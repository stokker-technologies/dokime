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
use std::collections::BTreeMap;
// Third Party Imports
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};
// Local Imports
use crate::records::BuiltinRecordType;
use crate::records::RecordSeverity;
use crate::transactions::prompts::{PromptSettlement, SettledPrompts, commit_settled_prompts, settle_prompts};
use crate::transactions::records::draft::BuiltinDraft;
use crate::transactions::records::draft::Drafts;
use crate::{runtime::Error, state::RuntimeState};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::CriterionStatus;
use dokime_domain::model::execution::result::CycleStatus;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::ResultStatus;
use dokime_domain::model::execution::result::ResultSummary;
use dokime_domain::model::execution::result::StepStatus;
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::transaction(id = "dokime.execution.cycle.policy", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateCyclePolicy {
    pub test_cycle_id: TestCycleId,
    pub continue_automatically: bool,
    pub continue_on_failure: bool,
    pub occurred_at: Timestamp,
}

#[harmos::transaction(id = "dokime.execution.cycle.control", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ControlCycle {
    pub test_cycle_id: TestCycleId,
    pub action: CycleControlAction,
    pub occurred_at: Timestamp,
}

#[harmos::transaction(id = "dokime.execution.cycle.reset", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResetCycle {
    pub test_cycle_id: TestCycleId,
    pub action: CycleResetAction,
    pub occurred_at: Timestamp,
}

struct CycleControlPlan {
    cycle: CycleView,
    records: Drafts,
    settled: SettledPrompts,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CycleControlAction {
    Pause,
    Resume,
    Stop,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum CycleResetAction {
    Delete { test_case_id: CaseId },
}

/* === Traits === */

/* === Implementations === */
impl Transaction<RuntimeState> for UpdateCyclePolicy {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        if state.shutting_down {
            return Err("runtime is shutting down".to_owned());
        }
        state.cycles.get(&self.test_cycle_id).ok_or("cycle not found")?.ensure_active().map_err(str::to_owned)
    }

    fn apply(&self, state: &mut RuntimeState) {
        let cycle = state.cycles.get_mut(&self.test_cycle_id).expect("Harmos checked the cycle before applying its policy");
        cycle.continue_automatically = self.continue_automatically;
        cycle.continue_on_failure = self.continue_on_failure;
        cycle.touch_at(self.occurred_at.clone());
    }
}

impl CycleControlPlan {
    fn new(state: &RuntimeState, control: &ControlCycle, cycle: CycleView, mut records: Drafts) -> Self {
        // A stopped cycle answers nothing: its open prompts settle with the stop
        // itself, so a provider blocked on one can wind down.
        let settled = if matches!(control.action, CycleControlAction::Stop) {
            settle_prompts(state, &mut records, PromptSettlement::Stopped, &control.occurred_at, |scope| {
                scope.test_cycle_id.as_ref() == Some(&control.test_cycle_id)
            })
        } else {
            SettledPrompts::default()
        };
        Self { cycle, records, settled }
    }
}

impl Transaction<RuntimeState> for ControlCycle {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        ensure_runtime_active(state)?;
        prepare_cycle_control(state, self).map(|_| ()).map_err(|error| error.to_string())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        prepare_cycle_control(state, self).expect("checked cycle control").records.collect(out);
    }
    fn apply(&self, state: &mut RuntimeState) {
        apply_cycle_control(state, self).expect("Harmos checked the cycle control before applying it");
    }
}

impl Transaction<RuntimeState> for ResetCycle {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        ensure_runtime_active(state)?;
        apply_cycle_reset(&mut state.clone(), self).map(|_| ()).map_err(|error| error.to_string())
    }

    fn apply(&self, state: &mut RuntimeState) {
        apply_cycle_reset(state, self).expect("Harmos checked the cycle reset before applying it");
    }
}

/* === Public Functions === */
pub fn apply_cycle_reset(state: &mut RuntimeState, reset: &ResetCycle) -> Result<Vec<TestExecutionId>, Error> {
    let mut cycle = state.cycle_view(&reset.test_cycle_id).ok_or_else(|| Error::not_found("not_found", "cycle not found"))?;
    let CycleResetAction::Delete { test_case_id } = &reset.action;
    let removed = delete_cycle_case(&mut cycle, test_case_id, reset.occurred_at.clone()).map_err(conflict)?;
    state.replace_cycle(cycle).map_err(conflict)?;
    Ok(removed)
}

/* === Private Functions === */
fn apply_cycle_control(state: &mut RuntimeState, control: &ControlCycle) -> Result<(), Error> {
    let CycleControlPlan { cycle, records, settled } = prepare_cycle_control(state, control)?;
    for request in state
        .run_requests
        .values_mut()
        .filter(|request| request.summary.cycle_id == control.test_cycle_id && !request.summary.phase.terminal())
    {
        match control.action {
            CycleControlAction::Pause => request.summary.paused = true,
            CycleControlAction::Resume => request.summary.paused = false,
            CycleControlAction::Stop => {
                request.summary.stop_requested = true;
                request.summary.paused = false;
            }
        }
    }
    state.replace_cycle(cycle).expect("cycle projection has valid ownership");
    records.apply(state);
    commit_settled_prompts(state, settled);
    Ok(())
}

fn prepare_cycle_control(state: &RuntimeState, control: &ControlCycle) -> Result<CycleControlPlan, Error> {
    if state.run_requests.values().any(|request| {
        request.summary.cycle_id == control.test_cycle_id && request.summary.phase == super::provider::request::RunPhase::Finishing
    }) {
        return Err(conflict("provider work has settled; execution checkpoints are finishing".into()));
    }
    let mut cycle = state.cycle_view(&control.test_cycle_id).ok_or_else(|| Error::not_found("not_found", "cycle not found"))?;
    cycle.ensure_active().map_err(|reason| conflict(reason.into()))?;
    match control.action {
        CycleControlAction::Pause => pause_cycle(&mut cycle, control.occurred_at.clone()),
        CycleControlAction::Resume => resume_cycle(&mut cycle, control.occurred_at.clone()),
        CycleControlAction::Stop => stop_cycle(&mut cycle, control.occurred_at.clone()),
    }
    .map_err(conflict)?;

    let mut records = Drafts::new(state);
    if state.run_requests.values().any(|request| request.summary.cycle_id == control.test_cycle_id && !request.summary.phase.terminal()) {
        let (name, message) = match control.action {
            CycleControlAction::Pause => (BuiltinRecordType::TestExecutionPauseRequested, "Cycle request paused."),
            CycleControlAction::Resume => (BuiltinRecordType::TestExecutionResumeRequested, "Cycle request resumed."),
            CycleControlAction::Stop => (BuiltinRecordType::TestExecutionStopRequested, "Cycle request stop requested."),
        };
        record_request(&mut records, &cycle, name, message, control.occurred_at.clone());
        return Ok(CycleControlPlan::new(state, control, cycle, records));
    }
    match control.action {
        CycleControlAction::Pause => {
            record_execution_lifecycle(
                &mut records,
                &cycle,
                BuiltinRecordType::TestExecutionPaused,
                "Test execution paused.",
                control.occurred_at.clone(),
            );
            record_request(
                &mut records,
                &cycle,
                BuiltinRecordType::TestExecutionPauseRequested,
                "Operator paused test execution.",
                control.occurred_at.clone(),
            );
        }
        CycleControlAction::Resume => {
            record_execution_lifecycle(
                &mut records,
                &cycle,
                BuiltinRecordType::TestExecutionResumed,
                "Test execution resumed.",
                control.occurred_at.clone(),
            );
            record_request(
                &mut records,
                &cycle,
                BuiltinRecordType::TestExecutionResumeRequested,
                "Operator resumed test execution.",
                control.occurred_at.clone(),
            );
        }
        CycleControlAction::Stop => record_request(
            &mut records,
            &cycle,
            BuiltinRecordType::TestExecutionStopRequested,
            "Operator requested test execution stop.",
            control.occurred_at.clone(),
        ),
    }
    Ok(CycleControlPlan::new(state, control, cycle, records))
}

fn ensure_runtime_active(state: &RuntimeState) -> Result<(), String> {
    if state.shutting_down { Err("runtime is shutting down".to_owned()) } else { Ok(()) }
}

fn pause_cycle(cycle: &mut CycleView, at: Timestamp) -> Result<(), String> {
    if cycle.lifecycle != CycleStatus::Running {
        return Err("only running cycles can be paused".to_owned());
    }
    cycle.lifecycle = cycle.lifecycle.try_transition(CycleStatus::Paused).map_err(|error| error.to_string())?;
    cycle.touch_at(at);
    Ok(())
}

fn resume_cycle(cycle: &mut CycleView, at: Timestamp) -> Result<(), String> {
    if cycle.lifecycle != CycleStatus::Paused {
        return Err("only paused cycles can be resumed".to_owned());
    }
    cycle.lifecycle = cycle.lifecycle.try_transition(CycleStatus::Running).map_err(|error| error.to_string())?;
    cycle.touch_at(at);
    Ok(())
}

fn stop_cycle(cycle: &mut CycleView, at: Timestamp) -> Result<(), String> {
    if matches!(
        cycle.lifecycle,
        CycleStatus::Completed | CycleStatus::Failed | CycleStatus::Blocked | CycleStatus::Stopping | CycleStatus::Stopped
    ) {
        return Err("cycle cannot be stopped from its current state".to_owned());
    }
    if matches!(cycle.lifecycle, CycleStatus::Running | CycleStatus::Paused) {
        cycle.lifecycle = cycle.lifecycle.try_transition(CycleStatus::Stopping).map_err(|error| error.to_string())?;
        cycle.touch_at(at);
        return Ok(());
    }
    for execution in &mut cycle.test_executions {
        if matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running) {
            execution.status = ExecutionStatus::Cancelled;
            execution.finished_at.get_or_insert_with(|| at.clone());
            execution.result = Some(cancelled_summary());
        }
        for step in &mut execution.steps {
            if matches!(step.status, StepStatus::Pending | StepStatus::Running) {
                step.status = StepStatus::Cancelled;
                step.finished_at.get_or_insert_with(|| at.clone());
            }
            for criterion in &mut step.criteria {
                if criterion.status == CriterionStatus::Pending {
                    criterion.status = CriterionStatus::Skipped;
                }
            }
        }
    }
    cycle.lifecycle = cycle.lifecycle.try_transition(CycleStatus::Stopped).unwrap_or(CycleStatus::Stopped);
    cycle.result = Some(cancelled_summary());
    cycle.touch_at(at);
    Ok(())
}

fn cancelled_summary() -> ResultSummary {
    ResultSummary { status: ResultStatus::Cancelled, passed: 0, failed: 0, blocked: 0, skipped: 0 }
}

fn delete_cycle_case(cycle: &mut CycleView, test_case_id: &CaseId, at: Timestamp) -> Result<Vec<TestExecutionId>, String> {
    ensure_case_deletable(cycle, test_case_id)?;
    let removed = cycle
        .test_executions
        .iter()
        .filter(|execution| &execution.test_case_id == test_case_id)
        .map(|execution| execution.test_execution_id.clone())
        .collect();
    // Removing an already absent plan is idempotent, including acknowledgement loss.
    cycle.test_executions.retain(|execution| &execution.test_case_id != test_case_id);
    cycle.touch_at(at);
    Ok(removed)
}

pub(crate) fn ensure_case_deletable(cycle: &CycleView, test_case_id: &CaseId) -> Result<(), String> {
    ensure_cycle_mutable(cycle)?;
    if cycle
        .test_executions
        .iter()
        .filter(|execution| &execution.test_case_id == test_case_id)
        .any(|execution| execution.status != ExecutionStatus::Skipped || execution.started_at.is_some() || execution.finished_at.is_some())
    {
        return Err("test cases with started or finished executions cannot be deleted".to_owned());
    }
    Ok(())
}

pub(crate) fn ensure_cycle_mutable(cycle: &CycleView) -> Result<(), String> {
    cycle.ensure_active().map_err(str::to_owned)
}

fn record_execution_lifecycle(records: &mut Drafts, cycle: &CycleView, record_type: BuiltinRecordType, message: &str, at: Timestamp) {
    for execution in cycle.test_executions.iter().filter(|execution| execution.status == ExecutionStatus::Running) {
        records.draft(
            BuiltinDraft {
                record_type,
                source: "runtime".to_owned(),
                severity: RecordSeverity::Info,
                message: Some(message.to_owned()),
                details: BTreeMap::from([("test_execution_id".to_owned(), Value::from(execution.test_execution_id.as_str()))]),
                scope: Some(execution.scope.clone()),
            },
            at.clone(),
        );
    }
}

fn record_request(records: &mut Drafts, cycle: &CycleView, record_type: BuiltinRecordType, message: &str, at: Timestamp) {
    records.draft(
        BuiltinDraft {
            record_type,
            source: "runtime".to_owned(),
            severity: RecordSeverity::Info,
            message: Some(message.to_owned()),
            details: BTreeMap::new(),
            scope: Some(RecordScope::default().with_cycle(cycle.test_cycle_id.clone())),
        },
        at,
    );
}

fn conflict(message: String) -> Error {
    Error::conflict("conflict", message)
}

/* === Tests === */
