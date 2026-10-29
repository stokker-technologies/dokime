/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use harmos::storage::{FileReference, ObjectVersion, StorageCursor};
use tokio::sync::Mutex;

use super::{cycle::CycleSnapshot, execution::ExecutionView};
use crate::RuntimeHandle;
use crate::runtime::Error;
use crate::runtime::Result;
use crate::runtime::progress::Operation;
use crate::storage::objects::cycle::Cycle;
use crate::storage::reports;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::TestCycle;
use dokime_domain::model::execution::saved::CycleSeal;
use dokime_domain::model::execution::saved::ExecutionRevision;
use dokime_domain::model::execution::saved::ExecutionSeal;
use dokime_domain::model::identity::TestCycleId;

pub(super) struct PendingCycle {
    snapshot: CycleSnapshot,
    executions: Arc<Vec<ExecutionSeal>>,
    report: Option<FileReference>,
    completed: Option<Arc<CompletedCycle>>,
}

pub(crate) struct Materialization {
    pub snapshot: CycleSnapshot,
    pub executions: Arc<Vec<ExecutionSeal>>,
    pub published: Option<FileReference>,
}

pub(crate) struct CompletedCycle {
    pub header: CompletedHeader,
}

pub(crate) struct CompletedHeader(harmos::storage::ReadBatch<TestCycle>);

impl std::ops::Deref for CompletedHeader {
    type Target = TestCycle;

    fn deref(&self) -> &TestCycle {
        &self.0
    }
}

impl super::executions::ExecutionStorage {
    pub(crate) async fn cycle_finalization_intent(&self, id: &TestCycleId) -> Result<Option<crate::transactions::CycleReportFinalization>> {
        Ok(self.state_manager.state().read(|state| state.cycle_report_finalizations.get(id).cloned()).await?.value)
    }

    pub(crate) async fn cycle_materialization_cursor(&self, id: &TestCycleId) -> Result<Option<StorageCursor>> {
        let owner = self.cycle_reports.lock().await.get(id).cloned();
        if let Some(owner) = owner {
            return Ok(Some(owner.lock().await.snapshot.saved().clone()));
        }
        Ok(self
            .state_manager
            .state()
            .read(|state| state.cycle_report_finalizations.get(id).map(|intent| intent.evidence_at.clone()))
            .await?
            .value)
    }

    pub(crate) async fn cycle_materialization(
        &self, id: &TestCycleId, at: &StorageCursor, operation: &Operation,
    ) -> Result<Option<Materialization>> {
        let owner = self.cycle_reports.lock().await.get(id).cloned();
        let owner = if let Some(owner) = owner {
            owner
        } else {
            let intent = self.cycle_finalization_intent(id).await?;
            let Some(intent) = intent else {
                return Ok(None);
            };
            if intent.evidence_at != *at {
                return Err(invalid("cycle request differs from persisted finalization boundary"));
            }
            let pending = self.prepare_cycle_report(Arc::new(intent.source), at, operation).await?;
            let owner = Arc::new(Mutex::new(pending));
            self.cycle_reports.lock().await.insert(id.clone(), owner.clone());
            owner
        };
        let pending = owner.lock().await;
        if pending.snapshot.saved() != at {
            return Err(invalid("cycle materialization changes its frozen cursor"));
        }
        Ok(Some(Materialization {
            snapshot: pending.snapshot.clone(),
            executions: pending.executions.clone(),
            published: pending.report.clone(),
        }))
    }

    pub(crate) async fn retain_cycle_report(&self, id: &TestCycleId, at: &StorageCursor, report: FileReference) -> Result<()> {
        let owner = self.cycle_reports.lock().await.get(id).cloned().ok_or_else(|| invalid("cycle finalization owner disappeared"))?;
        let mut pending = owner.lock().await;
        if pending.snapshot.saved() != at {
            return Err(invalid("cycle report boundary changed"));
        }
        if let Some(existing) = &pending.report {
            if *existing != report {
                return Err(invalid("cycle report was already published"));
            }
            return Ok(());
        }
        pending.report = Some(report);
        Ok(())
    }

    pub(crate) async fn retire_cycle_report(&self, id: &TestCycleId) {
        let Some(owner) = self.cycle_reports.lock().await.remove(id) else {
            return;
        };
        let pending = owner.lock().await;
        for member in &pending.snapshot.source().execution_ids {
            if let Some(acquisition) = self.acquisition(member).await {
                let mut acquisition = acquisition.lock().await;
                acquisition.cycle_context = None;
                acquisition.cycle_member = None;
            }
        }
    }
}

impl RuntimeHandle {
    // Members Release Their Own History When They Seal; A Reopened Cycle
    // Rebuilds Their Reports From The Anchors Those Releases Keep.
    async fn release_cycle_history(&self, cycle: &TestCycle, through: &StorageCursor) -> Result<()> {
        use harmos::catalog::Member;
        use harmos::storage::{ObjectDefinitions, ObjectKey};

        if cycle.saved.is_none() {
            return Err(invalid("unsealed cycle cannot release its history"));
        }
        let key_bytes =
            std::num::NonZeroUsize::new(cycle.test_cycle_id.as_str().len()).ok_or_else(|| invalid("cycle identity is empty"))?;
        let cycle_schema = <ObjectDefinitions as Member<Cycle>>::ENTRY.schema()?;
        let key = ObjectKey::new(cycle.test_cycle_id.as_str(), key_bytes)?;
        self.storage().release_object_history(&cycle_schema, &key, through).await?;
        Ok(())
    }

    async fn recover_saved_cycle(&self, intent: &crate::transactions::CycleReportFinalization) -> Result<Option<Arc<CompletedCycle>>> {
        let session = self.storage().clone();
        let Some(revision) = crate::storage::objects::cycle::read(&session, &intent.source.test_cycle_id, ObjectVersion::Latest).await?
        else {
            return Ok(None);
        };
        let Some(seal) = &revision.value.saved else {
            return Ok(None);
        };
        if !same_header(&revision.value, &intent.source) || seal.evidence_through != intent.evidence_at {
            return Err(invalid("durable cycle seal differs from its frozen finalization intent"));
        }
        seal.validate(&revision.value).map_err(crate::storage::Error::from)?;
        for member in &seal.executions {
            let execution = ExecutionView::open(self.storage(), &member.execution_id, ObjectVersion::Exact(member.version.clone()))
                .await?
                .ok_or_else(|| invalid("durable cycle member revision is missing"))?;
            if execution.revision()?.position != member.position {
                return Err(invalid("durable cycle member revision differs"));
            }
            if execution.execution().saved.is_none() {
                return Err(invalid("durable cycle member is unsealed"));
            }
        }
        self.release_cycle_history(&revision.value, &revision.position).await?;
        let header = revision.try_map(|revision| Ok::<_, Error>(revision.value))?;
        Ok(Some(Arc::new(CompletedCycle { header: CompletedHeader(header) })))
    }

    pub(crate) async fn finish_saved_cycle(&self, source: &CycleView, operation: &Operation) -> Result<Arc<CompletedCycle>> {
        let fallback = self.watch_state().latest();
        let session = source.session_id.as_ref().unwrap_or(&fallback.state().session_id);
        let mut frozen = header(source, None);
        frozen.session_id = Some(session.clone());
        let runtime = self.clone();
        let operation = operation.clone();
        // Saving Survives Caller Cancellation So A Started Seal Always Settles.
        tokio::spawn(async move {
            operation.phase("Waiting for Earlier Saves");
            operation.indeterminate();
            let order = runtime.execution_storage.cycle_save.lock().await;
            let id = frozen.test_cycle_id.clone();
            let intent = runtime.cycle_intent(frozen).await?;
            // Rendering Must Never Outrun The Durable Intent Journal.
            runtime.storage().flush_recovery().await?;
            if let Some(completed) = runtime.recover_saved_cycle(&intent).await? {
                return Ok(completed);
            }
            let owner = runtime.cycle_report_owner(&intent, &operation).await?;
            let at = owner.lock().await.snapshot.saved().clone();
            drop(order);
            runtime
                .operations()
                .call::<crate::operations::reports::MaterializeReport>(crate::operations::reports::MaterializeReport {
                    subject: dokime_domain::model::reports::wire::ReportSubject::Cycle(id.to_string()),
                    at,
                })
                .await?;
            let _order = runtime.execution_storage.cycle_save.lock().await;
            let mut pending = owner.lock().await;
            if let Some(completed) = &pending.completed {
                return Ok(completed.clone());
            }
            let completed = runtime.publish_cycle_seal(&pending, &operation).await?;
            pending.completed = Some(completed.clone());
            Ok(completed)
        })
        .await?
    }

    // The Durable Boundary Of A Cycle Report: The Persisted Intent, Or A New
    // One Chosen After Every Member Seal Is Durable.
    async fn cycle_intent(
        &self, frozen: dokime_domain::model::execution::result::TestCycle,
    ) -> Result<crate::transactions::CycleReportFinalization> {
        if let Some(intent) = self.execution_storage.cycle_finalization_intent(&frozen.test_cycle_id).await? {
            return Ok(intent);
        }
        let evidence_at = self.storage().flush().await?;
        for member in &frozen.execution_ids {
            let execution = ExecutionView::open(self.storage(), member, ObjectVersion::Latest)
                .await?
                .ok_or_else(|| invalid("cycle intent requires every persisted member"))?;
            let seal =
                execution.execution().saved.as_ref().ok_or_else(|| invalid("cycle intent requires every member seal to be durable"))?;
            if !evidence_at.covers(&execution.revision()?.position)? || !execution.revision()?.position.covers(&seal.dependencies_saved)? {
                return Err(invalid("cycle intent precedes a member seal dependency"));
            }
        }
        let intent = crate::transactions::CycleReportFinalization { source: frozen, evidence_at };
        self.commit(crate::transactions::RequestCycleReportFinalization { intent: intent.clone() })
            .await
            .map_err(|error| Error::Operation(error.to_string()))?;
        Ok(intent)
    }

    // The One Report Owner Of A Finalizing Cycle, Shared With Its Materialization.
    async fn cycle_report_owner(
        &self, intent: &crate::transactions::CycleReportFinalization, operation: &Operation,
    ) -> Result<Arc<Mutex<PendingCycle>>> {
        let frozen = Arc::new(intent.source.clone());
        let id = &frozen.test_cycle_id;
        let existing = self.execution_storage.cycle_reports.lock().await.get(id).cloned();
        if let Some(owner) = existing {
            if **owner.lock().await.snapshot.source() != *frozen {
                return Err(invalid("cycle finalization changed its frozen source"));
            }
            return Ok(owner);
        }
        let pending = self.execution_storage.prepare_cycle_report(frozen.clone(), &intent.evidence_at, operation).await?;
        let owner = Arc::new(Mutex::new(pending));
        self.execution_storage.cycle_reports.lock().await.insert(id.clone(), owner.clone());
        Ok(owner)
    }

    // Seal A Cycle Whose Report File Is Published: Save The Seal And The Header.
    async fn publish_cycle_seal(&self, pending: &PendingCycle, operation: &Operation) -> Result<Arc<CompletedCycle>> {
        operation.phase("Writing Report");
        operation.indeterminate();
        let report = pending.report.clone().ok_or_else(|| invalid("cycle materialization did not publish a file"))?;
        let saved = self.storage().flush_until(&report.position).await?;
        if !saved.covers(&report.position)? {
            return Err(invalid("cycle report is not saved"));
        }
        let seal = CycleSeal {
            owner_node_id: pending.snapshot.owner_node_id().clone(),
            evidence_through: pending.snapshot.saved().clone(),
            executions: (**pending.snapshot.revisions()).clone(),
            report,
            dependencies_saved: saved,
            sealed_at: pending.snapshot.captured_at().clone(),
        };
        seal.validate(pending.snapshot.source()).map_err(crate::storage::Error::from)?;
        operation.phase("Publishing Report");
        let inputs = pending.executions.iter().map(|execution| execution.source.context.clone()).collect::<Vec<_>>();
        reports::Prepared::new(
            self.storage(),
            &reports::Input {
                owner: reports::Owner::Cycle(&pending.snapshot.source().test_cycle_id),
                cycle: Some(pending.snapshot.source()),
                execution: None,
                generated_at: &seal.sealed_at,
                final_report: true,
                evidence_through: &seal.evidence_through,
                dependencies_saved: &seal.dependencies_saved,
                executions: &seal.executions,
                inputs: &inputs,
                file: &seal.report,
            },
            self.schema_settings(),
        )?
        .finish()
        .await?;

        operation.phase("Saving Seal");
        let source = &**pending.snapshot.source();
        let cycle = harmos::storage::ReadBatch::new(()).try_map(|_| {
            let mut cycle = source.clone();
            cycle.phase = dokime_domain::model::execution::result::CyclePhase::Sealed;
            cycle.saved = Some(seal);
            Ok::<_, Error>(cycle)
        })?;
        let prepared = cycle.try_map(|cycle| {
            let row = Cycle::try_from(&cycle)?;
            Ok::<_, Error>((cycle, row))
        })?;
        let receipt = self.storage().write_object(&prepared.1).await?;
        let saved = self.storage().flush_until(&receipt.position).await?;
        if !saved.covers(&receipt.position)? {
            return Err(invalid("cycle seal is not saved"));
        }
        let header = prepared.try_map(|(cycle, _row)| Ok::<_, Error>(cycle))?;
        let completed = Arc::new(CompletedCycle { header: CompletedHeader(header) });
        self.release_cycle_history(&completed.header, &saved).await?;
        Ok(completed)
    }
}

impl super::executions::ExecutionStorage {
    pub(super) async fn prepare_cycle_report(
        &self, source: Arc<TestCycle>, at: &StorageCursor, operation: &Operation,
    ) -> Result<PendingCycle> {
        operation.phase("Collecting Sealed Members");
        let session = self.storage.clone();
        let mut revisions = Vec::with_capacity(source.execution_ids.len());
        let mut executions = Vec::with_capacity(source.execution_ids.len());
        let mut appearance = None;
        let total = source.execution_ids.len() as u64;
        for (collected, id) in source.execution_ids.iter().enumerate() {
            operation.progress(collected as u64, total);
            if let Some((context, original, execution)) = self.original_cycle_member(id, at).await? {
                let seal = execution.saved.as_ref().expect("verified original sealed owner");
                executions.push(seal.clone());
                revisions.push((*original).clone());
                appearance.get_or_insert_with(|| context.appearance.clone());
                continue;
            }
            let inspection = ExecutionView::open(&self.storage, id, ObjectVersion::Latest)
                .await?
                .ok_or_else(|| invalid("cycle member has no saved revision"))?;
            let seal = inspection.execution().saved.as_ref().ok_or_else(|| invalid("cycle member is not sealed"))?;
            executions.push(seal.clone());
            let original = inspection.revision()?;
            revisions.push(ExecutionRevision {
                execution_id: id.clone(),
                position: original.position.clone(),
                version: original.version.clone(),
            });
            if appearance.is_none() {
                let snapshot = inspection.snapshot().await?;
                appearance = Some(snapshot.context()?.appearance.clone());
            }
        }
        let appearance = match appearance {
            Some(appearance) => appearance,
            None => self.report_appearance().await?,
        };
        // The Native Seal May Have Committed Before Managed Adoption Failed.
        // Current-Report Opens Reuse That Exact File Rather Than Publishing Again.
        let latest = crate::storage::objects::cycle::read(&session, &source.test_cycle_id, ObjectVersion::Latest).await?;
        let published = match latest.as_ref().and_then(|latest| latest.value.saved.as_ref()) {
            Some(seal) => {
                let latest = &latest.as_ref().expect("selected saved cycle").value;
                if !same_header(latest, &source) || seal.evidence_through != *at {
                    return Err(invalid("published cycle differs from frozen finalization source"));
                }
                seal.validate(latest).map_err(crate::storage::Error::from)?;
                Some(seal.report.clone())
            }
            None => None,
        };
        let snapshot = CycleSnapshot::capture(
            at.clone(),
            source,
            Arc::new(revisions),
            Arc::new(appearance),
            &self.state_manager.state().read(|state| state.identity.node.node_id.clone()).await?.value,
        )?;
        Ok(PendingCycle {
            snapshot,
            executions: Arc::new(executions),
            report: published,
            completed: None,
        })
    }
}

pub(crate) fn header(cycle: &CycleView, saved: Option<CycleSeal>) -> TestCycle {
    TestCycle {
        saved,
        test_cycle_id: cycle.test_cycle_id.clone(),
        session_id: cycle.session_id.clone(),
        metadata: cycle.metadata.clone(),
        phase: cycle.phase,
        lifecycle: cycle.lifecycle,
        created_at: cycle.created_at.clone(),
        started_at: cycle.started_at.clone(),
        updated_at: cycle.updated_at.clone(),
        execution_ids: cycle.test_executions.iter().map(|execution| execution.test_execution_id.clone()).collect(),
        records: cycle.records.clone(),
        continue_automatically: cycle.continue_automatically,
        result: cycle.result.clone(),
    }
}

fn same_header(left: &TestCycle, right: &TestCycle) -> bool {
    left.test_cycle_id == right.test_cycle_id
        && left.session_id == right.session_id
        && left.metadata == right.metadata
        && left.lifecycle == right.lifecycle
        && left.created_at == right.created_at
        && left.started_at == right.started_at
        && left.updated_at == right.updated_at
        && left.execution_ids == right.execution_ids
        && left.records == right.records
        && left.continue_automatically == right.continue_automatically
        && left.result == right.result
}

fn invalid(reason: &'static str) -> Error {
    Error::Storage(crate::storage::Error::Invalid(reason))
}
