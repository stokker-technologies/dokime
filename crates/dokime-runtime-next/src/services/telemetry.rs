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
impl v1::telemetry_server::Telemetry for Services {
    async fn list_samples(
        &self, request: tonic::Request<v1::TelemetryListSamplesRequest>,
    ) -> Result<tonic::Response<v1::TelemetrySamplesReply>, tonic::Status> {
        let reply = dokime_operations_next::client::telemetry_list_samples::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListSamplesStream = dokime_operations_next::client::Output<v1::TelemetryListSamplesRunOutput>;
    async fn run_list_samples(
        &self, request: tonic::Request<tonic::Streaming<v1::TelemetryListSamplesRunInput>>,
    ) -> Result<tonic::Response<Self::RunListSamplesStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Telemetry.RunListSamples: Not Yet Served by the New Host"))
    }
    type WatchSamplesStream = dokime_operations_next::client::Output<v1::TelemetryWatchSamplesOutput>;
    async fn watch_samples(
        &self, request: tonic::Request<v1::TelemetryWatchSamplesRequest>,
    ) -> Result<tonic::Response<Self::WatchSamplesStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Telemetry.WatchSamples: Not Yet Served by the New Host"))
    }
    type RunWatchSamplesStream = dokime_operations_next::client::Output<v1::TelemetryWatchSamplesRunOutput>;
    async fn run_watch_samples(
        &self, request: tonic::Request<tonic::Streaming<v1::TelemetryWatchSamplesRunInput>>,
    ) -> Result<tonic::Response<Self::RunWatchSamplesStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Telemetry.RunWatchSamples: Not Yet Served by the New Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
