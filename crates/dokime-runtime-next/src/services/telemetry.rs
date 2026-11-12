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
        let output = dokime_operations_next::client::run_call::unary(&self.context, request.into_inner(),
            |input| input.value.map(|value| match value {
                    v1::telemetry_list_samples_run_input::Value::Request(request) => dokime_operations_next::client::run_call::Input::Request(request),
                    v1::telemetry_list_samples_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::telemetry_list_samples_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                }), |context, request| async move { dokime_operations_next::client::telemetry_list_samples::handle(&context, request).await }, |reply| v1::TelemetryListSamplesRunOutput { value: Some(v1::telemetry_list_samples_run_output::Value::Response(reply)) }).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(output))
    }
    type WatchSamplesStream = dokime_operations_next::client::Output<v1::TelemetryWatchSamplesOutput>;
    async fn watch_samples(
        &self, request: tonic::Request<v1::TelemetryWatchSamplesRequest>,
    ) -> Result<tonic::Response<Self::WatchSamplesStream>, tonic::Status> {
        let output = dokime_operations_next::client::telemetry_watch_samples::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(output))
    }
    type RunWatchSamplesStream = dokime_operations_next::client::Output<v1::TelemetryWatchSamplesRunOutput>;
    async fn run_watch_samples(
        &self, request: tonic::Request<tonic::Streaming<v1::TelemetryWatchSamplesRunInput>>,
    ) -> Result<tonic::Response<Self::RunWatchSamplesStream>, tonic::Status> {
        let output = dokime_operations_next::client::run_call::streaming(&self.context, request.into_inner(),
            |input| input.value.map(|value| match value {
                    v1::telemetry_watch_samples_run_input::Value::Request(request) => dokime_operations_next::client::run_call::Input::Request(request),
                    v1::telemetry_watch_samples_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::telemetry_watch_samples_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                }), |context, request| async move { dokime_operations_next::client::telemetry_watch_samples::handle(&context, request).await }, |frame| v1::TelemetryWatchSamplesRunOutput { value: frame.value.map(|value| match value {
                    v1::telemetry_watch_samples_output::Value::Item(item) => v1::telemetry_watch_samples_run_output::Value::Item(item),
                    v1::telemetry_watch_samples_output::Value::Response(response) => v1::telemetry_watch_samples_run_output::Value::Response(response),
                }) }, || v1::TelemetryWatchSamplesRunOutput { value: Some(v1::telemetry_watch_samples_run_output::Value::Response(v1::ClientEmpty {})) }).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(output))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
