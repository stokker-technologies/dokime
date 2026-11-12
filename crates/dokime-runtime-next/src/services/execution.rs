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
        let reply = dokime_operations_next::client::execution_append_executions::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunAppendExecutionsStream = dokime_operations_next::client::Output<v1::ExecutionAppendExecutionsRunOutput>;
    async fn run_append_executions(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionAppendExecutionsRunInput>>,
    ) -> Result<tonic::Response<Self::RunAppendExecutionsStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionAppendExecutionsRunInput| match input.value? {
                v1::execution_append_executions_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_append_executions_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_append_executions_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionAppendExecutionsRunOutput {
                value: Some(v1::execution_append_executions_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_append_executions::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn cancel_execution(
        &self, request: tonic::Request<v1::ExecutionCancelExecutionRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_cancel_execution::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunCancelExecutionStream = dokime_operations_next::client::Output<v1::ExecutionCancelExecutionRunOutput>;
    async fn run_cancel_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionCancelExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunCancelExecutionStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionCancelExecutionRunInput| match input.value? {
                v1::execution_cancel_execution_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_cancel_execution_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_cancel_execution_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionCancelExecutionRunOutput {
                value: Some(v1::execution_cancel_execution_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_cancel_execution::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn create_cycle(
        &self, request: tonic::Request<v1::ExecutionCreateCycleRequest>,
    ) -> Result<tonic::Response<v1::CreateCycleReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_create_cycle::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunCreateCycleStream = dokime_operations_next::client::Output<v1::ExecutionCreateCycleRunOutput>;
    async fn run_create_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionCreateCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunCreateCycleStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionCreateCycleRunInput| match input.value? {
                v1::execution_create_cycle_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_create_cycle_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_create_cycle_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionCreateCycleRunOutput {
                value: Some(v1::execution_create_cycle_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_create_cycle::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn delete_case(
        &self, request: tonic::Request<v1::ExecutionDeleteCaseRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_delete_case::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunDeleteCaseStream = dokime_operations_next::client::Output<v1::ExecutionDeleteCaseRunOutput>;
    async fn run_delete_case(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionDeleteCaseRunInput>>,
    ) -> Result<tonic::Response<Self::RunDeleteCaseStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionDeleteCaseRunInput| match input.value? {
                v1::execution_delete_case_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_delete_case_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_delete_case_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionDeleteCaseRunOutput {
                value: Some(v1::execution_delete_case_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_delete_case::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn get_case(
        &self, request: tonic::Request<v1::ExecutionGetCaseRequest>,
    ) -> Result<tonic::Response<v1::CaseDetail>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_get_case::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetCaseStream = dokime_operations_next::client::Output<v1::ExecutionGetCaseRunOutput>;
    async fn run_get_case(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionGetCaseRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetCaseStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionGetCaseRunInput| match input.value? {
                v1::execution_get_case_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_get_case_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_get_case_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionGetCaseRunOutput {
                value: Some(v1::execution_get_case_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_get_case::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn get_cycle(
        &self, request: tonic::Request<v1::ExecutionGetCycleRequest>,
    ) -> Result<tonic::Response<v1::CycleSummary>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_get_cycle::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetCycleStream = dokime_operations_next::client::Output<v1::ExecutionGetCycleRunOutput>;
    async fn run_get_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionGetCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetCycleStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionGetCycleRunInput| match input.value? {
                v1::execution_get_cycle_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_get_cycle_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_get_cycle_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionGetCycleRunOutput {
                value: Some(v1::execution_get_cycle_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_get_cycle::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn get_execution(
        &self, request: tonic::Request<v1::ExecutionGetExecutionRequest>,
    ) -> Result<tonic::Response<v1::ExecutionDetail>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_get_execution::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetExecutionStream = dokime_operations_next::client::Output<v1::ExecutionGetExecutionRunOutput>;
    async fn run_get_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionGetExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetExecutionStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionGetExecutionRunInput| match input.value? {
                v1::execution_get_execution_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_get_execution_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_get_execution_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionGetExecutionRunOutput {
                value: Some(v1::execution_get_execution_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_get_execution::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn get_execution_command(
        &self, request: tonic::Request<v1::ExecutionGetExecutionCommandRequest>,
    ) -> Result<tonic::Response<v1::ExecutionCommandReceipt>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_get_execution_command::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetExecutionCommandStream = dokime_operations_next::client::Output<v1::ExecutionGetExecutionCommandRunOutput>;
    async fn run_get_execution_command(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionGetExecutionCommandRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetExecutionCommandStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply =
            dokime_operations_next::client::conversation::unary(
                self.context.clone(),
                request.into_inner(),
                |input: v1::ExecutionGetExecutionCommandRunInput| match input.value? {
                    v1::execution_get_execution_command_run_input::Value::Request(request) => Some(Input::Request(request)),
                    v1::execution_get_execution_command_run_input::Value::Cancel(_) => Some(Input::Cancel),
                    v1::execution_get_execution_command_run_input::Value::Finish(_) => Some(Input::Finish),
                },
                |response| v1::ExecutionGetExecutionCommandRunOutput {
                    value: Some(v1::execution_get_execution_command_run_output::Value::Response(response)),
                },
                |context, request| async move {
                    dokime_operations_next::client::execution_get_execution_command::handle(&context, request).await
                },
            )
            .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_case_parameters(
        &self, request: tonic::Request<v1::ExecutionListCaseParametersRequest>,
    ) -> Result<tonic::Response<v1::ListCaseParametersReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_list_case_parameters::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListCaseParametersStream = dokime_operations_next::client::Output<v1::ExecutionListCaseParametersRunOutput>;
    async fn run_list_case_parameters(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionListCaseParametersRunInput>>,
    ) -> Result<tonic::Response<Self::RunListCaseParametersStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply =
            dokime_operations_next::client::conversation::unary(
                self.context.clone(),
                request.into_inner(),
                |input: v1::ExecutionListCaseParametersRunInput| match input.value? {
                    v1::execution_list_case_parameters_run_input::Value::Request(request) => Some(Input::Request(request)),
                    v1::execution_list_case_parameters_run_input::Value::Cancel(_) => Some(Input::Cancel),
                    v1::execution_list_case_parameters_run_input::Value::Finish(_) => Some(Input::Finish),
                },
                |response| v1::ExecutionListCaseParametersRunOutput {
                    value: Some(v1::execution_list_case_parameters_run_output::Value::Response(response)),
                },
                |context, request| async move {
                    dokime_operations_next::client::execution_list_case_parameters::handle(&context, request).await
                },
            )
            .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_cases(
        &self, request: tonic::Request<v1::ExecutionListCasesRequest>,
    ) -> Result<tonic::Response<v1::ListCasesReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_list_cases::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListCasesStream = dokime_operations_next::client::Output<v1::ExecutionListCasesRunOutput>;
    async fn run_list_cases(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionListCasesRunInput>>,
    ) -> Result<tonic::Response<Self::RunListCasesStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionListCasesRunInput| match input.value? {
                v1::execution_list_cases_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_list_cases_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_list_cases_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionListCasesRunOutput {
                value: Some(v1::execution_list_cases_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_list_cases::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_cycles(
        &self, request: tonic::Request<v1::ExecutionListCyclesRequest>,
    ) -> Result<tonic::Response<v1::ListCyclesReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_list_cycles::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListCyclesStream = dokime_operations_next::client::Output<v1::ExecutionListCyclesRunOutput>;
    async fn run_list_cycles(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionListCyclesRunInput>>,
    ) -> Result<tonic::Response<Self::RunListCyclesStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionListCyclesRunInput| match input.value? {
                v1::execution_list_cycles_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_list_cycles_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_list_cycles_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionListCyclesRunOutput {
                value: Some(v1::execution_list_cycles_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_list_cycles::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_execution_parameters(
        &self, request: tonic::Request<v1::ExecutionListExecutionParametersRequest>,
    ) -> Result<tonic::Response<v1::ListMaterializedParametersReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_list_execution_parameters::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListExecutionParametersStream = dokime_operations_next::client::Output<v1::ExecutionListExecutionParametersRunOutput>;
    async fn run_list_execution_parameters(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionListExecutionParametersRunInput>>,
    ) -> Result<tonic::Response<Self::RunListExecutionParametersStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionListExecutionParametersRunInput| match input.value? {
                v1::execution_list_execution_parameters_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_list_execution_parameters_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_list_execution_parameters_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionListExecutionParametersRunOutput {
                value: Some(v1::execution_list_execution_parameters_run_output::Value::Response(response)),
            },
            |context, request| async move {
                dokime_operations_next::client::execution_list_execution_parameters::handle(&context, request).await
            },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_execution_steps(
        &self, request: tonic::Request<v1::ExecutionListExecutionStepsRequest>,
    ) -> Result<tonic::Response<v1::ListExecutionStepsReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_list_execution_steps::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListExecutionStepsStream = dokime_operations_next::client::Output<v1::ExecutionListExecutionStepsRunOutput>;
    async fn run_list_execution_steps(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionListExecutionStepsRunInput>>,
    ) -> Result<tonic::Response<Self::RunListExecutionStepsStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply =
            dokime_operations_next::client::conversation::unary(
                self.context.clone(),
                request.into_inner(),
                |input: v1::ExecutionListExecutionStepsRunInput| match input.value? {
                    v1::execution_list_execution_steps_run_input::Value::Request(request) => Some(Input::Request(request)),
                    v1::execution_list_execution_steps_run_input::Value::Cancel(_) => Some(Input::Cancel),
                    v1::execution_list_execution_steps_run_input::Value::Finish(_) => Some(Input::Finish),
                },
                |response| v1::ExecutionListExecutionStepsRunOutput {
                    value: Some(v1::execution_list_execution_steps_run_output::Value::Response(response)),
                },
                |context, request| async move {
                    dokime_operations_next::client::execution_list_execution_steps::handle(&context, request).await
                },
            )
            .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_executions(
        &self, request: tonic::Request<v1::ExecutionListExecutionsRequest>,
    ) -> Result<tonic::Response<v1::ListExecutionsReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_list_executions::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListExecutionsStream = dokime_operations_next::client::Output<v1::ExecutionListExecutionsRunOutput>;
    async fn run_list_executions(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionListExecutionsRunInput>>,
    ) -> Result<tonic::Response<Self::RunListExecutionsStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionListExecutionsRunInput| match input.value? {
                v1::execution_list_executions_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_list_executions_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_list_executions_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionListExecutionsRunOutput {
                value: Some(v1::execution_list_executions_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_list_executions::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn pause_cycle(
        &self, request: tonic::Request<v1::ExecutionPauseCycleRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_pause_cycle::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunPauseCycleStream = dokime_operations_next::client::Output<v1::ExecutionPauseCycleRunOutput>;
    async fn run_pause_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionPauseCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunPauseCycleStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionPauseCycleRunInput| match input.value? {
                v1::execution_pause_cycle_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_pause_cycle_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_pause_cycle_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionPauseCycleRunOutput {
                value: Some(v1::execution_pause_cycle_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_pause_cycle::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn pause_execution(
        &self, request: tonic::Request<v1::ExecutionPauseExecutionRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_pause_execution::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunPauseExecutionStream = dokime_operations_next::client::Output<v1::ExecutionPauseExecutionRunOutput>;
    async fn run_pause_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionPauseExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunPauseExecutionStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionPauseExecutionRunInput| match input.value? {
                v1::execution_pause_execution_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_pause_execution_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_pause_execution_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionPauseExecutionRunOutput {
                value: Some(v1::execution_pause_execution_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_pause_execution::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn rename_cycle(
        &self, request: tonic::Request<v1::ExecutionRenameCycleRequest>,
    ) -> Result<tonic::Response<v1::RenameReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_rename_cycle::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunRenameCycleStream = dokime_operations_next::client::Output<v1::ExecutionRenameCycleRunOutput>;
    async fn run_rename_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionRenameCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunRenameCycleStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionRenameCycleRunInput| match input.value? {
                v1::execution_rename_cycle_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_rename_cycle_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_rename_cycle_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionRenameCycleRunOutput {
                value: Some(v1::execution_rename_cycle_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_rename_cycle::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn rename_execution(
        &self, request: tonic::Request<v1::ExecutionRenameExecutionRequest>,
    ) -> Result<tonic::Response<v1::RenameReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_rename_execution::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunRenameExecutionStream = dokime_operations_next::client::Output<v1::ExecutionRenameExecutionRunOutput>;
    async fn run_rename_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionRenameExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunRenameExecutionStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionRenameExecutionRunInput| match input.value? {
                v1::execution_rename_execution_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_rename_execution_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_rename_execution_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionRenameExecutionRunOutput {
                value: Some(v1::execution_rename_execution_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_rename_execution::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn rerun_cycle(
        &self, request: tonic::Request<v1::ExecutionRerunCycleRequest>,
    ) -> Result<tonic::Response<v1::RerunCycleReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_rerun_cycle::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunRerunCycleStream = dokime_operations_next::client::Output<v1::ExecutionRerunCycleRunOutput>;
    async fn run_rerun_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionRerunCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunRerunCycleStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionRerunCycleRunInput| match input.value? {
                v1::execution_rerun_cycle_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_rerun_cycle_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_rerun_cycle_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionRerunCycleRunOutput {
                value: Some(v1::execution_rerun_cycle_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_rerun_cycle::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn resume_cycle(
        &self, request: tonic::Request<v1::ExecutionResumeCycleRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_resume_cycle::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunResumeCycleStream = dokime_operations_next::client::Output<v1::ExecutionResumeCycleRunOutput>;
    async fn run_resume_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionResumeCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunResumeCycleStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionResumeCycleRunInput| match input.value? {
                v1::execution_resume_cycle_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_resume_cycle_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_resume_cycle_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionResumeCycleRunOutput {
                value: Some(v1::execution_resume_cycle_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_resume_cycle::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn resume_execution(
        &self, request: tonic::Request<v1::ExecutionResumeExecutionRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_resume_execution::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunResumeExecutionStream = dokime_operations_next::client::Output<v1::ExecutionResumeExecutionRunOutput>;
    async fn run_resume_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionResumeExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunResumeExecutionStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionResumeExecutionRunInput| match input.value? {
                v1::execution_resume_execution_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_resume_execution_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_resume_execution_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionResumeExecutionRunOutput {
                value: Some(v1::execution_resume_execution_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_resume_execution::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn seal_cycle(
        &self, request: tonic::Request<v1::ExecutionSealCycleRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_seal_cycle::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunSealCycleStream = dokime_operations_next::client::Output<v1::ExecutionSealCycleRunOutput>;
    async fn run_seal_cycle(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionSealCycleRunInput>>,
    ) -> Result<tonic::Response<Self::RunSealCycleStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionSealCycleRunInput| match input.value? {
                v1::execution_seal_cycle_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_seal_cycle_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_seal_cycle_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionSealCycleRunOutput {
                value: Some(v1::execution_seal_cycle_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_seal_cycle::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn set_cycle_archived(
        &self, request: tonic::Request<v1::ExecutionSetCycleArchivedRequest>,
    ) -> Result<tonic::Response<v1::ArchiveReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_set_cycle_archived::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunSetCycleArchivedStream = dokime_operations_next::client::Output<v1::ExecutionSetCycleArchivedRunOutput>;
    async fn run_set_cycle_archived(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionSetCycleArchivedRunInput>>,
    ) -> Result<tonic::Response<Self::RunSetCycleArchivedStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionSetCycleArchivedRunInput| match input.value? {
                v1::execution_set_cycle_archived_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_set_cycle_archived_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_set_cycle_archived_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionSetCycleArchivedRunOutput {
                value: Some(v1::execution_set_cycle_archived_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_set_cycle_archived::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn set_execution_archived(
        &self, request: tonic::Request<v1::ExecutionSetExecutionArchivedRequest>,
    ) -> Result<tonic::Response<v1::ArchiveReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_set_execution_archived::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunSetExecutionArchivedStream = dokime_operations_next::client::Output<v1::ExecutionSetExecutionArchivedRunOutput>;
    async fn run_set_execution_archived(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionSetExecutionArchivedRunInput>>,
    ) -> Result<tonic::Response<Self::RunSetExecutionArchivedStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply =
            dokime_operations_next::client::conversation::unary(
                self.context.clone(),
                request.into_inner(),
                |input: v1::ExecutionSetExecutionArchivedRunInput| match input.value? {
                    v1::execution_set_execution_archived_run_input::Value::Request(request) => Some(Input::Request(request)),
                    v1::execution_set_execution_archived_run_input::Value::Cancel(_) => Some(Input::Cancel),
                    v1::execution_set_execution_archived_run_input::Value::Finish(_) => Some(Input::Finish),
                },
                |response| v1::ExecutionSetExecutionArchivedRunOutput {
                    value: Some(v1::execution_set_execution_archived_run_output::Value::Response(response)),
                },
                |context, request| async move {
                    dokime_operations_next::client::execution_set_execution_archived::handle(&context, request).await
                },
            )
            .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn start_execution(
        &self, request: tonic::Request<v1::ExecutionStartExecutionRequest>,
    ) -> Result<tonic::Response<v1::StartExecutionReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_start_execution::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunStartExecutionStream = dokime_operations_next::client::Output<v1::ExecutionStartExecutionRunOutput>;
    async fn run_start_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionStartExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunStartExecutionStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionStartExecutionRunInput| match input.value? {
                v1::execution_start_execution_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_start_execution_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_start_execution_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionStartExecutionRunOutput {
                value: Some(v1::execution_start_execution_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_start_execution::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn stop_execution(
        &self, request: tonic::Request<v1::ExecutionStopExecutionRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_stop_execution::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunStopExecutionStream = dokime_operations_next::client::Output<v1::ExecutionStopExecutionRunOutput>;
    async fn run_stop_execution(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionStopExecutionRunInput>>,
    ) -> Result<tonic::Response<Self::RunStopExecutionStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionStopExecutionRunInput| match input.value? {
                v1::execution_stop_execution_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_stop_execution_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_stop_execution_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionStopExecutionRunOutput {
                value: Some(v1::execution_stop_execution_run_output::Value::Response(response)),
            },
            |context, request| async move { dokime_operations_next::client::execution_stop_execution::handle(&context, request).await },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn submit_execution_command(
        &self, request: tonic::Request<v1::ExecutionSubmitExecutionCommandRequest>,
    ) -> Result<tonic::Response<v1::ExecutionCommandReceipt>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_submit_execution_command::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunSubmitExecutionCommandStream = dokime_operations_next::client::Output<v1::ExecutionSubmitExecutionCommandRunOutput>;
    async fn run_submit_execution_command(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionSubmitExecutionCommandRunInput>>,
    ) -> Result<tonic::Response<Self::RunSubmitExecutionCommandStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(
            self.context.clone(),
            request.into_inner(),
            |input: v1::ExecutionSubmitExecutionCommandRunInput| match input.value? {
                v1::execution_submit_execution_command_run_input::Value::Request(request) => Some(Input::Request(request)),
                v1::execution_submit_execution_command_run_input::Value::Cancel(_) => Some(Input::Cancel),
                v1::execution_submit_execution_command_run_input::Value::Finish(_) => Some(Input::Finish),
            },
            |response| v1::ExecutionSubmitExecutionCommandRunOutput {
                value: Some(v1::execution_submit_execution_command_run_output::Value::Response(response)),
            },
            |context, request| async move {
                dokime_operations_next::client::execution_submit_execution_command::handle(&context, request).await
            },
        )
        .await?;
        Ok(tonic::Response::new(reply))
    }
    async fn update_cycle_policy(
        &self, request: tonic::Request<v1::ExecutionUpdateCyclePolicyRequest>,
    ) -> Result<tonic::Response<v1::ControlReply>, tonic::Status> {
        let reply = dokime_operations_next::client::execution_update_cycle_policy::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        let reply = dokime_operations_next::client::bounded_reply(&self.context, reply).map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunUpdateCyclePolicyStream = dokime_operations_next::client::Output<v1::ExecutionUpdateCyclePolicyRunOutput>;
    async fn run_update_cycle_policy(
        &self, request: tonic::Request<tonic::Streaming<v1::ExecutionUpdateCyclePolicyRunInput>>,
    ) -> Result<tonic::Response<Self::RunUpdateCyclePolicyStream>, tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(), request.into_inner(), |input: v1::ExecutionUpdateCyclePolicyRunInput| match input.value? {
            v1::execution_update_cycle_policy_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::execution_update_cycle_policy_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::execution_update_cycle_policy_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::ExecutionUpdateCyclePolicyRunOutput { value: Some(v1::execution_update_cycle_policy_run_output::Value::Response(response)) }, |context, request| async move { dokime_operations_next::client::execution_update_cycle_policy::handle(&context, request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
