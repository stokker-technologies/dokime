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
        ObjectReceipt, ObjectVersion, QueryValue, SharedQueryValue, StorageProductionPreparation,
    },
};
use tokio::sync::Mutex;
// Local Imports
use super::{cycle::CycleSnapshot, execution::ExecutionView};
use crate::{Error, Result, RuntimeHandle, execution::CycleView, report::cycle::CycleReport};
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
    report: CycleReport,
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
    pub(crate) async fn finish_saved_cycle(
        &self,
        source: &CycleView,
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
        tokio::spawn(async move {
            let _order = runtime.execution_storage.cycle_save.lock().await;
            let id = &frozen.test_cycle_id;
            let existing = runtime
                .execution_storage
                .cycle_reports
                .lock()
                .await
                .get(id)
                .cloned();
            let owner = if let Some(owner) = existing {
                if **owner.lock().await.snapshot.source() != *frozen {
                    return Err(invalid("cycle finalization changed its frozen source"));
                }
                owner
            } else {
                let pending = runtime.prepare_cycle_report(frozen.clone()).await?;
                let owner = Arc::new(Mutex::new(pending));
                runtime
                    .execution_storage
                    .cycle_reports
                    .lock()
                    .await
                    .insert(id.clone(), owner.clone());
                owner
            };
            let mut pending = owner.lock().await;
            if let Some(completed) = &pending.completed {
                return Ok(completed.clone());
            }
            if pending.seal.is_none() {
                let report = pending.report.finish().await?;
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
            pending.completed = Some(completed.clone());
            Ok(completed)
        })
        .await?
    }

    async fn prepare_cycle_report(
        &self,
        source: SharedQueryValue<TestCycle>,
    ) -> Result<PendingCycle> {
        self.storage().flush().await?;
        let view = self
            .storage()
            .view_with_limits(self.config().storage_reads.execution)
            .await?;
        let limit = view.decoded_bytes()?;
        let mut revisions = self
            .storage()
            .produce(
                bound(std::mem::size_of::<Vec<ExecutionRevision>>())?,
                |_| Ok(Vec::new()),
            )?
            .try_into_value()?;
        let mut executions = self
            .storage()
            .produce(bound(std::mem::size_of::<Vec<ExecutionSeal>>())?, |_| {
                Ok(Vec::new())
            })?
            .try_into_value()?;
        let mut appearance = None;
        for id in &source.execution_ids {
            if let Some((context, original, execution)) = self
                .execution_storage
                .original_cycle_member(id, &view)
                .await?
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
                        self.storage()
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
                self.storage(),
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
            let snapshot = inspection.snapshot(self.storage()).await?;
            let context = snapshot.context()?;
            let original = inspection.revision();
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
                    self.storage()
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
            None => self.report_appearance(limit).await?.try_into_shared()?,
        };
        let revisions = revisions.try_into_shared()?;
        let snapshot = CycleSnapshot::capture(
            self.storage(),
            view,
            source.clone(),
            revisions,
            appearance,
            &self.state_read().identity.node.node_id,
        )?;
        let report = CycleReport::prepare(self, snapshot.clone()).await?;
        let bytes = std::mem::size_of::<Mutex<PendingCycle>>()
            + 2 * std::mem::size_of::<usize>()
            + harmos::btree_map_memory::<TestCycleId, Arc<Mutex<PendingCycle>>>(1)
            + source.test_cycle_id.memory_size();
        let registry = self.storage().prepare_production(bound(bytes)?)?;
        Ok(PendingCycle {
            snapshot,
            executions: executions.try_into_shared()?,
            report,
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
pub(super) fn header_memory(cycle: &CycleView) -> usize {
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

pub(super) fn header(cycle: &CycleView, saved: Option<CycleSeal>) -> TestCycle {
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
