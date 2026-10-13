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
use crate::execution::{CycleStatus, ExecutionStatus, StepRecord, StepStatus, TestExecution};
use crate::state::RuntimeState;
use crate::transactions::provider::request::RunPhase;
use dokime::core::{DokimeNodeIdentity, SessionId, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::transaction(id = "dokime.runtime.started", version = 4, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarkRuntimeStarted {
    pub session_id: SessionId,
    pub record_namespace: SessionId,
}

#[harmos::transaction(id = "dokime.runtime.identity.establish", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EstablishNodeIdentity {
    pub candidate: DokimeNodeIdentity,
}

#[harmos::transaction(id = "dokime.runtime.shutdown.start", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartShutdown;

#[harmos::transaction(id = "dokime.runtime.interrupted.recover", version = 4, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecoverInterruptedCycles {
    pub at: Timestamp,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Transaction<RuntimeState> for MarkRuntimeStarted {
    type Error = String;

    fn check(&self, _state: &RuntimeState) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        state.session_id = self.session_id.clone();
        state.record_namespace = self.record_namespace.clone();
        state.shutting_down = false;
    }
}

impl Transaction<RuntimeState> for EstablishNodeIdentity {
    type Error = String;

    fn check(&self, _state: &RuntimeState) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        state.establish_node_identity(self.candidate.clone());
    }
}

impl Transaction<RuntimeState> for StartShutdown {
    type Error = String;

    fn check(&self, _state: &RuntimeState) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        state.shutting_down = true;
    }
}

impl Transaction<RuntimeState> for RecoverInterruptedCycles {
    type Error = String;
    fn check(&self, _: &RuntimeState) -> Result<(), String> {
        Ok(())
    }
    fn apply(&self, state: &mut RuntimeState) {
        // The requester no longer exists after restart. Keep the original
        // question as evidence without fabricating an operator response.
        for (id, prompt) in &state.prompt_requests {
            if (prompt.scope.test_cycle_id.is_some() || prompt.scope.test_execution_id.is_some())
                && !state.prompt_responses.contains_key(id)
            {
                state
                    .closed_prompts
                    .entry(id.clone())
                    .or_insert_with(|| self.at.clone());
            }
        }

        for request in state.run_requests.values_mut() {
            if request.summary.phase.terminal() {
                continue;
            }

            request.summary.phase = RunPhase::Interrupted;
            request.summary.finished_at = Some(self.at.clone());
            request.summary.paused = false;
            request.pending.clear();
            request.groups.clear();
            request.conclusion = Some(CycleStatus::Stopped);
            request.failure = Some("runtime restarted before request completion".into());
        }

        // Sealed Cycles Are Served From Native History After A Restart
        state.retire_sealed_cycles();
        for cycle in state.cycles.values_mut() {
            let mut changed = matches!(
                cycle.lifecycle,
                CycleStatus::Ready
                    | CycleStatus::Running
                    | CycleStatus::Paused
                    | CycleStatus::Stopping
            );
            for id in &cycle.execution_ids {
                let execution = state
                    .executions
                    .get_mut(id)
                    .expect("cycle execution owner exists");
                changed |= interrupt_execution(execution, &self.at);
            }
            if changed {
                cycle.lifecycle = CycleStatus::Stopped;
                cycle.result = None;
                cycle.touch_at(self.at.clone());
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn interrupt_execution(execution: &mut TestExecution, at: &Timestamp) -> bool {
    if !matches!(
        execution.status,
        ExecutionStatus::Pending | ExecutionStatus::Running
    ) && !(execution.status == ExecutionStatus::Skipped && execution.finished_at.is_none())
    {
        return false;
    }

    execution.interrupt_step_attempt();
    execution.status = ExecutionStatus::Interrupted;
    execution.result = None;
    execution.finished_at = Some(at.clone());
    for step in &mut execution.steps {
        interrupt_step(step, at);
    }
    execution.refresh_step_records();
    true
}

fn interrupt_step(step: &mut StepRecord, at: &Timestamp) {
    if matches!(step.status, StepStatus::Pending | StepStatus::Running) {
        step.status = StepStatus::Cancelled;
        step.finished_at = Some(at.clone());
    }
}

/* === Tests === */
