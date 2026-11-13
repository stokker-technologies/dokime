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
impl v1::reports_server::Reports for Services {
    async fn publish_execution_report(
        &self, request: tonic::Request<v1::ReportsPublishExecutionReportRequest>,
    ) -> Result<tonic::Response<v1::ReportMetadata>, tonic::Status> {
        let reply = dokime_operations_next::client::reports_publish_execution_report::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunPublishExecutionReportStream = dokime_operations_next::client::Output<v1::ReportsPublishExecutionReportRunOutput>;
    async fn run_publish_execution_report(
        &self, request: tonic::Request<tonic::Streaming<v1::ReportsPublishExecutionReportRunInput>>,
    ) -> Result<tonic::Response<Self::RunPublishExecutionReportStream>, tonic::Status> {
        let reply =
            dokime_operations_next::client::run_call::unary(
                &self.context,
                request.into_inner(),
                |input| {
                    input.value.map(|value| match value {
                        v1::reports_publish_execution_report_run_input::Value::Request(request) => {
                            dokime_operations_next::client::run_call::Input::Request(request)
                        }
                        v1::reports_publish_execution_report_run_input::Value::Cancel(_) => {
                            dokime_operations_next::client::run_call::Input::Cancel
                        }
                        v1::reports_publish_execution_report_run_input::Value::Finish(_) => {
                            dokime_operations_next::client::run_call::Input::Finish
                        }
                    })
                },
                |context, request| async move {
                    dokime_operations_next::client::reports_publish_execution_report::handle(&context, request).await
                },
                |reply| v1::ReportsPublishExecutionReportRunOutput {
                    value: Some(v1::reports_publish_execution_report_run_output::Value::Response(reply)),
                },
            )
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type DownloadCycleReportStream = dokime_operations_next::client::Output<v1::ReportsDownloadCycleReportOutput>;
    async fn download_cycle_report(
        &self, request: tonic::Request<v1::ReportsDownloadCycleReportRequest>,
    ) -> Result<tonic::Response<Self::DownloadCycleReportStream>, tonic::Status> {
        let reply = dokime_operations_next::client::reports_download_cycle_report::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunDownloadCycleReportStream = dokime_operations_next::client::Output<v1::ReportsDownloadCycleReportRunOutput>;
    async fn run_download_cycle_report(
        &self, request: tonic::Request<tonic::Streaming<v1::ReportsDownloadCycleReportRunInput>>,
    ) -> Result<tonic::Response<Self::RunDownloadCycleReportStream>, tonic::Status> {
        let reply=dokime_operations_next::client::run_call::finite(&self.context,request.into_inner(),|input|input.value.map(|value|match value{v1::reports_download_cycle_report_run_input::Value::Request(request)=>dokime_operations_next::client::run_call::Input::Request(request),v1::reports_download_cycle_report_run_input::Value::Cancel(_)=>dokime_operations_next::client::run_call::Input::Cancel,v1::reports_download_cycle_report_run_input::Value::Finish(_)=>dokime_operations_next::client::run_call::Input::Finish}),|context,request|async move{dokime_operations_next::client::reports_download_cycle_report::handle(&context,request).await},
                    |frame|v1::ReportsDownloadCycleReportRunOutput {value:frame.value.map(|value|match value{v1::reports_download_cycle_report_output::Value::Item(item)=>v1::reports_download_cycle_report_run_output::Value::Item(item),v1::reports_download_cycle_report_output::Value::Response(response)=>v1::reports_download_cycle_report_run_output::Value::Response(response)})},
                    ||v1::ReportsDownloadCycleReportRunOutput {value:Some(v1::reports_download_cycle_report_run_output::Value::Response(v1::ClientEmpty {}))}).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type DownloadExecutionReportStream = dokime_operations_next::client::Output<v1::ReportsDownloadExecutionReportOutput>;
    async fn download_execution_report(
        &self, request: tonic::Request<v1::ReportsDownloadExecutionReportRequest>,
    ) -> Result<tonic::Response<Self::DownloadExecutionReportStream>, tonic::Status> {
        let reply = dokime_operations_next::client::reports_download_execution_report::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunDownloadExecutionReportStream = dokime_operations_next::client::Output<v1::ReportsDownloadExecutionReportRunOutput>;
    async fn run_download_execution_report(
        &self, request: tonic::Request<tonic::Streaming<v1::ReportsDownloadExecutionReportRunInput>>,
    ) -> Result<tonic::Response<Self::RunDownloadExecutionReportStream>, tonic::Status> {
        let reply =
            dokime_operations_next::client::run_call::finite(
                &self.context,
                request.into_inner(),
                |input| {
                    input.value.map(|value| match value {
                        v1::reports_download_execution_report_run_input::Value::Request(request) => {
                            dokime_operations_next::client::run_call::Input::Request(request)
                        }
                        v1::reports_download_execution_report_run_input::Value::Cancel(_) => {
                            dokime_operations_next::client::run_call::Input::Cancel
                        }
                        v1::reports_download_execution_report_run_input::Value::Finish(_) => {
                            dokime_operations_next::client::run_call::Input::Finish
                        }
                    })
                },
                |context, request| async move {
                    dokime_operations_next::client::reports_download_execution_report::handle(&context, request).await
                },
                |frame| v1::ReportsDownloadExecutionReportRunOutput {
                    value: frame.value.map(|value| match value {
                        v1::reports_download_execution_report_output::Value::Item(item) => {
                            v1::reports_download_execution_report_run_output::Value::Item(item)
                        }
                        v1::reports_download_execution_report_output::Value::Response(response) => {
                            v1::reports_download_execution_report_run_output::Value::Response(response)
                        }
                    }),
                },
                || v1::ReportsDownloadExecutionReportRunOutput {
                    value: Some(v1::reports_download_execution_report_run_output::Value::Response(v1::ClientEmpty {})),
                },
            )
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn materialize_report(
        &self, request: tonic::Request<v1::ReportsMaterializeReportRequest>,
    ) -> Result<tonic::Response<v1::MaterializedReport>, tonic::Status> {
        let reply = dokime_operations_next::client::reports_materialize_report::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunMaterializeReportStream = dokime_operations_next::client::Output<v1::ReportsMaterializeReportRunOutput>;
    async fn run_materialize_report(
        &self, request: tonic::Request<tonic::Streaming<v1::ReportsMaterializeReportRunInput>>,
    ) -> Result<tonic::Response<Self::RunMaterializeReportStream>, tonic::Status> {
        let reply = dokime_operations_next::client::run_call::unary(
            &self.context,
            request.into_inner(),
            |input| {
                input.value.map(|value| match value {
                    v1::reports_materialize_report_run_input::Value::Request(request) => {
                        dokime_operations_next::client::run_call::Input::Request(request)
                    }
                    v1::reports_materialize_report_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::reports_materialize_report_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                })
            },
            |context, request| async move { dokime_operations_next::client::reports_materialize_report::handle(&context, request).await },
            |reply| v1::ReportsMaterializeReportRunOutput {
                value: Some(v1::reports_materialize_report_run_output::Value::Response(reply)),
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
