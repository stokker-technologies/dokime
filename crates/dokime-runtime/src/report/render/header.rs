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

// Local Imports
use crate::{
    document::capture::Capture,
    report::{input::CycleReportInput, value::Value},
    runtime::storage::cycle::CycleSnapshot,
};
use crate::{
    core::{DocumentId, Metadata, SessionId, TestCycleId, Timestamp},
    execution::{CycleStatus, ExecutionStatus, ResultStatus, ResultSummary},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy)]
pub(in crate::report) struct Header<'a> {
    pub session_id: &'a SessionId,
    pub cycle: Cycle<'a>,
    pub documents: &'a BTreeMap<DocumentId, Capture>,
    pub plots: Option<&'a BTreeMap<String, Value>>,
    pub generated_at: &'a Timestamp,
}

#[derive(Clone, Copy)]
pub(in crate::report) struct Cycle<'a> {
    pub test_cycle_id: &'a TestCycleId,
    pub metadata: &'a Metadata,
    pub lifecycle: CycleStatus,
    pub created_at: &'a Timestamp,
    pub result: Option<&'a ResultSummary>,
    pub continue_on_failure: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<'a> Header<'a> {
    pub(in crate::report) fn cycle(
        snapshot: &'a CycleSnapshot,
        documents: &'a BTreeMap<DocumentId, Capture>,
    ) -> Self {
        let source = snapshot.source();
        Self {
            session_id: snapshot.session_id(),
            cycle: Cycle {
                test_cycle_id: &source.test_cycle_id,
                metadata: &source.metadata,
                lifecycle: source.lifecycle,
                created_at: &source.created_at,
                result: source.result.as_ref(),
                continue_on_failure: source.continue_on_failure,
            },
            documents,
            plots: None,
            generated_at: snapshot.captured_at(),
        }
    }
}

impl<'a> From<&'a CycleReportInput> for Header<'a> {
    fn from(input: &'a CycleReportInput) -> Self {
        Self {
            session_id: &input.session_id,
            cycle: Cycle {
                test_cycle_id: &input.cycle.test_cycle_id,
                metadata: &input.cycle.metadata,
                lifecycle: captured_status(input),
                created_at: &input.cycle.created_at,
                result: input.cycle.result.as_ref(),
                continue_on_failure: input.cycle.continue_on_failure,
            },
            documents: &input.documents,
            plots: Some(&input.plots),
            generated_at: &input.generated_at,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
// A Captured Report States Its Cycle By Its Members: Running, Else Its Result.
fn captured_status(input: &CycleReportInput) -> CycleStatus {
    if input
        .cycle
        .test_executions
        .iter()
        .any(|execution| matches!(execution.status, ExecutionStatus::Running))
    {
        return CycleStatus::Running;
    }
    match input.cycle.result.as_ref().map(|result| &result.status) {
        Some(ResultStatus::Passed) => CycleStatus::Completed,
        Some(ResultStatus::Failed | ResultStatus::Blocked) => CycleStatus::Failed,
        Some(ResultStatus::Cancelled) => CycleStatus::Stopped,
        Some(ResultStatus::Pending) | None => CycleStatus::Ready,
    }
}

/* === Tests === */
