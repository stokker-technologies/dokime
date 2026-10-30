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
    sync::{Arc, RwLock},
};

// Third Party Imports
use harmos::{
    state::Position,
    storage::{FileReference, ObjectReceipt, ReadBatch, StorageCursor},
};
use tokio::sync::Mutex;

// Local Imports
use crate::app::StateManager;
use crate::installed_plugins::RuntimeAppearanceFrame;
use crate::report::PreparedExecutionReport;
use crate::runtime::Error;
use crate::runtime::Result;
use crate::storage::Manager;
use crate::storage::objects::execution::Execution;
use crate::storage::reports;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::execution::saved::ExecutionOutput;
use dokime_domain::model::execution::saved::ExecutionRevision;
use dokime_domain::model::execution::saved::ExecutionSeal;
use dokime_domain::model::execution::saved::validate_outputs;
use dokime_domain::model::identity::ArtifactId;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct TailStop {
    pub(crate) lease: Mutex<Option<crate::services::Lease>>,
}

pub(crate) struct TailStopPreparation {
    name: String,
}

pub(crate) struct ExecutionStorage {
    pub(crate) schema_settings: harmos::config::SchemaSettings,
    pub(crate) storage: Manager,
    pub(super) state_manager: StateManager,
    pub(super) context_epoch: SessionId,
    pub(super) brand: Arc<RwLock<Option<crate::installed_plugins::ResolvedBrand>>>,
    pub(crate) cycle_save: Arc<Mutex<()>>,
    pub(super) cycle_reports:
        Mutex<BTreeMap<dokime_domain::model::identity::TestCycleId, Arc<Mutex<super::cycle_finalization::PendingCycle>>>>,
    pub(super) active: Mutex<BTreeMap<TestExecutionId, Arc<Mutex<Acquisition>>>>,

    // Scopes Of Owners Still Admitting Evidence; Telemetry Routes Through It Without Locking Each Owner.
    pub(super) open_scopes: Arc<std::sync::Mutex<BTreeMap<TestExecutionId, dokime_domain::model::execution::RecordScope>>>,

    pub(super) codecs: crate::storage::telemetry::CodecCache,
    pub(crate) tail_stops: Mutex<BTreeMap<String, Arc<TailStop>>>,
    pub(crate) capture_failure: std::sync::Mutex<Option<Arc<Error>>>,
    pub(crate) captures: super::captures::Captures,

    #[cfg(any(test, feature = "fault-injection", feature = "testing"))]
    pub(super) log_faults: std::sync::Mutex<BTreeMap<TestExecutionId, (usize, std::io::ErrorKind)>>,
}

pub(crate) struct Acquisition {
    pub(super) storage: Manager,
    pub(super) execution_id: TestExecutionId,
    pub(super) execution: Arc<TestExecution>,
    execution_owner: ReadBatch<Arc<TestExecution>>,
    pub(super) gate: Arc<Mutex<()>>,
    materialization_gate: Arc<Mutex<()>>,
    pub(super) finish_requested: bool,
    pub(super) outputs_sealed: bool,
    pub(super) outputs_quiescent: bool,
    pub(super) artifacts: BTreeMap<ArtifactId, super::artifacts::Upload>,
    pub(super) native_outputs: BTreeMap<String, super::native_outputs::NativeOutput>,
    observed_at: Option<Position>,
    settled_outputs: Option<Arc<ReadBatch<Vec<ExecutionOutput>>>>,
    working: Option<Arc<ExecutionRevision>>,
    pending: Option<PendingRevision>,
    finalizing: Option<Arc<TestExecution>>,
    published: Option<PublishedReport>,
    pub(super) cycle_context: Option<Arc<CycleMemberContext>>,
    // The Final Report's Materialized Input, Reused By Cycle Reports Until The Cycle Retires.
    pub(super) cycle_member: Option<(FileReference, Arc<crate::report::cycle::CapturedMember>)>,
}

pub(super) struct CycleMemberContext {
    pub(super) context: FileReference,
    pub(super) appearance: RuntimeAppearanceFrame,
}

pub(crate) struct CaptureInputs {
    pub execution: Arc<TestExecution>,
    pub _execution_owner: ReadBatch<Arc<TestExecution>>,
    pub lifecycle_id: String,
    // The Revision A Lifecycle Froze, Which Anchors Its Report.
    pub anchor: Option<ExecutionRevision>,
    pub outputs: Arc<ReadBatch<Vec<ExecutionOutput>>>,
    pub unavailable_outputs: ReadBatch<Vec<String>>,
    pub through: StorageCursor,
}

struct PendingRevision {
    execution: Arc<TestExecution>,
    receipt: Option<ObjectReceipt>,
}

// A Final Report Published In Full, Waiting For Finalization To Seal Its Owner.
struct PublishedReport {
    seal: Arc<ExecutionSeal>,
    anchor: Option<ExecutionRevision>,
    appearance: RuntimeAppearanceFrame,
}

/* === Enums === */

pub(crate) enum FrozenExecution {
    Live(Arc<TestExecution>),
    Saved(harmos::storage::ReadBatch<harmos::storage::ObjectRevision<TestExecution>>),
}

/* === Traits === */

/* === Implementations === */

impl TailStopPreparation {
    pub(crate) fn into_entry(self, lease: crate::services::Lease) -> (String, Arc<TailStop>) {
        let stop = TailStop { lease: Mutex::new(Some(lease)) };
        (self.name, Arc::new(stop))
    }
}

impl Acquisition {
    pub(super) async fn settle_outputs(&mut self) -> Result<Arc<ReadBatch<Vec<ExecutionOutput>>>> {
        if !self.finish_requested || !self.outputs_sealed || !self.outputs_quiescent {
            return Err(invalid("execution output settlement requires closed producer admission"));
        }
        if let Some(outputs) = &self.settled_outputs {
            return Ok(outputs.clone());
        }
        super::artifacts::abandon_dropped(self).await?;
        if super::artifacts::has_live_upload(self) {
            return Err(Error::conflict("execution_uploads_open", "execution still has live artifact producers"));
        }
        super::artifacts::close_incomplete(self).await?;
        super::native_outputs::capture(self).await?;
        if self.artifacts.values().any(|source| source.descriptor().is_none()) {
            return Err(invalid("settled artifact descriptor is missing"));
        }
        let outputs = self.capture_outputs().await?;
        self.settled_outputs = Some(outputs.clone());
        Ok(outputs)
    }

    pub(super) async fn snapshot_outputs(&mut self) -> Result<Arc<ReadBatch<Vec<ExecutionOutput>>>> {
        super::artifacts::checkpoint(self).await?;
        super::native_outputs::snapshot(self, false).await?;
        self.capture_outputs().await
    }

    async fn capture_outputs(&self) -> Result<Arc<ReadBatch<Vec<ExecutionOutput>>>> {
        let count = self.native_outputs.len() + self.artifacts.values().filter(|source| source.descriptor().is_some()).count();
        let captured = harmos::storage::ReadBatch::new(()).try_map(|_| -> Result<_> {
            let mut captured = Vec::new();
            captured.try_reserve_exact(count).map_err(|_| invalid("captured output allocation failed"))?;
            let descriptors = self
                .native_outputs
                .values()
                .map(|source| source.descriptor())
                .chain(self.artifacts.values().filter_map(|source| source.descriptor()).map(Some));
            for output in descriptors {
                captured.push(output.ok_or_else(|| invalid("captured output descriptor is missing"))?.clone());
            }
            Ok(captured)
        })?;
        let through = self.storage.status().accepted;
        let saved = self.storage.flush_until(&through).await?;
        if !saved.covers(&through)? {
            return Err(invalid("captured output save does not cover original files"));
        }
        validate_outputs(&self.execution_id, &through, captured.as_slice()).map_err(crate::storage::Error::from)?;
        Ok(Arc::new(captured))
    }
}

impl ExecutionStorage {
    pub(crate) fn prepare_tail_stop(&self, name: &str) -> Result<TailStopPreparation> {
        Ok(TailStopPreparation { name: name.to_owned() })
    }

    pub(crate) async fn tail_names(&self, execution: Option<&TestExecutionId>) -> Result<Vec<String>> {
        let stops = self.tail_stops.lock().await;
        Ok(stops
            .keys()
            .filter(|name| {
                execution.is_none_or(|id| {
                    name.strip_prefix("execution-log:")
                        .and_then(|name| name.strip_prefix(id.as_str()))
                        .is_some_and(|suffix| suffix.starts_with(':'))
                })
            })
            .cloned()
            .collect())
    }

    pub(crate) async fn new(
        storage: Manager, state_manager: StateManager, context_epoch: SessionId,
        brand: Arc<RwLock<Option<crate::installed_plugins::ResolvedBrand>>>, schema_settings: harmos::config::SchemaSettings,
    ) -> Result<Self> {
        let captures = super::captures::Captures::new();
        Ok(Self {
            schema_settings,
            storage,
            state_manager,
            context_epoch,
            brand,
            cycle_save: Arc::new(Mutex::new(())),
            cycle_reports: Mutex::new(BTreeMap::new()),
            active: Mutex::new(BTreeMap::new()),
            open_scopes: Default::default(),
            codecs: Default::default(),
            tail_stops: Mutex::new(BTreeMap::new()),
            capture_failure: Default::default(),
            captures,
            #[cfg(any(test, feature = "fault-injection", feature = "testing"))]
            log_faults: std::sync::Mutex::new(BTreeMap::new()),
        })
    }

    pub(crate) fn capture_failure_result(&self) -> Result<()> {
        let failure = self.capture_failure.lock().expect("capture failure lock poisoned").clone();
        if let Some(source) = failure {
            return Err(Error::Harmos(harmos::Error::TaskFailed { domain: "execution file capture", source }));
        }
        Ok(())
    }

    pub(crate) async fn durable_seal(&self, id: &TestExecutionId) -> Result<Option<ReadBatch<ExecutionSeal>>> {
        let latest = super::execution::ExecutionView::open(&self.storage, id, harmos::storage::ObjectVersion::Latest).await?;
        latest.as_ref().and_then(|latest| latest.execution().saved.as_ref()).map(retain_seal).transpose()
    }

    pub(crate) async fn final_seal(&self, id: &TestExecutionId) -> Result<Option<ReadBatch<ExecutionSeal>>> {
        let Some(owner) = self.acquisition(id).await else { return Ok(None) };
        let acquisition = owner.lock().await;
        let Some(seal) = acquisition.execution.saved.as_ref() else { return Ok(None) };
        Ok(Some(retain_seal(seal)?))
    }

    // The Final Seal, Or The Seal Of A Final Report Published Ahead Of It.
    pub(crate) async fn published_seal(&self, id: &TestExecutionId) -> Result<Option<ReadBatch<ExecutionSeal>>> {
        let Some(owner) = self.acquisition(id).await else { return Ok(None) };
        let acquisition = owner.lock().await;
        let seal = acquisition.execution.saved.as_ref().or_else(|| acquisition.published.as_ref().map(|published| published.seal.as_ref()));
        let Some(seal) = seal else { return Ok(None) };
        Ok(Some(retain_seal(seal)?))
    }

    pub(crate) fn retain_seal(&self, seal: &ExecutionSeal) -> Result<ReadBatch<ExecutionSeal>> {
        retain_seal(seal)
    }

    pub(crate) async fn acquisition(&self, id: &TestExecutionId) -> Option<Arc<Mutex<Acquisition>>> {
        self.active.lock().await.get(id).cloned()
    }

    pub(super) async fn original_cycle_member(
        &self, id: &TestExecutionId, saved: &StorageCursor,
    ) -> Result<Option<(Arc<CycleMemberContext>, Arc<ExecutionRevision>, Arc<TestExecution>)>> {
        let Some(owner) = self.acquisition(id).await else {
            return Ok(None);
        };
        let acquisition = owner.lock().await;
        let Some(context) = &acquisition.cycle_context else {
            return Ok(None);
        };
        let revision = final_revision(&acquisition)?;
        if !saved.covers(&revision.position)?
            || &revision.execution_id != id
            || Some(&context.context) != acquisition.execution.saved.as_ref().map(|seal| &seal.source.context)
        {
            return Err(invalid("original cycle member differs from saved producer proof"));
        }
        Ok(Some((context.clone(), revision, acquisition.execution.clone())))
    }

    // The Materialized Input Of This Exact Sealed Report, When This Process Published It.
    pub(crate) async fn sealed_cycle_member(&self, seal: &ExecutionSeal) -> Option<Arc<crate::report::cycle::CapturedMember>> {
        let owner = self.acquisition(&seal.source.execution.execution_id).await?;
        let acquisition = owner.lock().await;
        acquisition.cycle_member.as_ref().filter(|(report, _)| *report == seal.report).map(|(_, member)| member.clone())
    }

    pub(crate) async fn published_report_view(&self, id: &TestExecutionId) -> Result<Option<super::execution::ExecutionView>> {
        let Some(owner) = self.acquisition(id).await else { return Ok(None) };
        let seal = owner.lock().await.published.as_ref().map(|report| report.seal.clone());
        let Some(seal) = seal else { return Ok(None) };
        Ok(Some(super::execution::ExecutionView::from_seal_source(&self.storage, &seal).await?))
    }

    pub(crate) async fn prepare_execution(self: &Arc<Self>, cycle: &CycleView, execution: &TestExecution) -> Result<()> {
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

    pub(crate) async fn materialization_gate(&self, execution: &TestExecutionId) -> Result<Arc<Mutex<()>>> {
        let owner = self.acquisition(execution).await.ok_or_else(|| invalid("report has no acquisition owner"))?;
        let gate = owner.lock().await.materialization_gate.clone();
        Ok(gate)
    }

    pub(crate) async fn prepare_capture(
        self: &Arc<Self>, cycle: &CycleView, execution: &TestExecution, at: &StorageCursor, final_report: bool,
    ) -> Result<CaptureInputs> {
        validate_owner(cycle, execution)?;
        if execution.saved.is_some()
            || (final_report
                && (matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running)
                    || execution.finished_at.is_none()
                    || execution.active_step_attempt.is_some()))
        {
            return Err(invalid("execution finalization requires an unsealed settled input"));
        }
        let owner = match self.acquisition(&execution.test_execution_id).await {
            Some(owner) => owner,
            None => self.acquire(cycle, execution).await?,
        };
        // The Caller's Execution Is The Frozen Input; The Owner's Copy May Lag Behind It.
        let (frozen, execution_owner) = {
            let acquisition = owner.lock().await;
            validate_identity(&acquisition, execution)?;
            if **acquisition.execution_owner.as_ref() == *execution {
                (acquisition.execution_owner.as_ref().clone(), acquisition.execution_owner.clone())
            } else {
                retained_execution(execution)?
            }
        };
        if !self.storage.cursor_saved().covers(at)? {
            return Err(invalid("report cursor is not durable"));
        }
        let intent =
            self.state_manager.state().read(|state| state.report_finalizations.get(&execution.test_execution_id).cloned()).await?.value;
        let report_boundary = intent.as_ref().map(|intent| intent.evidence_at.clone());
        let (lifecycle_id, anchor) = match intent.and_then(|intent| intent.lifecycle) {
            Some(lifecycle) => {
                self.adopt_revision(execution, &lifecycle.execution).await?;
                (lifecycle.id.to_string(), Some(lifecycle.execution))
            }
            None if final_report => {
                return Err(invalid("final report requires an immutable lifecycle intent"));
            }
            None => (uuid::Uuid::now_v7().to_string(), None),
        };
        if report_boundary.as_ref().is_some_and(|boundary| boundary != at) {
            return Err(invalid("report request differs from its persisted finalization boundary"));
        }
        let report_boundary = at.clone();
        let open_scopes = self.open_scopes.clone();
        let operation = async move {
            let gate = owner.lock().await.gate.clone();
            let _boundary = gate.lock().await;
            let mut acquisition = owner.lock().await;
            validate_identity(&acquisition, &frozen)?;
            if acquisition.published.is_some() || acquisition.execution.saved.is_some() {
                return Err(invalid("final report is already published"));
            }
            if acquisition.finalizing.as_ref().is_some_and(|original| **original != *frozen) {
                return Err(invalid("report capture changed the original finalization input"));
            }
            if final_report && acquisition.finalizing.is_none() {
                // Retain The Exact Input Before Closing Admission Or Awaiting A Save
                acquisition.finalizing = Some(frozen.clone());
                acquisition.finish_requested = true;
                open_scopes.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(&acquisition.execution_id);
            }
            let frozen =
                if final_report { acquisition.finalizing.as_ref().expect("original finalization retained").clone() } else { frozen };
            let outputs = if final_report { acquisition.settle_outputs().await? } else { acquisition.snapshot_outputs().await? };
            // A Final Report Seals Every Settled Output; Settlement May Follow The Evidence Cut.
            let pending =
                |output: &ExecutionOutput| -> harmos::Result<bool> { Ok(!final_report && !report_boundary.covers(&output.file.position)?) };
            let mut included_count = 0usize;
            let mut unavailable_count = 0usize;
            for output in outputs.iter() {
                if pending(output)? {
                    unavailable_count = unavailable_count.checked_add(1).ok_or_else(|| invalid("unavailable output count overflows"))?;
                } else {
                    included_count = included_count.checked_add(1).ok_or_else(|| invalid("included output count overflows"))?;
                }
            }
            let included = harmos::storage::ReadBatch::new(()).try_map(|_| -> Result<_> {
                let mut included = Vec::new();
                included.try_reserve_exact(included_count).map_err(|_| invalid("included output allocation failed"))?;
                for output in outputs.iter() {
                    if !pending(output)? {
                        included.push(output.clone());
                    }
                }
                Ok(included)
            })?;
            let unavailable_outputs = harmos::storage::ReadBatch::new(()).try_map(|_| -> Result<_> {
                let mut unavailable = Vec::new();
                unavailable.try_reserve_exact(unavailable_count).map_err(|_| invalid("unavailable output allocation failed"))?;
                for output in outputs.iter() {
                    if pending(output)? {
                        unavailable.push(output.relative_path.clone());
                    }
                }
                Ok(unavailable)
            })?;
            // Freeze Evidence While This Owner Still Excludes New Admissions.
            Ok::<_, Error>(CaptureInputs {
                execution: frozen,
                _execution_owner: execution_owner,
                lifecycle_id,
                anchor,
                outputs: Arc::new(included),
                unavailable_outputs,
                through: report_boundary,
            })
        };
        tokio::spawn(operation).await.map_err(|source| Error::Join { source })?
    }

    pub(crate) async fn has_report_lifecycle(&self, id: &TestExecutionId) -> Result<bool> {
        Ok(self
            .state_manager
            .state()
            .read(|state| state.report_finalizations.get(id).is_some_and(|intent| intent.lifecycle.is_some()))
            .await?
            .value)
    }

    pub(crate) async fn freeze_revision(&self, execution: &TestExecution) -> Result<Arc<ExecutionRevision>> {
        let owner =
            self.acquisition(&execution.test_execution_id).await.ok_or_else(|| invalid("report lifecycle has no acquisition owner"))?;
        if matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running)
            || execution.finished_at.is_none()
            || execution.saved.is_some()
        {
            return Err(invalid("report lifecycle must be terminal and unsealed"));
        }
        let (execution, execution_owner) = retained_execution(execution)?;
        // Saving Survives Caller Cancellation So A Started Freeze Always Settles.
        tokio::spawn(async move {
            let gate = owner.lock().await.gate.clone();
            let _boundary = gate.lock().await;
            let mut acquisition = owner.lock().await;
            acquisition.execution_owner = execution_owner;
            validate_identity(&acquisition, &execution)?;
            if acquisition.finalizing.as_ref().is_some_and(|frozen| **frozen != *execution) {
                return Err(invalid("report lifecycle differs from its frozen input"));
            }
            bind_publication_revision(&mut acquisition, &execution).await?;
            acquisition.working.clone().ok_or_else(|| invalid("report lifecycle has no saved revision"))
        })
        .await
        .map_err(|source| Error::Join { source })?
    }

    pub(crate) async fn frozen_execution(
        &self, session: &crate::storage::Manager, revision: &ExecutionRevision,
    ) -> Result<FrozenExecution> {
        if let Some(owner) = self.acquisition(&revision.execution_id).await {
            let acquisition = owner.lock().await;
            if acquisition.working.as_deref() == Some(revision) && acquisition.pending.is_none() {
                return Ok(FrozenExecution::Live(acquisition.execution.clone()));
            }
        }
        let stored = crate::storage::objects::execution::read(
            session,
            &revision.execution_id,
            harmos::storage::ObjectVersion::Exact(revision.version.clone()),
        )
        .await?
        .ok_or_else(|| invalid("report lifecycle revision is missing"))?;
        if stored.position != revision.position || stored.value.test_execution_id != revision.execution_id || stored.value.saved.is_some() {
            return Err(invalid("report lifecycle revision differs from its reference"));
        }
        Ok(FrozenExecution::Saved(stored))
    }

    pub(crate) async fn adopt_revision(&self, execution: &TestExecution, revision: &ExecutionRevision) -> Result<()> {
        let owner =
            self.acquisition(&execution.test_execution_id).await.ok_or_else(|| invalid("report lifecycle has no acquisition owner"))?;
        let gate = owner.lock().await.gate.clone();
        let _boundary = gate.lock().await;
        let mut acquisition = owner.lock().await;
        validate_identity(&acquisition, execution)?;
        if acquisition.execution.saved.is_some() {
            return Err(invalid("final report is already published"));
        }
        if acquisition.working.as_deref() == Some(revision) && *acquisition.execution == *execution && acquisition.pending.is_none() {
            return Ok(());
        }
        let stored = acquisition
            .storage
            .read_object::<Execution>(revision.execution_id.as_str(), harmos::storage::ObjectVersion::Exact(revision.version.clone()))
            .await?
            .ok_or_else(|| invalid("report lifecycle revision is missing"))?;
        let same_revision = revision.execution_id == execution.test_execution_id
            && stored.position == revision.position
            && stored.version == revision.version;
        if !same_revision || acquisition.pending.is_some() {
            return Err(invalid("capture differs from immutable lifecycle intent"));
        }

        let native = Execution::try_from(execution)?;
        if stored.value != native {
            let fields = stored.value.differing_fields(&native);
            tracing::error!(execution_id = %execution.test_execution_id, ?fields, "execution changed after its report lifecycle was frozen");
            return Err(invalid("capture differs from immutable lifecycle intent"));
        }
        drop(native);
        drop(stored);

        let (execution, execution_owner) = retained_execution(execution)?;
        acquisition.execution = execution;
        acquisition.execution_owner = execution_owner;
        acquisition.working = Some(Arc::new(revision.clone()));
        Ok(())
    }

    pub(crate) async fn publication_revision(&self, execution: &TestExecution) -> Result<Arc<ExecutionRevision>> {
        let owner = self.acquisition(&execution.test_execution_id).await.ok_or_else(|| invalid("report has no acquisition"))?;
        let gate = owner.lock().await.gate.clone();
        let _guard = gate.lock().await;
        let mut acquisition = owner.lock().await;
        validate_identity(&acquisition, execution)?;
        if acquisition.pending.is_some() || acquisition.working.is_none() {
            save_working_revision(&mut acquisition).await?;
        }
        acquisition.working.clone().ok_or_else(|| invalid("publication has no native revision"))
    }

    pub(crate) async fn materialize_final_report(
        self: &Arc<Self>, ctx: &crate::report::Scope, inspection: super::execution::ExecutionView, prepared: PreparedExecutionReport,
        sealed_at: Timestamp,
    ) -> Result<dokime_domain::model::reports::wire::MaterializedReport> {
        inspection.captured_source().ok_or_else(|| invalid("report requires an original captured source"))?;
        let storage = self.clone();
        // Spooled Before The Owner Lock; A Failed Spool Only Costs A Later Cycle Report Its Reuse.
        let member = crate::report::cycle::CapturedMember::spool(prepared.captured_input())
            .inspect_err(|error| tracing::debug!(%error, "final report input was not spooled for cycle reuse"))
            .ok()
            .map(Arc::new);
        let work = async move {
            let owner = storage
                .acquisition(&prepared.captured_input().execution_id)
                .await
                .ok_or_else(|| invalid("report has no original acquisition owner"))?;
            let gate = owner.lock().await.gate.clone();
            let _boundary = gate.lock().await;
            let mut acquisition = owner.lock().await;
            bind_publication_revision(&mut acquisition, prepared.captured_input().execution()?).await?;
            validate_report(&acquisition, &inspection, &prepared)?;
            if acquisition.published.is_some() {
                return Err(invalid("final report is already published"));
            }
            let published = publish_final(ctx, &mut acquisition, prepared, inspection, &sealed_at).await?;
            acquisition.cycle_member = member.map(|member| (published.report.clone(), member));
            Ok(published)
        };
        work.await
    }

    pub(crate) async fn published_final_report(
        &self, execution: &TestExecutionId, at: &StorageCursor,
    ) -> Result<Option<dokime_domain::model::reports::wire::MaterializedReport>> {
        let Some(owner) = self.acquisition(execution).await else {
            return Ok(None);
        };
        let acquisition = owner.lock().await;
        let Some(published) = acquisition.published.as_ref() else {
            return Ok(None);
        };
        if published.seal.source.evidence_through != *at {
            return Err(invalid("report request changes the frozen evidence cursor"));
        }
        Ok(Some(dokime_domain::model::reports::wire::MaterializedReport { report: published.seal.report.clone() }))
    }

    pub(crate) async fn seal_final_report(&self, execution: &TestExecutionId) -> Result<Arc<ExecutionRevision>> {
        let owner = self.acquisition(execution).await.ok_or_else(|| invalid("report seal has no acquisition owner"))?;
        // Saving Survives Caller Cancellation So A Started Seal Always Settles.
        tokio::spawn(async move {
            let gate = owner.lock().await.gate.clone();
            let _boundary = gate.lock().await;
            let mut acquisition = owner.lock().await;
            seal_final(&mut acquisition).await
        })
        .await
        .map_err(|source| Error::Join { source })?
    }

    pub(crate) async fn checkpoint_targets(self: &Arc<Self>, cycle: &CycleView, targets: &[TestExecutionId]) -> Result<()> {
        let maximum = 10_000;
        if cycle.test_executions.len() > maximum || targets.len() > maximum {
            return Err(invalid("execution checkpoint fanout exceeds its source bound"));
        }
        let mut members = BTreeMap::new();
        for execution in &cycle.test_executions {
            if execution.test_cycle_id != cycle.test_cycle_id || members.insert(&execution.test_execution_id, execution).is_some() {
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
            let owner = self.acquisition(id).await.ok_or_else(|| invalid("execution checkpoint has no admitted producer owner"))?;
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
                        if current.saved.is_some() || acquisition.finalizing.as_ref().is_some_and(|frozen| **frozen != *current) {
                            return Err(invalid("execution checkpoint changed its frozen ownership"));
                        }
                        retained_execution(current)
                    })
                    .await
                    .map_err(Error::Harmos)?;
                let (current, current_owner) = observed.value?;
                if *acquisition.execution != *current || acquisition.working.is_none() {
                    acquisition.execution = current;
                    acquisition.execution_owner = current_owner;
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

    pub(crate) fn with_active_membership<T>(&self, inspect: impl FnOnce(&dyn Fn(&TestExecutionId) -> bool) -> Result<T>) -> Result<T> {
        let active = self.active.try_lock().map_err(|_| {
            Error::Harmos(harmos::Error::ResourceExhausted {
                resource: "active execution inventory registry busy",
            })
        })?;
        inspect(&|id| active.contains_key(id))
    }
    pub(crate) async fn acquire(&self, cycle: &CycleView, execution: &TestExecution) -> Result<Arc<Mutex<Acquisition>>> {
        validate_owner(cycle, execution)?;
        if execution.saved.is_some() {
            return Err(invalid("sealed execution cannot acquire mutable producers"));
        }
        if let Some(owner) = self.acquisition(&execution.test_execution_id).await {
            validate_existing(&*owner.lock().await, execution)?;
            return Ok(owner);
        }
        let (frozen, execution_owner) = retained_execution(execution)?;
        let id = execution.test_execution_id.clone();
        let owner = Arc::new(Mutex::new(Acquisition {
            storage: self.storage.clone(),
            execution_id: id.clone(),
            execution: frozen,
            execution_owner,
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
            published: None,
            cycle_context: None,
            cycle_member: None,
        }));
        let existing = {
            let mut active = self.active.lock().await;
            match active.entry(id) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    self.open_scopes
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .insert(entry.key().clone(), execution.scope.clone());
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

fn retained_execution(execution: &TestExecution) -> Result<(Arc<TestExecution>, ReadBatch<Arc<TestExecution>>)> {
    let owner = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(Arc::new(execution.clone())))?;
    let value = owner.as_ref().clone();
    Ok((value, owner))
}

fn retain_seal(seal: &ExecutionSeal) -> Result<ReadBatch<ExecutionSeal>> {
    harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(seal.clone()))
}

fn validate_report(
    acquisition: &Acquisition, inspection: &super::execution::ExecutionView, prepared: &PreparedExecutionReport,
) -> Result<()> {
    let source = inspection.captured_source().ok_or_else(|| invalid("report requires the original pre-report inspection"))?;
    let captured = prepared.captured_input();
    let terminal = acquisition.finalizing.as_ref().ok_or_else(|| invalid("report has no original frozen finalization input"))?;
    let revision = acquisition.working.as_ref().ok_or_else(|| invalid("report has no saved pre-report revision"))?;
    let outputs = acquisition.settled_outputs.as_ref().ok_or_else(|| invalid("report has no settled native output closure"))?;
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
        || source.outputs.iter().any(|output| !outputs.contains(output))
        || prepared.settled_outputs() != source.outputs.as_slice()
        || captured.input.cycle.test_executions.as_slice() != std::slice::from_ref(&**terminal)
        || inspection.execution() != &**terminal
        || *acquisition.execution != **terminal
    {
        return Err(invalid("report differs from its frozen acquisition and settled closure"));
    }
    Ok(())
}

#[tracing::instrument(name = "report.publish_final", skip_all)]
async fn publish_final(
    ctx: &crate::report::Scope, acquisition: &mut Acquisition, mut prepared: PreparedExecutionReport,
    inspection: super::execution::ExecutionView, sealed_at: &Timestamp,
) -> Result<dokime_domain::model::reports::wire::MaterializedReport> {
    let revision = acquisition.working.as_ref().ok_or_else(|| invalid("publication has no real native revision"))?;
    let seal = prepared.save_dependencies(&acquisition.storage, sealed_at, revision).await?;
    let captured = prepared.captured_input();
    if seal.source.execution.execution_id != captured.execution_id
        || seal.source.owner_node_id != captured.owner_node_id
        || seal.source.evidence_through != captured.evidence_through
        || seal.source.outputs.as_slice() != prepared.settled_outputs()
    {
        return Err(invalid("saved report dependencies differ from their input"));
    }
    reports::Prepared::new(
        &acquisition.storage,
        &reports::Input {
            owner: reports::Owner::Execution(&acquisition.execution_id),
            cycle: None,
            execution: Some(&seal),
            generated_at: &seal.sealed_at,
            final_report: true,
            evidence_through: &seal.source.evidence_through,
            dependencies_saved: &seal.dependencies_saved,
            executions: std::slice::from_ref(&seal.source.execution),
            inputs: std::slice::from_ref(&seal.source.context),
            file: &seal.report,
        },
        &ctx.resources().schema_settings,
    )?
    .finish()
    .await?;
    let report = seal.report.clone();
    acquisition.published = Some(PublishedReport {
        seal: Arc::new(seal),
        anchor: inspection.captured_source().and_then(|source| source.execution.clone()),
        appearance: captured.context.appearance.clone(),
    });
    Ok(dokime_domain::model::reports::wire::MaterializedReport { report })
}

#[tracing::instrument(name = "report.seal_final", skip_all)]
async fn seal_final(acquisition: &mut Acquisition) -> Result<Arc<ExecutionRevision>> {
    if acquisition.execution.saved.is_some() {
        return complete_final_report(acquisition);
    }
    // A Reopened Cycle Rebuilds This Report From Its Captured Anchor; Only
    // Revisions Before That Anchor Become Stale Once The Seal Is Saved.
    let anchor = acquisition.published.as_ref().and_then(|published| published.anchor.clone());
    if acquisition.pending.is_none() {
        let seal = acquisition.published.as_ref().ok_or_else(|| invalid("seal requires a published final report"))?.seal.clone();
        let terminal = acquisition.finalizing.as_ref().ok_or_else(|| invalid("seal has no frozen terminal execution"))?;
        let sealed_owner = harmos::storage::ReadBatch::new(()).try_map(|_| -> Result<_> {
            let mut sealed = (**terminal).clone();
            sealed.saved = Some((*seal).clone());
            Ok(Arc::new(sealed))
        })?;
        let sealed = sealed_owner.as_ref().clone();
        acquisition.execution_owner = sealed_owner;
        // Keep The Mutable Head Unsealed Until This Exact Pending Admission Is Saved
        acquisition.pending = Some(PendingRevision { execution: sealed, receipt: None });
    }
    save_working_revision(acquisition).await?;
    if let Some(anchor) = anchor {
        release_before(&acquisition.storage, &anchor).await?;
    }
    complete_final_report(acquisition)
}

async fn release_before(storage: &Manager, anchor: &ExecutionRevision) -> Result<()> {
    use harmos::catalog::Member;
    use harmos::storage::{ObjectDefinitions, ObjectKey};

    let Some(predecessor) = anchor.position.sequence.checked_sub(1) else {
        return Ok(());
    };
    let schema = <ObjectDefinitions as Member<Execution>>::ENTRY.schema()?;
    let key_bytes =
        std::num::NonZeroUsize::new(anchor.execution_id.as_str().len()).ok_or_else(|| invalid("execution identity is empty"))?;
    let key = ObjectKey::new(anchor.execution_id.as_str(), key_bytes)?;
    let through = StorageCursor {
        history: anchor.position.history.clone(),
        sequence: predecessor,
    };
    storage.release_object_history(&schema, &key, &through).await?;
    Ok(())
}

fn complete_final_report(acquisition: &mut Acquisition) -> Result<Arc<ExecutionRevision>> {
    if acquisition.pending.is_some() {
        return Err(invalid("sealed owner still has an unresolved admission"));
    }
    let revision = final_revision(acquisition)?;
    if acquisition.cycle_context.is_none()
        && let Some(published) = &acquisition.published
    {
        let file = &acquisition.execution.saved.as_ref().expect("saved original seal").source.context;
        acquisition.cycle_context = Some(Arc::new(CycleMemberContext {
            context: file.clone(),
            appearance: published.appearance.clone(),
        }));
    }
    acquisition.published = None;
    acquisition.finalizing = None;
    acquisition.settled_outputs = None;
    Ok(revision)
}

fn final_revision(acquisition: &Acquisition) -> Result<Arc<ExecutionRevision>> {
    let revision = acquisition.working.as_ref().ok_or_else(|| invalid("sealed owner has no saved original revision"))?;
    let seal = acquisition.execution.saved.as_ref().ok_or_else(|| invalid("original final seal has not been saved"))?;
    revision.version.check_position(&revision.position)?;
    if !acquisition.storage.status().saved.covers(&revision.position)? {
        return Err(invalid("original final seal has not reached native durability"));
    }
    super::execution::check_seal_position(&revision.position, &seal.dependencies_saved)?;
    Ok(revision.clone())
}

async fn bind_publication_revision(acquisition: &mut Acquisition, execution: &TestExecution) -> Result<()> {
    if acquisition.pending.is_some() {
        save_working_revision(acquisition).await?;
    }
    if acquisition.execution.saved.is_some() {
        return Err(invalid("cannot replace a sealed execution"));
    }
    validate_identity(acquisition, execution)?;
    if *acquisition.execution != *execution || acquisition.working.is_none() {
        let (frozen, execution_owner) = retained_execution(execution)?;
        acquisition.execution = frozen;
        acquisition.execution_owner = execution_owner;
        save_working_revision(acquisition).await?;
    }
    Ok(())
}

async fn save_working_revision(acquisition: &mut Acquisition) -> Result<()> {
    if acquisition.pending.is_none() {
        acquisition.pending = Some(PendingRevision {
            execution: acquisition.execution.clone(),
            receipt: None,
        });
    }

    let pending = acquisition.pending.as_mut().expect("original execution save retained");
    if pending.receipt.is_none() {
        if acquisition.storage.status().failure.is_some() {
            acquisition.storage.flush().await?;
        }
        let native = Execution::try_from(&*pending.execution)?;
        let receipt = acquisition.storage.write_object(&native).await.inspect_err(|error| {
            tracing::error!(
                ?error,
                execution_id = %pending.execution.test_execution_id,
                sealed = pending.execution.saved.is_some(),
                "execution revision admission failed"
            );
        })?;
        let receipts = [receipt];
        pending.receipt = receipts.into_iter().next();
    }

    let receipt = pending.receipt.as_ref().expect("original execution receipt retained");
    receipt.version.check_position(&receipt.position)?;
    let saved = acquisition.storage.flush_until(&receipt.position).await?;
    if !saved.covers(&receipt.position)? {
        return Err(invalid("execution save does not cover its original admission"));
    }
    let revision = Arc::new(ExecutionRevision {
        execution_id: pending.execution.test_execution_id.clone(),
        position: receipt.position.clone(),
        version: receipt.version.clone(),
    });
    acquisition.execution = pending.execution.clone();
    acquisition.working = Some(revision);
    acquisition.pending = None;
    Ok(())
}

fn validate_owner(cycle: &CycleView, execution: &TestExecution) -> Result<()> {
    if execution.test_cycle_id != cycle.test_cycle_id
        || execution.test_case_id != execution.snapshot.case().test_case_id
        || cycle.test_executions.iter().filter(|member| member.test_execution_id == execution.test_execution_id).count() != 1
        || cycle
            .test_executions
            .iter()
            .find(|member| member.test_execution_id == execution.test_execution_id)
            .is_some_and(|member| member != execution)
        || execution.scope.test_execution_id.as_ref().is_some_and(|id| id != &execution.test_execution_id)
        || execution.scope.test_cycle_id.as_ref().is_some_and(|id| id != &execution.test_cycle_id)
        || execution.scope.test_case_id.as_ref().is_some_and(|id| id != &execution.test_case_id)
    {
        return Err(invalid("execution acquisition differs from its admitted owner"));
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
        return Err(invalid("execution identity was reused with different frozen inputs"));
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

fn invalid(reason: &'static str) -> Error {
    Error::Storage(crate::storage::Error::Invalid(reason))
}

/* === Tests === */
