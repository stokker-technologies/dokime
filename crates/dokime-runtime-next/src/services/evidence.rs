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
        let reply = dokime_operations_next::client::evidence_get_captured_record_schema::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetCapturedRecordSchemaStream = dokime_operations_next::client::Output<v1::EvidenceGetCapturedRecordSchemaRunOutput>;
    async fn run_get_captured_record_schema(
        &self, request: tonic::Request<tonic::Streaming<v1::EvidenceGetCapturedRecordSchemaRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetCapturedRecordSchemaStream>, tonic::Status> {
        let output = dokime_operations_next::client::run_call::unary(&self.context, request.into_inner(),
            |input| input.value.map(|value| match value {
                    v1::evidence_get_captured_record_schema_run_input::Value::Request(request) => dokime_operations_next::client::run_call::Input::Request(request),
                    v1::evidence_get_captured_record_schema_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::evidence_get_captured_record_schema_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                }), |context, request| async move { dokime_operations_next::client::evidence_get_captured_record_schema::handle(&context, request).await }, |reply| v1::EvidenceGetCapturedRecordSchemaRunOutput { value: Some(v1::evidence_get_captured_record_schema_run_output::Value::Response(reply)) }).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(output))
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
        let output = dokime_operations_next::client::run_call::unary(&self.context, request.into_inner(),
            |input| input.value.map(|value| match value {
                    v1::evidence_list_records_run_input::Value::Request(request) => dokime_operations_next::client::run_call::Input::Request(request),
                    v1::evidence_list_records_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::evidence_list_records_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                }), |context, request| async move { dokime_operations_next::client::evidence_list_records::handle(&context, request).await }, |reply| v1::EvidenceListRecordsRunOutput { value: Some(v1::evidence_list_records_run_output::Value::Response(reply)) }).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(output))
    }
    type WatchRecordsStream = dokime_operations_next::client::Output<v1::EvidenceWatchRecordsOutput>;
    async fn watch_records(
        &self, request: tonic::Request<v1::EvidenceWatchRecordsRequest>,
    ) -> Result<tonic::Response<Self::WatchRecordsStream>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_watch_records::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunWatchRecordsStream = dokime_operations_next::client::Output<v1::EvidenceWatchRecordsRunOutput>;
    async fn run_watch_records(
        &self, request: tonic::Request<tonic::Streaming<v1::EvidenceWatchRecordsRunInput>>,
    ) -> Result<tonic::Response<Self::RunWatchRecordsStream>, tonic::Status> {
        let output = dokime_operations_next::client::run_call::streaming(&self.context, request.into_inner(),
            |input| input.value.map(|value| match value {
                    v1::evidence_watch_records_run_input::Value::Request(request) => dokime_operations_next::client::run_call::Input::Request(request),
                    v1::evidence_watch_records_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::evidence_watch_records_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                }), |context, request| async move { dokime_operations_next::client::evidence_watch_records::handle(&context, request).await }, |frame| v1::EvidenceWatchRecordsRunOutput { value: frame.value.map(|value| match value {
                    v1::evidence_watch_records_output::Value::Item(item) => v1::evidence_watch_records_run_output::Value::Item(item),
                    v1::evidence_watch_records_output::Value::Response(response) => v1::evidence_watch_records_run_output::Value::Response(response),
                }) }, || v1::EvidenceWatchRecordsRunOutput { value: Some(v1::evidence_watch_records_run_output::Value::Response(v1::ClientEmpty {})) }).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(output))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
