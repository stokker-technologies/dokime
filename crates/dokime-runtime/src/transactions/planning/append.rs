/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Atomically Bind Explicit Executions To Registered Cases

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};

// Local Imports
use crate::state::RuntimeState;
use dokime_domain::model::identity::{CaseId, TestCycleId, TestExecutionId};
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "dokime.execution.executions.append", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppendExecutions {
    pub cycle_id: TestCycleId,
    pub max_executions: u64,
    pub executions: Vec<(TestExecutionId, CaseId)>,
    pub occurred_at: Timestamp,
    pub settings: BTreeMap<String, BTreeMap<String, Value>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Transaction<RuntimeState> for AppendExecutions {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        super::ensure_runtime_active(state)?;
        prepare(state, self).map(|_| ())
    }

    fn apply(&self, state: &mut RuntimeState) {
        let cycle = prepare(state, self).expect("Harmos checked explicit execution append");
        state.replace_cycle(cycle).expect("appended executions have validated ownership");
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn prepare(state: &RuntimeState, append: &AppendExecutions) -> Result<super::CycleView, String> {
    if append.executions.is_empty() || append.executions.len() as u64 > append.max_executions {
        return Err("append requires a nonempty plan within the configured execution limit".into());
    }
    let mut unique = BTreeSet::new();
    for (id, _) in &append.executions {
        if !unique.insert(id) {
            return Err("append repeats an execution identity".into());
        }
    }

    let mut cycle = state.cycle_view(&append.cycle_id).ok_or("cycle not found")?;
    let existing = append.executions.iter().filter(|(id, _)| state.executions.contains_key(id)).count();
    if existing != 0 {
        check_retry(&cycle, append)?;
        return Ok(cycle);
    }

    let cases = append
        .executions
        .iter()
        .map(|(_, id)| state.suites.case(id).cloned().ok_or_else(|| format!("registered case {id} not found")))
        .collect::<Result<Vec<_>, String>>()?;
    state.suites.validate_cases(&cases)?;
    for case in &cases {
        let revision = state.case_definition_revision(case)?;
        if state.suites.case_revisions().get(&case.test_case_id) != Some(&revision) {
            return Err("materialized case definition or provider changed; materialize a new case".into());
        }
    }
    super::append_cases(&mut cycle, cases.clone(), append.occurred_at.clone())?;
    let ids = append.executions.iter().map(|(id, _)| id.clone());
    super::materialize_executions(state, &mut cycle, &cases, ids, &append.settings).map_err(|error| error.to_string())?;
    state.validate_cycle_view(&cycle)?;
    Ok(cycle)
}

fn check_retry(cycle: &super::CycleView, append: &AppendExecutions) -> Result<(), String> {
    let first = &append.executions[0].0;
    let offset = cycle
        .test_executions
        .iter()
        .position(|execution| &execution.test_execution_id == first)
        .ok_or("execution identity already belongs to another append")?;
    let members = cycle.test_executions.iter().skip(offset).take(append.executions.len());
    if members.count() != append.executions.len() {
        return Err("partial execution append conflicts with existing membership".into());
    }
    for (execution, (id, case)) in cycle.test_executions.iter().skip(offset).zip(&append.executions) {
        if &execution.test_execution_id != id || &execution.snapshot.case().test_case_id != case {
            return Err("execution identity has a different case, cycle or order".into());
        }
    }
    Ok(())
}

/* === Tests === */
