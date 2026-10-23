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
use crate::records::{BuiltinRecordType, RecordSeverity};
use crate::state::RuntimeState;
use crate::transactions::records::draft::{BuiltinDraft, Drafts};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "dokime.execution.label.rename", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenameLabel {
    pub subject: LabelSubject,
    pub label: String,
    pub edited_at: Timestamp,
}

/* === Enums === */

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
pub enum LabelSubject {
    Cycle(TestCycleId),
    Execution(TestExecutionId),
}

/* === Traits === */

/* === Implementations === */

impl Transaction<RuntimeState> for RenameLabel {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        if self.label.trim().is_empty() {
            return Err("label must not be empty".to_owned());
        }
        match &self.subject {
            LabelSubject::Cycle(id) => {
                let cycle = state.cycles.get(id).ok_or("cycle not found")?;
                cycle.ensure_active().map_err(str::to_owned)?;
                if state.archived_cycles.contains(id) {
                    return Err("cycle is archived; unarchive it before renaming".to_owned());
                }
                Ok(())
            }
            LabelSubject::Execution(id) => {
                let execution = state.executions.get(id).ok_or("execution not found")?;
                state.cycles.get(&execution.test_cycle_id).ok_or("execution cycle not found")?.ensure_active().map_err(str::to_owned)?;
                if case_attempts(state, id)
                    .iter()
                    .any(|attempt| state.executions.get(attempt).is_some_and(|execution| execution.saved.is_some()))
                {
                    return Err("execution is sealed; its report keeps the label it froze".to_owned());
                }
                Ok(())
            }
        }
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        self.plan(state).collect(out);
    }

    fn apply(&self, state: &mut RuntimeState) {
        let records = self.plan(state);
        match &self.subject {
            LabelSubject::Cycle(id) => {
                if let Some(cycle) = state.cycles.get_mut(id) {
                    cycle.metadata.label = self.label.clone();
                }
            }
            LabelSubject::Execution(id) => {
                if let Some(case_id) = state.executions.get(id).map(|execution| execution.test_case_id.clone()) {
                    state.suites.relabel_case(&case_id, &self.label);
                }
                for attempt in case_attempts(state, id) {
                    if let Some(execution) = state.executions.get_mut(&attempt) {
                        execution.snapshot.relabel_case(self.label.clone());
                    }
                }
            }
        }
        records.apply(state);
    }
}

impl RenameLabel {
    fn plan(&self, state: &RuntimeState) -> Drafts {
        let mut records = Drafts::new(state);
        let current = match &self.subject {
            LabelSubject::Cycle(id) => {
                state.cycles.get(id).map(|cycle| (cycle.metadata.label.clone(), RecordScope::default().with_cycle(id.clone())))
            }
            LabelSubject::Execution(id) => {
                state.executions.get(id).map(|execution| (execution.snapshot.case().metadata.label.clone(), execution.scope.clone()))
            }
        };
        if let Some((old_label, scope)) = current {
            records.draft(
                BuiltinDraft {
                    record_type: BuiltinRecordType::OperatorLabelEdited,
                    source: "operator".to_owned(),
                    severity: RecordSeverity::Info,
                    message: Some("Operator label edited.".to_owned()),
                    details: BTreeMap::from([
                        ("old_label".to_owned(), Value::from(old_label)),
                        ("new_label".to_owned(), Value::from(self.label.clone())),
                    ]),
                    scope: Some(scope),
                },
                self.edited_at.clone(),
            );
        }
        records
    }
}

/* === Public Functions === */

pub fn case_attempts(state: &RuntimeState, id: &TestExecutionId) -> Vec<TestExecutionId> {
    let Some(execution) = state.executions.get(id) else {
        return Vec::new();
    };
    state
        .cycles
        .get(&execution.test_cycle_id)
        .map(|cycle| {
            cycle
                .execution_ids
                .iter()
                .filter(|member| state.executions.get(*member).is_some_and(|attempt| attempt.test_case_id == execution.test_case_id))
                .cloned()
                .collect()
        })
        .unwrap_or_else(|| vec![id.clone()])
}

/* === Private Functions === */

/* === Tests === */
