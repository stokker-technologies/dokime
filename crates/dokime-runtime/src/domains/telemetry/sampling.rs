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

// Local Imports
use crate::execution::{ExecutionStatus, StepStatus};
use crate::{execution::CycleStatus, runtime::state::RuntimeState};
use crate::core::{CaseId, RecordScope, TestCycleId, TestExecutionId};
use crate::telemetry::SamplingScope;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SamplingPhaseState {
    Running,
    Paused,
    Stopped,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/* === Functions === */

pub(crate) fn record_scope_from_sampling_scope(scope: &SamplingScope) -> RecordScope {
    let mut record_scope = RecordScope::default();
    if let Some(test_cycle_id) = &scope.test_cycle_id
        && let Ok(test_cycle_id) = TestCycleId::from_uuid(test_cycle_id.clone())
    {
        record_scope = record_scope.with_cycle(test_cycle_id);
    }
    if let Some(case_id) = &scope.test_case_id
        && let Ok(case_id) = CaseId::from_wire(case_id.clone())
    {
        record_scope = record_scope.with_test_case(case_id);
    }
    if let Some(test_execution_id) = &scope.test_execution_id
        && let Ok(test_execution_id) = TestExecutionId::from_uuid(test_execution_id.clone())
    {
        record_scope = record_scope.with_test_execution(test_execution_id);
    }
    record_scope
}

pub(crate) fn sampling_phase_state(
    state: &RuntimeState,
    scope: &RecordScope,
    start_after: Option<&str>,
) -> SamplingPhaseState {
    let execution = scope
        .test_execution_id
        .as_ref()
        .and_then(|id| state.executions.get(id));
    if scope.test_execution_id.is_some() && execution.is_none() {
        return SamplingPhaseState::Stopped;
    }
    // Pending executions own their capture before the first recorded step.
    if let Some(execution) = execution
        && !matches!(
            execution.status,
            ExecutionStatus::Pending | ExecutionStatus::Running
        )
    {
        return SamplingPhaseState::Stopped;
    }
    let test_cycle_id = scope
        .test_cycle_id
        .as_ref()
        .or_else(|| execution.map(|execution| &execution.test_cycle_id));
    let Some(test_cycle_id) = test_cycle_id else {
        return SamplingPhaseState::Running;
    };
    if execution.is_some_and(|execution| &execution.test_cycle_id != test_cycle_id) {
        return SamplingPhaseState::Stopped;
    }
    let Some(cycle) = state.cycles.get(test_cycle_id) else {
        return SamplingPhaseState::Stopped;
    };
    match cycle.lifecycle {
        CycleStatus::Running => {}
        CycleStatus::Paused => return SamplingPhaseState::Paused,
        _ => return SamplingPhaseState::Stopped,
    }
    let Some(start_after) = start_after else {
        return SamplingPhaseState::Running;
    };

    let started = execution
        .into_iter()
        .chain(
            scope
                .test_execution_id
                .is_none()
                .then_some(())
                .into_iter()
                .flat_map(|_| state.executions_for_cycle(cycle)),
        )
        .any(|execution| {
            execution.steps.iter().any(|step| {
                step.metadata.identifier.as_str() == start_after
                    && matches!(
                        step.status,
                        StepStatus::Passed
                            | StepStatus::Failed
                            | StepStatus::Skipped
                            | StepStatus::Cancelled
                    )
            })
        });
    if started {
        SamplingPhaseState::Running
    } else {
        SamplingPhaseState::Paused
    }
}

/* === Private Functions === */

/* === Tests === */
