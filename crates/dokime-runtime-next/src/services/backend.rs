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
impl v1::backend_server::Backend for Services {
    type CollectDiagnosticsStream = dokime_operations_next::client::Output<v1::BackendCollectDiagnosticsOutput>;
    async fn collect_diagnostics(
        &self, request: tonic::Request<v1::CollectBackendDiagnostics>,
    ) -> Result<tonic::Response<Self::CollectDiagnosticsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Backend.CollectDiagnostics: Not Yet Served by the New Host"))
    }
    type RunCollectDiagnosticsStream = dokime_operations_next::client::Output<v1::BackendCollectDiagnosticsRunOutput>;
    async fn run_collect_diagnostics(
        &self, request: tonic::Request<tonic::Streaming<v1::BackendCollectDiagnosticsRunInput>>,
    ) -> Result<tonic::Response<Self::RunCollectDiagnosticsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Backend.RunCollectDiagnostics: Not Yet Served by the New Host"))
    }
    async fn get_availability(
        &self, request: tonic::Request<v1::GetBackendAvailability>,
    ) -> Result<tonic::Response<v1::BackendAvailability>, tonic::Status> {
        let reply = dokime_operations_next::client::backend_get_availability::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetAvailabilityStream = dokime_operations_next::client::Output<v1::BackendGetAvailabilityRunOutput>;
    async fn run_get_availability(
        &self, request: tonic::Request<tonic::Streaming<v1::BackendGetAvailabilityRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetAvailabilityStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Backend.RunGetAvailability: Not Yet Served by the New Host"))
    }
    async fn get_health(&self, request: tonic::Request<v1::GetBackendHealth>) -> Result<tonic::Response<v1::BackendHealth>, tonic::Status> {
        let reply = dokime_operations_next::client::backend_get_health::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetHealthStream = dokime_operations_next::client::Output<v1::BackendGetHealthRunOutput>;
    async fn run_get_health(
        &self, request: tonic::Request<tonic::Streaming<v1::BackendGetHealthRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetHealthStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Backend.RunGetHealth: Not Yet Served by the New Host"))
    }
    async fn get_identity(
        &self, request: tonic::Request<v1::GetBackendIdentity>,
    ) -> Result<tonic::Response<v1::BackendIdentity>, tonic::Status> {
        let reply = dokime_operations_next::client::backend_get_identity::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetIdentityStream = dokime_operations_next::client::Output<v1::BackendGetIdentityRunOutput>;
    async fn run_get_identity(
        &self, request: tonic::Request<tonic::Streaming<v1::BackendGetIdentityRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetIdentityStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Backend.RunGetIdentity: Not Yet Served by the New Host"))
    }
    type WatchLogsStream = dokime_operations_next::client::Output<v1::BackendWatchLogsOutput>;
    async fn watch_logs(
        &self, request: tonic::Request<v1::WatchBackendLogs>,
    ) -> Result<tonic::Response<Self::WatchLogsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Backend.WatchLogs: Not Yet Served by the New Host"))
    }
    type RunWatchLogsStream = dokime_operations_next::client::Output<v1::BackendWatchLogsRunOutput>;
    async fn run_watch_logs(
        &self, request: tonic::Request<tonic::Streaming<v1::BackendWatchLogsRunInput>>,
    ) -> Result<tonic::Response<Self::RunWatchLogsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Backend.RunWatchLogs: Not Yet Served by the New Host"))
    }
    type WatchTraceStream = dokime_operations_next::client::Output<v1::BackendWatchTraceOutput>;
    async fn watch_trace(
        &self, request: tonic::Request<v1::WatchBackendTrace>,
    ) -> Result<tonic::Response<Self::WatchTraceStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Backend.WatchTrace: Not Yet Served by the New Host"))
    }
    type RunWatchTraceStream = dokime_operations_next::client::Output<v1::BackendWatchTraceRunOutput>;
    async fn run_watch_trace(
        &self, request: tonic::Request<tonic::Streaming<v1::BackendWatchTraceRunInput>>,
    ) -> Result<tonic::Response<Self::RunWatchTraceStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Backend.RunWatchTrace: Not Yet Served by the New Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
