/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Derive Work Changes From The Checked Transition Without Reimplementing It

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::state::Transaction;

// Local Imports
use crate::state::RuntimeState;
use dokime_domain::{
    model::{execution::result::TestCycle, identity::TestCycleId},
    state::records::work::{Cycle, Execution, Work},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn collect(
    state: &RuntimeState, transition: &impl Transaction<RuntimeState>, cycle: Option<&TestCycleId>, out: &mut harmos::state::Records,
) {
    let mut after = state.clone();
    transition.apply(&mut after);
    let mut work = Work {
        executions: Vec::new(),
        cycles: Vec::new(),
    };
    if let Some(id) = cycle {
        if let Some(cycle) = after.cycles.get(id) {
            collect_cycle(state, &after, cycle, &mut work);
        }
    } else {
        for cycle in after.cycles.values() {
            collect_cycle(state, &after, cycle, &mut work);
        }
    }
    if !work.executions.is_empty() || !work.cycles.is_empty() {
        let _ = out.push(work);
    }
}

/* === Private Functions === */

fn collect_cycle(before: &RuntimeState, after: &RuntimeState, cycle: &TestCycle, work: &mut Work) {
    for id in &cycle.execution_ids {
        let (Some(previous), Some(execution)) = (before.executions.get(id), after.executions.get(id)) else { continue };
        if previous.status == execution.status
            && previous.started_at == execution.started_at
            && previous.finished_at == execution.finished_at
            && previous.result == execution.result
        {
            continue;
        }
        work.executions.push(Execution::from(execution));
    }
    if before.cycles.get(&cycle.test_cycle_id).is_some_and(|previous| previous.lifecycle != cycle.lifecycle) {
        work.cycles.push(Cycle::from(cycle));
    }
}

/* === Tests === */
