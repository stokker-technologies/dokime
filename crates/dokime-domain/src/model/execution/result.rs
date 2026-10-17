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
pub use crate::model::execution::ExecutionMode;

/* === Imports === */
// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};
// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use crate::model::identity::CaseId;
use crate::model::identity::CriterionId;
use crate::model::identity::DocumentId;
use crate::model::values::MeasurementUnit;
use crate::model::identity::Metadata;
use crate::model::execution::RecordScope;
use crate::model::identity::SessionId;
use crate::model::identity::StepId;
use crate::model::identity::TestCycleId;
use crate::model::identity::TestExecutionId;
use crate::model::time::Timestamp;
use crate::model::values::Value;
use crate::model::definition::{MaterializedParameter, TestCase};
use crate::model::diagnostics::Diagnostic;
use crate::model::document::DocumentDefinition;
use crate::state::records::set::TestRecordSet;

/* === Constants === */

/* === Macros === */
product_error!(IllegalCycleTransition, formatter {
    Self { from, to } => write!(formatter, "illegal cycle transition from {from:?} to {to:?}"),
});

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestCycle {
    #[serde(default)]
    pub saved: Option<super::saved::CycleSeal>,
    pub test_cycle_id: TestCycleId,
    pub session_id: Option<SessionId>,
    pub metadata: Metadata,
    pub lifecycle: CycleStatus,
    pub created_at: Timestamp,
    pub started_at: Option<Timestamp>,
    pub updated_at: Timestamp,
    pub execution_ids: Vec<TestExecutionId>,
    pub records: TestRecordSet,
    pub continue_on_failure: bool,
    pub result: Option<ResultSummary>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CycleView {
    #[serde(default)]
    pub saved: Option<super::saved::CycleSeal>,
    pub test_cycle_id: TestCycleId,
    #[serde(default)]
    pub session_id: Option<SessionId>,
    pub metadata: Metadata,
    #[serde(default)]
    pub lifecycle: CycleStatus,
    pub created_at: Timestamp,
    pub started_at: Option<Timestamp>,
    pub updated_at: Timestamp,
    #[serde(default)]
    pub test_executions: Vec<TestExecution>,
    #[serde(default)]
    pub records: TestRecordSet,
    #[serde(default)]
    pub continue_on_failure: bool,
    pub result: Option<ResultSummary>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestExecution {
    #[serde(default)]
    pub saved: Option<super::saved::ExecutionSeal>,
    #[serde(default)]
    pub mode: ExecutionMode,
    pub test_cycle_id: TestCycleId,
    pub test_execution_id: TestExecutionId,
    pub test_case_id: CaseId,
    #[serde(default)]
    pub suite: Option<Metadata>,
    pub status: ExecutionStatus,
    pub snapshot: crate::model::execution::snapshot::ExecutionSnapshot,
    #[serde(default)]
    pub invalidation: Option<ExecutionInvalidation>,
    pub scope: RecordScope,
    #[serde(default)]
    pub output_parameters: BTreeMap<String, MaterializedParameter>,
    #[serde(default)]
    pub documents: BTreeMap<DocumentId, DocumentDefinition>,
    #[serde(default)]
    pub records: TestRecordSet,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    #[serde(default)]
    pub steps: Vec<StepRecord>,
    #[serde(default)]
    pub step_attempts: Vec<TestStepAttempt>,
    #[serde(default)]
    pub active_step_attempt: Option<StepAttemptStart>,
    #[serde(default)]
    pub capture_phase: crate::model::execution::CapturePhase,
    #[serde(default)]
    pub traceability: Vec<crate::model::traceability::TraceabilitySection>,
    pub result: Option<ResultSummary>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepAttemptStart {
    pub attempt_id: crate::model::identity::StepAttemptId,
    pub step_id: StepId,
    pub started_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestStepAttempt {
    pub attempt_id: crate::model::identity::StepAttemptId,
    pub step_id: StepId,
    pub started_at: Timestamp,
    pub finished_at: Option<Timestamp>,
    pub termination: super::TerminationReason,
    pub detail: Option<Diagnostic>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StepRecord {
    pub step_id: StepId,
    pub metadata: Metadata,
    pub status: StepStatus,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    #[serde(default)]
    pub measurements: Vec<MeasurementRecord>,
    #[serde(default)]
    pub criteria: Vec<CriterionResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Diagnostic>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeasurementRecord {
    pub metadata: Metadata,
    pub value: Value,
    pub unit: Option<MeasurementUnit>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CriterionResult {
    pub criterion_id: CriterionId,
    pub metadata: Metadata,
    pub status: CriterionStatus,
    pub observed: Option<Value>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultSummary {
    pub status: ResultStatus,
    pub passed: u32,
    pub failed: u32,
    pub blocked: u32,
    pub skipped: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IllegalCycleTransition {
    pub from: CycleStatus,
    pub to: CycleStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionInvalidation {
    pub previous: crate::model::execution::snapshot::ExecutionPluginIdentity,
    pub current: Option<crate::model::execution::snapshot::ExecutionPluginIdentity>,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CycleStatus {
    #[default]
    Ready,
    Running,
    Paused,
    Stopping,
    Completed,
    Failed,
    Blocked,
    Stopped,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestCaseStatus {
    Skipped,
    Invalidated,
    Planned,
    Queued,
    Running,
    Passed,
    Failed,
    Blocked,
    Cancelled,
    Interrupted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Skipped,
    Invalidated,
    Pending,
    Running,
    Passed,
    Failed,
    Blocked,
    Cancelled,
    Interrupted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriterionStatus {
    Pending,
    Passed,
    Failed,
    Indeterminate,
    Skipped,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultStatus {
    Pending,
    Passed,
    Failed,
    Blocked,
    Cancelled,
}

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for CycleView {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.saved),
                std::mem::size_of_val(&self.saved),
            ),
            (
                harmos::MemorySize::memory_size(&self.test_cycle_id),
                std::mem::size_of_val(&self.test_cycle_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.session_id),
                std::mem::size_of_val(&self.session_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.metadata),
                std::mem::size_of_val(&self.metadata),
            ),
            (
                harmos::MemorySize::memory_size(&self.created_at),
                std::mem::size_of_val(&self.created_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.started_at),
                std::mem::size_of_val(&self.started_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.updated_at),
                std::mem::size_of_val(&self.updated_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.test_executions),
                std::mem::size_of_val(&self.test_executions),
            ),
            (
                harmos::MemorySize::memory_size(&self.records),
                std::mem::size_of_val(&self.records),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl harmos::MemorySize for TestExecution {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.saved),
                std::mem::size_of_val(&self.saved),
            ),
            (
                harmos::MemorySize::memory_size(&self.test_cycle_id),
                std::mem::size_of_val(&self.test_cycle_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.test_execution_id),
                std::mem::size_of_val(&self.test_execution_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.test_case_id),
                std::mem::size_of_val(&self.test_case_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.suite),
                std::mem::size_of_val(&self.suite),
            ),
            (
                harmos::MemorySize::memory_size(&self.snapshot),
                std::mem::size_of_val(&self.snapshot),
            ),
            (
                harmos::MemorySize::memory_size(&self.invalidation),
                std::mem::size_of_val(&self.invalidation),
            ),
            (
                harmos::MemorySize::memory_size(&self.scope),
                std::mem::size_of_val(&self.scope),
            ),
            (
                harmos::MemorySize::memory_size(&self.output_parameters),
                std::mem::size_of_val(&self.output_parameters),
            ),
            (
                harmos::MemorySize::memory_size(&self.documents),
                std::mem::size_of_val(&self.documents),
            ),
            (
                harmos::MemorySize::memory_size(&self.records),
                std::mem::size_of_val(&self.records),
            ),
            (
                harmos::MemorySize::memory_size(&self.started_at),
                std::mem::size_of_val(&self.started_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.finished_at),
                std::mem::size_of_val(&self.finished_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.steps),
                std::mem::size_of_val(&self.steps),
            ),
            (
                harmos::MemorySize::memory_size(&self.step_attempts),
                std::mem::size_of_val(&self.step_attempts),
            ),
            (
                harmos::MemorySize::memory_size(&self.active_step_attempt),
                std::mem::size_of_val(&self.active_step_attempt),
            ),
            (
                harmos::MemorySize::memory_size(&self.traceability),
                std::mem::size_of_val(&self.traceability),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl harmos::MemorySize for StepAttemptStart {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.attempt_id),
                std::mem::size_of_val(&self.attempt_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.step_id),
                std::mem::size_of_val(&self.step_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.started_at),
                std::mem::size_of_val(&self.started_at),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl harmos::MemorySize for TestStepAttempt {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.attempt_id),
                std::mem::size_of_val(&self.attempt_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.step_id),
                std::mem::size_of_val(&self.step_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.started_at),
                std::mem::size_of_val(&self.started_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.finished_at),
                std::mem::size_of_val(&self.finished_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.detail),
                std::mem::size_of_val(&self.detail),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl harmos::MemorySize for MeasurementRecord {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.metadata),
                std::mem::size_of_val(&self.metadata),
            ),
            (
                harmos::MemorySize::memory_size(&self.value),
                std::mem::size_of_val(&self.value),
            ),
            (
                harmos::MemorySize::memory_size(&self.unit),
                std::mem::size_of_val(&self.unit),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl harmos::MemorySize for CriterionResult {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.criterion_id),
                std::mem::size_of_val(&self.criterion_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.metadata),
                std::mem::size_of_val(&self.metadata),
            ),
            (
                harmos::MemorySize::memory_size(&self.observed),
                std::mem::size_of_val(&self.observed),
            ),
            (
                harmos::MemorySize::memory_size(&self.message),
                std::mem::size_of_val(&self.message),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl harmos::MemorySize for StepRecord {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.step_id),
                std::mem::size_of_val(&self.step_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.metadata),
                std::mem::size_of_val(&self.metadata),
            ),
            (
                harmos::MemorySize::memory_size(&self.started_at),
                std::mem::size_of_val(&self.started_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.finished_at),
                std::mem::size_of_val(&self.finished_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.measurements),
                std::mem::size_of_val(&self.measurements),
            ),
            (
                harmos::MemorySize::memory_size(&self.criteria),
                std::mem::size_of_val(&self.criteria),
            ),
            (
                harmos::MemorySize::memory_size(&self.detail),
                std::mem::size_of_val(&self.detail),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl harmos::MemorySize for ExecutionInvalidation {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.previous),
                std::mem::size_of_val(&self.previous),
            ),
            (
                harmos::MemorySize::memory_size(&self.current),
                std::mem::size_of_val(&self.current),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl harmos::MemorySize for TestCycle {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.saved),
                std::mem::size_of_val(&self.saved),
            ),
            (
                harmos::MemorySize::memory_size(&self.test_cycle_id),
                std::mem::size_of_val(&self.test_cycle_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.session_id),
                std::mem::size_of_val(&self.session_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.metadata),
                std::mem::size_of_val(&self.metadata),
            ),
            (
                harmos::MemorySize::memory_size(&self.created_at),
                std::mem::size_of_val(&self.created_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.started_at),
                std::mem::size_of_val(&self.started_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.updated_at),
                std::mem::size_of_val(&self.updated_at),
            ),
            (
                harmos::MemorySize::memory_size(&self.execution_ids),
                std::mem::size_of_val(&self.execution_ids),
            ),
            (
                harmos::MemorySize::memory_size(&self.records),
                std::mem::size_of_val(&self.records),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl TestCycle {
    pub fn touch_at(&mut self, at: Timestamp) {
        self.updated_at = at;
    }
}

impl CycleView {
    pub fn cases(&self) -> impl Iterator<Item = &TestCase> {
        let mut seen = BTreeSet::new();
        self.test_executions.iter().filter_map(move |execution| {
            let case = execution.snapshot.case();
            seen.insert(&case.test_case_id).then_some(case)
        })
    }

    pub fn touch_at(&mut self, at: Timestamp) {
        self.updated_at = at;
    }
}

impl CycleStatus {
    fn allows(self, target: CycleStatus) -> bool {
        use CycleStatus::{Blocked, Completed, Failed, Paused, Ready, Running, Stopped, Stopping};
        if self == target {
            return true;
        }
        matches!(
            (self, target),
            (Completed | Failed | Blocked | Stopped, Ready)
                | (Ready | Paused, Running)
                | (Running, Paused)
                | (Running | Paused, Stopping)
                | (Ready | Running | Stopping, Stopped)
                | (Running | Stopping, Completed | Failed | Blocked)
        )
    }

    pub fn try_transition(
        self,
        target: CycleStatus,
    ) -> Result<CycleStatus, IllegalCycleTransition> {
        if self.allows(target) {
            Ok(target)
        } else {
            Err(IllegalCycleTransition {
                from: self,
                to: target,
            })
        }
    }
}

impl TestStepAttempt {
    pub fn start(&self) -> StepAttemptStart {
        StepAttemptStart {
            attempt_id: self.attempt_id.clone(),
            step_id: self.step_id.clone(),
            started_at: self.started_at.clone(),
        }
    }
}

impl TestExecution {
    pub fn capture_scope(&self) -> RecordScope {
        let mut scope = self.scope.clone();
        scope.test_cycle_id = Some(self.test_cycle_id.clone());
        scope.test_case_id = Some(self.test_case_id.clone());
        scope.test_execution_id = Some(self.test_execution_id.clone());
        scope.criterion_id = None;
        scope.step_id = self
            .active_step_attempt
            .as_ref()
            .map(|start| start.step_id.clone());
        scope.origin = self.active_step_attempt.as_ref().map_or_else(
            || self.capture_phase.into(),
            |start| crate::model::execution::EvidenceOrigin::Step {
                attempt_id: start.attempt_id.clone(),
            },
        );
        scope
    }

    pub fn validate_step_start(&self, start: &StepAttemptStart) -> Result<(), &'static str> {
        self.validate_step_origin(&start.step_id, &start.started_at)
    }

    fn validate_step_origin(
        &self,
        step_id: &StepId,
        started_at: &Timestamp,
    ) -> Result<(), &'static str> {
        started_at
            .to_datetime()
            .map_err(|_| "invalid step attempt start time")?;
        if !self
            .snapshot
            .template()
            .is_some_and(|template| template.phases.iter().any(|step| &step.step_id == step_id))
        {
            return Err("step attempt has no frozen definition");
        }
        Ok(())
    }

    pub fn validate_step_invocation(&self, start: &StepAttemptStart) -> Result<(), &'static str> {
        self.validate_step_start(start)?;
        let template = self
            .snapshot
            .template()
            .ok_or("step has no frozen template")?;
        validate_invocation(template, &self.step_attempts, &start.step_id)
    }

    pub fn effective_step_attempts(&self) -> BTreeMap<StepId, &TestStepAttempt> {
        let Some(template) = self.snapshot.template() else {
            return BTreeMap::new();
        };
        effective_attempts(
            template,
            &self.step_attempts,
            self.active_step_attempt.as_ref(),
        )
    }

    /// Present each template phase from its effective step attempt. Once an
    /// execution records attempts, the attempt ledger alone decides every
    /// template phase; an execution that reports phases without attempts keeps
    /// each explicitly recorded outcome.
    pub fn refresh_step_records(&mut self) {
        let Some(template) = self.snapshot.template() else {
            return;
        };
        let ledger = !self.step_attempts.is_empty() || self.active_step_attempt.is_some();
        let selected = self.effective_step_attempts();
        let mut steps = self.steps.clone();
        for phase in &template.phases {
            let selected = selected.get(&phase.step_id).copied();
            let active = self
                .active_step_attempt
                .as_ref()
                .filter(|start| start.step_id == phase.step_id);
            let (status, started_at, finished_at, detail) = if let Some(active) = active {
                (
                    StepStatus::Running,
                    Some(active.started_at.clone()),
                    None,
                    None,
                )
            } else if let Some(attempt) = selected {
                let status = match attempt.termination {
                    super::TerminationReason::Completed => StepStatus::Passed,
                    super::TerminationReason::Cancelled => StepStatus::Cancelled,
                    super::TerminationReason::Failed | super::TerminationReason::Interrupted => {
                        StepStatus::Failed
                    }
                };
                (
                    status,
                    Some(attempt.started_at.clone()),
                    attempt.finished_at.clone(),
                    attempt.detail.clone(),
                )
            } else if let Some(row) = steps
                .iter()
                .find(|step| step.step_id == phase.step_id)
                .filter(|row| !ledger && recorded_outcome(&row.status, self.finished_at.is_some()))
            {
                (
                    row.status.clone(),
                    row.started_at.clone(),
                    row.finished_at.clone(),
                    row.detail.clone(),
                )
            } else {
                let status = if self.finished_at.is_some() {
                    StepStatus::Skipped
                } else {
                    StepStatus::Pending
                };
                (status, None, None, None)
            };
            let row =
                if let Some(index) = steps.iter().position(|step| step.step_id == phase.step_id) {
                    &mut steps[index]
                } else {
                    steps.push(StepRecord {
                        step_id: phase.step_id.clone(),
                        metadata: phase.metadata.clone(),
                        status: StepStatus::Pending,
                        started_at: None,
                        finished_at: None,
                        measurements: Vec::new(),
                        criteria: Vec::new(),
                        detail: None,
                    });
                    steps.last_mut().expect("inserted step presentation")
                };
            if active.is_some() || row.started_at != started_at {
                row.measurements.clear();
                row.criteria.clear();
            }
            row.metadata = phase.metadata.clone();
            row.status = status;
            row.started_at = started_at;
            row.finished_at = finished_at;
            row.detail = detail;
        }
        self.steps = steps;
    }

    pub fn validate_step_attempt(&self, attempt: &TestStepAttempt) -> Result<(), &'static str> {
        self.validate_step_origin(&attempt.step_id, &attempt.started_at)?;
        if let Some(finished) = &attempt.finished_at
            && finished
                .to_datetime()
                .map_err(|_| "invalid step attempt finish time")?
                < attempt
                    .started_at
                    .to_datetime()
                    .map_err(|_| "invalid step attempt start time")?
        {
            return Err("step attempt finishes before it starts");
        }
        if (attempt.termination == super::TerminationReason::Interrupted)
            != attempt.finished_at.is_none()
        {
            return Err("step attempt completion time differs from its termination");
        }
        let diagnostic = matches!(
            attempt.termination,
            super::TerminationReason::Failed | super::TerminationReason::Interrupted
        );
        if diagnostic != attempt.detail.is_some() {
            return Err("step attempt diagnostic differs from its termination");
        }
        Ok(())
    }

    pub fn interrupt_step_attempt(&mut self) {
        let Some(start) = self.active_step_attempt.take() else {
            return;
        };
        self.step_attempts.push(TestStepAttempt {
            attempt_id: start.attempt_id,
            step_id: start.step_id,
            started_at: start.started_at,
            finished_at: None,
            termination: super::TerminationReason::Interrupted,
            detail: Some(
                Diagnostic::of("step runner ended without a completion receipt").redacted(),
            ),
        });
        self.refresh_step_records();
    }

    pub fn effective_parameters(&self) -> BTreeMap<String, MaterializedParameter> {
        let mut parameters = self.snapshot.parameters().clone();
        parameters.extend(self.output_parameters.clone());
        parameters
    }
}

/* === Public Functions === */

/* === Private Functions === */
pub fn validate_invocation(
    template: &crate::model::definition::TestCaseTemplate,
    attempts: &[TestStepAttempt],
    step_id: &StepId,
) -> Result<(), &'static str> {
    let position = template
        .phases
        .iter()
        .position(|step| step.step_id == *step_id)
        .ok_or("step has no frozen definition")?;
    let mut invoked = BTreeSet::new();
    for attempt in attempts {
        if attempt.termination != super::TerminationReason::Completed {
            return Err("a terminated step requires execution cleanup");
        }
        invoked.insert(&attempt.step_id);
    }
    if invoked.contains(step_id) && !template.phases[position].repeatable {
        return Err("step does not permit repeated invocation");
    }
    if template.phases[position + 1..]
        .iter()
        .any(|step| !step.repeatable && invoked.contains(&step.step_id))
    {
        return Err("step would invalidate an invoked non-repeatable downstream step");
    }
    Ok(())
}

// An Explicit Phase Outcome Survives Refresh: A Settled One Always, A Running
// One Until Its Execution Finishes Without Settling It.
fn recorded_outcome(status: &StepStatus, finished: bool) -> bool {
    match status {
        StepStatus::Pending => false,
        StepStatus::Running => !finished,
        _ => true,
    }
}

pub fn effective_attempts<'a>(
    template: &crate::model::definition::TestCaseTemplate,
    attempts: &'a [TestStepAttempt],
    active: Option<&StepAttemptStart>,
) -> BTreeMap<StepId, &'a TestStepAttempt> {
    let positions: BTreeMap<_, _> = template
        .phases
        .iter()
        .enumerate()
        .map(|(position, step)| (&step.step_id, position))
        .collect();
    let mut selected = BTreeMap::new();
    for attempt in attempts {
        let Some(&position) = positions.get(&attempt.step_id) else {
            continue;
        };
        selected.split_off(&position);
        selected.insert(position, attempt);
    }
    if let Some(active) = active
        && let Some(position) = positions.get(&active.step_id)
    {
        selected.split_off(position);
    }
    selected
        .into_values()
        .map(|attempt| (attempt.step_id.clone(), attempt))
        .collect()
}

/* === Tests === */
