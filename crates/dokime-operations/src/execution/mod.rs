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
mod cancel_execution;
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
mod pause_cycle;
mod pause_execution;
mod rename_cycle;
mod rename_execution;
mod resume_cycle;
mod resume_execution;
mod run_provider_cycle;
mod seal_cycle;
mod set_cycle_archived;
mod set_execution_archived;
mod start_execution;
mod stop_execution;
mod submit_execution_command;
pub mod transport;
mod update_cycle_policy;

/* === Re-Exports === */

pub use append_executions::{AppendExecutions, ExecutionBinding};
pub use cancel_execution::CancelExecution;
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
pub use pause_cycle::PauseCycle;
pub use pause_execution::PauseExecution;
pub use rename_cycle::RenameCycle;
pub use rename_execution::RenameExecution;
pub use resume_cycle::ResumeCycle;
pub use resume_execution::ResumeExecution;
pub use run_provider_cycle::RunProviderCycle;
pub use seal_cycle::SealCycle;
pub use set_cycle_archived::SetCycleArchived;
pub use set_execution_archived::SetExecutionArchived;
pub use start_execution::StartExecution;
pub use stop_execution::StopExecution;
pub use submit_execution_command::SubmitExecutionCommand;
pub use transport::{EvidenceBatch, LeaseAsk, OperatorQuestion};
pub use update_cycle_policy::UpdateCyclePolicy;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{MUTATION, ORDINARY};

use dokime_domain::model::execution::wire_cursor::Cursor;
use dokime_domain::model::identity::wire::Metadata;

use dokime_domain::model::execution::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

// Detail pages preserve native planned-case / execution-attempt vector order.
// Ordinals are zero-based positions at this cursor, not permanent identities.
// Continuations require the same epoch/revision and an existing after-ID in the
// requested cycle; changed revisions abort and unknown markers are invalid.
// Current-runtime detail. Published historical checkpoints are a separate API.

#[harmos::catalog]
pub enum Operations {
    ListExecutionSteps(ListExecutionSteps),
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
    CancelExecution(CancelExecution),
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

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
