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
use dokime_domain::model::execution::result::CycleStatus;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "dokime.execution.archive")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetArchived {
    pub subject: ArchiveSubject,
    pub archived: bool,
}

/* === Enums === */

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
pub enum ArchiveSubject {
    Cycle(TestCycleId),
    Execution(TestExecutionId),
}

/* === Traits === */

/* === Implementations === */

impl Transaction<RuntimeState> for SetArchived {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        match &self.subject {
            ArchiveSubject::Cycle(id) => match state.cycles.get(id) {
                Some(cycle) if cycle.phase != dokime_domain::model::execution::result::CyclePhase::Sealed => {
                    Err("only a sealed cycle can be archived or unarchived".to_owned())
                }
                _ => Ok(()),
            },
            ArchiveSubject::Execution(id) => match state.executions.get(id) {
                Some(execution) if !execution_terminal(&execution.status) => {
                    Err("only a terminal execution can be archived or unarchived".to_owned())
                }
                _ => Ok(()),
            },
        }
    }

    fn apply(&self, state: &mut RuntimeState) {
        match (&self.subject, self.archived) {
            (ArchiveSubject::Cycle(id), true) => {
                state.archived_cycles.insert(id.clone());
                state.cycle_report_finalizations.remove(id);
                if let Some(cycle) = state.cycles.get(id) {
                    for member in &cycle.execution_ids {
                        if state.executions.get(member).is_some_and(|execution| execution.saved.is_some()) {
                            state.report_finalizations.remove(member);
                        }
                    }
                }
            }
            (ArchiveSubject::Cycle(id), false) => {
                state.archived_cycles.remove(id);
            }
            (ArchiveSubject::Execution(id), true) => {
                state.archived_executions.insert(id.clone());
                if state.executions.get(id).is_some_and(|execution| execution.saved.is_some()) {
                    state.report_finalizations.remove(id);
                }
            }
            (ArchiveSubject::Execution(id), false) => {
                state.archived_executions.remove(id);
            }
        }
    }
}

/* === Public Functions === */

pub fn cycle_terminal(lifecycle: CycleStatus) -> bool {
    matches!(lifecycle, CycleStatus::Completed | CycleStatus::Failed | CycleStatus::Blocked | CycleStatus::Stopped)
}

pub fn execution_terminal(status: &ExecutionStatus) -> bool {
    !matches!(status, ExecutionStatus::Pending | ExecutionStatus::Running)
}

/* === Private Functions === */

/* === Tests === */
