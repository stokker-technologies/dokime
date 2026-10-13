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
use crate::execution::result::ExecutionInvalidation;
use crate::execution::{ExecutionPluginIdentity, ExecutionStatus, StepStatus};
use crate::state::RuntimeState;
use dokime::core::Timestamp;

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
impl Transaction<RuntimeState> for InvalidateExecutions {
    type Error = String;

    fn check(&self, _state: &RuntimeState) -> Result<(), String> {
        if self.current.as_ref() == Some(&self.previous) {
            return Err("the admitted executable identity has not changed".into());
        }
        if self
            .current
            .as_ref()
            .is_some_and(|current| current.source != self.previous.source)
        {
            return Err("replacement identity must belong to the same provider".into());
        }
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        for cycle in state.cycles.values_mut() {
            let mut changed = false;
            for id in &cycle.execution_ids {
                let execution = state
                    .executions
                    .get_mut(id)
                    .expect("cycle execution owner exists");
                if execution.snapshot.plugin() != Some(&self.previous)
                    || execution.finished_at.is_some()
                    || !matches!(
                        execution.status,
                        ExecutionStatus::Pending | ExecutionStatus::Running
                    )
                {
                    continue;
                }
                execution.status = ExecutionStatus::Invalidated;
                execution.invalidation = Some(ExecutionInvalidation {
                    previous: self.previous.clone(),
                    current: self.current.clone(),
                });
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
