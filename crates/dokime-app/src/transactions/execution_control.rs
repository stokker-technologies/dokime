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
use harmos::Transaction;
use serde::{Deserialize, Serialize};

// Local Imports
use crate::execution::CycleStatus;
use crate::records::{ProductRecordDraft, RecordSeverity};
use crate::state::RuntimeState;
use crate::transactions::{
    CycleControlAction,
    provider::request::{RunPhase, RunRequest},
};
use dokime::core::{Identifier, TestExecutionId, Timestamp};

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
    pub(crate) fn request<'a>(&self, state: &'a RuntimeState) -> Result<&'a RunRequest, String> {
        if state.shutting_down {
            return Err("runtime is shutting down".into());
        }
        let execution = state
            .executions
            .get(&self.test_execution_id)
            .ok_or("execution not found")?;
        let request = state
            .run_requests
            .values()
            .find(|request| {
                request.test_execution_id.as_ref() == Some(&self.test_execution_id)
                    && !request.summary.phase.terminal()
            })
            .ok_or("execution has no active execution-addressed request")?;
        if request.summary.cycle_id != execution.test_cycle_id {
            return Err("execution request belongs to another cycle".into());
        }
        let cycle = state
            .cycles
            .get(&execution.test_cycle_id)
            .ok_or("execution cycle not found")?;
        if !cycle.execution_ids.contains(&self.test_execution_id) {
            return Err("execution is not a member of its cycle".into());
        }
        if !matches!(
            request.summary.phase,
            RunPhase::Queued | RunPhase::Preparing | RunPhase::Running
        ) {
            return Err("provider work has settled; execution checkpoints are finishing".into());
        }
        match self.action {
            CycleControlAction::Pause
                if request.summary.paused || request.summary.stop_requested =>
            {
                Err("execution request cannot be paused from its current state".into())
            }
            CycleControlAction::Resume
                if !request.summary.paused || request.summary.stop_requested =>
            {
                Err("execution request cannot be resumed from its current state".into())
            }
            _ => Ok(request),
        }
    }

    fn drafts(&self, state: &RuntimeState) -> crate::transactions::records::draft::Drafts {
        let mut records = crate::transactions::records::draft::Drafts::new(state);
        let execution = &state.executions[&self.test_execution_id];
        let (name, message) = match self.action {
            CycleControlAction::Pause => (
                "test.execution.pause_requested",
                "Execution request paused.",
            ),
            CycleControlAction::Resume => (
                "test.execution.resume_requested",
                "Execution request resumed.",
            ),
            CycleControlAction::Stop => (
                "test.execution.stop_requested",
                "Execution request stop requested.",
            ),
        };
        records.draft(
            ProductRecordDraft {
                record_type: Identifier::new(name).expect("builtin record"),
                source: "runtime".into(),
                severity: RecordSeverity::Info,
                message: Some(message.into()),
                details: Default::default(),
                scope: Some(execution.scope.clone()),
            },
            self.occurred_at.clone(),
        );
        records
    }
}

impl Transaction<RuntimeState> for ControlExecution {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        self.request(state).map(|_| ())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::Records) {
        self.drafts(state).collect(out);
    }

    fn apply(&self, state: &mut RuntimeState) {
        let id = self
            .request(state)
            .expect("checked execution control")
            .summary
            .request_id
            .clone();
        let records = self.drafts(state);
        let request = state.run_requests.get_mut(&id).expect("checked request");
        let cycle = state
            .cycles
            .get_mut(&request.summary.cycle_id)
            .expect("checked cycle");
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
        records.apply(state);
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
