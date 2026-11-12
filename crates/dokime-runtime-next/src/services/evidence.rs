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
impl v1::evidence_server::Evidence for Services {
    async fn get_captured_record_schema(
        &self, request: tonic::Request<v1::EvidenceGetCapturedRecordSchemaRequest>,
    ) -> Result<tonic::Response<v1::RecordSchema>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Evidence.GetCapturedRecordSchema: Not Yet Served by the New Host"))
    }
    type RunGetCapturedRecordSchemaStream = dokime_operations_next::client::Output<v1::EvidenceGetCapturedRecordSchemaRunOutput>;
    async fn run_get_captured_record_schema(
        &self, request: tonic::Request<tonic::Streaming<v1::EvidenceGetCapturedRecordSchemaRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetCapturedRecordSchemaStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Evidence.RunGetCapturedRecordSchema: Not Yet Served by the New Host"))
    }
    async fn list_records(
        &self, request: tonic::Request<v1::EvidenceListRecordsRequest>,
    ) -> Result<tonic::Response<v1::RecordPage>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_list_records::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListRecordsStream = dokime_operations_next::client::Output<v1::EvidenceListRecordsRunOutput>;
    async fn run_list_records(
        &self, request: tonic::Request<tonic::Streaming<v1::EvidenceListRecordsRunInput>>,
    ) -> Result<tonic::Response<Self::RunListRecordsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Evidence.RunListRecords: Not Yet Served by the New Host"))
    }
    type WatchRecordsStream = dokime_operations_next::client::Output<v1::EvidenceWatchRecordsOutput>;
    async fn watch_records(
        &self, request: tonic::Request<v1::EvidenceWatchRecordsRequest>,
    ) -> Result<tonic::Response<Self::WatchRecordsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Evidence.WatchRecords: Not Yet Served by the New Host"))
    }
    type RunWatchRecordsStream = dokime_operations_next::client::Output<v1::EvidenceWatchRecordsRunOutput>;
    async fn run_watch_records(
        &self, request: tonic::Request<tonic::Streaming<v1::EvidenceWatchRecordsRunInput>>,
    ) -> Result<tonic::Response<Self::RunWatchRecordsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Evidence.RunWatchRecords: Not Yet Served by the New Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
