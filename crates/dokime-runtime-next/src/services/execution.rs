/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::execution_server::Execution for Services {
    async fn append_executions(
        &self, request: tonic::Request<v1::ExecutionAppendExecutionsRequest>,
    ) -> Result<tonic::Response<v1::AppendCasesReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.AppendExecutions: Not Yet Served by the New Host"))
    }
    type RunAppendExecutionsStream = dokime_operations_next::client::Output<v1::ExecutionAppendExecutionsRunOutput>;
    async fn run_append_executions(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionAppendExecutionsRunInput>>,
    ) -> Result<tonic::Response<Self::RunAppendExecutionsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunAppendExecutions: Not Yet Served by the New Host"))
    }
    async fn cancel_execution(
        &self, request: tonic::Request<v1::ExecutionCancelExecutionRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.CancelExecution: Not Yet Served by the New Host"))
    }
    type RunCancelExecutionStream = dokime_operations_next::client::Output<v1::ExecutionCancelExecutionRunOutput>;
    async fn run_cancel_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionCancelExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunCancelExecutionStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunCancelExecution: Not Yet Served by the New Host"))
    }
    async fn create_cycle(
        &self, request: tonic::Request<v1::ExecutionCreateCycleRequest>,
    ) -> Result<tonic::Response<v1::CreateCycleReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_create_cycle::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunCreateCycleStream = dokime_operations_next::client::Output<v1::ExecutionCreateCycleRunOutput>;
    async fn run_create_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionCreateCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunCreateCycleStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunCreateCycle: Not Yet Served by the New Host"))
    }
    async fn delete_case(
        &self, request: tonic::Request<v1::ExecutionDeleteCaseRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.DeleteCase: Not Yet Served by the New Host"))
    }
    type RunDeleteCaseStream = dokime_operations_next::client::Output<v1::ExecutionDeleteCaseRunOutput>;
    async fn run_delete_case(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionDeleteCaseRunInput>>,
    ) -> Result<tonic::Response<Self::RunDeleteCaseStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunDeleteCase: Not Yet Served by the New Host"))
    }
    async fn get_case(
        &self, request: tonic::Request<v1::ExecutionGetCaseRequest>,
    ) -> Result<tonic::Response<v1::CaseDetail>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.GetCase: Not Yet Served by the New Host"))
    }
    type RunGetCaseStream = dokime_operations_next::client::Output<v1::ExecutionGetCaseRunOutput>;
    async fn run_get_case(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionGetCaseRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetCaseStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunGetCase: Not Yet Served by the New Host"))
    }
    async fn get_cycle(
        &self, request: tonic::Request<v1::ExecutionGetCycleRequest>,
    ) -> Result<tonic::Response<v1::CycleSummary>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.GetCycle: Not Yet Served by the New Host"))
    }
    type RunGetCycleStream = dokime_operations_next::client::Output<v1::ExecutionGetCycleRunOutput>;
    async fn run_get_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionGetCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetCycleStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunGetCycle: Not Yet Served by the New Host"))
    }
    async fn get_execution(
        &self, request: tonic::Request<v1::ExecutionGetExecutionRequest>,
    ) -> Result<tonic::Response<v1::ExecutionDetail>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_get_execution::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetExecutionStream = dokime_operations_next::client::Output<v1::ExecutionGetExecutionRunOutput>;
    async fn run_get_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionGetExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetExecutionStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunGetExecution: Not Yet Served by the New Host"))
    }
    async fn get_execution_command(
        &self, request: tonic::Request<v1::ExecutionGetExecutionCommandRequest>,
    ) -> Result<tonic::Response<v1::ExecutionCommandReceipt>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.GetExecutionCommand: Not Yet Served by the New Host"))
    }
    type RunGetExecutionCommandStream = dokime_operations_next::client::Output<v1::ExecutionGetExecutionCommandRunOutput>;
    async fn run_get_execution_command(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionGetExecutionCommandRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetExecutionCommandStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunGetExecutionCommand: Not Yet Served by the New Host"))
    }
    async fn list_case_parameters(
        &self, request: tonic::Request<v1::ExecutionListCaseParametersRequest>,
    ) -> Result<tonic::Response<v1::ListCaseParametersReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.ListCaseParameters: Not Yet Served by the New Host"))
    }
    type RunListCaseParametersStream = dokime_operations_next::client::Output<v1::ExecutionListCaseParametersRunOutput>;
    async fn run_list_case_parameters(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionListCaseParametersRunInput>>,
    ) -> Result<tonic::Response<Self::RunListCaseParametersStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunListCaseParameters: Not Yet Served by the New Host"))
    }
    async fn list_cases(
        &self, request: tonic::Request<v1::ExecutionListCasesRequest>,
    ) -> Result<tonic::Response<v1::ListCasesReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.ListCases: Not Yet Served by the New Host"))
    }
    type RunListCasesStream = dokime_operations_next::client::Output<v1::ExecutionListCasesRunOutput>;
    async fn run_list_cases(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionListCasesRunInput>>,
    ) -> Result<tonic::Response<Self::RunListCasesStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunListCases: Not Yet Served by the New Host"))
    }
    async fn list_cycles(
        &self, request: tonic::Request<v1::ExecutionListCyclesRequest>,
    ) -> Result<tonic::Response<v1::ListCyclesReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.ListCycles: Not Yet Served by the New Host"))
    }
    type RunListCyclesStream = dokime_operations_next::client::Output<v1::ExecutionListCyclesRunOutput>;
    async fn run_list_cycles(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionListCyclesRunInput>>,
    ) -> Result<tonic::Response<Self::RunListCyclesStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunListCycles: Not Yet Served by the New Host"))
    }
    async fn list_execution_parameters(
        &self, request: tonic::Request<v1::ExecutionListExecutionParametersRequest>,
    ) -> Result<tonic::Response<v1::ListMaterializedParametersReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.ListExecutionParameters: Not Yet Served by the New Host"))
    }
    type RunListExecutionParametersStream = dokime_operations_next::client::Output<v1::ExecutionListExecutionParametersRunOutput>;
    async fn run_list_execution_parameters(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionListExecutionParametersRunInput>>,
    ) -> Result<tonic::Response<Self::RunListExecutionParametersStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunListExecutionParameters: Not Yet Served by the New Host"))
    }
    async fn list_execution_steps(
        &self, request: tonic::Request<v1::ExecutionListExecutionStepsRequest>,
    ) -> Result<tonic::Response<v1::ListExecutionStepsReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.ListExecutionSteps: Not Yet Served by the New Host"))
    }
    type RunListExecutionStepsStream = dokime_operations_next::client::Output<v1::ExecutionListExecutionStepsRunOutput>;
    async fn run_list_execution_steps(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionListExecutionStepsRunInput>>,
    ) -> Result<tonic::Response<Self::RunListExecutionStepsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunListExecutionSteps: Not Yet Served by the New Host"))
    }
    async fn list_executions(
        &self, request: tonic::Request<v1::ExecutionListExecutionsRequest>,
    ) -> Result<tonic::Response<v1::ListExecutionsReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.ListExecutions: Not Yet Served by the New Host"))
    }
    type RunListExecutionsStream = dokime_operations_next::client::Output<v1::ExecutionListExecutionsRunOutput>;
    async fn run_list_executions(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionListExecutionsRunInput>>,
    ) -> Result<tonic::Response<Self::RunListExecutionsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunListExecutions: Not Yet Served by the New Host"))
    }
    async fn pause_cycle(
        &self, request: tonic::Request<v1::ExecutionPauseCycleRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.PauseCycle: Not Yet Served by the New Host"))
    }
    type RunPauseCycleStream = dokime_operations_next::client::Output<v1::ExecutionPauseCycleRunOutput>;
    async fn run_pause_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionPauseCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunPauseCycleStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunPauseCycle: Not Yet Served by the New Host"))
    }
    async fn pause_execution(
        &self, request: tonic::Request<v1::ExecutionPauseExecutionRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.PauseExecution: Not Yet Served by the New Host"))
    }
    type RunPauseExecutionStream = dokime_operations_next::client::Output<v1::ExecutionPauseExecutionRunOutput>;
    async fn run_pause_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionPauseExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunPauseExecutionStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunPauseExecution: Not Yet Served by the New Host"))
    }
    async fn rename_cycle(
        &self, request: tonic::Request<v1::ExecutionRenameCycleRequest>,
    ) -> Result<tonic::Response<v1::RenameReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RenameCycle: Not Yet Served by the New Host"))
    }
    type RunRenameCycleStream = dokime_operations_next::client::Output<v1::ExecutionRenameCycleRunOutput>;
    async fn run_rename_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionRenameCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunRenameCycleStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunRenameCycle: Not Yet Served by the New Host"))
    }
    async fn rename_execution(
        &self, request: tonic::Request<v1::ExecutionRenameExecutionRequest>,
    ) -> Result<tonic::Response<v1::RenameReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RenameExecution: Not Yet Served by the New Host"))
    }
    type RunRenameExecutionStream = dokime_operations_next::client::Output<v1::ExecutionRenameExecutionRunOutput>;
    async fn run_rename_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionRenameExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunRenameExecutionStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunRenameExecution: Not Yet Served by the New Host"))
    }
    async fn rerun_cycle(
        &self, request: tonic::Request<v1::ExecutionRerunCycleRequest>,
    ) -> Result<tonic::Response<v1::RerunCycleReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RerunCycle: Not Yet Served by the New Host"))
    }
    type RunRerunCycleStream = dokime_operations_next::client::Output<v1::ExecutionRerunCycleRunOutput>;
    async fn run_rerun_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionRerunCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunRerunCycleStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunRerunCycle: Not Yet Served by the New Host"))
    }
    async fn resume_cycle(
        &self, request: tonic::Request<v1::ExecutionResumeCycleRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.ResumeCycle: Not Yet Served by the New Host"))
    }
    type RunResumeCycleStream = dokime_operations_next::client::Output<v1::ExecutionResumeCycleRunOutput>;
    async fn run_resume_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionResumeCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunResumeCycleStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunResumeCycle: Not Yet Served by the New Host"))
    }
    async fn resume_execution(
        &self, request: tonic::Request<v1::ExecutionResumeExecutionRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.ResumeExecution: Not Yet Served by the New Host"))
    }
    type RunResumeExecutionStream = dokime_operations_next::client::Output<v1::ExecutionResumeExecutionRunOutput>;
    async fn run_resume_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionResumeExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunResumeExecutionStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunResumeExecution: Not Yet Served by the New Host"))
    }
    async fn seal_cycle(
        &self, request: tonic::Request<v1::ExecutionSealCycleRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.SealCycle: Not Yet Served by the New Host"))
    }
    type RunSealCycleStream = dokime_operations_next::client::Output<v1::ExecutionSealCycleRunOutput>;
    async fn run_seal_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionSealCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunSealCycleStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunSealCycle: Not Yet Served by the New Host"))
    }
    async fn set_cycle_archived(
        &self, request: tonic::Request<v1::ExecutionSetCycleArchivedRequest>,
    ) -> Result<tonic::Response<v1::ArchiveReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.SetCycleArchived: Not Yet Served by the New Host"))
    }
    type RunSetCycleArchivedStream = dokime_operations_next::client::Output<v1::ExecutionSetCycleArchivedRunOutput>;
    async fn run_set_cycle_archived(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionSetCycleArchivedRunInput>>,
    ) -> Result<tonic::Response<Self::RunSetCycleArchivedStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunSetCycleArchived: Not Yet Served by the New Host"))
    }
    async fn set_execution_archived(
        &self, request: tonic::Request<v1::ExecutionSetExecutionArchivedRequest>,
    ) -> Result<tonic::Response<v1::ArchiveReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.SetExecutionArchived: Not Yet Served by the New Host"))
    }
    type RunSetExecutionArchivedStream = dokime_operations_next::client::Output<v1::ExecutionSetExecutionArchivedRunOutput>;
    async fn run_set_execution_archived(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionSetExecutionArchivedRunInput>>,
    ) -> Result<tonic::Response<Self::RunSetExecutionArchivedStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunSetExecutionArchived: Not Yet Served by the New Host"))
    }
    async fn start_execution(
        &self, request: tonic::Request<v1::ExecutionStartExecutionRequest>,
    ) -> Result<tonic::Response<v1::StartExecutionReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.StartExecution: Not Yet Served by the New Host"))
    }
    type RunStartExecutionStream = dokime_operations_next::client::Output<v1::ExecutionStartExecutionRunOutput>;
    async fn run_start_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionStartExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunStartExecutionStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunStartExecution: Not Yet Served by the New Host"))
    }
    async fn stop_execution(
        &self, request: tonic::Request<v1::ExecutionStopExecutionRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.StopExecution: Not Yet Served by the New Host"))
    }
    type RunStopExecutionStream = dokime_operations_next::client::Output<v1::ExecutionStopExecutionRunOutput>;
    async fn run_stop_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionStopExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunStopExecutionStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunStopExecution: Not Yet Served by the New Host"))
    }
    async fn submit_execution_command(
        &self, request: tonic::Request<v1::ExecutionSubmitExecutionCommandRequest>,
    ) -> Result<tonic::Response<v1::ExecutionCommandReceipt>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.SubmitExecutionCommand: Not Yet Served by the New Host"))
    }
    type RunSubmitExecutionCommandStream = dokime_operations_next::client::Output<v1::ExecutionSubmitExecutionCommandRunOutput>;
    async fn run_submit_execution_command(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionSubmitExecutionCommandRunInput>>,
    ) -> Result<tonic::Response<Self::RunSubmitExecutionCommandStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunSubmitExecutionCommand: Not Yet Served by the New Host"))
    }
    async fn update_cycle_policy(
        &self, request: tonic::Request<v1::ExecutionUpdateCyclePolicyRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.UpdateCyclePolicy: Not Yet Served by the New Host"))
    }
    type RunUpdateCyclePolicyStream = dokime_operations_next::client::Output<v1::ExecutionUpdateCyclePolicyRunOutput>;
    async fn run_update_cycle_policy(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionUpdateCyclePolicyRunInput>>,
    ) -> Result<tonic::Response<Self::RunUpdateCyclePolicyStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Execution.RunUpdateCyclePolicy: Not Yet Served by the New Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
