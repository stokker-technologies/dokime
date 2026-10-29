/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

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

use super::Endpoint;
use super::{MUTATION, ORDINARY};

#[allow(unused_imports)]
use dokime_domain::model::definition::wire::*;
use dokime_domain::model::execution::wire_cursor::Cursor;
use dokime_domain::model::identity::wire::Metadata;

#[allow(unused_imports)]
use dokime_domain::model::identity::wire::*;

use dokime_domain::model::execution::wire::*;
#[allow(unused_imports)]
use dokime_domain::model::planning::wire::*;

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

impl Endpoint for ListExecutionSteps {
    const PATH: &'static str = "/dokime.v1.Execution/ListExecutionSteps";
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

impl Endpoint for CancelExecution {
    const PATH: &'static str = "/dokime.v1.Execution/CancelExecution";
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
