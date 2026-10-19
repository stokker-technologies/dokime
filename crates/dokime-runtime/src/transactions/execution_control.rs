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

// Third Party Imports
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};

// Local Imports
use crate::records::BuiltinRecordType;
use crate::records::RecordSeverity;
use crate::state::RuntimeState;
use crate::transactions::prompts::{PromptSettlement, SettledPrompts, commit_settled_prompts, settle_prompts};
use crate::transactions::records::draft::BuiltinDraft;
use crate::transactions::records::draft::Drafts;
use crate::transactions::{
    CycleControlAction,
    provider::request::{RunPhase, RunRequest},
};
use dokime_domain::model::execution::result::CycleStatus;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::transaction(id = "dokime.execution.control", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlExecution {
    pub test_execution_id: TestExecutionId,
    pub action: CycleControlAction,
    pub occurred_at: Timestamp,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ControlExecution {
    pub(crate) fn owning_request<'a>(state: &'a RuntimeState, id: &TestExecutionId) -> Option<&'a RunRequest> {
        state.executions.get(id)?;
        state.run_requests.values().find(|request| !request.summary.phase.terminal() && request.selected_execution_ids.contains(id))
    }

    pub(crate) fn request<'a>(&self, state: &'a RuntimeState) -> Result<&'a RunRequest, String> {
        if state.shutting_down {
            return Err("runtime is shutting down".into());
        }
        let execution = state.executions.get(&self.test_execution_id).ok_or("execution not found")?;
        let request = Self::owning_request(state, &self.test_execution_id).ok_or("execution has no active request")?;
        if request.summary.cycle_id != execution.test_cycle_id {
            return Err("execution request belongs to another cycle".into());
        }
        let cycle = state.cycles.get(&execution.test_cycle_id).ok_or("execution cycle not found")?;
        if !cycle.execution_ids.contains(&self.test_execution_id) {
            return Err("execution is not a member of its cycle".into());
        }
        if !matches!(request.summary.phase, RunPhase::Queued | RunPhase::Preparing | RunPhase::Running) {
            return Err("provider work has settled; execution checkpoints are finishing".into());
        }
        match self.action {
            CycleControlAction::Pause if request.summary.paused || request.summary.stop_requested => {
                Err("execution request cannot be paused from its current state".into())
            }
            CycleControlAction::Resume if !request.summary.paused || request.summary.stop_requested => {
                Err("execution request cannot be resumed from its current state".into())
            }
            _ => Ok(request),
        }
    }

    fn drafts(&self, state: &RuntimeState) -> (Drafts, SettledPrompts) {
        let mut records = Drafts::new(state);
        let execution = &state.executions[&self.test_execution_id];
        let (name, message) = match self.action {
            CycleControlAction::Pause => (BuiltinRecordType::TestExecutionPauseRequested, "Execution request paused."),
            CycleControlAction::Resume => (BuiltinRecordType::TestExecutionResumeRequested, "Execution request resumed."),
            CycleControlAction::Stop => (BuiltinRecordType::TestExecutionStopRequested, "Execution request stop requested."),
        };
        records.draft(
            BuiltinDraft {
                record_type: name,
                source: "runtime".into(),
                severity: RecordSeverity::Info,
                message: Some(message.into()),
                details: Default::default(),
                scope: Some(execution.scope.clone()),
            },
            self.occurred_at.clone(),
        );
        // A stopped execution answers nothing: its open prompts settle with the
        // stop itself, so a provider blocked on one can wind down.
        let settled = match self.action {
            CycleControlAction::Stop => settle_prompts(state, &mut records, PromptSettlement::Stopped, &self.occurred_at, |scope| {
                scope.test_execution_id.as_ref() == Some(&self.test_execution_id)
            }),
            CycleControlAction::Pause | CycleControlAction::Resume => SettledPrompts::default(),
        };
        (records, settled)
    }
}

impl Transaction<RuntimeState> for ControlExecution {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        if matches!(self.action, CycleControlAction::Stop) {
            let execution = state.executions.get(&self.test_execution_id).ok_or("execution not found")?;
            state.cycles.get(&execution.test_cycle_id).ok_or("execution cycle not found")?;
            return Ok(());
        }
        self.request(state).map(|_| ())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        self.drafts(state).0.collect(out);
    }

    fn apply(&self, state: &mut RuntimeState) {
        if matches!(self.action, CycleControlAction::Stop) {
            self.stop(state);
            return;
        }
        let id = self.request(state).expect("checked execution control").summary.request_id.clone();
        let (records, settled) = self.drafts(state);
        let request = state.run_requests.get_mut(&id).expect("checked request");
        let cycle = state.cycles.get_mut(&request.summary.cycle_id).expect("checked cycle");
        match self.action {
            CycleControlAction::Pause => {
                request.summary.paused = true;
                cycle.lifecycle = CycleStatus::Paused;
            }
            CycleControlAction::Resume => {
                request.summary.paused = false;
                cycle.lifecycle = CycleStatus::Running;
            }
            CycleControlAction::Stop => {
                request.summary.stop_requested = true;
                request.summary.paused = false;
                cycle.lifecycle = CycleStatus::Stopping;
            }
        }
        cycle.touch_at(self.occurred_at.clone());
        let cycle_id = cycle.test_cycle_id.clone();
        super::provider::request::sync_cycle_activity(state, &cycle_id);
        records.apply(state);
        commit_settled_prompts(state, settled);
    }
}

impl ControlExecution {
    fn stop(&self, state: &mut RuntimeState) {
        use dokime_domain::model::execution::result::{ExecutionStatus, ResultStatus, ResultSummary, StepStatus};
        let (records, settled) = self.drafts(state);
        let request_id = Self::owning_request(state, &self.test_execution_id).map(|request| request.summary.request_id.clone());
        let execution = state.executions.get_mut(&self.test_execution_id).expect("checked execution");
        let cycle = state.cycles.get_mut(&execution.test_cycle_id).expect("checked cycle");
        if cycle.phase == dokime_domain::model::execution::result::CyclePhase::Active {
            cycle.continue_automatically = false;
            cycle.touch_at(self.occurred_at.clone());
        }
        if execution.started_at.is_none() && execution.finished_at.is_none() {
            execution.status = ExecutionStatus::Cancelled;
            execution.finished_at = Some(self.occurred_at.clone());
            execution.result = Some(ResultSummary { passed: 0, failed: 0, blocked: 0, skipped: 1, status: ResultStatus::Cancelled });
            for step in &mut execution.steps {
                if step.status == StepStatus::Pending {
                    step.status = StepStatus::Skipped;
                    step.finished_at = Some(self.occurred_at.clone());
                }
            }
        }
        if let Some(id) = request_id {
            let request = state.run_requests.get_mut(&id).expect("execution request exists");
            request.summary.stop_requested = true;
            request.summary.paused = false;
        }
        records.apply(state);
        commit_settled_prompts(state, settled);
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
