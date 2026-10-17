/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod append_planned_cases;
mod copy_cycle;
mod create_cycle;
mod delete_case;
mod get_case;
mod get_cycle;
mod get_execution;
mod get_execution_command;
mod get_run_request;
mod list_case_parameters;
mod list_cases;
mod list_cycles;
mod list_execution_parameters;
mod list_execution_steps;
mod list_executions;
mod list_step_criteria;
mod pause_cycle;
mod pause_execution;
mod rename_cycle;
mod rename_execution;
mod resume_cycle;
mod resume_execution;
mod run_case;
mod run_cycle;
mod run_execution;
mod schedule_case_rerun;
mod set_cycle_archived;
mod set_execution_archived;
mod stop_cycle;
mod stop_execution;
mod submit_execution_command;
mod update_cycle_policy;

/* === Re-Exports === */
pub use append_planned_cases::AppendPlannedCases;
pub use copy_cycle::CopyCycle;
pub use create_cycle::CreateCycle;
pub use delete_case::DeleteCase;
pub use get_case::GetCase;
pub use get_cycle::GetCycle;
pub use get_execution::GetExecution;
pub use get_execution_command::GetExecutionCommand;
pub use get_run_request::GetRunRequest;
pub use list_case_parameters::ListCaseParameters;
pub use list_cases::ListCases;
pub use list_cycles::ListCycles;
pub use list_execution_parameters::ListExecutionParameters;
pub use list_execution_steps::ListExecutionSteps;
pub use list_executions::ListExecutions;
pub use list_step_criteria::ListStepCriteria;
pub use pause_cycle::PauseCycle;
pub use pause_execution::PauseExecution;
pub use rename_cycle::RenameCycle;
pub use rename_execution::RenameExecution;
pub use resume_cycle::ResumeCycle;
pub use resume_execution::ResumeExecution;
pub use run_case::RunCase;
pub use run_cycle::RunCycle;
pub use run_execution::RunExecution;
pub use schedule_case_rerun::ScheduleCaseRerun;
pub use set_cycle_archived::SetCycleArchived;
pub use set_execution_archived::SetExecutionArchived;
pub use stop_cycle::StopCycle;
pub use stop_execution::StopExecution;
pub use submit_execution_command::SubmitExecutionCommand;
pub use update_cycle_policy::UpdateCyclePolicy;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Access;
use super::Endpoint;
use super::ORDINARY;
#[allow(unused_imports)]
use crate::model::definition::wire::*;
use crate::model::execution::wire_cursor::Cursor;
use crate::model::identity::wire::Metadata;
#[allow(unused_imports)]
use crate::model::identity::wire::*;
#[allow(unused_imports)]
use crate::model::planning::wire::*;

use crate::model::execution::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Copies the current plan, authored Definition/Operator cycle inputs, documents,
// and policy into a new ready cycle. No execution evidence or derived parameters
// are copied. The source remains untouched, including if currently running.
// Copy does not certify provider availability; Run validates that separately.

// Append cases generated from the current registered catalog to an existing
// mutable cycle. The plan is resolved and validated against one captured
// registry snapshot before the existing append transaction is committed.

// `filter` and `order` must be repeated unchanged on every continuation, and
// `after_cycle_id` must still be a member of the filtered result. A changed
// selection, a changed order, or a continuation ID the filter excludes is
// rejected rather than silently restarted.

// Archiving is orthogonal to the lifecycle and admitted only on a terminal
// cycle or execution: it hides the row from lists that do not ask for archived
// rows and changes nothing else.

// A rename is admitted until the subject seals: a sealed, archived or History
// cycle and an execution whose case has a sealed attempt refuse it with
// FailedPrecondition, since the sealed report froze the label. An execution's
// label is its case's label, so every attempt of that case is renamed together.

// DeleteCase only removes an unexecuted case. ScheduleCaseRerun makes the
// cycle ready without altering prior attempts or starting execution. A subsequent
// RunCase dispatches the next attempt; prior attempts and evidence remain retained.

// Optional keys are exact opaque values: 1–256 UTF-8 bytes, nonblank, no control
// characters. Namespace: stable node + authenticated principal, shared across
// RunCycle and RunCase. Reuse requires the identical cycle/case selection;
// correlation labels do not affect identity. Persist a key before first send.
// Repeating the original request resolves its durable receipt after restart;
// it never resumes an interrupted acquisition. Omission provides no retry safety.

// Lookup is independent of live cycles and epoch cursors. A Read principal may
// inspect its own receipts; Admin may inspect any. Unknown and other-owner IDs
// both return NotFound. A native receipt with no submitter is Admin-only.

// Commands belong to an admitted interactive run. Reuse the same command UUID
// and identical action to recover admission; a transport failure is uncertain.

// Detail pages preserve native planned-case / execution-attempt vector order.
// Ordinals are zero-based positions at this cursor, not permanent identities.
// Continuations require the same epoch/revision and an existing after-ID in the
// requested cycle; changed revisions abort and unknown markers are invalid.

// Current-runtime detail. Published historical checkpoints are a separate API.

/* === Enums === */

#[harmos_domain::catalog]
pub enum Operations {
    ListExecutionSteps(ListExecutionSteps),
    ListStepCriteria(ListStepCriteria),
    CopyCycle(CopyCycle),
    CreateCycle(CreateCycle),
    GetCycle(GetCycle),
    GetCase(GetCase),
    GetExecution(GetExecution),
    ListCaseParameters(ListCaseParameters),
    ListExecutionParameters(ListExecutionParameters),
    ListCycles(ListCycles),
    ListCases(ListCases),
    ListExecutions(ListExecutions),
    AppendPlannedCases(AppendPlannedCases),
    RunCycle(RunCycle),
    RunCase(RunCase),
    RunExecution(RunExecution),
    PauseExecution(PauseExecution),
    ResumeExecution(ResumeExecution),
    StopExecution(StopExecution),
    GetRunRequest(GetRunRequest),
    SubmitExecutionCommand(SubmitExecutionCommand),
    GetExecutionCommand(GetExecutionCommand),
    PauseCycle(PauseCycle),
    ResumeCycle(ResumeCycle),
    StopCycle(StopCycle),
    DeleteCase(DeleteCase),
    ScheduleCaseRerun(ScheduleCaseRerun),
    SetCycleArchived(SetCycleArchived),
    SetExecutionArchived(SetExecutionArchived),
    RenameCycle(RenameCycle),
    RenameExecution(RenameExecution),
    UpdateCyclePolicy(UpdateCyclePolicy),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for ListExecutionSteps {
    const PATH: &'static str = "/dokime.v1.Execution/ListExecutionSteps";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListStepCriteria {
    const PATH: &'static str = "/dokime.v1.Execution/ListStepCriteria";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for CopyCycle {
    const PATH: &'static str = "/dokime.v1.Execution/CopyCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for CreateCycle {
    const PATH: &'static str = "/dokime.v1.Execution/CreateCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for GetCycle {
    const PATH: &'static str = "/dokime.v1.Execution/GetCycle";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetCase {
    const PATH: &'static str = "/dokime.v1.Execution/GetCase";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetExecution {
    const PATH: &'static str = "/dokime.v1.Execution/GetExecution";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListCaseParameters {
    const PATH: &'static str = "/dokime.v1.Execution/ListCaseParameters";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListExecutionParameters {
    const PATH: &'static str = "/dokime.v1.Execution/ListExecutionParameters";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListCycles {
    const PATH: &'static str = "/dokime.v1.Execution/ListCycles";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListCases {
    const PATH: &'static str = "/dokime.v1.Execution/ListCases";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListExecutions {
    const PATH: &'static str = "/dokime.v1.Execution/ListExecutions";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for AppendPlannedCases {
    const PATH: &'static str = "/dokime.v1.Execution/AppendPlannedCases";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for RunCycle {
    const PATH: &'static str = "/dokime.v1.Execution/RunCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for RunCase {
    const PATH: &'static str = "/dokime.v1.Execution/RunCase";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for RunExecution {
    const PATH: &'static str = "/dokime.v1.Execution/RunExecution";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for PauseExecution {
    const PATH: &'static str = "/dokime.v1.Execution/PauseExecution";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for ResumeExecution {
    const PATH: &'static str = "/dokime.v1.Execution/ResumeExecution";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for StopExecution {
    const PATH: &'static str = "/dokime.v1.Execution/StopExecution";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for GetRunRequest {
    const PATH: &'static str = "/dokime.v1.Execution/GetRunRequest";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for SubmitExecutionCommand {
    const PATH: &'static str = "/dokime.v1.Execution/SubmitExecutionCommand";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for GetExecutionCommand {
    const PATH: &'static str = "/dokime.v1.Execution/GetExecutionCommand";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for PauseCycle {
    const PATH: &'static str = "/dokime.v1.Execution/PauseCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for ResumeCycle {
    const PATH: &'static str = "/dokime.v1.Execution/ResumeCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for StopCycle {
    const PATH: &'static str = "/dokime.v1.Execution/StopCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for DeleteCase {
    const PATH: &'static str = "/dokime.v1.Execution/DeleteCase";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for ScheduleCaseRerun {
    const PATH: &'static str = "/dokime.v1.Execution/ScheduleCaseRerun";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for SetCycleArchived {
    const PATH: &'static str = "/dokime.v1.Execution/SetCycleArchived";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for SetExecutionArchived {
    const PATH: &'static str = "/dokime.v1.Execution/SetExecutionArchived";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for RenameCycle {
    const PATH: &'static str = "/dokime.v1.Execution/RenameCycle";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for RenameExecution {
    const PATH: &'static str = "/dokime.v1.Execution/RenameExecution";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for UpdateCyclePolicy {
    const PATH: &'static str = "/dokime.v1.Execution/UpdateCyclePolicy";
    const ACCESS: Access = Access::Control;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
