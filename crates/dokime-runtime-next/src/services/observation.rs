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
impl v1::observation_server::Observation for Services {
    async fn get_runtime_shell(&self, request: tonic::Request<v1::ObservationGetRuntimeShellRequest>) -> Result<tonic::Response<v1::RuntimeShellSnapshot>,tonic::Status> {
        let reply = dokime_operations_next::client::observation_get_runtime_shell::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetRuntimeShellStream = dokime_operations_next::client::Output<v1::ObservationGetRuntimeShellRunOutput>;
    async fn run_get_runtime_shell(&self, request: tonic::Request<tonic::Streaming<v1::ObservationGetRuntimeShellRunInput>>) -> Result<tonic::Response<Self::RunGetRuntimeShellStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(),request.into_inner(),|input: v1::ObservationGetRuntimeShellRunInput| match input.value? {
            v1::observation_get_runtime_shell_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::observation_get_runtime_shell_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::observation_get_runtime_shell_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::ObservationGetRuntimeShellRunOutput { value: Some(v1::observation_get_runtime_shell_run_output::Value::Response(response)) }, |context,request| async move { dokime_operations_next::client::observation_get_runtime_shell::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    async fn get_traceability(&self, request: tonic::Request<v1::ObservationGetTraceabilityRequest>) -> Result<tonic::Response<v1::TraceabilityPage>,tonic::Status> {
        let reply = dokime_operations_next::client::observation_get_traceability::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetTraceabilityStream = dokime_operations_next::client::Output<v1::ObservationGetTraceabilityRunOutput>;
    async fn run_get_traceability(&self, request: tonic::Request<tonic::Streaming<v1::ObservationGetTraceabilityRunInput>>) -> Result<tonic::Response<Self::RunGetTraceabilityStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(),request.into_inner(),|input: v1::ObservationGetTraceabilityRunInput| match input.value? {
            v1::observation_get_traceability_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::observation_get_traceability_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::observation_get_traceability_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::ObservationGetTraceabilityRunOutput { value: Some(v1::observation_get_traceability_run_output::Value::Response(response)) }, |context,request| async move { dokime_operations_next::client::observation_get_traceability::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    type WatchExecutionProgressStream = dokime_operations_next::client::Output<v1::ObservationWatchExecutionProgressOutput>;
    async fn watch_execution_progress(&self, request: tonic::Request<v1::ObservationWatchExecutionProgressRequest>) -> Result<tonic::Response<Self::WatchExecutionProgressStream>,tonic::Status> {
        let reply = dokime_operations_next::client::observation_watch_execution_progress::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunWatchExecutionProgressStream = dokime_operations_next::client::Output<v1::ObservationWatchExecutionProgressRunOutput>;
    async fn run_watch_execution_progress(&self, request: tonic::Request<tonic::Streaming<v1::ObservationWatchExecutionProgressRunInput>>) -> Result<tonic::Response<Self::RunWatchExecutionProgressStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::streaming(self.context.clone(),request.into_inner(),|input: v1::ObservationWatchExecutionProgressRunInput| match input.value? {
            v1::observation_watch_execution_progress_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::observation_watch_execution_progress_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::observation_watch_execution_progress_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |output: v1::ObservationWatchExecutionProgressOutput| v1::ObservationWatchExecutionProgressRunOutput { value: output.value.map(|value| match value { v1::observation_watch_execution_progress_output::Value::Item(item) => v1::observation_watch_execution_progress_run_output::Value::Item(item),v1::observation_watch_execution_progress_output::Value::Response(response) => v1::observation_watch_execution_progress_run_output::Value::Response(response) }) }, |context,request| async move { dokime_operations_next::client::observation_watch_execution_progress::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    type WatchRuntimeStream = dokime_operations_next::client::Output<v1::ObservationWatchRuntimeOutput>;
    async fn watch_runtime(&self, request: tonic::Request<v1::ObservationWatchRuntimeRequest>) -> Result<tonic::Response<Self::WatchRuntimeStream>,tonic::Status> {
        let reply = dokime_operations_next::client::observation_watch_runtime::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunWatchRuntimeStream = dokime_operations_next::client::Output<v1::ObservationWatchRuntimeRunOutput>;
    async fn run_watch_runtime(&self, request: tonic::Request<tonic::Streaming<v1::ObservationWatchRuntimeRunInput>>) -> Result<tonic::Response<Self::RunWatchRuntimeStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::streaming(self.context.clone(),request.into_inner(),|input: v1::ObservationWatchRuntimeRunInput| match input.value? {
            v1::observation_watch_runtime_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::observation_watch_runtime_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::observation_watch_runtime_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |output: v1::ObservationWatchRuntimeOutput| v1::ObservationWatchRuntimeRunOutput { value: output.value.map(|value| match value { v1::observation_watch_runtime_output::Value::Item(item) => v1::observation_watch_runtime_run_output::Value::Item(item),v1::observation_watch_runtime_output::Value::Response(response) => v1::observation_watch_runtime_run_output::Value::Response(response) }) }, |context,request| async move { dokime_operations_next::client::observation_watch_runtime::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
