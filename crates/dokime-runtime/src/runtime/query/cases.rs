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

pub use dokime_domain::model::execution::result::TestCaseStatus as CaseStatus;

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::state::RuntimeState;
use crate::transactions::cycle::ensure_case_deletable;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::TestCycle;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

const HISTORICAL: &str = "historical checkpoints cannot be modified";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseProjection {
    pub status: Option<CaseStatus>,
    pub latest_execution_id: Option<TestExecutionId>,
    pub latest_execution_status: Option<ExecutionStatus>,
    pub attempt_count: u32,
    pub actions: CaseActions,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseTally {
    pub running: u64,
    pub queued: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseActions {
    pub delete: bool,
    pub unavailable_reasons: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn case_projection(_state: &RuntimeState, cycle: &CycleView, test_case_id: &CaseId) -> CaseProjection {
    let attempts = attempts(cycle, test_case_id);
    let latest = attempts.last().copied();
    let mut actions = CaseActions {
        delete: true,
        unavailable_reasons: Vec::new(),
    };
    admit(&mut actions.delete, &mut actions.unavailable_reasons, ensure_case_deletable(cycle, test_case_id));

    CaseProjection {
        status: Some(status(latest)),
        latest_execution_id: latest.map(|execution| execution.test_execution_id.clone()),
        latest_execution_status: latest.map(|execution| execution.status.clone()),
        attempt_count: attempts.len() as u32,
        actions,
    }
}

pub fn case_tally(state: &RuntimeState, cycle: &TestCycle) -> CaseTally {
    let mut latest = BTreeMap::new();
    for execution in state.executions_for_cycle(cycle) {
        latest.insert(&execution.test_case_id, execution);
    }

    let mut tally = CaseTally::default();
    for execution in latest.into_values() {
        match status(Some(execution)) {
            CaseStatus::Running => tally.running += 1,
            CaseStatus::Planned | CaseStatus::Queued => tally.queued += 1,
            _ => {}
        }
    }
    tally
}

pub fn historical_case_projection(cycle: &CycleView, test_case_id: &CaseId) -> CaseProjection {
    let (latest, attempt_count) = historical_attempts(cycle, test_case_id);
    CaseProjection {
        status: latest.map(|execution| status(Some(execution))),
        latest_execution_id: latest.map(|execution| execution.test_execution_id.clone()),
        latest_execution_status: latest.map(|execution| execution.status.clone()),
        attempt_count,
        actions: CaseActions {
            unavailable_reasons: vec![HISTORICAL.to_owned()],
            ..Default::default()
        },
    }
}

/* === Private Functions === */

fn historical_attempts<'a>(cycle: &'a CycleView, test_case_id: &CaseId) -> (Option<&'a TestExecution>, u32) {
    cycle
        .test_executions
        .iter()
        .filter(|execution| &execution.test_case_id == test_case_id)
        .fold((None, 0u32), |(_, count), execution| (Some(execution), count.wrapping_add(1)))
}

fn attempts<'a>(cycle: &'a CycleView, test_case_id: &CaseId) -> Vec<&'a TestExecution> {
    cycle.test_executions.iter().filter(|execution| &execution.test_case_id == test_case_id).collect()
}

fn status(latest: Option<&TestExecution>) -> CaseStatus {
    let Some(execution) = latest else {
        return CaseStatus::Planned;
    };
    // A materialized attempt nobody dispatched is the case's queued work, not
    // a skip: `Skipped` is how the runtime spells "planned" on an attempt.
    if execution.status == ExecutionStatus::Skipped && execution.started_at.is_none() && execution.finished_at.is_none() {
        return CaseStatus::Queued;
    }
    match execution.status {
        ExecutionStatus::Skipped => CaseStatus::Skipped,
        ExecutionStatus::Invalidated => CaseStatus::Invalidated,
        ExecutionStatus::Pending => CaseStatus::Queued,
        ExecutionStatus::Running => CaseStatus::Running,
        ExecutionStatus::Passed => CaseStatus::Passed,
        ExecutionStatus::Failed => CaseStatus::Failed,
        ExecutionStatus::Blocked => CaseStatus::Blocked,
        ExecutionStatus::Cancelled => CaseStatus::Cancelled,
        ExecutionStatus::Interrupted => CaseStatus::Interrupted,
    }
}

fn admit(allowed: &mut bool, reasons: &mut Vec<String>, outcome: Result<(), String>) {
    if let Err(reason) = outcome {
        *allowed = false;
        if !reasons.contains(&reason) {
            reasons.push(reason);
        }
    }
}

/* === Tests === */
