/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// History

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::Services;
use dokime_protocol::v1;

// Standard Library Imports

// Third Party Imports

// Local Imports


/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::history_server::History for Services {
    async fn browse_cycles(&self,request: tonic::Request<v1::HistoryBrowseCyclesRequest>) -> Result<tonic::Response<v1::CycleBrowsePage>,tonic::Status> {
        let reply = dokime_operations_next::client::history_browse_cycles::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunBrowseCyclesStream = dokime_operations_next::client::Output<v1::HistoryBrowseCyclesRunOutput>;
    async fn run_browse_cycles(&self,request: tonic::Request<tonic::Streaming<v1::HistoryBrowseCyclesRunInput>>) -> Result<tonic::Response<Self::RunBrowseCyclesStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(),request.into_inner(),|input: v1::HistoryBrowseCyclesRunInput| match input.value? {
            v1::history_browse_cycles_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::history_browse_cycles_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::history_browse_cycles_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::HistoryBrowseCyclesRunOutput { value: Some(v1::history_browse_cycles_run_output::Value::Response(response)) }, |context,request| async move { dokime_operations_next::client::history_browse_cycles::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    async fn browse_executions(&self,request: tonic::Request<v1::HistoryBrowseExecutionsRequest>) -> Result<tonic::Response<v1::ExecutionBrowsePage>,tonic::Status> {
        let reply = dokime_operations_next::client::history_browse_executions::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunBrowseExecutionsStream = dokime_operations_next::client::Output<v1::HistoryBrowseExecutionsRunOutput>;
    async fn run_browse_executions(&self,request: tonic::Request<tonic::Streaming<v1::HistoryBrowseExecutionsRunInput>>) -> Result<tonic::Response<Self::RunBrowseExecutionsStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(),request.into_inner(),|input: v1::HistoryBrowseExecutionsRunInput| match input.value? {
            v1::history_browse_executions_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::history_browse_executions_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::history_browse_executions_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::HistoryBrowseExecutionsRunOutput { value: Some(v1::history_browse_executions_run_output::Value::Response(response)) }, |context,request| async move { dokime_operations_next::client::history_browse_executions::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    async fn delete_execution(&self,request: tonic::Request<v1::HistoryDeleteExecutionRequest>) -> Result<tonic::Response<v1::DeleteExecutionResponse>,tonic::Status> {
        let reply = dokime_operations_next::client::history_delete_execution::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunDeleteExecutionStream = dokime_operations_next::client::Output<v1::HistoryDeleteExecutionRunOutput>;
    async fn run_delete_execution(&self,request: tonic::Request<tonic::Streaming<v1::HistoryDeleteExecutionRunInput>>) -> Result<tonic::Response<Self::RunDeleteExecutionStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(),request.into_inner(),|input: v1::HistoryDeleteExecutionRunInput| match input.value? {
            v1::history_delete_execution_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::history_delete_execution_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::history_delete_execution_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::HistoryDeleteExecutionRunOutput { value: Some(v1::history_delete_execution_run_output::Value::Response(response)) }, |context,request| async move { dokime_operations_next::client::history_delete_execution::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    type DownloadCheckpointArtifactStream = dokime_operations_next::client::Output<v1::HistoryDownloadCheckpointArtifactOutput>;
    async fn download_checkpoint_artifact(&self,request: tonic::Request<v1::HistoryDownloadCheckpointArtifactRequest>) -> Result<tonic::Response<Self::DownloadCheckpointArtifactStream>,tonic::Status> {
        let reply = dokime_operations_next::client::history_download_checkpoint_artifact::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunDownloadCheckpointArtifactStream = dokime_operations_next::client::Output<v1::HistoryDownloadCheckpointArtifactRunOutput>;
    async fn run_download_checkpoint_artifact(&self,request: tonic::Request<tonic::Streaming<v1::HistoryDownloadCheckpointArtifactRunInput>>) -> Result<tonic::Response<Self::RunDownloadCheckpointArtifactStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::streaming(self.context.clone(),request.into_inner(),|input: v1::HistoryDownloadCheckpointArtifactRunInput| match input.value? {
            v1::history_download_checkpoint_artifact_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::history_download_checkpoint_artifact_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::history_download_checkpoint_artifact_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |output: v1::HistoryDownloadCheckpointArtifactOutput| v1::HistoryDownloadCheckpointArtifactRunOutput { value: output.value.map(|value| match value { v1::history_download_checkpoint_artifact_output::Value::Item(item) => v1::history_download_checkpoint_artifact_run_output::Value::Item(item),v1::history_download_checkpoint_artifact_output::Value::Response(response) => v1::history_download_checkpoint_artifact_run_output::Value::Response(response) }) }, |context,request| async move { dokime_operations_next::client::history_download_checkpoint_artifact::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    async fn get_checkpoint_context(&self,request: tonic::Request<v1::HistoryGetCheckpointContextRequest>) -> Result<tonic::Response<v1::CheckpointContext>,tonic::Status> {
        let reply = dokime_operations_next::client::history_get_checkpoint_context::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetCheckpointContextStream = dokime_operations_next::client::Output<v1::HistoryGetCheckpointContextRunOutput>;
    async fn run_get_checkpoint_context(&self,request: tonic::Request<tonic::Streaming<v1::HistoryGetCheckpointContextRunInput>>) -> Result<tonic::Response<Self::RunGetCheckpointContextStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(),request.into_inner(),|input: v1::HistoryGetCheckpointContextRunInput| match input.value? {
            v1::history_get_checkpoint_context_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::history_get_checkpoint_context_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::history_get_checkpoint_context_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::HistoryGetCheckpointContextRunOutput { value: Some(v1::history_get_checkpoint_context_run_output::Value::Response(response)) }, |context,request| async move { dokime_operations_next::client::history_get_checkpoint_context::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    async fn get_checkpoint_document(&self,request: tonic::Request<v1::HistoryGetCheckpointDocumentRequest>) -> Result<tonic::Response<v1::CheckpointDocument>,tonic::Status> {
        let reply = dokime_operations_next::client::history_get_checkpoint_document::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetCheckpointDocumentStream = dokime_operations_next::client::Output<v1::HistoryGetCheckpointDocumentRunOutput>;
    async fn run_get_checkpoint_document(&self,request: tonic::Request<tonic::Streaming<v1::HistoryGetCheckpointDocumentRunInput>>) -> Result<tonic::Response<Self::RunGetCheckpointDocumentStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(),request.into_inner(),|input: v1::HistoryGetCheckpointDocumentRunInput| match input.value? {
            v1::history_get_checkpoint_document_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::history_get_checkpoint_document_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::history_get_checkpoint_document_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::HistoryGetCheckpointDocumentRunOutput { value: Some(v1::history_get_checkpoint_document_run_output::Value::Response(response)) }, |context,request| async move { dokime_operations_next::client::history_get_checkpoint_document::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    async fn get_checkpoint_evidence(&self,request: tonic::Request<v1::HistoryGetCheckpointEvidenceRequest>) -> Result<tonic::Response<v1::CheckpointEvidence>,tonic::Status> {
        let reply = dokime_operations_next::client::history_get_checkpoint_evidence::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetCheckpointEvidenceStream = dokime_operations_next::client::Output<v1::HistoryGetCheckpointEvidenceRunOutput>;
    async fn run_get_checkpoint_evidence(&self,request: tonic::Request<tonic::Streaming<v1::HistoryGetCheckpointEvidenceRunInput>>) -> Result<tonic::Response<Self::RunGetCheckpointEvidenceStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(),request.into_inner(),|input: v1::HistoryGetCheckpointEvidenceRunInput| match input.value? {
            v1::history_get_checkpoint_evidence_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::history_get_checkpoint_evidence_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::history_get_checkpoint_evidence_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::HistoryGetCheckpointEvidenceRunOutput { value: Some(v1::history_get_checkpoint_evidence_run_output::Value::Response(response)) }, |context,request| async move { dokime_operations_next::client::history_get_checkpoint_evidence::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    async fn get_execution_checkpoint(&self,request: tonic::Request<v1::HistoryGetExecutionCheckpointRequest>) -> Result<tonic::Response<v1::ExecutionCheckpoint>,tonic::Status> {
        let reply = dokime_operations_next::client::history_get_execution_checkpoint::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetExecutionCheckpointStream = dokime_operations_next::client::Output<v1::HistoryGetExecutionCheckpointRunOutput>;
    async fn run_get_execution_checkpoint(&self,request: tonic::Request<tonic::Streaming<v1::HistoryGetExecutionCheckpointRunInput>>) -> Result<tonic::Response<Self::RunGetExecutionCheckpointStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(),request.into_inner(),|input: v1::HistoryGetExecutionCheckpointRunInput| match input.value? {
            v1::history_get_execution_checkpoint_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::history_get_execution_checkpoint_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::history_get_execution_checkpoint_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::HistoryGetExecutionCheckpointRunOutput { value: Some(v1::history_get_execution_checkpoint_run_output::Value::Response(response)) }, |context,request| async move { dokime_operations_next::client::history_get_execution_checkpoint::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_checkpoint_records(&self,request: tonic::Request<v1::HistoryListCheckpointRecordsRequest>) -> Result<tonic::Response<v1::CheckpointRecordPage>,tonic::Status> {
        let reply = dokime_operations_next::client::history_list_checkpoint_records::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListCheckpointRecordsStream = dokime_operations_next::client::Output<v1::HistoryListCheckpointRecordsRunOutput>;
    async fn run_list_checkpoint_records(&self,request: tonic::Request<tonic::Streaming<v1::HistoryListCheckpointRecordsRunInput>>) -> Result<tonic::Response<Self::RunListCheckpointRecordsStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(),request.into_inner(),|input: v1::HistoryListCheckpointRecordsRunInput| match input.value? {
            v1::history_list_checkpoint_records_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::history_list_checkpoint_records_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::history_list_checkpoint_records_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::HistoryListCheckpointRecordsRunOutput { value: Some(v1::history_list_checkpoint_records_run_output::Value::Response(response)) }, |context,request| async move { dokime_operations_next::client::history_list_checkpoint_records::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_checkpoint_telemetry(&self,request: tonic::Request<v1::HistoryListCheckpointTelemetryRequest>) -> Result<tonic::Response<v1::CheckpointTelemetryPage>,tonic::Status> {
        let reply = dokime_operations_next::client::history_list_checkpoint_telemetry::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListCheckpointTelemetryStream = dokime_operations_next::client::Output<v1::HistoryListCheckpointTelemetryRunOutput>;
    async fn run_list_checkpoint_telemetry(&self,request: tonic::Request<tonic::Streaming<v1::HistoryListCheckpointTelemetryRunInput>>) -> Result<tonic::Response<Self::RunListCheckpointTelemetryStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(),request.into_inner(),|input: v1::HistoryListCheckpointTelemetryRunInput| match input.value? {
            v1::history_list_checkpoint_telemetry_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::history_list_checkpoint_telemetry_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::history_list_checkpoint_telemetry_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::HistoryListCheckpointTelemetryRunOutput { value: Some(v1::history_list_checkpoint_telemetry_run_output::Value::Response(response)) }, |context,request| async move { dokime_operations_next::client::history_list_checkpoint_telemetry::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
    async fn read_checkpoint_output(&self,request: tonic::Request<v1::HistoryReadCheckpointOutputRequest>) -> Result<tonic::Response<v1::CheckpointFileWindow>,tonic::Status> {
        let reply = dokime_operations_next::client::history_read_checkpoint_output::handle(&self.context,request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunReadCheckpointOutputStream = dokime_operations_next::client::Output<v1::HistoryReadCheckpointOutputRunOutput>;
    async fn run_read_checkpoint_output(&self,request: tonic::Request<tonic::Streaming<v1::HistoryReadCheckpointOutputRunInput>>) -> Result<tonic::Response<Self::RunReadCheckpointOutputStream>,tonic::Status> {
        use dokime_operations_next::client::conversation::Input;
        let reply = dokime_operations_next::client::conversation::unary(self.context.clone(),request.into_inner(),|input: v1::HistoryReadCheckpointOutputRunInput| match input.value? {
            v1::history_read_checkpoint_output_run_input::Value::Request(request) => Some(Input::Request(request)),
            v1::history_read_checkpoint_output_run_input::Value::Cancel(_) => Some(Input::Cancel),
            v1::history_read_checkpoint_output_run_input::Value::Finish(_) => Some(Input::Finish),
        }, |response| v1::HistoryReadCheckpointOutputRunOutput { value: Some(v1::history_read_checkpoint_output_run_output::Value::Response(response)) }, |context,request| async move { dokime_operations_next::client::history_read_checkpoint_output::handle(&context,request).await }).await?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
