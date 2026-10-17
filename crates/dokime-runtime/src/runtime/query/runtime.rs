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
use std::{collections::BTreeMap, sync::Arc};
// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use crate::RuntimeHandle;
use crate::definition::{MaterializedParameter, ParameterSource, TestCase};
use crate::evidence_runtime::EvidenceCounts;
use crate::installed_plugins::InstalledPluginErrorRecord;
use crate::queries::RuntimeCursor;
use crate::runtime::progress::{OperationSnapshot, OperationWatch};
use crate::state::RuntimeState;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::CycleStatus;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::ResultSummary;
use dokime_domain::model::execution::result::StepRecord;
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::DokimeIdentity;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeOverviewSnapshot {
    pub revision: u64,
    pub shutting_down: bool,
    pub session_id: SessionId,
    pub identity: DokimeIdentity,
    pub hub_metadata: Metadata,
    pub test_suite_count: usize,
    pub cycle_summaries: Vec<RuntimeCycleSummary>,
    #[serde(skip)]
    pub cycles: Vec<CycleView>,
    pub evidence_counts: EvidenceCounts,
    pub installed_plugin_errors: Vec<InstalledPluginErrorRecord>,
    #[serde(skip)]
    pub retention: RuntimeRetentionDiagnostics,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeRetentionDiagnostics {
    pub state_cycles: usize,
    pub state_executions_total: usize,
    pub state_record_count: u64,
    pub telemetry_cursor_entries: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeStateNotice {
    pub cursor: RuntimeCursor,
}

#[derive(Clone)]
pub struct RuntimeStateSnapshot {
    state: Arc<RuntimeState>,
    revision: u64,
}

pub struct RuntimeStateWatch {
    receiver: tokio::sync::watch::Receiver<RuntimeStateSnapshot>,
    operations: OperationWatch,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeCycleSummary {
    pub test_cycle_id: TestCycleId,
    pub session_id: Option<SessionId>,
    pub metadata: Metadata,
    pub lifecycle: CycleStatus,
    pub created_at: Timestamp,
    pub started_at: Option<Timestamp>,
    pub updated_at: Timestamp,
    pub test_cases: Vec<TestCase>,
    pub test_executions: Vec<RuntimeExecutionSummary>,
    pub continue_on_failure: bool,
    pub result: Option<ResultSummary>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeExecutionSummary {
    pub test_execution_id: TestExecutionId,
    pub test_case_id: CaseId,
    pub status: ExecutionStatus,
    pub invalidation: Option<dokime_domain::model::execution::result::ExecutionInvalidation>,
    pub scope: RecordScope,
    pub parameters: BTreeMap<String, MaterializedParameter>,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub steps: Vec<StepRecord>,
    pub result: Option<ResultSummary>,
}

/* === Enums === */
/// What changed since a [`RuntimeStateWatch`] last answered.
pub enum RuntimeChange {
    /// Runtime state committed a new revision.
    State(RuntimeStateSnapshot),
    /// Only the operation list changed; state is where it was.
    Operations(OperationSnapshot),
}

/* === Traits === */

/* === Implementations === */
impl RuntimeStateSnapshot {
    pub(crate) fn new(state: Arc<RuntimeState>, revision: u64) -> Self {
        Self { state, revision }
    }

    pub fn state(&self) -> &RuntimeState {
        &self.state
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
}

impl RuntimeStateWatch {
    pub fn latest(&self) -> RuntimeStateSnapshot {
        self.receiver.borrow().clone()
    }

    pub async fn next(&mut self) -> Option<RuntimeStateSnapshot> {
        self.receiver.changed().await.ok()?;
        Some(self.receiver.borrow_and_update().clone())
    }

    /// The operations the runtime is performing now.
    pub fn operations(&self) -> OperationSnapshot {
        self.operations.latest()
    }

    /// The next change of state or of the operation list. State wins when
    /// both changed, so a committed revision is never delayed by progress.
    pub async fn next_change(&mut self) -> Option<RuntimeChange> {
        tokio::select! {
            biased;
            changed = self.receiver.changed() => {
                changed.ok()?;
                Some(RuntimeChange::State(self.receiver.borrow_and_update().clone()))
            }
            Some(operations) = self.operations.next() => Some(RuntimeChange::Operations(operations)),
        }
    }

    pub async fn wait_until(&mut self, predicate: impl Fn(&RuntimeStateSnapshot) -> bool) -> Option<RuntimeStateSnapshot> {
        self.receiver.wait_for(|snapshot| predicate(snapshot)).await.ok().map(|snapshot| snapshot.clone())
    }
}

impl RuntimeCycleSummary {
    pub fn from_cycle(cycle: &CycleView) -> Self {
        Self {
            test_cycle_id: cycle.test_cycle_id.clone(),
            session_id: cycle.session_id.clone(),
            metadata: cycle.metadata.clone(),
            lifecycle: cycle.lifecycle,
            created_at: cycle.created_at.clone(),
            started_at: cycle.started_at.clone(),
            updated_at: cycle.updated_at.clone(),
            test_cases: cycle.cases().cloned().collect::<Vec<_>>(),
            test_executions: cycle.test_executions.iter().map(RuntimeExecutionSummary::from_execution).collect(),
            continue_on_failure: cycle.continue_on_failure,
            result: cycle.result.clone(),
        }
    }
}

impl RuntimeExecutionSummary {
    fn from_execution(execution: &dokime_domain::model::execution::result::TestExecution) -> Self {
        Self {
            test_execution_id: execution.test_execution_id.clone(),
            test_case_id: execution.test_case_id.clone(),
            status: execution.status.clone(),
            invalidation: execution.invalidation.clone(),
            scope: execution.scope.clone(),
            parameters: public_parameters(&execution.effective_parameters()),
            started_at: execution.started_at.clone(),
            finished_at: execution.finished_at.clone(),
            steps: execution.steps.clone(),
            result: execution.result.clone(),
        }
    }
}

impl RuntimeHandle {
    pub async fn runtime_overview(&self) -> crate::runtime::Result<RuntimeOverviewSnapshot> {
        let artifact_count = self.execution_storage.artifact_count().await?;
        let telemetry_cursor_entries = self.telemetry_cursors.lock().unwrap_or_else(|error| error.into_inner()).len();
        let state = self.state_read();
        // Product revisions advance with transactions. Records share Harmos's
        // ordered history but do not mutate this state image, so report the
        // exact position paired with the cached projection rather than a later
        // record-only journal frontier.
        let revision = self.state_watch.borrow().revision();
        let cycles = state.cycle_views().collect();
        let evidence_counts = crate::runtime_state::evidence_counts(&state, artifact_count);
        let config = self.config();
        Ok(crate::runtime_state::runtime_overview_snapshot(&state, &config, revision, cycles, evidence_counts, telemetry_cursor_entries))
    }

    pub fn watch_state(&self) -> RuntimeStateWatch {
        RuntimeStateWatch { receiver: self.state_watch.subscribe(), operations: self.operation_progress.watch() }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn public_parameters(parameters: &BTreeMap<String, MaterializedParameter>) -> BTreeMap<String, MaterializedParameter> {
    parameters
        .iter()
        .filter(|(_, parameter)| {
            matches!(
                parameter.source,
                ParameterSource::Definition
                    | ParameterSource::Traceability
                    | ParameterSource::Environment
                    | ParameterSource::Resource
                    | ParameterSource::Operator
                    | ParameterSource::Runtime
                    | ParameterSource::Telemetry
                    | ParameterSource::Record
                    | ParameterSource::Output
            )
        })
        .map(|(key, parameter)| (key.clone(), parameter.clone()))
        .collect()
}

/* === Tests === */
