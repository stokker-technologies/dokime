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
use std::{
    collections::BTreeMap,
    num::NonZeroUsize,
    sync::{Arc, RwLock},
};

// Third Party Imports
use harmos::{
    MemorySize,
    state::Position,
    storage::{
        FileReference, ObjectReceipt, ObjectVersionToken, QueryValue, SharedQueryValue,
        StorageCursor, StorageProductionPreparation,
    },
};
use tokio::sync::Mutex;

// Local Imports
use crate::{
    Error, Result, app::StateManager, execution::CycleView,
    installed_plugins::RuntimeAppearanceFrame, report::PreparedExecutionReport,
};
use dokime::{
    core::{ArtifactId, SessionId, TestExecutionId, Timestamp},
    execution::{
        ExecutionStatus, TestExecution,
        saved::{
            ExecutionOutput, ExecutionRevision, ExecutionSeal, output_validation_memory,
            validate_outputs,
        },
    },
    storage::{Manager, objects::execution::Execution, reports},
};

/* === Constants === */
const MAX_ACTIVE_OWNERS: usize = 10_000;

/* === Macros === */

/* === Type Aliases === */
/* === Structs === */
pub(crate) struct TailStop {
    pub(crate) lease: Mutex<Option<crate::services::Lease>>,
    _admission: StorageProductionPreparation,
}

pub(crate) struct TailStopPreparation {
    name: String,
    admission: StorageProductionPreparation,
}

pub(crate) struct ExecutionStorage {
    pub(crate) storage: Manager,
    pub(super) state_manager: StateManager,
    pub(super) context_epoch: SessionId,
    pub(super) brand: Arc<RwLock<Option<crate::installed_plugins::ResolvedBrand>>>,
    pub(crate) cycle_save: Arc<Mutex<()>>,
    pub(super) cycle_reports: Mutex<
        BTreeMap<dokime::core::TestCycleId, Arc<Mutex<super::cycle_finalization::PendingCycle>>>,
    >,
    pub(super) active: Mutex<BTreeMap<TestExecutionId, Arc<Mutex<Acquisition>>>>,
    pub(super) artifact_admission: Arc<tokio::sync::Semaphore>,
    pub(crate) tail_stops: Mutex<BTreeMap<String, Arc<TailStop>>>,
    pub(crate) captures: super::captures::Captures,
}

pub(crate) struct Acquisition {
    pub(super) storage: Manager,
    pub(super) execution_id: TestExecutionId,
    pub(super) execution: SharedQueryValue<TestExecution>,
    pub(super) gate: Arc<Mutex<()>>,
    materialization_gate: Arc<Mutex<()>>,
    pub(super) finish_requested: bool,
    pub(super) outputs_sealed: bool,
    pub(super) outputs_quiescent: bool,
    pub(super) artifacts: BTreeMap<ArtifactId, super::artifacts::Upload>,
    pub(super) native_outputs: BTreeMap<String, super::native_outputs::NativeOutput>,
    observed_at: Option<Position>,
    settled_outputs: Option<SharedQueryValue<Vec<ExecutionOutput>>>,
    working: Option<SharedQueryValue<ExecutionRevision>>,
    pending: Option<PendingRevision>,
    finalizing: Option<SharedQueryValue<TestExecution>>,
    report: Option<PendingReport>,
    pub(super) cycle_context: Option<SharedQueryValue<CycleMemberContext>>,
    _registry: StorageProductionPreparation,
}

pub(super) struct CycleMemberContext {
    pub(super) context: FileReference,
    pub(super) appearance: RuntimeAppearanceFrame,
}

pub(crate) struct CaptureInputs {
    pub execution: SharedQueryValue<TestExecution>,
    pub lifecycle_id: String,
    pub outputs: SharedQueryValue<Vec<ExecutionOutput>>,
    pub unavailable_outputs: QueryValue<Vec<String>>,
    pub through: QueryValue<StorageCursor>,
    _memory: StorageProductionPreparation,
}

struct PendingRevision {
    execution: SharedQueryValue<TestExecution>,
    receipt: Option<ObjectReceipt>,
    _admission: StorageProductionPreparation,
}

struct PendingReport {
    prepared: PreparedExecutionReport,
    inspection: super::execution::ExecutionView,
    sealed_at: QueryValue<Timestamp>,
    seal: Option<SharedQueryValue<ExecutionSeal>>,
    publication: Option<reports::Prepared>,
    _admission: StorageProductionPreparation,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl MemorySize for CycleMemberContext {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(
                self.context
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<FileReference>()),
            )
            .saturating_add(
                self.appearance
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<RuntimeAppearanceFrame>()),
            )
    }
}

impl TailStopPreparation {
    pub(crate) fn into_entry(self, lease: crate::services::Lease) -> (String, Arc<TailStop>) {
        let stop = TailStop {
            lease: Mutex::new(Some(lease)),
            _admission: self.admission,
        };
        (self.name, Arc::new(stop))
    }
}

impl Acquisition {
    pub(super) async fn settle_outputs(
        &mut self,
    ) -> Result<SharedQueryValue<Vec<ExecutionOutput>>> {
        if !self.finish_requested || !self.outputs_sealed || !self.outputs_quiescent {
            return Err(invalid(
                "execution output settlement requires closed producer admission",
            ));
        }
        if let Some(outputs) = &self.settled_outputs {
            return Ok(outputs.clone());
        }
        super::artifacts::abandon_dropped(self).await?;
        if super::artifacts::has_live_upload(self) {
            return Err(Error::conflict(
                "execution_uploads_open",
                "execution still has live artifact producers",
            ));
        }
        super::artifacts::close_incomplete(self).await?;
        super::native_outputs::capture(self).await?;
        if self
            .artifacts
            .values()
            .any(|source| source.descriptor().is_none())
        {
            return Err(invalid("settled artifact descriptor is missing"));
        }
        let outputs = self.capture_outputs().await?;
        self.settled_outputs = Some(outputs.clone());
        Ok(outputs)
    }

    pub(super) async fn snapshot_outputs(
        &mut self,
    ) -> Result<SharedQueryValue<Vec<ExecutionOutput>>> {
        super::artifacts::checkpoint(self).await?;
        super::native_outputs::snapshot(self, false).await?;
        self.capture_outputs().await
    }

    async fn capture_outputs(&self) -> Result<SharedQueryValue<Vec<ExecutionOutput>>> {
        let count = self
            .native_outputs
            .len()
            .checked_add(self.artifacts.len())
            .ok_or_else(|| invalid("execution output source count overflows"))?;
        let descriptors = || {
            self.native_outputs
                .values()
                .map(|source| source.descriptor())
                .chain(
                    self.artifacts
                        .values()
                        .filter_map(|source| source.descriptor())
                        .map(Some),
                )
        };
        let mut bytes = std::mem::size_of::<Vec<ExecutionOutput>>()
            .checked_add(
                count
                    .checked_mul(std::mem::size_of::<ExecutionOutput>())
                    .ok_or_else(|| invalid("captured output slots overflow"))?,
            )
            .ok_or_else(|| invalid("captured output memory overflows"))?;
        for output in descriptors() {
            let output = output.ok_or_else(|| invalid("captured output descriptor is missing"))?;
            let memory = output.memory_size();
            if memory == usize::MAX {
                return Err(invalid("captured output descriptor memory overflows"));
            }
            bytes = bytes
                .checked_add(memory.saturating_sub(std::mem::size_of::<ExecutionOutput>()))
                .ok_or_else(|| invalid("captured output memory overflows"))?;
        }
        // Include Every Validation Set Before Building Or Cloning The Bundle
        let scratch = harmos::btree_map_memory::<&String, ()>(count)
            .saturating_add(harmos::btree_map_memory::<&str, ()>(count))
            .saturating_add(harmos::btree_map_memory::<&ArtifactId, ()>(
                self.artifacts.len(),
            ));
        let claim = self.storage.prepare_production(allowance(
            bytes
                .checked_add(scratch)
                .ok_or_else(|| invalid("captured output validation memory overflows"))?,
        )?)?;
        let mut captured = Vec::new();
        captured.try_reserve_exact(count).map_err(|_| {
            Error::Harmos(harmos::Error::ResourceExhausted {
                resource: "captured output slots",
            })
        })?;
        if captured.capacity() > count {
            return Err(Error::Harmos(harmos::Error::ResourceExhausted {
                resource: "captured output slot capacity",
            }));
        }
        for output in descriptors() {
            captured.push(
                output
                    .expect("all descriptors checked before cloning")
                    .clone(),
            );
        }
        if output_validation_memory(&captured) > scratch {
            return Err(invalid(
                "captured output validation exceeds its original claim",
            ));
        }
        let through = self.storage.status().accepted;
        let saved = self.storage.flush_until(&through).await?;
        if !saved.covers(&through)? {
            return Err(invalid(
                "captured output save does not cover original files",
            ));
        }
        validate_outputs(&self.execution_id, &through, &captured)
            .map_err(dokime::storage::Error::from)?;
        let outputs = claim
            .finish(captured)?
            .try_into_value()?
            .try_into_shared()?;
        Ok(outputs)
    }
}

impl ExecutionStorage {
    pub(crate) fn prepare_tail_stop(&self, name: &str) -> Result<TailStopPreparation> {
        let node = harmos::btree_map_memory::<String, Arc<TailStop>>(1);
        let bytes = node
            .checked_add(std::mem::size_of::<TailStop>())
            .and_then(|bytes| bytes.checked_add(2 * std::mem::size_of::<usize>()))
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<TailStopPreparation>()))
            .and_then(|bytes| bytes.checked_add(name.len()))
            .ok_or_else(|| invalid("log tail registry memory overflows"))?;
        let admission = self.storage.prepare_production(allowance(bytes)?)?;
        let name = copy_tail_name(name)?;
        Ok(TailStopPreparation { name, admission })
    }

    pub(crate) async fn tail_names(
        &self,
        execution: Option<&TestExecutionId>,
    ) -> Result<QueryValue<Vec<String>>> {
        let stops = self.tail_stops.lock().await;
        let selected = || {
            stops.keys().filter(|name| {
                execution.is_none_or(|id| {
                    name.strip_prefix("execution-log:")
                        .and_then(|name| name.strip_prefix(id.as_str()))
                        .is_some_and(|suffix| suffix.starts_with(':'))
                })
            })
        };
        let mut count = 0usize;
        let mut bytes = std::mem::size_of::<Vec<String>>();
        for name in selected() {
            count = count
                .checked_add(1)
                .ok_or_else(|| invalid("log tail stop count overflows"))?;
            bytes = bytes
                .checked_add(std::mem::size_of::<String>())
                .and_then(|bytes| bytes.checked_add(name.len()))
                .ok_or_else(|| invalid("log tail stop batch memory overflows"))?;
        }
        let claim = self.storage.prepare_production(allowance(bytes)?)?;
        let mut names = Vec::new();
        names.try_reserve_exact(count).map_err(|_| {
            Error::Harmos(harmos::Error::ResourceExhausted {
                resource: "log tail stop slots",
            })
        })?;
        if names.capacity() > count {
            return Err(Error::Harmos(harmos::Error::ResourceExhausted {
                resource: "log tail stop slot capacity",
            }));
        }
        for name in selected() {
            names.push(copy_tail_name(name)?);
        }
        Ok(claim.finish(names)?.try_into_value()?)
    }

    pub(crate) async fn new(
        storage: Manager,
        state_manager: StateManager,
        context_epoch: SessionId,
        brand: Arc<RwLock<Option<crate::installed_plugins::ResolvedBrand>>>,
    ) -> Result<Self> {
        Ok(Self {
            storage,
            state_manager,
            context_epoch,
            brand,
            cycle_save: Arc::new(Mutex::new(())),
            cycle_reports: Mutex::new(BTreeMap::new()),
            active: Mutex::new(BTreeMap::new()),
            artifact_admission: Arc::new(tokio::sync::Semaphore::new(4)),
            tail_stops: Mutex::new(BTreeMap::new()),
            captures: Default::default(),
        })
    }

    pub(crate) async fn acquisition(
        &self,
        id: &TestExecutionId,
    ) -> Option<Arc<Mutex<Acquisition>>> {
        self.active.lock().await.get(id).cloned()
    }

    pub(super) async fn original_cycle_member(
        &self,
        id: &TestExecutionId,
        view: &harmos::storage::StorageView,
    ) -> Result<
        Option<(
            SharedQueryValue<CycleMemberContext>,
            SharedQueryValue<ExecutionRevision>,
            SharedQueryValue<TestExecution>,
        )>,
    > {
        let Some(owner) = self.acquisition(id).await else {
            return Ok(None);
        };
        let acquisition = owner.lock().await;
        let Some(context) = &acquisition.cycle_context else {
            return Ok(None);
        };
        let revision = final_revision(&acquisition)?;
        context.check_origin(&self.storage)?;
        if !view.saved()?.covers(&revision.position)?
            || &revision.execution_id != id
            || Some(&context.context)
                != acquisition
                    .execution
                    .saved
                    .as_ref()
                    .map(|seal| &seal.source.context)
        {
            return Err(invalid(
                "original cycle member differs from saved producer proof",
            ));
        }
        Ok(Some((
            context.clone(),
            revision,
            acquisition.execution.clone(),
        )))
    }

    pub(crate) async fn retained_report_view(
        &self,
        id: &TestExecutionId,
    ) -> Option<super::execution::ExecutionView> {
        let owner = self.acquisition(id).await?;
        let acquisition = owner.lock().await;
        acquisition
            .report
            .as_ref()
            .map(|report| report.inspection.clone())
    }

    pub(crate) async fn prepare_execution(
        self: &Arc<Self>,
        cycle: &CycleView,
        execution: &TestExecution,
    ) -> Result<()> {
        let owner = self.acquire(cycle, execution).await?;
        tokio::spawn(async move {
            let mut acquisition = owner.lock().await;
            if acquisition.working.is_some() && acquisition.pending.is_none() {
                return Ok(());
            }
            save_working_revision(&mut acquisition).await
        })
        .await
        .map_err(|source| Error::Join { source })?
    }

    pub(crate) async fn materialization_gate(
        &self,
        execution: &TestExecutionId,
    ) -> Result<Arc<Mutex<()>>> {
        let owner = self
            .acquisition(execution)
            .await
            .ok_or_else(|| invalid("report has no acquisition owner"))?;
        let gate = owner.lock().await.materialization_gate.clone();
        Ok(gate)
    }

    pub(crate) async fn prepare_capture(
        self: &Arc<Self>,
        cycle: &CycleView,
        execution: &TestExecution,
        at: &StorageCursor,
        final_report: bool,
    ) -> Result<CaptureInputs> {
        validate_owner(cycle, execution)?;
        if execution.saved.is_some()
            || (final_report
                && (matches!(
                    execution.status,
                    ExecutionStatus::Pending | ExecutionStatus::Running
                ) || execution.finished_at.is_none()
                    || execution.active_step_attempt.is_some()))
        {
            return Err(invalid(
                "execution finalization requires an unsealed settled input",
            ));
        }
        let owner = match self.acquisition(&execution.test_execution_id).await {
            Some(owner) => owner,
            None => self.acquire(cycle, execution).await?,
        };
        let frozen = self
            .storage
            .produce(allowance(execution.memory_size())?, |_| {
                Ok(execution.clone())
            })?
            .try_into_value()?
            .try_into_shared()?;
        let storage = self.storage.clone();
        if !self.storage.cursor_saved().covers(at)? {
            return Err(invalid("report cursor is not durable"));
        }
        let intent = self
            .state_manager
            .state()
            .read(|state| {
                state
                    .report_finalizations
                    .get(&execution.test_execution_id)
                    .cloned()
            })
            .await?
            .value;
        let report_boundary = intent.as_ref().map(|intent| intent.evidence_at.clone());
        let lifecycle_id = match intent.and_then(|intent| intent.lifecycle) {
            Some(lifecycle) if lifecycle.execution == *execution => lifecycle.id.to_string(),
            Some(_) => return Err(invalid("capture differs from immutable lifecycle intent")),
            None if final_report => {
                return Err(invalid(
                    "final report requires an immutable lifecycle intent",
                ));
            }
            None => uuid::Uuid::now_v7().to_string(),
        };
        if report_boundary
            .as_ref()
            .is_some_and(|boundary| boundary != at)
        {
            return Err(invalid(
                "report request differs from its persisted finalization boundary",
            ));
        }
        let report_boundary = at.clone();
        let operation = async move {
            let gate = owner.lock().await.gate.clone();
            let _boundary = gate.lock().await;
            let mut acquisition = owner.lock().await;
            validate_identity(&acquisition, &frozen)?;
            if acquisition.report.is_some() || acquisition.execution.saved.is_some() {
                return Err(invalid(
                    "final report is already retained; retry its original save",
                ));
            }
            if acquisition
                .finalizing
                .as_ref()
                .is_some_and(|original| **original != *frozen)
            {
                return Err(invalid(
                    "report capture changed the original finalization input",
                ));
            }
            if final_report && acquisition.finalizing.is_none() {
                // Retain The Exact Input Before Closing Admission Or Awaiting A Save
                acquisition.finalizing = Some(frozen.clone());
                acquisition.finish_requested = true;
            }
            let frozen = if final_report {
                acquisition
                    .finalizing
                    .as_ref()
                    .expect("original finalization retained")
                    .clone()
            } else {
                frozen
            };
            let outputs = if final_report {
                acquisition.settle_outputs().await?
            } else {
                acquisition.snapshot_outputs().await?
            };
            let unavailable_outputs = storage
                .produce(allowance(outputs.memory_size())?, |_| {
                    let mut unavailable = Vec::new();
                    for output in outputs.iter() {
                        if !report_boundary.covers(&output.file.position)? {
                            unavailable.push(output.relative_path.clone());
                        }
                    }
                    Ok(unavailable)
                })?
                .try_into_value()?;
            let outputs = outputs
                .try_project(allowance(outputs.memory_size())?, |outputs, _| {
                    let mut included = Vec::with_capacity(outputs.len());
                    for output in outputs {
                        if report_boundary.covers(&output.file.position)? {
                            included.push(output.clone());
                        }
                    }
                    Ok(included)
                })?
                .try_into_shared()?;
            // Freeze Evidence While This Owner Still Excludes New Admissions.
            let through = storage
                .produce(allowance(report_boundary.memory_size())?, |_| {
                    Ok(report_boundary)
                })?
                .try_into_value()?;
            let memory = storage.prepare_production(allowance(
                std::mem::size_of::<CaptureInputs>().saturating_add(lifecycle_id.capacity()),
            )?)?;
            Ok::<_, Error>(CaptureInputs {
                execution: frozen,
                lifecycle_id,
                outputs,
                unavailable_outputs,
                through,
                _memory: memory,
            })
        };
        let bytes = std::mem::size_of_val(&operation)
            .checked_add(std::mem::size_of::<StorageProductionPreparation>())
            .ok_or_else(|| invalid("capture operation memory overflows"))?;
        let memory = self.storage.prepare_production(allowance(bytes)?)?;
        tokio::spawn(async move {
            let _memory = memory;
            operation.await
        })
        .await
        .map_err(|source| Error::Join { source })?
    }

    pub(crate) async fn has_report_lifecycle(&self, id: &TestExecutionId) -> Result<bool> {
        Ok(self
            .state_manager
            .state()
            .read(|state| {
                state
                    .report_finalizations
                    .get(id)
                    .is_some_and(|intent| intent.lifecycle.is_some())
            })
            .await?
            .value)
    }

    pub(crate) async fn publication_revision(
        &self,
        execution: &TestExecution,
    ) -> Result<SharedQueryValue<ExecutionRevision>> {
        let owner = self
            .acquisition(&execution.test_execution_id)
            .await
            .ok_or_else(|| invalid("report has no acquisition"))?;
        let gate = owner.lock().await.gate.clone();
        let _guard = gate.lock().await;
        let mut acquisition = owner.lock().await;
        validate_identity(&acquisition, execution)?;
        if acquisition.pending.is_some() || acquisition.working.is_none() {
            save_working_revision(&mut acquisition).await?;
        }
        acquisition
            .working
            .clone()
            .ok_or_else(|| invalid("publication has no native revision"))
    }

    pub(crate) async fn materialize_final_report(
        self: &Arc<Self>,
        ctx: &harmos::operations::Context<'_>,
        inspection: super::execution::ExecutionView,
        prepared: PreparedExecutionReport,
        sealed_at: Timestamp,
    ) -> Result<dokime::operations::reports::MaterializedReport> {
        prepared.captured_input().check_origin(&self.storage)?;
        prepared.settled_outputs().check_origin(&self.storage)?;
        if let Ok(revision) = inspection.revision() {
            revision.check_origin(&self.storage)?;
        }
        inspection
            .captured_source()
            .ok_or_else(|| invalid("report requires an original captured source"))?
            .check_origin(&self.storage)?;
        let preparation = inspection.view().prepare_work()?;
        let storage = self.clone();
        let work = async move {
            let owner = storage
                .acquisition(&prepared.captured_input().execution_id)
                .await
                .ok_or_else(|| invalid("report has no original acquisition owner"))?;
            let gate = owner.lock().await.gate.clone();
            let _boundary = gate.lock().await;
            let mut acquisition = owner.lock().await;
            bind_publication_revision(&mut acquisition, prepared.captured_input().execution()?)
                .await?;
            let computation = inspection.view().begin_computation()?;
            validate_report(&acquisition, &inspection, &prepared)?;
            if acquisition.report.is_some() {
                return Err(invalid("report already retained; retry its original save"));
            }
            let memory = acquisition.storage.prepare_production(allowance(
                std::mem::size_of::<PendingReport>()
                    .checked_add(sealed_at.memory_size())
                    .ok_or_else(|| invalid("retained report memory overflows"))?,
            )?)?;
            let sealed_at = acquisition
                .storage
                .produce(allowance(sealed_at.memory_size())?, |_| Ok(sealed_at))?
                .try_into_value()?;
            computation.check()?;
            drop(computation);
            acquisition.report = Some(PendingReport {
                prepared,
                inspection,
                sealed_at,
                seal: None,
                publication: None,
                _admission: memory,
            });
            publish_materialized_final(ctx, &mut acquisition).await
        };
        preparation.boxed(work)?.await
    }

    pub(super) async fn has_final_report(&self, execution: &TestExecutionId) -> bool {
        let Some(owner) = self.acquisition(execution).await else {
            return false;
        };
        let acquisition = owner.lock().await;
        acquisition.report.is_some() || acquisition.execution.saved.is_some()
    }

    pub(crate) async fn retry_materialized_report(
        &self,
        ctx: &harmos::operations::Context<'_>,
        execution: &TestExecutionId,
        at: &StorageCursor,
    ) -> Result<Option<dokime::operations::reports::MaterializedReport>> {
        let Some(owner) = self.acquisition(execution).await else {
            return Ok(None);
        };
        let gate = owner.lock().await.gate.clone();
        let _guard = gate.lock().await;
        let mut acquisition = owner.lock().await;
        let Some(report) = acquisition.report.as_ref() else {
            return Ok(None);
        };
        if report.prepared.captured_input().evidence_through != *at {
            return Err(invalid("report retry changes the frozen evidence cursor"));
        }
        publish_materialized_final(ctx, &mut acquisition)
            .await
            .map(Some)
    }

    pub(crate) async fn retry_final_report(
        &self,
        execution: &TestExecutionId,
    ) -> Result<SharedQueryValue<ExecutionRevision>> {
        let owner = self
            .acquisition(execution)
            .await
            .ok_or_else(|| invalid("report retry has no original acquisition owner"))?;
        let preparation = {
            let mut acquisition = owner.lock().await;
            if acquisition.execution.saved.is_some() && acquisition.pending.is_none() {
                return complete_final_report(&mut acquisition);
            }
            let report = acquisition
                .report
                .as_ref()
                .ok_or_else(|| invalid("report retry has no retained prepared input"))?;
            report.inspection.view().prepare_work()?
        };
        let work = async move {
            let gate = owner.lock().await.gate.clone();
            let _boundary = gate.lock().await;
            let mut acquisition = owner.lock().await;
            settle_final_report(&mut acquisition).await
        };
        let work = preparation.boxed(work)?;
        tokio::spawn(work)
            .await
            .map_err(|source| Error::Join { source })?
    }

    pub(crate) async fn checkpoint_targets(
        self: &Arc<Self>,
        cycle: &CycleView,
        targets: &[TestExecutionId],
    ) -> Result<()> {
        if cycle.test_executions.len() > MAX_ACTIVE_OWNERS || targets.len() > MAX_ACTIVE_OWNERS {
            return Err(invalid(
                "execution checkpoint fanout exceeds its source bound",
            ));
        }
        let slots = harmos::btree_map_memory::<&TestExecutionId, &TestExecution>(
            cycle.test_executions.len(),
        );
        let _index = self.storage.prepare_production(allowance(slots)?)?;
        let mut members = BTreeMap::new();
        for execution in &cycle.test_executions {
            if execution.test_cycle_id != cycle.test_cycle_id
                || members
                    .insert(&execution.test_execution_id, execution)
                    .is_some()
            {
                return Err(invalid("execution checkpoint membership is inconsistent"));
            }
        }
        // Refuse Invalid Fanout Before Saving Any Selected Member
        for id in targets {
            if !members.contains_key(id) {
                return Err(invalid("execution checkpoint target is outside its cycle"));
            }
        }
        for id in targets {
            let owner = self
                .acquisition(id)
                .await
                .ok_or_else(|| invalid("execution checkpoint has no admitted producer owner"))?;
            let storage = self.clone();
            let expected = members[id];
            validate_identity(&*owner.lock().await, expected)?;
            tokio::spawn(async move {
                let mut acquisition = owner.lock().await;
                // Settle The Original Admission Before Reading Any Newer Journal State
                if acquisition.pending.is_some() {
                    save_working_revision(&mut acquisition).await?;
                }
                let observed = storage
                    .state_manager
                    .state()
                    .read(|state| {
                        let current = state
                            .executions
                            .get(&acquisition.execution_id)
                            .ok_or_else(|| invalid("execution checkpoint member is missing"))?;
                        validate_identity(&acquisition, current)?;
                        if current.saved.is_some()
                            || acquisition
                                .finalizing
                                .as_ref()
                                .is_some_and(|frozen| **frozen != *current)
                        {
                            return Err(invalid(
                                "execution checkpoint changed its frozen ownership",
                            ));
                        }
                        storage
                            .storage
                            .produce(allowance(current.memory_size())?, |_| Ok(current.clone()))?
                            .try_into_value()?
                            .try_into_shared()
                            .map_err(Error::from)
                    })
                    .await
                    .map_err(Error::Harmos)?;
                let current = observed.value?;
                if *acquisition.execution != *current || acquisition.working.is_none() {
                    acquisition.execution = current;
                    save_working_revision(&mut acquisition).await?;
                }
                acquisition.observed_at = Some(observed.position);
                Ok::<_, Error>(())
            })
            .await
            .map_err(|source| Error::Join { source })??;
        }
        Ok(())
    }

    pub(crate) fn active_ids(
        &self,
        projection: NonZeroUsize,
    ) -> Result<QueryValue<Vec<TestExecutionId>>> {
        let active = self.active.try_lock().map_err(|_| {
            Error::Harmos(harmos::Error::ResourceExhausted {
                resource: "active execution inventory registry busy",
            })
        })?;
        if active.len() > MAX_ACTIVE_OWNERS {
            return Err(invalid(
                "active execution inventory exceeds its source bound",
            ));
        }
        let slots = active
            .len()
            .checked_mul(std::mem::size_of::<TestExecutionId>())
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Vec<TestExecutionId>>()))
            .ok_or_else(|| invalid("active execution inventory memory overflows"))?;
        let bytes = active.keys().try_fold(slots, |bytes, id| {
            bytes
                .checked_add(id.as_str().len())
                .ok_or_else(|| invalid("active execution inventory memory overflows"))
        })?;
        if bytes > projection.get() {
            return Err(Error::Harmos(harmos::Error::ResourceExhausted {
                resource: "active execution inventory projection",
            }));
        }
        // Keep This Same Claim Through Slot Allocation And Every ID Copy
        let claim = self.storage.prepare_production(allowance(bytes)?)?;
        let mut ids = Vec::new();
        ids.try_reserve_exact(active.len()).map_err(|_| {
            Error::Harmos(harmos::Error::ResourceExhausted {
                resource: "active execution inventory slots",
            })
        })?;
        if ids.capacity() > active.len() {
            return Err(Error::Harmos(harmos::Error::ResourceExhausted {
                resource: "active execution inventory slot capacity",
            }));
        }
        ids.extend(active.keys().cloned());
        drop(active);
        Ok(claim.finish(ids)?.try_into_value()?)
    }

    pub(crate) async fn acquire(
        &self,
        cycle: &CycleView,
        execution: &TestExecution,
    ) -> Result<Arc<Mutex<Acquisition>>> {
        validate_owner(cycle, execution)?;
        if execution.saved.is_some() {
            return Err(invalid("sealed execution cannot acquire mutable producers"));
        }
        if let Some(owner) = self.acquisition(&execution.test_execution_id).await {
            validate_existing(&*owner.lock().await, execution)?;
            return Ok(owner);
        }
        // Claim The Frozen Graph Before Cloning Any Nested Field
        let frozen = self
            .storage
            .produce(allowance(execution.memory_size())?, |_| {
                Ok(execution.clone())
            })?
            .try_into_value()?
            .try_into_shared()?;
        let registry_bytes =
            harmos::btree_map_memory::<TestExecutionId, Arc<Mutex<Acquisition>>>(1)
                .checked_add(std::mem::size_of::<Mutex<Acquisition>>())
                .and_then(|bytes| bytes.checked_add(2 * std::mem::size_of::<usize>()))
                .and_then(|bytes| bytes.checked_add(2 * std::mem::size_of::<Mutex<()>>()))
                .and_then(|bytes| bytes.checked_add(4 * std::mem::size_of::<usize>()))
                .and_then(|bytes| bytes.checked_add(execution.test_execution_id.as_str().len()))
                .and_then(|bytes| bytes.checked_add(execution.test_execution_id.as_str().len()))
                .ok_or_else(|| invalid("acquisition registry memory overflows"))?;
        let registry = self
            .storage
            .prepare_production(allowance(registry_bytes)?)?;
        let id = execution.test_execution_id.clone();
        let owner = Arc::new(Mutex::new(Acquisition {
            storage: self.storage.clone(),
            execution_id: id.clone(),
            execution: frozen,
            gate: Arc::new(Mutex::new(())),
            materialization_gate: Arc::new(Mutex::new(())),
            finish_requested: false,
            outputs_sealed: false,
            outputs_quiescent: false,
            artifacts: BTreeMap::new(),
            native_outputs: BTreeMap::new(),
            observed_at: None,
            settled_outputs: None,
            working: None,
            pending: None,
            finalizing: None,
            report: None,
            cycle_context: None,
            _registry: registry,
        }));
        let existing = {
            let mut active = self.active.lock().await;
            match active.entry(id) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(owner.clone());
                    None
                }
                std::collections::btree_map::Entry::Occupied(entry) => Some(entry.get().clone()),
            }
        };
        if let Some(existing) = existing {
            validate_existing(&*existing.lock().await, execution)?;
            return Ok(existing);
        }
        Ok(owner)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn validate_report(
    acquisition: &Acquisition,
    inspection: &super::execution::ExecutionView,
    prepared: &PreparedExecutionReport,
) -> Result<()> {
    let source = inspection
        .captured_source()
        .ok_or_else(|| invalid("report requires the original pre-report inspection"))?;
    let captured = prepared.captured_input();
    let terminal = acquisition
        .finalizing
        .as_ref()
        .ok_or_else(|| invalid("report has no original frozen finalization input"))?;
    let revision = acquisition
        .working
        .as_ref()
        .ok_or_else(|| invalid("report has no saved pre-report revision"))?;
    let outputs = acquisition
        .settled_outputs
        .as_ref()
        .ok_or_else(|| invalid("report has no settled native output closure"))?;
    if !acquisition.finish_requested
        || !acquisition.outputs_sealed
        || !acquisition.outputs_quiescent
        || acquisition.execution.saved.is_some()
        || acquisition.pending.is_some()
        || captured.execution != source.execution
        || captured.execution_id != revision.execution_id
        || captured.lifecycle_id != source.lifecycle_id
        || captured.owner_node_id != source.owner_node_id
        || captured.evidence_through != source.evidence_through
        || captured.context != source.context
        || captured.telemetry_contracts != source.telemetry_contracts
        || captured.record_contracts != source.record_contracts
        || source
            .outputs
            .iter()
            .any(|output| !outputs.contains(output))
        || prepared.settled_outputs().as_slice() != source.outputs.as_slice()
        || captured.input.cycle.test_executions.as_slice() != std::slice::from_ref(&**terminal)
        || inspection.execution() != &**terminal
        || *acquisition.execution != **terminal
    {
        return Err(invalid(
            "report differs from its frozen acquisition and settled closure",
        ));
    }
    Ok(())
}

async fn publish_materialized_final(
    ctx: &harmos::operations::Context<'_>,
    acquisition: &mut Acquisition,
) -> Result<dokime::operations::reports::MaterializedReport> {
    if let Some(seal) = &acquisition.execution.saved {
        return Ok(dokime::operations::reports::MaterializedReport {
            report: seal.report.clone(),
        });
    }
    if acquisition
        .report
        .as_ref()
        .is_none_or(|report| report.seal.is_none())
    {
        let report = acquisition
            .report
            .as_mut()
            .ok_or_else(|| invalid("finalization has no retained report"))?;
        let revision = acquisition
            .working
            .as_ref()
            .ok_or_else(|| invalid("publication has no real native revision"))?;
        let seal = report
            .prepared
            .save_dependencies(ctx, &report.sealed_at, revision)
            .await?;
        seal.check_origin(&acquisition.storage)?;
        let computation = report.inspection.view().begin_computation()?;
        if seal.source.execution.execution_id != report.prepared.captured_input().execution_id
            || seal.source.owner_node_id != report.prepared.captured_input().owner_node_id
            || seal.source.evidence_through != report.prepared.captured_input().evidence_through
            || seal.source.outputs.as_slice() != report.prepared.settled_outputs().as_slice()
        {
            return Err(invalid(
                "saved report dependencies differ from retained input",
            ));
        }
        report.seal = Some(seal.try_into_shared()?);
        computation.check()?;
    }
    let report = acquisition
        .report
        .as_mut()
        .expect("original report retained");
    if report.publication.is_none() {
        let seal = report
            .seal
            .as_ref()
            .expect("original dependency seal retained");
        report.publication = Some(reports::Prepared::new(
            &acquisition.storage,
            &reports::Input {
                owner: reports::Owner::Execution(&acquisition.execution_id),
                cycle: None,
                execution: Some(seal),
                generated_at: &seal.sealed_at,
                final_report: true,
                evidence_through: &seal.source.evidence_through,
                dependencies_saved: &seal.dependencies_saved,
                executions: std::slice::from_ref(&seal.source.execution),
                inputs: std::slice::from_ref(&seal.source.context),
                file: &seal.report,
            },
        )?);
    }
    report
        .publication
        .as_mut()
        .expect("original report publication retained")
        .finish()
        .await?;

    let seal = acquisition
        .report
        .as_ref()
        .and_then(|report| report.seal.as_ref())
        .ok_or_else(|| invalid("published report has no retained dependencies"))?;
    Ok(dokime::operations::reports::MaterializedReport {
        report: seal.report.clone(),
    })
}

async fn settle_final_report(
    acquisition: &mut Acquisition,
) -> Result<SharedQueryValue<ExecutionRevision>> {
    if acquisition.execution.saved.is_some() {
        return complete_final_report(acquisition);
    }
    let report = acquisition
        .report
        .as_mut()
        .ok_or_else(|| invalid("finalization has no retained report"))?;
    if report.seal.is_none() {
        return Err(invalid("materialize report dependencies before sealing"));
    }
    report
        .publication
        .as_mut()
        .ok_or_else(|| invalid("materialize report publication before sealing"))?
        .finish()
        .await?;
    if acquisition.pending.is_none() {
        let report = acquisition
            .report
            .as_ref()
            .expect("original report retained");
        let seal = report
            .seal
            .as_ref()
            .expect("original dependency seal retained");
        let terminal = acquisition
            .finalizing
            .as_ref()
            .expect("original terminal retained");
        let bytes = terminal
            .memory_size()
            .checked_add(seal.memory_size())
            .ok_or_else(|| invalid("final execution construction memory overflows"))?;
        let sealed = report
            .inspection
            .view()
            .project_shared(terminal, allowance(bytes)?, |terminal, _| {
                let mut execution = terminal.clone();
                execution.saved = Some((**seal).clone());
                Ok(execution)
            })?
            .try_into_shared()?;
        // Keep The Mutable Head Unsealed Until This Exact Pending Admission Is Saved
        acquisition.pending = Some(pending_revision(&acquisition.storage, sealed)?);
    }
    save_working_revision(acquisition).await?;
    complete_final_report(acquisition)
}

fn complete_final_report(
    acquisition: &mut Acquisition,
) -> Result<SharedQueryValue<ExecutionRevision>> {
    if acquisition.pending.is_some() {
        return Err(invalid("sealed owner still has an unresolved admission"));
    }
    let revision = final_revision(acquisition)?;
    if acquisition.cycle_context.is_none() && acquisition.report.is_some() {
        let report = acquisition
            .report
            .as_ref()
            .ok_or_else(|| invalid("sealed member has no original report input"))?;
        let computation = report.inspection.view().begin_computation()?;
        let captured = report.prepared.captured_input();
        let file = &acquisition
            .execution
            .saved
            .as_ref()
            .expect("saved original seal")
            .source
            .context;
        let bytes = file
            .memory_size()
            .saturating_add(captured.context.appearance.memory_size())
            .saturating_add(std::mem::size_of::<CycleMemberContext>());
        acquisition.cycle_context = Some(
            acquisition
                .storage
                .produce(allowance(bytes)?, |_| {
                    Ok(CycleMemberContext {
                        context: file.clone(),
                        appearance: captured.context.appearance.clone(),
                    })
                })?
                .try_into_value()?
                .try_into_shared()?,
        );
        computation.check()?;
    }
    acquisition.report = None;
    acquisition.finalizing = None;
    acquisition.settled_outputs = None;
    Ok(revision)
}

fn final_revision(acquisition: &Acquisition) -> Result<SharedQueryValue<ExecutionRevision>> {
    let revision = acquisition
        .working
        .as_ref()
        .ok_or_else(|| invalid("sealed owner has no saved original revision"))?;
    let seal = acquisition
        .execution
        .saved
        .as_ref()
        .ok_or_else(|| invalid("original final seal has not been saved"))?;
    revision.version.check_position(&revision.position)?;
    if !acquisition
        .storage
        .status()
        .saved
        .covers(&revision.position)?
    {
        return Err(invalid(
            "original final seal has not reached native durability",
        ));
    }
    super::execution::check_seal_position(&revision.position, &seal.dependencies_saved)?;
    Ok(revision.clone())
}

fn copy_tail_name(source: &str) -> Result<String> {
    let mut name = String::new();
    name.try_reserve_exact(source.len()).map_err(|_| {
        Error::Harmos(harmos::Error::ResourceExhausted {
            resource: "log tail name",
        })
    })?;
    if name.capacity() > source.len() {
        return Err(Error::Harmos(harmos::Error::ResourceExhausted {
            resource: "log tail name capacity",
        }));
    }
    name.push_str(source);
    Ok(name)
}

async fn bind_publication_revision(
    acquisition: &mut Acquisition,
    execution: &TestExecution,
) -> Result<()> {
    if acquisition.pending.is_some() {
        save_working_revision(acquisition).await?;
    }
    if acquisition.execution.saved.is_some() {
        return Err(invalid("cannot replace a sealed execution"));
    }
    if *acquisition.execution != *execution || acquisition.working.is_none() {
        acquisition.execution = acquisition
            .storage
            .produce(allowance(execution.memory_size())?, |_| {
                Ok(execution.clone())
            })?
            .try_into_value()?
            .try_into_shared()?;
        save_working_revision(acquisition).await?;
    }
    Ok(())
}

async fn save_working_revision(acquisition: &mut Acquisition) -> Result<()> {
    if acquisition.pending.is_none() {
        acquisition.pending = Some(pending_revision(
            &acquisition.storage,
            acquisition.execution.clone(),
        )?);
    }
    let pending = acquisition
        .pending
        .as_mut()
        .expect("original execution save retained");
    if pending.receipt.is_none() {
        if acquisition.storage.status().failure.is_some() {
            acquisition.storage.flush().await?;
        }
        let bytes = Execution::encoding_memory(&pending.execution)?;
        let encode = |execution: &TestExecution| {
            Execution::try_from(execution).map_err(|source| {
                harmos::Error::backend(
                    "encode native execution",
                    harmos::BackendFailure::Permanent,
                    source,
                )
            })
        };
        let native = match &acquisition.report {
            Some(report) => report.inspection.view().project_shared(
                &pending.execution,
                allowance(bytes)?,
                |execution, _| encode(execution),
            )?,
            None => acquisition
                .storage
                .produce(allowance(bytes)?, |_| encode(&pending.execution))?
                .try_into_value()?,
        };
        // Preserve Admission Before Any Fallible Flush Or Receipt Projection
        // Bound This One Revision Instead Of Reserving A Maximum-Sized Group.
        // The Original Capture Allowance Also Covers Descriptor And Receipt Work.
        let admission = bytes
            .checked_add(4 * 1024 * 1024)
            .ok_or_else(|| invalid("execution admission memory overflows"))?;
        let receipts = acquisition
            .storage
            .write_object_group(allowance(admission)?, NonZeroUsize::MIN, |group| {
                group.write(&*native)
            })
            .await
            .inspect_err(|error| {
                tracing::error!(
                    ?error,
                    execution_id = %pending.execution.test_execution_id,
                    encoding_memory = bytes,
                    sealed = pending.execution.saved.is_some(),
                    "execution revision admission failed"
                );
            })?;
        pending.receipt = receipts.into_iter().next();
    }
    let receipt = pending
        .receipt
        .as_ref()
        .expect("original execution receipt retained");
    receipt.version.check_position(&receipt.position)?;
    let saved = acquisition.storage.flush_until(&receipt.position).await?;
    if !saved.covers(&receipt.position)? {
        return Err(invalid(
            "execution save does not cover its original admission",
        ));
    }
    let bytes = std::mem::size_of::<ExecutionRevision>()
        .checked_add(pending.execution.test_execution_id.as_str().len())
        .and_then(|bytes| {
            bytes.checked_add(
                receipt
                    .position
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<StorageCursor>()),
            )
        })
        .and_then(|bytes| {
            bytes.checked_add(
                receipt
                    .version
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<ObjectVersionToken>()),
            )
        })
        .ok_or_else(|| invalid("execution revision memory overflows"))?;
    let revision = acquisition
        .storage
        .produce(allowance(bytes)?, |_| {
            Ok(ExecutionRevision {
                execution_id: pending.execution.test_execution_id.clone(),
                position: receipt.position.clone(),
                version: receipt.version.clone(),
            })
        })?
        .try_into_value()?
        .try_into_shared()?;
    acquisition.execution = pending.execution.clone();
    acquisition.working = Some(revision);
    acquisition.pending = None;
    Ok(())
}

fn pending_revision(
    storage: &Manager,
    execution: SharedQueryValue<TestExecution>,
) -> Result<PendingRevision> {
    let cursor = storage.status().accepted;
    let history = cursor
        .memory_size()
        .checked_sub(std::mem::size_of::<StorageCursor>())
        .ok_or_else(|| invalid("execution receipt history memory overflows"))?;
    let bytes = std::mem::size_of::<PendingRevision>()
        .checked_add(history)
        .and_then(|bytes| bytes.checked_add(history))
        .ok_or_else(|| invalid("execution receipt memory overflows"))?;
    Ok(PendingRevision {
        execution,
        receipt: None,
        _admission: storage.prepare_production(allowance(bytes)?)?,
    })
}

fn validate_owner(cycle: &CycleView, execution: &TestExecution) -> Result<()> {
    if execution.test_cycle_id != cycle.test_cycle_id
        || execution.test_case_id != execution.snapshot.case().test_case_id
        || cycle
            .test_executions
            .iter()
            .filter(|member| member.test_execution_id == execution.test_execution_id)
            .count()
            != 1
        || cycle
            .test_executions
            .iter()
            .find(|member| member.test_execution_id == execution.test_execution_id)
            .is_some_and(|member| member != execution)
        || execution
            .scope
            .test_execution_id
            .as_ref()
            .is_some_and(|id| id != &execution.test_execution_id)
        || execution
            .scope
            .test_cycle_id
            .as_ref()
            .is_some_and(|id| id != &execution.test_cycle_id)
        || execution
            .scope
            .test_case_id
            .as_ref()
            .is_some_and(|id| id != &execution.test_case_id)
    {
        return Err(invalid(
            "execution acquisition differs from its admitted owner",
        ));
    }
    Ok(())
}

fn validate_identity(owner: &Acquisition, execution: &TestExecution) -> Result<()> {
    if owner.execution_id != execution.test_execution_id
        || owner.execution.test_execution_id != execution.test_execution_id
        || owner.execution.test_cycle_id != execution.test_cycle_id
        || owner.execution.test_case_id != execution.test_case_id
        || owner.execution.snapshot != execution.snapshot
        || owner.execution.scope != execution.scope
    {
        return Err(invalid(
            "execution identity was reused with different frozen inputs",
        ));
    }
    Ok(())
}

fn validate_existing(owner: &Acquisition, execution: &TestExecution) -> Result<()> {
    validate_identity(owner, execution)?;
    if owner.finish_requested || owner.execution.saved.is_some() {
        return Err(invalid("execution no longer accepts mutable producers"));
    }
    Ok(())
}

fn allowance(bytes: usize) -> Result<NonZeroUsize> {
    if bytes == usize::MAX {
        return Err(invalid("execution acquisition memory overflows"));
    }
    NonZeroUsize::new(bytes).ok_or_else(|| invalid("empty execution acquisition allowance"))
}

fn invalid(reason: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(reason))
}

/* === Tests === */
