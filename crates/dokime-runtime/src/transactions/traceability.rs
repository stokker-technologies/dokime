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
use std::collections::BTreeSet;
// Third Party Imports
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};
// Local Imports
use crate::records::BuiltinRecordType;
use crate::transactions::records::draft::BuiltinDraft;
use crate::{runtime::Error, state::RuntimeState};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::CycleStatus;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::StepRecord;
use dokime_domain::model::execution::result::StepStatus;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::identity::StepId;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::traceability::TraceabilitySection;
use dokime_domain::model::traceability::derived_traceability_group_named;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::transaction(id = "dokime.traceability.phase.update", version = 3, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateExecutionPhase {
    pub scope: RecordScope,
    pub identifier: Identifier,
    pub label: String,
    pub status: String,
    pub occurred_at: Timestamp,
}

#[harmos::transaction(id = "dokime.execution.capture.phase", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetCapturePhase {
    pub scope: RecordScope,
    pub phase: dokime_domain::model::execution::CapturePhase,
    pub occurred_at: Timestamp,
}

#[harmos::transaction(id = "dokime.execution.step.attempt.begin", version = 2, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeginStepAttempt {
    pub scope: RecordScope,
    pub start: dokime_domain::model::execution::result::StepAttemptStart,
}

#[harmos::transaction(id = "dokime.execution.step.attempt.record", version = 2, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordStepAttempt {
    pub scope: RecordScope,
    pub attempt: dokime_domain::model::execution::result::TestStepAttempt,
}

#[harmos::transaction(id = "dokime.traceability.template.record", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordExecutionTraceability {
    pub scope: RecordScope,
    pub sections: Vec<TraceabilitySection>,
    pub occurred_at: Timestamp,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Transaction<RuntimeState> for RecordExecutionTraceability {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        if state.shutting_down {
            return Err("runtime is shutting down".to_owned());
        }
        record_execution_traceability(&mut state.clone(), self.clone()).map_err(|error| error.to_string())
    }

    fn apply(&self, state: &mut RuntimeState) {
        record_execution_traceability(state, self.clone()).expect("Harmos checked the authored traceability before applying it");
    }
}

impl Transaction<RuntimeState> for SetCapturePhase {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        set_capture_phase(&mut state.clone(), self).map_err(|error| error.to_string())
    }
    fn apply(&self, state: &mut RuntimeState) {
        set_capture_phase(state, self).expect("checked capture phase");
    }
}

impl Transaction<RuntimeState> for BeginStepAttempt {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        begin_step_attempt(&mut state.clone(), self).map_err(|error| error.to_string())
    }
    fn apply(&self, state: &mut RuntimeState) {
        begin_step_attempt(state, self).expect("Harmos checked step admission before activation");
    }
}

impl Transaction<RuntimeState> for RecordStepAttempt {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        admit_step_attempt(&mut state.clone(), self).map_err(|error| error.to_string())
    }
    fn apply(&self, state: &mut RuntimeState) {
        admit_step_attempt(state, self).expect("Harmos checked the step attempt before admission");
    }
}

impl Transaction<RuntimeState> for UpdateExecutionPhase {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        if state.shutting_down {
            return Err("runtime is shutting down".to_owned());
        }
        update_execution_phase(&mut state.clone(), self.clone()).map_err(|error| error.to_string())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        self.start_records(state).collect(out);
    }
    fn apply(&self, state: &mut RuntimeState) {
        let records = self.start_records(state);
        update_execution_phase(state, self.clone()).expect("Harmos checked the execution phase before applying it");
        records.apply(state);
    }
}

impl UpdateExecutionPhase {
    fn start_records(&self, state: &RuntimeState) -> crate::transactions::records::draft::Drafts {
        let mut records = crate::transactions::records::draft::Drafts::new(state);
        let execution = self.scope.test_cycle_id.as_ref().and_then(|id| state.cycles.get(id)).and_then(|cycle| {
            state.executions_for_cycle(cycle).find(|execution| {
                if let Some(id) = &self.scope.test_execution_id {
                    &execution.test_execution_id == id
                } else {
                    Some(&execution.test_case_id) == self.scope.test_case_id.as_ref()
                }
            })
        });
        if let Some(execution) = execution
            && execution.started_at.is_none()
            && phase_step_status(&self.status) != StepStatus::Pending
        {
            records.draft(
                BuiltinDraft {
                    record_type: BuiltinRecordType::TestExecutionStarted,
                    source: "runtime".into(),
                    severity: crate::records::RecordSeverity::Info,
                    message: Some("Test execution started.".into()),
                    details: Default::default(),
                    scope: Some(self.scope.clone().with_test_execution(execution.test_execution_id.clone())),
                },
                self.occurred_at.clone(),
            );
        }
        records
    }
}

/* === Public Functions === */
fn set_capture_phase(state: &mut RuntimeState, update: &SetCapturePhase) -> Result<(), Error> {
    let id = update.scope.test_execution_id.as_ref().ok_or_else(|| conflict("capture phase requires an execution"))?;
    let execution = state.executions.get_mut(id).ok_or_else(|| not_found("execution not found"))?;
    if update.scope.test_cycle_id.as_ref() != Some(&execution.test_cycle_id)
        || update.scope.test_case_id.as_ref() != Some(&execution.test_case_id)
    {
        return Err(conflict("capture phase scope differs from its execution owner"));
    }
    if !matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running) || execution.finished_at.is_some() {
        return Err(conflict("capture phase requires an unsealed execution"));
    }
    if execution.active_step_attempt.is_some() {
        return Err(conflict("capture phase cannot close an active step"));
    }
    if update.phase == dokime_domain::model::execution::CapturePhase::Initialization
        && (!execution.step_attempts.is_empty() || execution.capture_phase == dokime_domain::model::execution::CapturePhase::Cleanup)
    {
        return Err(conflict("initialization cannot resume after a step attempt"));
    }
    execution.capture_phase = update.phase;
    if update.phase == dokime_domain::model::execution::CapturePhase::Initialization {
        execution.status = ExecutionStatus::Running;
        execution.started_at.get_or_insert_with(|| update.occurred_at.clone());
    }
    Ok(())
}

fn begin_step_attempt(state: &mut RuntimeState, update: &BeginStepAttempt) -> Result<(), Error> {
    if state.shutting_down {
        return Err(conflict("runtime is shutting down"));
    }
    let cycle_id = update.scope.test_cycle_id.as_ref().ok_or_else(|| not_found("step admission requires a cycle"))?;
    let cycle = state.cycles.get(cycle_id).ok_or_else(|| not_found("cycle not found"))?;
    if !matches!(cycle.lifecycle, CycleStatus::Running) {
        return Err(conflict("step admission requires a running cycle"));
    }
    let id = update.scope.test_execution_id.as_ref().ok_or_else(|| not_found("step admission requires an execution"))?;
    let execution = state.executions.get_mut(id).ok_or_else(|| not_found("execution not found"))?;
    if &execution.test_cycle_id != cycle_id || update.scope.test_case_id.as_ref() != Some(&execution.test_case_id) {
        return Err(conflict("step admission scope differs from its execution owner"));
    }
    if execution.capture_phase == dokime_domain::model::execution::CapturePhase::Cleanup {
        return Err(conflict("step admission cannot interrupt cleanup"));
    }
    execution.validate_step_start(&update.start).map_err(conflict)?;
    if !matches!(execution.status, ExecutionStatus::Running) || execution.finished_at.is_some() {
        return Err(conflict("step admission requires a running unsealed execution"));
    }
    if execution.step_attempts.iter().any(|attempt| attempt.attempt_id == update.start.attempt_id) {
        return Err(conflict("completed step attempt cannot be invoked again"));
    }
    if let Some(active) = &execution.active_step_attempt {
        return if active == &update.start { Ok(()) } else { Err(conflict("another step attempt is already active")) };
    }
    execution.validate_step_invocation(&update.start).map_err(conflict)?;
    execution.capture_phase = dokime_domain::model::execution::CapturePhase::Default;
    execution.active_step_attempt = Some(update.start.clone());
    execution.refresh_step_records();
    Ok(())
}

fn admit_step_attempt(state: &mut RuntimeState, update: &RecordStepAttempt) -> Result<(), Error> {
    let id = update.scope.test_execution_id.as_ref().ok_or_else(|| not_found("step attempt requires an execution"))?;
    let execution = state.executions.get_mut(id).ok_or_else(|| not_found("execution not found"))?;
    if update.scope.test_cycle_id.as_ref() != Some(&execution.test_cycle_id)
        || update.scope.test_case_id.as_ref() != Some(&execution.test_case_id)
    {
        return Err(conflict("step attempt scope differs from its execution owner"));
    }
    execution.validate_step_attempt(&update.attempt).map_err(conflict)?;
    if let Some(existing) = execution.step_attempts.iter().find(|attempt| attempt.attempt_id == update.attempt.attempt_id) {
        return if existing == &update.attempt { Ok(()) } else { Err(conflict("step attempt identity has different content")) };
    }
    if !matches!(execution.status, ExecutionStatus::Running) || execution.finished_at.is_some() {
        return Err(conflict("step attempt requires a running unsealed execution"));
    }
    if execution.active_step_attempt.as_ref() != Some(&update.attempt.start()) {
        return Err(conflict("completion differs from the admitted active step attempt"));
    }
    execution.step_attempts.push(update.attempt.clone());
    execution.active_step_attempt = None;
    execution.refresh_step_records();
    Ok(())
}

pub fn update_execution_phase(state: &mut RuntimeState, update: UpdateExecutionPhase) -> Result<(), Error> {
    let step_id = phase_step_id(&update);
    let test_cycle_id = update.scope.test_cycle_id.clone().ok_or_else(|| not_found("execution phase scope missing cycle"))?;
    let step_status = phase_step_status(&update.status);
    let cycle = state.cycles.get_mut(&test_cycle_id).ok_or_else(|| not_found("cycle not found"))?;
    let execution_index = if let Some(id) = &update.scope.test_execution_id {
        cycle.execution_ids.iter().position(|execution_id| execution_id == id)
    } else if let Some(id) = &update.scope.test_case_id {
        let mut matches = cycle
            .execution_ids
            .iter()
            .enumerate()
            .filter(|(_, execution_id)| state.executions.get(*execution_id).is_some_and(|execution| &execution.test_case_id == id));
        let first = matches.next().map(|(index, _)| index);
        if matches.next().is_some() {
            return Err(conflict("execution phase scope is ambiguous; provide an execution id"));
        }
        first
    } else {
        None
    }
    .ok_or_else(|| not_found("execution not found"))?;
    let execution = state.executions.get_mut(&cycle.execution_ids[execution_index]).expect("cycle execution owner exists");

    if !matches!(cycle.lifecycle, CycleStatus::Running | CycleStatus::Ready)
        || !matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running)
    {
        return Err(conflict("execution phase updates require an active cycle and execution"));
    }

    if execution.status == ExecutionStatus::Pending && step_status != StepStatus::Pending {
        execution.status = ExecutionStatus::Running;
        execution.started_at = Some(update.occurred_at.clone());
    }
    if execution.started_at.is_none() && execution.status == ExecutionStatus::Running {
        execution.started_at = Some(update.occurred_at.clone());
    }
    if execution.status != ExecutionStatus::Running {
        return Err(conflict("execution is not running"));
    }

    let step = match execution.steps.iter_mut().find(|step| step.metadata.identifier == update.identifier) {
        Some(step) => step,
        None => {
            execution.steps.push(StepRecord {
                step_id,
                metadata: Metadata::new(update.identifier.clone(), update.label.clone(), None),
                status: StepStatus::Pending,
                started_at: None,
                finished_at: None,
                measurements: Vec::new(),
                criteria: Vec::new(),
                detail: None,
            });
            execution.steps.last_mut().expect("step was just pushed")
        }
    };
    step.metadata = Metadata::new(update.identifier, update.label, step.metadata.description.clone());
    step.status = step_status.clone();
    if step_status == StepStatus::Running && step.started_at.is_none() {
        step.started_at = Some(update.occurred_at.clone());
    }
    if is_terminal_step(&step_status) {
        step.finished_at = Some(update.occurred_at.clone());
    }
    execution.refresh_step_records();
    cycle.touch_at(update.occurred_at);
    Ok(())
}

/* === Private Functions === */
fn record_execution_traceability(state: &mut RuntimeState, update: RecordExecutionTraceability) -> Result<(), Error> {
    admissible_sections(&update.sections)?;
    let test_cycle_id = update.scope.test_cycle_id.clone().ok_or_else(|| not_found("authored traceability scope missing cycle"))?;
    let test_execution_id =
        update.scope.test_execution_id.clone().ok_or_else(|| not_found("authored traceability scope missing execution"))?;
    let execution = state
        .executions
        .get_mut(&test_execution_id)
        .filter(|execution| execution.test_cycle_id == test_cycle_id)
        .ok_or_else(|| not_found("execution not found"))?;

    for section in update.sections {
        match execution.traceability.iter_mut().find(|stated| stated.identifier == section.identifier) {
            Some(stated) => *stated = section,
            None => execution.traceability.push(section),
        }
    }
    state.cycles.get_mut(&test_cycle_id).expect("the cycle was found above").touch_at(update.occurred_at);
    Ok(())
}

fn admissible_sections(sections: &[TraceabilitySection]) -> Result<(), Error> {
    let mut stated = BTreeSet::new();
    for section in sections {
        let identifier = section.identifier.as_str();
        if let Some(derived) = derived_traceability_group_named(identifier) {
            return Err(validation(format!(
                "traceability section \"{identifier}\" claims the \"{}\" group the host derives \
                 for every execution; rename the section",
                derived.label
            )));
        }
        if !stated.insert(identifier) {
            return Err(validation(format!(
                "traceability section \"{identifier}\" was stated twice in one delivery; state \
                 each section once"
            )));
        }
    }
    Ok(())
}

fn phase_step_id(update: &UpdateExecutionPhase) -> StepId {
    let scope_key = crate::traceability::scope_discriminator(&update.scope);
    StepId::from_seed(format!("{}:{}:{scope_key}", update.occurred_at, update.identifier))
}

fn phase_step_status(status: &str) -> StepStatus {
    match status {
        "running" => StepStatus::Running,
        "passed" | "completed" => StepStatus::Passed,
        "failed" | "blocked" => StepStatus::Failed,
        "skipped" => StepStatus::Skipped,
        "cancelled" | "canceled" | "stopped" => StepStatus::Cancelled,
        _ => StepStatus::Pending,
    }
}

fn is_terminal_step(status: &StepStatus) -> bool {
    matches!(status, StepStatus::Passed | StepStatus::Failed | StepStatus::Skipped | StepStatus::Cancelled)
}

fn not_found(message: impl Into<String>) -> Error {
    Error::not_found("not_found", message)
}

fn conflict(message: impl Into<String>) -> Error {
    Error::conflict("conflict", message)
}

fn validation(message: impl Into<String>) -> Error {
    Error::validation("invalid_traceability_section", message, vec![])
}

/* === Tests === */
