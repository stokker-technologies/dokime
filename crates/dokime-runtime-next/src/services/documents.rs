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
impl v1::documents_server::Documents for Services {
    type CaptureCurrentDocumentStream = dokime_operations_next::client::Output<v1::DocumentsCaptureCurrentDocumentOutput>;
    async fn capture_current_document(
        &self, request: tonic::Request<v1::DocumentsCaptureCurrentDocumentRequest>,
    ) -> Result<tonic::Response<Self::CaptureCurrentDocumentStream>, tonic::Status> {
        let reply = dokime_operations_next::client::documents_capture_current_document::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunCaptureCurrentDocumentStream = dokime_operations_next::client::Output<v1::DocumentsCaptureCurrentDocumentRunOutput>;
    async fn run_capture_current_document(
        &self, request: tonic::Request<tonic::Streaming<v1::DocumentsCaptureCurrentDocumentRunInput>>,
    ) -> Result<tonic::Response<Self::RunCaptureCurrentDocumentStream>, tonic::Status> {
        let reply = dokime_operations_next::client::run_call::finite(
            &self.context,
            request.into_inner(),
            |input| {
                input.value.map(|value| match value {
                    v1::documents_capture_current_document_run_input::Value::Request(request) => {
                        dokime_operations_next::client::run_call::Input::Request(request)
                    }
                    v1::documents_capture_current_document_run_input::Value::Cancel(_) => {
                        dokime_operations_next::client::run_call::Input::Cancel
                    }
                    v1::documents_capture_current_document_run_input::Value::Finish(_) => {
                        dokime_operations_next::client::run_call::Input::Finish
                    }
                })
            },
            |context, request| async move {
                dokime_operations_next::client::documents_capture_current_document::handle(&context, request).await
            },
            |frame| v1::DocumentsCaptureCurrentDocumentRunOutput {
                value: frame.value.map(|value| match value {
                    v1::documents_capture_current_document_output::Value::Item(item) => {
                        v1::documents_capture_current_document_run_output::Value::Item(item)
                    }
                    v1::documents_capture_current_document_output::Value::Response(response) => {
                        v1::documents_capture_current_document_run_output::Value::Response(response)
                    }
                }),
            },
            || v1::DocumentsCaptureCurrentDocumentRunOutput {
                value: Some(v1::documents_capture_current_document_run_output::Value::Response(v1::ClientEmpty {})),
            },
        )
        .await
        .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn update_plot_settings(
        &self, request: tonic::Request<v1::DocumentsUpdatePlotSettingsRequest>,
    ) -> Result<tonic::Response<v1::ClientEmpty>, tonic::Status> {
        let reply = dokime_operations_next::client::documents_update_plot_settings::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunUpdatePlotSettingsStream = dokime_operations_next::client::Output<v1::DocumentsUpdatePlotSettingsRunOutput>;
    async fn run_update_plot_settings(
        &self, request: tonic::Request<tonic::Streaming<v1::DocumentsUpdatePlotSettingsRunInput>>,
    ) -> Result<tonic::Response<Self::RunUpdatePlotSettingsStream>, tonic::Status> {
        let reply =
            dokime_operations_next::client::run_call::unary(
                &self.context,
                request.into_inner(),
                |input| {
                    input.value.map(|value| match value {
                        v1::documents_update_plot_settings_run_input::Value::Request(request) => {
                            dokime_operations_next::client::run_call::Input::Request(request)
                        }
                        v1::documents_update_plot_settings_run_input::Value::Cancel(_) => {
                            dokime_operations_next::client::run_call::Input::Cancel
                        }
                        v1::documents_update_plot_settings_run_input::Value::Finish(_) => {
                            dokime_operations_next::client::run_call::Input::Finish
                        }
                    })
                },
                |context, request| async move {
                    dokime_operations_next::client::documents_update_plot_settings::handle(&context, request).await
                },
                |reply| v1::DocumentsUpdatePlotSettingsRunOutput {
                    value: Some(v1::documents_update_plot_settings_run_output::Value::Response(reply)),
                },
            )
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
