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
    async fn get_runtime_shell(
        &self, request: tonic::Request<v1::ObservationGetRuntimeShellRequest>,
    ) -> Result<tonic::Response<v1::RuntimeShellSnapshot>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Observation.GetRuntimeShell: Not Yet Served by the New Host"))
    }
    type RunGetRuntimeShellStream = dokime_operations_next::client::Output<v1::ObservationGetRuntimeShellRunOutput>;
    async fn run_get_runtime_shell(
        &self, request: tonic::Request<tonic::Streaming<v1::ObservationGetRuntimeShellRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetRuntimeShellStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Observation.RunGetRuntimeShell: Not Yet Served by the New Host"))
    }
    async fn get_traceability(
        &self, request: tonic::Request<v1::ObservationGetTraceabilityRequest>,
    ) -> Result<tonic::Response<v1::TraceabilityPage>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Observation.GetTraceability: Not Yet Served by the New Host"))
    }
    type RunGetTraceabilityStream = dokime_operations_next::client::Output<v1::ObservationGetTraceabilityRunOutput>;
    async fn run_get_traceability(
        &self, request: tonic::Request<tonic::Streaming<v1::ObservationGetTraceabilityRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetTraceabilityStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Observation.RunGetTraceability: Not Yet Served by the New Host"))
    }
    type WatchExecutionProgressStream = dokime_operations_next::client::Output<v1::ObservationWatchExecutionProgressOutput>;
    async fn watch_execution_progress(
        &self, request: tonic::Request<v1::ObservationWatchExecutionProgressRequest>,
    ) -> Result<tonic::Response<Self::WatchExecutionProgressStream>, tonic::Status> {
        let reply = dokime_operations_next::client::observation_watch_execution_progress::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunWatchExecutionProgressStream = dokime_operations_next::client::Output<v1::ObservationWatchExecutionProgressRunOutput>;
    async fn run_watch_execution_progress(
        &self, request: tonic::Request<tonic::Streaming<v1::ObservationWatchExecutionProgressRunInput>>,
    ) -> Result<tonic::Response<Self::RunWatchExecutionProgressStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Observation.RunWatchExecutionProgress: Not Yet Served by the New Host"))
    }
    type WatchRuntimeStream = dokime_operations_next::client::Output<v1::ObservationWatchRuntimeOutput>;
    async fn watch_runtime(
        &self, request: tonic::Request<v1::ObservationWatchRuntimeRequest>,
    ) -> Result<tonic::Response<Self::WatchRuntimeStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Observation.WatchRuntime: Not Yet Served by the New Host"))
    }
    type RunWatchRuntimeStream = dokime_operations_next::client::Output<v1::ObservationWatchRuntimeRunOutput>;
    async fn run_watch_runtime(
        &self, request: tonic::Request<tonic::Streaming<v1::ObservationWatchRuntimeRunInput>>,
    ) -> Result<tonic::Response<Self::RunWatchRuntimeStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Observation.RunWatchRuntime: Not Yet Served by the New Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
