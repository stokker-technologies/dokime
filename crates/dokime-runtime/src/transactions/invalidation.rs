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
use crate::state::RuntimeState;
use crate::transactions::prompts::{PromptSettlement, SettledPrompts, commit_settled_prompts, settle_prompts};
use crate::transactions::records::draft::Drafts;
use dokime_domain::model::execution::result::ExecutionInvalidation;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::StepStatus;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::execution::snapshot::ExecutionPluginIdentity;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::transaction(id = "dokime.execution.invalidate", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvalidateExecutions {
    pub previous: ExecutionPluginIdentity,
    pub current: Option<ExecutionPluginIdentity>,
    pub invalidated_at: Timestamp,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl InvalidateExecutions {
    fn invalidates(&self, execution: &TestExecution) -> bool {
        execution.snapshot.plugin() == Some(&self.previous)
            && execution.finished_at.is_none()
            && matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running)
    }

    /// An invalidated execution answers nothing: its open prompts expire in
    /// the same transaction.
    fn prompts(&self, state: &RuntimeState) -> (Drafts, SettledPrompts) {
        let mut records = Drafts::new(state);
        let settled = settle_prompts(state, &mut records, PromptSettlement::Ended, &self.invalidated_at, |scope| {
            scope.test_execution_id.as_ref().and_then(|id| state.executions.get(id)).is_some_and(|execution| self.invalidates(execution))
        });
        (records, settled)
    }
}

impl Transaction<RuntimeState> for InvalidateExecutions {
    type Error = String;

    fn check(&self, _state: &RuntimeState) -> Result<(), String> {
        if self.current.as_ref() == Some(&self.previous) {
            return Err("the admitted executable identity has not changed".into());
        }
        if self.current.as_ref().is_some_and(|current| current.source != self.previous.source) {
            return Err("replacement identity must belong to the same provider".into());
        }
        Ok(())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        self.prompts(state).0.collect(out);
    }

    fn apply(&self, state: &mut RuntimeState) {
        let (records, settled) = self.prompts(state);
        records.apply(state);
        commit_settled_prompts(state, settled);
        for cycle in state.cycles.values_mut() {
            let mut changed = false;
            for id in &cycle.execution_ids {
                let execution = state.executions.get_mut(id).expect("cycle execution owner exists");
                if !self.invalidates(execution) {
                    continue;
                }
                execution.status = ExecutionStatus::Invalidated;
                execution.invalidation = Some(ExecutionInvalidation { previous: self.previous.clone(), current: self.current.clone() });
                execution.finished_at = Some(self.invalidated_at.clone());
                execution.result = None;
                for step in &mut execution.steps {
                    if matches!(step.status, StepStatus::Pending | StepStatus::Running) {
                        step.status = StepStatus::Cancelled;
                        step.finished_at = Some(self.invalidated_at.clone());
                    }
                }
                changed = true;
            }
            if changed {
                cycle.touch_at(self.invalidated_at.clone());
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
