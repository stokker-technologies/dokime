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
pub use crate::execution::TestCaseStatus as CaseStatus;

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use crate::execution::{CycleView, ExecutionStatus, TestCycle, TestExecution};
use crate::state::RuntimeState;
use crate::transactions::cycle::{ensure_case_deletable, ensure_case_restartable};
use crate::transactions::provider::{ensure_case_runnable, planned_attempt};
use crate::core::{CaseId, TestExecutionId};

/* === Constants === */
const HISTORICAL: &str = "historical checkpoints cannot be modified";

const NO_MATERIALIZED_ATTEMPT: &str = "case has no materialized execution";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseProjection {
    pub status: Option<CaseStatus>,
    pub latest_execution_id: Option<TestExecutionId>,
    pub latest_execution_status: Option<ExecutionStatus>,
    pub attempt_count: u32,
    pub rerun_scheduled: bool,
    pub actions: CaseActions,
}

/// A cycle's planned cases counted by their latest attempt's projected status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseTally {
    /// Cases whose latest attempt is running.
    pub running: u64,
    /// Cases still waiting to run: planned, undispatched, or admitted but not started.
    pub queued: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseActions {
    pub run: bool,
    pub schedule_rerun: bool,
    pub delete: bool,
    pub unavailable_reasons: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl CaseProjection {
    pub fn historical_memory(cycle: &CycleView, test_case_id: &CaseId) -> usize {
        let (latest, _) = historical_attempts(cycle, test_case_id);
        std::mem::size_of::<Self>()
            .saturating_add(
                latest.map_or(0, |execution| execution.test_execution_id.as_str().len()),
            )
            .saturating_add(std::mem::size_of::<String>())
            .saturating_add(HISTORICAL.len())
    }
}

/* === Public Functions === */
pub fn case_projection(
    state: &RuntimeState,
    cycle: &CycleView,
    test_case_id: &CaseId,
) -> CaseProjection {
    let attempts = attempts(cycle, test_case_id);
    let latest = attempts.last().copied();
    let mut actions = CaseActions {
        run: true,
        schedule_rerun: true,
        delete: true,
        unavailable_reasons: Vec::new(),
    };

    admit(
        &mut actions.run,
        &mut actions.unavailable_reasons,
        ensure_case_runnable(state, cycle, test_case_id),
    );
    admit(
        &mut actions.schedule_rerun,
        &mut actions.unavailable_reasons,
        restartable(cycle, test_case_id, "schedule a case rerun"),
    );
    admit(
        &mut actions.delete,
        &mut actions.unavailable_reasons,
        ensure_case_deletable(cycle, test_case_id),
    );

    CaseProjection {
        status: Some(status(latest)),
        latest_execution_id: latest.map(|execution| execution.test_execution_id.clone()),
        latest_execution_status: latest.map(|execution| execution.status.clone()),
        attempt_count: attempts.len() as u32,
        rerun_scheduled: rerun_scheduled(cycle, test_case_id, &attempts),
        actions,
    }
}

/// Count a cycle's cases by the status their latest attempt projects, in one
/// pass over the cycle's attempts.
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
        rerun_scheduled: false,
        actions: CaseActions {
            unavailable_reasons: vec![HISTORICAL.to_owned()],
            ..Default::default()
        },
    }
}

/* === Private Functions === */
fn historical_attempts<'a>(
    cycle: &'a CycleView,
    test_case_id: &CaseId,
) -> (Option<&'a TestExecution>, u32) {
    cycle
        .test_executions
        .iter()
        .filter(|execution| &execution.test_case_id == test_case_id)
        .fold((None, 0u32), |(_, count), execution| {
            (Some(execution), count.wrapping_add(1))
        })
}

fn attempts<'a>(cycle: &'a CycleView, test_case_id: &CaseId) -> Vec<&'a TestExecution> {
    cycle
        .test_executions
        .iter()
        .filter(|execution| &execution.test_case_id == test_case_id)
        .collect()
}

fn status(latest: Option<&TestExecution>) -> CaseStatus {
    let Some(execution) = latest else {
        return CaseStatus::Planned;
    };
    // A materialized attempt nobody dispatched is the case's queued work, not
    // a skip: `Skipped` is how the runtime spells "planned" on an attempt.
    if execution.status == ExecutionStatus::Skipped
        && execution.started_at.is_none()
        && execution.finished_at.is_none()
    {
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

fn rerun_scheduled(cycle: &CycleView, test_case_id: &CaseId, attempts: &[&TestExecution]) -> bool {
    planned_attempt(cycle, test_case_id).is_some()
        && attempts.iter().rev().skip(1).any(|execution| {
            execution.finished_at.is_some() || execution.status != ExecutionStatus::Skipped
        })
}

fn restartable(cycle: &CycleView, test_case_id: &CaseId, operation: &str) -> Result<(), String> {
    crate::transactions::cycle::ensure_cycle_mutable(cycle)?;
    ensure_case_restartable(cycle, test_case_id, operation)?;
    if cycle
        .test_executions
        .iter()
        .all(|execution| &execution.test_case_id != test_case_id)
    {
        return Err(NO_MATERIALIZED_ATTEMPT.to_owned());
    }
    Ok(())
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
