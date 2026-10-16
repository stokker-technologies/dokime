/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{num::NonZeroUsize, sync::Arc};
// Third Party Imports
use harmos::{
    MemorySize,
    storage::{
        FileReference, ObjectReceipt, ObjectVersion, QueryValue, SharedQueryValue, StorageCursor,
        StorageProductionPreparation,
    },
};
use tokio::sync::Mutex;
// Local Imports
use super::{cycle::CycleSnapshot, execution::ExecutionView};
use crate::{Error, Result, RuntimeHandle, execution::CycleView, runtime::progress::Operation};
use dokime::{
    core::TestCycleId,
    execution::{
        TestCycle,
        saved::{CycleSeal, ExecutionRevision, ExecutionSeal},
    },
    storage::{objects::cycle::Cycle, reports},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct PendingCycle {
    snapshot: CycleSnapshot,
    executions: SharedQueryValue<Vec<ExecutionSeal>>,
    report: Option<SharedQueryValue<FileReference>>,
    prepared: Arc<Mutex<Option<crate::report::cycle::CycleReport>>>,
    seal: Option<SharedQueryValue<CycleSeal>>,
    native: Option<QueryValue<Cycle>>,
    publication: Option<reports::Prepared>,
    receipt: Option<ObjectReceipt>,
    completed: Option<SharedQueryValue<CompletedCycle>>,
    _registry: StorageProductionPreparation,
}

pub(crate) struct CompletedCycle {
    pub header: TestCycle,
    pub executions: Vec<ExecutionSeal>,
}
impl MemorySize for CompletedCycle {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(
                self.header
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<TestCycle>()),
            )
            .saturating_add(
                self.executions
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<Vec<ExecutionSeal>>()),
            )
    }
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl super::executions::ExecutionStorage {
    pub(crate) async fn cycle_finalization_intent(
        &self,
        id: &TestCycleId,
    ) -> Result<Option<QueryValue<crate::transactions::CycleReportFinalization>>> {
        self.state_manager
            .state()
            .read(|state| {
                state
                    .cycle_report_finalizations
                    .get(id)
                    .map(|intent| {
                        self.storage
                            .produce(bound(intent.memory_size())?, |_| Ok(intent.clone()))?
                            .try_into_value()
                            .map_err(Error::from)
                    })
                    .transpose()
            })
            .await?
            .value
    }

    pub(crate) async fn cycle_materialization_cursor(
        &self,
        id: &TestCycleId,
    ) -> Result<Option<StorageCursor>> {
        let owner = self.cycle_reports.lock().await.get(id).cloned();
        if let Some(owner) = owner {
            return Ok(Some(owner.lock().await.snapshot.view().saved()?));
        }
        Ok(self
            .state_manager
            .state()
            .read(|state| {
                state
                    .cycle_report_finalizations
                    .get(id)
                    .map(|intent| intent.evidence_at.clone())
            })
            .await?
            .value)
    }

    pub(crate) async fn cycle_materialization(
        &self,
        id: &TestCycleId,
        at: &StorageCursor,
        limits: harmos::storage::QueryLimits,
        operation: &Operation,
    ) -> Result<
        Option<(
            CycleSnapshot,
            SharedQueryValue<Vec<ExecutionSeal>>,
            Option<SharedQueryValue<FileReference>>,
            Arc<Mutex<Option<crate::report::cycle::CycleReport>>>,
        )>,
    > {
        let owner = self.cycle_reports.lock().await.get(id).cloned();
        let owner = if let Some(owner) = owner {
            owner
        } else {
            let intent = self.cycle_finalization_intent(id).await?;
            let Some(intent) = intent else {
                return Ok(None);
            };
            if intent.evidence_at != *at {
                return Err(invalid(
                    "cycle request differs from persisted finalization boundary",
                ));
            }
            let source = self
                .storage
                .produce(bound(intent.source.memory_size())?, |_| {
                    Ok(intent.source.clone())
                })?
                .try_into_value()?
                .try_into_shared()?;
            let pending = self
                .prepare_cycle_report(source, at, limits, operation)
                .await?;
            let owner = Arc::new(Mutex::new(pending));
            self.cycle_reports
                .lock()
                .await
                .insert(id.clone(), owner.clone());
            owner
        };
        let pending = owner.lock().await;
        if pending.snapshot.view().saved()? != *at {
            return Err(invalid("cycle materialization changes its frozen cursor"));
        }
        Ok(Some((
            pending.snapshot.clone(),
            pending.executions.clone(),
            pending.report.clone(),
            pending.prepared.clone(),
        )))
    }

    pub(crate) async fn retain_cycle_report(
        &self,
        id: &TestCycleId,
        at: &StorageCursor,
        report: FileReference,
    ) -> Result<()> {
        let owner = self
            .cycle_reports
            .lock()
            .await
            .get(id)
            .cloned()
            .ok_or_else(|| invalid("cycle finalization owner disappeared"))?;
        let mut pending = owner.lock().await;
        if pending.snapshot.view().saved()? != *at {
            return Err(invalid("cycle report boundary changed"));
        }
        if let Some(existing) = &pending.report {
            if **existing != report {
                return Err(invalid("cycle report was already published"));
            }
            return Ok(());
        }
        pending.report = Some(
            self.storage
                .produce(bound(report.memory_size())?, |_| Ok(report))?
                .try_into_value()?
                .try_into_shared()?,
        );
        Ok(())
    }

    pub(crate) async fn retire_cycle_report(&self, id: &TestCycleId) {
        let Some(owner) = self.cycle_reports.lock().await.remove(id) else {
            return;
        };
        let pending = owner.lock().await;
        for member in &pending.snapshot.source().execution_ids {
            if let Some(acquisition) = self.acquisition(member).await {
                acquisition.lock().await.cycle_context = None;
            }
        }
    }
}

impl RuntimeHandle {
    async fn release_cycle_history(
        &self,
        cycle: &TestCycle,
        through: &StorageCursor,
    ) -> Result<()> {
        use dokime::storage::objects::execution::Execution;
        use harmos::catalog::Member;
        use harmos::storage::{ObjectDefinitions, ObjectKey};

        if cycle.saved.is_none() {
            return Err(invalid("unsealed cycle cannot release member history"));
        }
        let key_bytes = cycle
            .execution_ids
            .iter()
            .fold(cycle.test_cycle_id.as_str().len(), |maximum, id| {
                maximum.max(id.as_str().len())
            });
        let _memory = self
            .storage()
            .prepare_production(bound(key_bytes.saturating_add(4096))?)?;
        let execution_schema = <ObjectDefinitions as Member<Execution>>::ENTRY.schema()?;
        let cycle_schema = <ObjectDefinitions as Member<Cycle>>::ENTRY.schema()?;
        for id in &cycle.execution_ids {
            let key = ObjectKey::new(id.as_str(), bound(key_bytes)?)?;
            self.storage()
                .release_object_history(&execution_schema, &key, through)
                .await?;
        }
        let key = ObjectKey::new(cycle.test_cycle_id.as_str(), bound(key_bytes)?)?;
        self.storage()
            .release_object_history(&cycle_schema, &key, through)
            .await?;
        Ok(())
    }

    async fn recover_saved_cycle(
        &self,
        intent: &crate::transactions::CycleReportFinalization,
    ) -> Result<Option<SharedQueryValue<CompletedCycle>>> {
        let view = self
            .storage()
            .view_with_limits(self.config().storage.finalization())
            .await?;
        let limit = view.decoded_bytes()?;
        let Some(revision) = dokime::storage::objects::cycle::read_in(
            self.storage(),
            &view,
            &intent.source.test_cycle_id,
            ObjectVersion::Latest,
            limit,
        )
        .await?
        else {
            return Ok(None);
        };
        let Some(seal) = &revision.value.saved else {
            return Ok(None);
        };
        if !same_header(&revision.value, &intent.source)
            || seal.evidence_through != intent.evidence_at
        {
            return Err(invalid(
                "durable cycle seal differs from its frozen finalization intent",
            ));
        }
        seal.validate(&revision.value)
            .map_err(dokime::storage::Error::from)?;
        let mut executions = self
            .storage()
            .produce(bound(std::mem::size_of::<Vec<ExecutionSeal>>())?, |_| {
                Ok(Vec::new())
            })?
            .try_into_value()?;
        for member in &seal.executions {
            let execution = ExecutionView::open(
                self.storage(),
                view.clone(),
                &member.execution_id,
                ObjectVersion::Exact(member.version.clone()),
                limit,
            )
            .await?
            .ok_or_else(|| invalid("durable cycle member revision is missing"))?;
            if execution.revision()?.position != member.position {
                return Err(invalid("durable cycle member revision differs"));
            }
            let member_seal = execution
                .execution()
                .saved
                .as_ref()
                .ok_or_else(|| invalid("durable cycle member is unsealed"))?;
            let bytes = executions
                .memory_size()
                .saturating_add(member_seal.memory_size());
            executions = executions.try_map(bound(bytes)?, |mut executions, _| {
                executions.try_reserve_exact(1).map_err(|_| exhausted())?;
                executions.push(member_seal.clone());
                Ok(executions)
            })?;
        }
        self.release_cycle_history(&revision.value, &revision.position)
            .await?;
        let bytes = std::mem::size_of::<CompletedCycle>()
            .saturating_add(revision.value.memory_size())
            .saturating_add(executions.memory_size());
        Ok(Some(
            self.storage()
                .produce(bound(bytes)?, |_| {
                    Ok(CompletedCycle {
                        header: revision.value.clone(),
                        executions: (*executions).clone(),
                    })
                })?
                .try_into_value()?
                .try_into_shared()?,
        ))
    }

    pub(crate) async fn finish_saved_cycle(
        &self,
        source: &CycleView,
        operation: &Operation,
    ) -> Result<SharedQueryValue<CompletedCycle>> {
        let fallback = self.watch_state().latest();
        let session = source
            .session_id
            .as_ref()
            .unwrap_or(&fallback.state().session_id);
        let frozen = self
            .storage()
            .produce(
                bound(header_memory(source).saturating_add(session.memory_size()))?,
                |_| {
                    let mut frozen = header(source, None);
                    frozen.session_id = Some(session.clone());
                    Ok(frozen)
                },
            )?
            .try_into_value()?
            .try_into_shared()?;
        let runtime = self.clone();
        let operation = operation.clone();
        tokio::spawn(async move {
            operation.phase("Waiting for Earlier Saves");
            operation.indeterminate();
            let _order = runtime.execution_storage.cycle_save.lock().await;
            let id = frozen.test_cycle_id.clone();
            let intent = runtime
                .execution_storage
                .cycle_finalization_intent(&id)
                .await?;
            let intent = if let Some(intent) = intent {
                intent
            } else {
                // Child Seals Must Be Durable Before The Cycle Boundary Is Chosen.
                let evidence_at = runtime.storage().flush().await?;
                let verified = runtime
                    .storage()
                    .view_at(&evidence_at, runtime.config().storage.finalization())
                    .await?;
                for member in &frozen.execution_ids {
                    let execution = ExecutionView::open(
                        runtime.storage(),
                        verified.clone(),
                        member,
                        ObjectVersion::Latest,
                        verified.decoded_bytes()?,
                    )
                    .await?
                    .ok_or_else(|| invalid("cycle intent requires every persisted member"))?;
                    let seal = execution.execution().saved.as_ref().ok_or_else(|| {
                        invalid("cycle intent requires every member seal to be durable")
                    })?;
                    if !evidence_at.covers(&execution.revision()?.position)?
                        || !execution
                            .revision()?
                            .position
                            .covers(&seal.dependencies_saved)?
                    {
                        return Err(invalid("cycle intent precedes a member seal dependency"));
                    }
                }
                drop(verified);
                let bytes = frozen
                    .memory_size()
                    .saturating_add(evidence_at.memory_size())
                    .saturating_add(std::mem::size_of::<
                        crate::transactions::CycleReportFinalization,
                    >());
                let intent = runtime
                    .storage()
                    .produce(bound(bytes)?, |_| {
                        Ok(crate::transactions::CycleReportFinalization {
                            source: (*frozen).clone(),
                            evidence_at,
                        })
                    })?
                    .try_into_value()?;
                let _journal_copy = runtime
                    .storage()
                    .prepare_production(bound(intent.memory_size())?)?;
                runtime
                    .commit(crate::transactions::RequestCycleReportFinalization {
                        intent: (*intent).clone(),
                    })
                    .await
                    .map_err(|error| Error::Operation(error.to_string()))?;
                intent
            };
            // Rendering Must Never Outrun The Durable Intent Journal.
            runtime.storage().flush_recovery().await?;
            let frozen = runtime
                .storage()
                .produce(bound(intent.source.memory_size())?, |_| {
                    Ok(intent.source.clone())
                })?
                .try_into_value()?
                .try_into_shared()?;
            if let Some(completed) = runtime.recover_saved_cycle(&intent).await? {
                return Ok(completed);
            }
            let existing = runtime
                .execution_storage
                .cycle_reports
                .lock()
                .await
                .get(&id)
                .cloned();
            let owner = if let Some(owner) = existing {
                if **owner.lock().await.snapshot.source() != *frozen {
                    return Err(invalid("cycle finalization changed its frozen source"));
                }
                owner
            } else {
                let pending = runtime
                    .execution_storage
                    .prepare_cycle_report(
                        frozen.clone(),
                        &intent.evidence_at,
                        runtime.config().storage.finalization(),
                        &operation,
                    )
                    .await?;
                let owner = Arc::new(Mutex::new(pending));
                runtime
                    .execution_storage
                    .cycle_reports
                    .lock()
                    .await
                    .insert(id.clone(), owner.clone());
                owner
            };
            let at = owner.lock().await.snapshot.view().saved()?;
            drop(_order);
            runtime
                .operations()
                .call_as::<dokime::operations::reports::MaterializeReport>(
                    dokime::operations::reports::MaterializeReportRequest {
                        subject: dokime::operations::reports::ReportSubject::Cycle(id.to_string()),
                        at,
                    },
                )
                .await?;
            let _order = runtime.execution_storage.cycle_save.lock().await;
            let mut pending = owner.lock().await;
            if let Some(completed) = &pending.completed {
                return Ok(completed.clone());
            }
            if pending.seal.is_none() {
                operation.phase("Writing Report");
                operation.indeterminate();
                let report = (**pending
                    .report
                    .as_ref()
                    .ok_or_else(|| invalid("cycle materialization did not publish a file"))?)
                .clone();
                let saved = runtime.storage().flush_until(&report.position).await?;
                if !saved.covers(&report.position)? {
                    return Err(invalid("cycle report is not saved"));
                }
                let bytes = std::mem::size_of::<CycleSeal>()
                    .saturating_add(report.memory_size())
                    .saturating_add(pending.snapshot.revisions().memory_size())
                    .saturating_add(
                        pending
                            .snapshot
                            .view()
                            .saved()?
                            .memory_size()
                            .saturating_mul(2),
                    )
                    .saturating_add(pending.snapshot.owner_node_id().memory_size())
                    .saturating_add(pending.snapshot.captured_at().memory_size());
                let seal = runtime
                    .storage()
                    .produce(bound(bytes)?, |_| {
                        Ok(CycleSeal {
                            owner_node_id: pending.snapshot.owner_node_id().clone(),
                            evidence_through: pending.snapshot.view().saved()?,
                            executions: (**pending.snapshot.revisions()).clone(),
                            report,
                            dependencies_saved: saved,
                            sealed_at: pending.snapshot.captured_at().clone(),
                        })
                    })?
                    .try_into_value()?
                    .try_into_shared()?;
                seal.validate(pending.snapshot.source())
                    .map_err(dokime::storage::Error::from)?;
                pending.seal = Some(seal);
            }
            operation.phase("Publishing Report");
            if pending.publication.is_none() {
                let seal = pending.seal.as_ref().expect("prepared cycle seal");
                let bytes = pending.executions.iter().fold(
                    std::mem::size_of::<Vec<harmos::storage::FileReference>>(),
                    |bytes, execution| bytes.saturating_add(execution.source.context.memory_size()),
                );
                let inputs = runtime
                    .storage()
                    .produce(bound(bytes)?, |_| {
                        let mut inputs = Vec::with_capacity(pending.executions.len());
                        for execution in pending.executions.iter() {
                            inputs.push(execution.source.context.clone());
                        }
                        Ok(inputs)
                    })?
                    .try_into_value()?;
                let publication = reports::Prepared::new(
                    runtime.storage(),
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
                )?;
                pending.publication = Some(publication);
            }
            pending
                .publication
                .as_mut()
                .expect("prepared report publication")
                .finish()
                .await?;

            operation.phase("Saving Seal");
            if pending.native.is_none() {
                let seal = pending.seal.as_ref().unwrap();
                let header_bytes = pending
                    .snapshot
                    .source()
                    .memory_size()
                    .saturating_add(seal.memory_size());
                let header = runtime.storage().produce(bound(header_bytes)?, |_| {
                    Ok({
                        let mut header = (**pending.snapshot.source()).clone();
                        header.saved = Some((**seal).clone());
                        header
                    })
                })?;
                let bytes = Cycle::encoding_memory(header.value())?;
                let native = runtime
                    .storage()
                    .produce(bound(bytes)?, |_| {
                        Cycle::try_from(header.value()).map_err(|source| codec(source.into()))
                    })?
                    .try_into_value()?;
                pending.native = Some(native);
            }
            if pending.receipt.is_none() {
                pending.receipt = Some(
                    runtime
                        .storage()
                        .write_object(&**pending.native.as_ref().unwrap())
                        .await?,
                );
            }
            let receipt = pending.receipt.as_ref().unwrap();
            let saved = runtime.storage().flush_until(&receipt.position).await?;
            if !saved.covers(&receipt.position)? {
                return Err(invalid("cycle seal is not saved"));
            }
            let seal = pending.seal.as_ref().unwrap();
            let bytes = pending
                .snapshot
                .source()
                .memory_size()
                .saturating_add(seal.memory_size())
                .saturating_add(pending.executions.memory_size())
                .saturating_add(std::mem::size_of::<CompletedCycle>());
            let completed = runtime
                .storage()
                .produce(bound(bytes)?, |_| {
                    let mut cycle = (**pending.snapshot.source()).clone();
                    cycle.saved = Some((**seal).clone());
                    Ok(CompletedCycle {
                        header: cycle,
                        executions: (*pending.executions).clone(),
                    })
                })?
                .try_into_value()?
                .try_into_shared()?;
            runtime
                .release_cycle_history(&completed.header, &saved)
                .await?;
            pending.completed = Some(completed.clone());
            Ok(completed)
        })
        .await?
    }
}

impl super::executions::ExecutionStorage {
    pub(super) async fn prepare_cycle_report(
        &self,
        source: SharedQueryValue<TestCycle>,
        at: &StorageCursor,
        limits: harmos::storage::QueryLimits,
        operation: &Operation,
    ) -> Result<PendingCycle> {
        operation.phase("Collecting Sealed Members");
        let view = self.storage.view_at(at, limits).await?;
        let limit = view.decoded_bytes()?;
        let mut revisions = self
            .storage
            .produce(
                bound(std::mem::size_of::<Vec<ExecutionRevision>>())?,
                |_| Ok(Vec::new()),
            )?
            .try_into_value()?;
        let mut executions = self
            .storage
            .produce(bound(std::mem::size_of::<Vec<ExecutionSeal>>())?, |_| {
                Ok(Vec::new())
            })?
            .try_into_value()?;
        let mut appearance = None;
        let total = source.execution_ids.len() as u64;
        for (collected, id) in source.execution_ids.iter().enumerate() {
            operation.progress(collected as u64, total);
            if let Some((context, original, execution)) =
                self.original_cycle_member(id, &view).await?
            {
                let computation = view.begin_computation()?;
                let seal = execution
                    .saved
                    .as_ref()
                    .expect("verified original sealed owner");
                let bytes = executions.memory_size().saturating_add(seal.memory_size());
                executions = executions.try_map(bound(bytes)?, |mut executions, _| {
                    executions.try_reserve_exact(1).map_err(|_| exhausted())?;
                    executions.push(seal.clone());
                    Ok(executions)
                })?;
                let bytes = revisions
                    .memory_size()
                    .saturating_add(original.memory_size());
                revisions = revisions.try_map(bound(bytes)?, |mut revisions, _| {
                    revisions.try_reserve_exact(1).map_err(|_| exhausted())?;
                    revisions.push((*original).clone());
                    Ok(revisions)
                })?;
                if appearance.is_none() {
                    appearance = Some(
                        self.storage
                            .produce(bound(context.appearance.memory_size())?, |_| {
                                Ok(context.appearance.clone())
                            })?
                            .try_into_value()?
                            .try_into_shared()?,
                    );
                }
                computation.check()?;
                continue;
            }
            let inspection = ExecutionView::open(
                &self.storage,
                view.clone(),
                id,
                ObjectVersion::Latest,
                limit,
            )
            .await?
            .ok_or_else(|| invalid("cycle member has no saved revision"))?;
            if inspection.execution().saved.is_none() {
                return Err(invalid("cycle member is not sealed"));
            }
            let seal = inspection.execution().saved.as_ref().unwrap();
            let bytes = executions.memory_size().saturating_add(seal.memory_size());
            executions = executions.try_map(bound(bytes)?, |mut executions, _| {
                executions.try_reserve_exact(1).map_err(|_| exhausted())?;
                executions.push(seal.clone());
                Ok(executions)
            })?;
            let snapshot = inspection.snapshot(&self.storage).await?;
            let context = snapshot.context()?;
            let original = inspection.revision()?;
            let revision_bytes = std::mem::size_of::<ExecutionRevision>()
                .saturating_add(id.memory_size())
                .saturating_add(original.position.memory_size())
                .saturating_add(original.version.memory_size());
            let bytes = revisions.memory_size().saturating_add(revision_bytes);
            revisions = revisions.try_map(bound(bytes)?, |mut revisions, _| {
                revisions.try_reserve_exact(1).map_err(|_| exhausted())?;
                revisions.push(ExecutionRevision {
                    execution_id: id.clone(),
                    position: original.position.clone(),
                    version: original.version.clone(),
                });
                Ok(revisions)
            })?;
            if appearance.is_none() {
                appearance = Some(
                    self.storage
                        .produce(bound(context.appearance.memory_size())?, |_| {
                            Ok(context.appearance.clone())
                        })?
                        .try_into_value()?
                        .try_into_shared()?,
                );
            }
        }
        let appearance = match appearance {
            Some(appearance) => appearance,
            None => self
                .report_appearance(&view, limit)
                .await?
                .try_into_shared()?,
        };
        // The Native Seal May Have Committed Before Managed Adoption Failed.
        // Current-Report Opens Reuse That Exact File Rather Than Publishing Again.
        let latest_view = self.storage.view_with_limits(limits).await?;
        let latest = dokime::storage::objects::cycle::read_in(
            &self.storage,
            &latest_view,
            &source.test_cycle_id,
            ObjectVersion::Latest,
            limit,
        )
        .await?;
        let published = if let Some(latest) = &latest {
            if let Some(seal) = &latest.value.saved {
                if !same_header(&latest.value, &source) || seal.evidence_through != *at {
                    return Err(invalid(
                        "published cycle differs from frozen finalization source",
                    ));
                }
                seal.validate(&latest.value)
                    .map_err(dokime::storage::Error::from)?;
                Some(
                    self.storage
                        .produce(bound(seal.report.memory_size())?, |_| {
                            Ok(seal.report.clone())
                        })?
                        .try_into_value()?
                        .try_into_shared()?,
                )
            } else {
                None
            }
        } else {
            None
        };
        drop(latest);
        drop(latest_view);
        let revisions = revisions.try_into_shared()?;
        let snapshot = CycleSnapshot::capture(
            &self.storage,
            view,
            source.clone(),
            revisions,
            appearance,
            &self
                .state_manager
                .state()
                .read(|state| state.identity.node.node_id.clone())
                .await?
                .value,
        )?;
        let bytes = std::mem::size_of::<Mutex<PendingCycle>>()
            + 4 * std::mem::size_of::<usize>()
            + std::mem::size_of::<Mutex<Option<crate::report::cycle::CycleReport>>>()
            + harmos::btree_map_memory::<TestCycleId, Arc<Mutex<PendingCycle>>>(1)
            + source.test_cycle_id.memory_size();
        let registry = self.storage.prepare_production(bound(bytes)?)?;
        Ok(PendingCycle {
            snapshot,
            executions: executions.try_into_shared()?,
            report: published,
            prepared: Arc::new(Mutex::new(None)),
            seal: None,
            native: None,
            publication: None,
            receipt: None,
            completed: None,
            _registry: registry,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
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
        && left.continue_on_failure == right.continue_on_failure
        && left.result == right.result
}

pub(crate) fn header_memory(cycle: &CycleView) -> usize {
    let fields = [
        (
            cycle.test_cycle_id.memory_size(),
            std::mem::size_of_val(&cycle.test_cycle_id),
        ),
        (
            cycle.session_id.memory_size(),
            std::mem::size_of_val(&cycle.session_id),
        ),
        (
            cycle.metadata.memory_size(),
            std::mem::size_of_val(&cycle.metadata),
        ),
        (
            cycle.created_at.memory_size(),
            std::mem::size_of_val(&cycle.created_at),
        ),
        (
            cycle.started_at.memory_size(),
            std::mem::size_of_val(&cycle.started_at),
        ),
        (
            cycle.updated_at.memory_size(),
            std::mem::size_of_val(&cycle.updated_at),
        ),
        (
            cycle.records.memory_size(),
            std::mem::size_of_val(&cycle.records),
        ),
    ];
    let header = fields.into_iter().fold(
        std::mem::size_of::<TestCycle>(),
        |bytes, (owned, inline)| {
            if owned == usize::MAX {
                usize::MAX
            } else {
                bytes.saturating_add(owned.saturating_sub(inline))
            }
        },
    );
    cycle
        .test_executions
        .iter()
        .fold(header, |bytes, execution| {
            bytes.saturating_add(execution.test_execution_id.memory_size())
        })
}

pub(crate) fn header(cycle: &CycleView, saved: Option<CycleSeal>) -> TestCycle {
    TestCycle {
        saved,
        test_cycle_id: cycle.test_cycle_id.clone(),
        session_id: cycle.session_id.clone(),
        metadata: cycle.metadata.clone(),
        lifecycle: cycle.lifecycle,
        created_at: cycle.created_at.clone(),
        started_at: cycle.started_at.clone(),
        updated_at: cycle.updated_at.clone(),
        execution_ids: cycle
            .test_executions
            .iter()
            .map(|execution| execution.test_execution_id.clone())
            .collect(),
        records: cycle.records.clone(),
        continue_on_failure: cycle.continue_on_failure,
        result: cycle.result.clone(),
    }
}
fn bound(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes)
        .filter(|_| bytes != usize::MAX)
        .ok_or_else(|| invalid("cycle report source memory overflows"))
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "cycle final report memory",
    }
}
fn invalid(reason: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(reason))
}
fn codec(source: Error) -> harmos::Error {
    harmos::Error::backend(
        "encode final cycle",
        harmos::BackendFailure::Permanent,
        source,
    )
}

/* === Tests === */
