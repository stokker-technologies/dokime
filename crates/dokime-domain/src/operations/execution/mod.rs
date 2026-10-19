/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod append_executions;
mod copy_cycle;
mod create_cycle;
mod delete_case;
mod get_case;
mod get_cycle;
mod get_execution;
mod get_execution_command;
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
mod seal_cycle;
mod set_cycle_archived;
mod set_execution_archived;
mod start_execution;
mod stop_execution;
mod submit_execution_command;
mod update_cycle_policy;

/* === Re-Exports === */

pub use append_executions::{AppendExecutions, ExecutionBinding};
pub use copy_cycle::CopyCycle;
pub use create_cycle::CreateCycle;
pub use delete_case::DeleteCase;
pub use get_case::GetCase;
pub use get_cycle::GetCycle;
pub use get_execution::GetExecution;
pub use get_execution_command::GetExecutionCommand;
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
pub use seal_cycle::SealCycle;
pub use set_cycle_archived::SetCycleArchived;
pub use set_execution_archived::SetExecutionArchived;
pub use start_execution::StartExecution;
pub use stop_execution::StopExecution;
pub use submit_execution_command::SubmitExecutionCommand;
pub use update_cycle_policy::UpdateCyclePolicy;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Endpoint;
use super::ORDINARY;

#[allow(unused_imports)]
use crate::model::definition::wire::*;
use crate::model::execution::wire_cursor::Cursor;
use crate::model::identity::wire::Metadata;

#[allow(unused_imports)]
use crate::model::identity::wire::*;

use crate::model::execution::wire::*;
#[allow(unused_imports)]
use crate::model::planning::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

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

// DeleteCase Only Removes An Unexecuted Case. Reruns Bind A New Explicit Execution Identity.

// Commands belong to an admitted interactive run. Reuse the same command UUID

// and identical action to recover admission; a transport failure is uncertain.

// Detail pages preserve native planned-case / execution-attempt vector order.

// Ordinals are zero-based positions at this cursor, not permanent identities.

// Continuations require the same epoch/revision and an existing after-ID in the

// requested cycle; changed revisions abort and unknown markers are invalid.

// Current-runtime detail. Published historical checkpoints are a separate API.

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
    AppendExecutions(AppendExecutions),
    StartExecution(StartExecution),
    PauseExecution(PauseExecution),
    ResumeExecution(ResumeExecution),
    StopExecution(StopExecution),
    SubmitExecutionCommand(SubmitExecutionCommand),
    GetExecutionCommand(GetExecutionCommand),
    PauseCycle(PauseCycle),
    ResumeCycle(ResumeCycle),
    SealCycle(SealCycle),
    DeleteCase(DeleteCase),
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
}

impl Endpoint for ListStepCriteria {
    const PATH: &'static str = "/dokime.v1.Execution/ListStepCriteria";
}

impl Endpoint for CopyCycle {
    const PATH: &'static str = "/dokime.v1.Execution/CopyCycle";
}

impl Endpoint for CreateCycle {
    const PATH: &'static str = "/dokime.v1.Execution/CreateCycle";
}

impl Endpoint for GetCycle {
    const PATH: &'static str = "/dokime.v1.Execution/GetCycle";
}

impl Endpoint for GetCase {
    const PATH: &'static str = "/dokime.v1.Execution/GetCase";
}

impl Endpoint for GetExecution {
    const PATH: &'static str = "/dokime.v1.Execution/GetExecution";
}

impl Endpoint for ListCaseParameters {
    const PATH: &'static str = "/dokime.v1.Execution/ListCaseParameters";
}

impl Endpoint for ListExecutionParameters {
    const PATH: &'static str = "/dokime.v1.Execution/ListExecutionParameters";
}

impl Endpoint for ListCycles {
    const PATH: &'static str = "/dokime.v1.Execution/ListCycles";
}

impl Endpoint for ListCases {
    const PATH: &'static str = "/dokime.v1.Execution/ListCases";
}

impl Endpoint for ListExecutions {
    const PATH: &'static str = "/dokime.v1.Execution/ListExecutions";
}

impl Endpoint for AppendExecutions {
    const PATH: &'static str = "/dokime.v1.Execution/AppendExecutions";
}

impl Endpoint for StartExecution {
    const PATH: &'static str = "/dokime.v1.Execution/StartExecution";
}

impl Endpoint for PauseExecution {
    const PATH: &'static str = "/dokime.v1.Execution/PauseExecution";
}

impl Endpoint for ResumeExecution {
    const PATH: &'static str = "/dokime.v1.Execution/ResumeExecution";
}

impl Endpoint for StopExecution {
    const PATH: &'static str = "/dokime.v1.Execution/StopExecution";
}

impl Endpoint for SubmitExecutionCommand {
    const PATH: &'static str = "/dokime.v1.Execution/SubmitExecutionCommand";
}

impl Endpoint for GetExecutionCommand {
    const PATH: &'static str = "/dokime.v1.Execution/GetExecutionCommand";
}

impl Endpoint for PauseCycle {
    const PATH: &'static str = "/dokime.v1.Execution/PauseCycle";
}

impl Endpoint for ResumeCycle {
    const PATH: &'static str = "/dokime.v1.Execution/ResumeCycle";
}

impl Endpoint for DeleteCase {
    const PATH: &'static str = "/dokime.v1.Execution/DeleteCase";
}

impl Endpoint for SetCycleArchived {
    const PATH: &'static str = "/dokime.v1.Execution/SetCycleArchived";
}

impl Endpoint for SetExecutionArchived {
    const PATH: &'static str = "/dokime.v1.Execution/SetExecutionArchived";
}

impl Endpoint for RenameCycle {
    const PATH: &'static str = "/dokime.v1.Execution/RenameCycle";
}

impl Endpoint for RenameExecution {
    const PATH: &'static str = "/dokime.v1.Execution/RenameExecution";
}

impl Endpoint for SealCycle {
    const PATH: &'static str = "/dokime.v1.Execution/SealCycle";
}

impl Endpoint for UpdateCyclePolicy {
    const PATH: &'static str = "/dokime.v1.Execution/UpdateCyclePolicy";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
