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
        let _ = request;
        Err(tonic::Status::unimplemented("Reports.RunPublishExecutionReport: Not Yet Served by the New Host"))
    }
    type DownloadCycleReportStream = dokime_operations_next::client::Output<v1::ReportsDownloadCycleReportOutput>;
    async fn download_cycle_report(
        &self, request: tonic::Request<v1::ReportsDownloadCycleReportRequest>,
    ) -> Result<tonic::Response<Self::DownloadCycleReportStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Reports.DownloadCycleReport: Not Yet Served by the New Host"))
    }
    type RunDownloadCycleReportStream = dokime_operations_next::client::Output<v1::ReportsDownloadCycleReportRunOutput>;
    async fn run_download_cycle_report(
        &self, request: tonic::Request<tonic::Streaming<v1::ReportsDownloadCycleReportRunInput>>,
    ) -> Result<tonic::Response<Self::RunDownloadCycleReportStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Reports.RunDownloadCycleReport: Not Yet Served by the New Host"))
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
        let _ = request;
        Err(tonic::Status::unimplemented("Reports.RunDownloadExecutionReport: Not Yet Served by the New Host"))
    }
    async fn materialize_report(
        &self, request: tonic::Request<v1::ReportsMaterializeReportRequest>,
    ) -> Result<tonic::Response<v1::MaterializedReport>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Reports.MaterializeReport: Not Yet Served by the New Host"))
    }
    type RunMaterializeReportStream = dokime_operations_next::client::Output<v1::ReportsMaterializeReportRunOutput>;
    async fn run_materialize_report(
        &self, request: tonic::Request<tonic::Streaming<v1::ReportsMaterializeReportRunInput>>,
    ) -> Result<tonic::Response<Self::RunMaterializeReportStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Reports.RunMaterializeReport: Not Yet Served by the New Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
