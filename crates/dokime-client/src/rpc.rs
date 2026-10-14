/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{Client, RequestStream, ResponseStream, Result, contracts};
use dokime_contract::operations::Endpoint;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Client {
    pub async fn content_follow_execution_file(
        &self,
        message: contracts::content::FollowExecutionFileRequest,
    ) -> Result<ResponseStream<contracts::content::ExecutionFileFollowFrame>> {
        self.download::<contracts::content::FollowExecutionFile>(
            contracts::content::FollowExecutionFile::PATH,
            message,
        )
        .await
    }

    pub async fn appearance_get_appearance(
        &self,
        message: contracts::appearance::GetAppearanceRequest,
    ) -> Result<contracts::appearance::AppearanceSnapshot> {
        let reply = self
            .unary::<contracts::appearance::GetAppearance>(
                contracts::appearance::GetAppearance::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn backend_get_identity(
        &self,
        message: contracts::backend::GetIdentityRequest,
    ) -> Result<contracts::backend::BackendIdentity> {
        let reply = self
            .unary::<contracts::backend::GetIdentity>(
                contracts::backend::GetIdentity::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn backend_get_health(
        &self,
        message: contracts::backend::GetHealthRequest,
    ) -> Result<contracts::backend::BackendHealth> {
        let reply = self
            .unary::<contracts::backend::GetHealth>(contracts::backend::GetHealth::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn backend_get_availability(
        &self,
        message: contracts::backend::GetAvailabilityRequest,
    ) -> Result<contracts::backend::BackendAvailability> {
        let reply = self
            .unary::<contracts::backend::GetAvailability>(
                contracts::backend::GetAvailability::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn backend_get_diagnostics(
        &self,
        message: contracts::backend::GetDiagnosticsRequest,
    ) -> Result<contracts::backend::DiagnosticsResponse> {
        let reply = self
            .unary::<contracts::backend::GetDiagnostics>(
                contracts::backend::GetDiagnostics::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn backend_collect_diagnostics(
        &self,
        message: contracts::backend::CollectDiagnosticsRequest,
    ) -> Result<ResponseStream<contracts::backend::DiagnosticsBundleFrame>> {
        self.download::<contracts::backend::CollectDiagnostics>(
            contracts::backend::CollectDiagnostics::PATH,
            message,
        )
        .await
    }

    pub async fn backend_watch_logs(
        &self,
        message: contracts::backend::WatchLogsRequest,
    ) -> Result<ResponseStream<contracts::backend::LogFrame>> {
        self.download::<contracts::backend::WatchLogs>(contracts::backend::WatchLogs::PATH, message)
            .await
    }

    pub async fn catalog_list_suites(
        &self,
        message: contracts::catalog::ListSuitesRequest,
    ) -> Result<contracts::catalog::ListSuitesReply> {
        let reply = self
            .unary::<contracts::catalog::ListSuites>(contracts::catalog::ListSuites::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn catalog_get_suite(
        &self,
        message: contracts::catalog::GetSuiteRequest,
    ) -> Result<contracts::catalog::SuiteDefinition> {
        let reply = self
            .unary::<contracts::catalog::GetSuite>(contracts::catalog::GetSuite::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn catalog_list_record_schemas(
        &self,
        message: contracts::catalog::ListRecordSchemasRequest,
    ) -> Result<contracts::catalog::ListRecordSchemasReply> {
        let reply = self
            .unary::<contracts::catalog::ListRecordSchemas>(
                contracts::catalog::ListRecordSchemas::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn catalog_get_record_schema(
        &self,
        message: contracts::catalog::GetRecordSchemaRequest,
    ) -> Result<contracts::schema::RecordSchema> {
        let reply = self
            .unary::<contracts::catalog::GetRecordSchema>(
                contracts::catalog::GetRecordSchema::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn catalog_list_telemetry_schemas(
        &self,
        message: contracts::catalog::ListTelemetrySchemasRequest,
    ) -> Result<contracts::catalog::ListTelemetrySchemasReply> {
        let reply = self
            .unary::<contracts::catalog::ListTelemetrySchemas>(
                contracts::catalog::ListTelemetrySchemas::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn catalog_get_telemetry_schema(
        &self,
        message: contracts::catalog::GetTelemetrySchemaRequest,
    ) -> Result<contracts::schema::TelemetrySchema> {
        let reply = self
            .unary::<contracts::catalog::GetTelemetrySchema>(
                contracts::catalog::GetTelemetrySchema::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn content_download_appearance_asset(
        &self,
        message: contracts::content::DownloadAppearanceAssetRequest,
    ) -> Result<ResponseStream<contracts::content::AppearanceAssetFrame>> {
        self.download::<contracts::content::DownloadAppearanceAsset>(
            contracts::content::DownloadAppearanceAsset::PATH,
            message,
        )
        .await
    }

    pub async fn content_list_execution_files(
        &self,
        message: contracts::content::ListExecutionFilesRequest,
    ) -> Result<contracts::content::ListExecutionFilesReply> {
        let reply = self
            .unary::<contracts::content::ListExecutionFiles>(
                contracts::content::ListExecutionFiles::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn content_read_execution_file(
        &self,
        message: contracts::content::ReadExecutionFileRequest,
    ) -> Result<contracts::content::ExecutionFileWindow> {
        let reply = self
            .unary::<contracts::content::ReadExecutionFile>(
                contracts::content::ReadExecutionFile::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn content_download_execution_file(
        &self,
        message: contracts::content::DownloadExecutionFileRequest,
    ) -> Result<ResponseStream<contracts::content::ExecutionFileDownloadFrame>> {
        self.download::<contracts::content::DownloadExecutionFile>(
            contracts::content::DownloadExecutionFile::PATH,
            message,
        )
        .await
    }

    pub async fn content_list_artifacts(
        &self,
        message: contracts::content::ListArtifactsRequest,
    ) -> Result<contracts::content::ListArtifactsReply> {
        let reply = self
            .unary::<contracts::content::ListArtifacts>(
                contracts::content::ListArtifacts::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn content_download_artifact(
        &self,
        message: contracts::content::DownloadArtifactRequest,
    ) -> Result<ResponseStream<contracts::content::ArtifactFrame>> {
        self.download::<contracts::content::DownloadArtifact>(
            contracts::content::DownloadArtifact::PATH,
            message,
        )
        .await
    }

    pub async fn content_upload_artifact(
        &self,
        message: impl tokio_stream::Stream<Item = contracts::content::UploadArtifactFrame>
        + Send
        + 'static,
    ) -> Result<contracts::content::ArtifactMetadata> {
        self.upload::<contracts::content::UploadArtifact, _>(
            contracts::content::UploadArtifact::PATH,
            (),
            message,
        )
        .await
    }

    pub fn content_upload_artifact_open(
        &self,
    ) -> Result<
        RequestStream<
            contracts::content::UploadArtifactFrame,
            contracts::content::ArtifactMetadata,
        >,
    > {
        self.require(contracts::content::UploadArtifact::PATH)?;
        let client = self.clone();
        RequestStream::new(self, move |stream| async move {
            client.content_upload_artifact(stream).await
        })
    }

    pub async fn content_download_cycle_bundle(
        &self,
        message: contracts::content::DownloadCycleBundleRequest,
    ) -> Result<ResponseStream<contracts::content::CycleBundleFrame>> {
        self.download::<contracts::content::DownloadCycleBundle>(
            contracts::content::DownloadCycleBundle::PATH,
            message,
        )
        .await
    }

    pub async fn content_get_cycle_folder_path(
        &self,
        message: contracts::content::GetCycleFolderPathRequest,
    ) -> Result<contracts::content::LocalPathResponse> {
        let reply = self
            .unary::<contracts::content::GetCycleFolderPath>(
                contracts::content::GetCycleFolderPath::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn documents_capture_current_document(
        &self,
        message: contracts::documents::CaptureCurrentDocumentRequest,
    ) -> Result<ResponseStream<contracts::documents::DocumentFrame>> {
        self.download::<contracts::documents::CaptureCurrentDocument>(
            contracts::documents::CaptureCurrentDocument::PATH,
            message,
        )
        .await
    }

    pub async fn evidence_list_records(
        &self,
        message: contracts::evidence::ListRecordsRequest,
    ) -> Result<contracts::evidence::RecordPage> {
        let reply = self
            .unary::<contracts::evidence::ListRecords>(
                contracts::evidence::ListRecords::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn evidence_watch_records(
        &self,
        message: contracts::evidence::WatchRecordsRequest,
    ) -> Result<ResponseStream<contracts::evidence::RecordPage>> {
        self.download::<contracts::evidence::WatchRecords>(
            contracts::evidence::WatchRecords::PATH,
            message,
        )
        .await
    }

    pub async fn evidence_get_captured_record_schema(
        &self,
        message: contracts::evidence::GetCapturedRecordSchemaRequest,
    ) -> Result<contracts::schema::RecordSchema> {
        let reply = self
            .unary::<contracts::evidence::GetCapturedRecordSchema>(
                contracts::evidence::GetCapturedRecordSchema::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_execution_steps(
        &self,
        message: contracts::execution::ListExecutionStepsRequest,
    ) -> Result<contracts::execution::ListExecutionStepsReply> {
        let reply = self
            .unary::<contracts::execution::ListExecutionSteps>(
                contracts::execution::ListExecutionSteps::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_step_criteria(
        &self,
        message: contracts::execution::ListStepEvidenceRequest,
    ) -> Result<contracts::execution::ListStepCriteriaReply> {
        let reply = self
            .unary::<contracts::execution::ListStepCriteria>(
                contracts::execution::ListStepCriteria::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_copy_cycle(
        &self,
        message: contracts::execution::CopyCycleRequest,
    ) -> Result<contracts::execution::CreateCycleReply> {
        let reply = self
            .unary::<contracts::execution::CopyCycle>(
                contracts::execution::CopyCycle::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_create_cycle(
        &self,
        message: contracts::execution::CreateCycleRequest,
    ) -> Result<contracts::execution::CreateCycleReply> {
        let reply = self
            .unary::<contracts::execution::CreateCycle>(
                contracts::execution::CreateCycle::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_get_cycle(
        &self,
        message: contracts::execution::GetCycleRequest,
    ) -> Result<contracts::execution::CycleSummary> {
        let reply = self
            .unary::<contracts::execution::GetCycle>(contracts::execution::GetCycle::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_get_case(
        &self,
        message: contracts::execution::GetCaseRequest,
    ) -> Result<contracts::execution::CaseDetail> {
        let reply = self
            .unary::<contracts::execution::GetCase>(contracts::execution::GetCase::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_get_execution(
        &self,
        message: contracts::execution::GetExecutionRequest,
    ) -> Result<contracts::execution::ExecutionDetail> {
        let reply = self
            .unary::<contracts::execution::GetExecution>(
                contracts::execution::GetExecution::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_case_parameters(
        &self,
        message: contracts::execution::ListCaseParametersRequest,
    ) -> Result<contracts::execution::ListCaseParametersReply> {
        let reply = self
            .unary::<contracts::execution::ListCaseParameters>(
                contracts::execution::ListCaseParameters::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_execution_parameters(
        &self,
        message: contracts::execution::ListExecutionParametersRequest,
    ) -> Result<contracts::execution::ListMaterializedParametersReply> {
        let reply = self
            .unary::<contracts::execution::ListExecutionParameters>(
                contracts::execution::ListExecutionParameters::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_cycles(
        &self,
        message: contracts::execution::ListCyclesRequest,
    ) -> Result<contracts::execution::ListCyclesReply> {
        let reply = self
            .unary::<contracts::execution::ListCycles>(
                contracts::execution::ListCycles::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_cases(
        &self,
        message: contracts::execution::ListCasesRequest,
    ) -> Result<contracts::execution::ListCasesReply> {
        let reply = self
            .unary::<contracts::execution::ListCases>(
                contracts::execution::ListCases::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_executions(
        &self,
        message: contracts::execution::ListExecutionsRequest,
    ) -> Result<contracts::execution::ListExecutionsReply> {
        let reply = self
            .unary::<contracts::execution::ListExecutions>(
                contracts::execution::ListExecutions::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_append_planned_cases(
        &self,
        message: contracts::execution::AppendPlannedCasesRequest,
    ) -> Result<contracts::execution::ControlReply> {
        let reply = self
            .unary::<contracts::execution::AppendPlannedCases>(
                contracts::execution::AppendPlannedCases::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_run_cycle(
        &self,
        message: contracts::execution::RunCycleRequest,
    ) -> Result<contracts::execution::RunReply> {
        let reply = self
            .unary::<contracts::execution::RunCycle>(contracts::execution::RunCycle::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_run_case(
        &self,
        message: contracts::execution::RunCaseRequest,
    ) -> Result<contracts::execution::RunReply> {
        let reply = self
            .unary::<contracts::execution::RunCase>(contracts::execution::RunCase::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_run_execution(
        &self,
        message: contracts::execution::RunExecutionRequest,
    ) -> Result<contracts::execution::RunReply> {
        let reply = self
            .unary::<contracts::execution::RunExecution>(
                contracts::execution::RunExecution::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_pause_execution(
        &self,
        message: contracts::execution::ExecutionControlRequest,
    ) -> Result<contracts::execution::ControlReply> {
        let reply = self
            .unary::<contracts::execution::PauseExecution>(
                contracts::execution::PauseExecution::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_resume_execution(
        &self,
        message: contracts::execution::ExecutionControlRequest,
    ) -> Result<contracts::execution::ControlReply> {
        let reply = self
            .unary::<contracts::execution::ResumeExecution>(
                contracts::execution::ResumeExecution::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_stop_execution(
        &self,
        message: contracts::execution::ExecutionControlRequest,
    ) -> Result<contracts::execution::ControlReply> {
        let reply = self
            .unary::<contracts::execution::StopExecution>(
                contracts::execution::StopExecution::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_get_run_request(
        &self,
        message: contracts::execution::GetRunRequestRequest,
    ) -> Result<contracts::execution::RunRequestReceipt> {
        let reply = self
            .unary::<contracts::execution::GetRunRequest>(
                contracts::execution::GetRunRequest::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_submit_execution_command(
        &self,
        message: contracts::execution::SubmitExecutionCommandRequest,
    ) -> Result<contracts::execution::ExecutionCommandReceipt> {
        let reply = self
            .unary::<contracts::execution::SubmitExecutionCommand>(
                contracts::execution::SubmitExecutionCommand::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_get_execution_command(
        &self,
        message: contracts::execution::GetExecutionCommandRequest,
    ) -> Result<contracts::execution::ExecutionCommandReceipt> {
        let reply = self
            .unary::<contracts::execution::GetExecutionCommand>(
                contracts::execution::GetExecutionCommand::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_pause_cycle(
        &self,
        message: contracts::execution::CycleControlRequest,
    ) -> Result<contracts::execution::ControlReply> {
        let reply = self
            .unary::<contracts::execution::PauseCycle>(
                contracts::execution::PauseCycle::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_resume_cycle(
        &self,
        message: contracts::execution::CycleControlRequest,
    ) -> Result<contracts::execution::ControlReply> {
        let reply = self
            .unary::<contracts::execution::ResumeCycle>(
                contracts::execution::ResumeCycle::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_stop_cycle(
        &self,
        message: contracts::execution::CycleControlRequest,
    ) -> Result<contracts::execution::ControlReply> {
        let reply = self
            .unary::<contracts::execution::StopCycle>(
                contracts::execution::StopCycle::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_delete_case(
        &self,
        message: contracts::execution::CaseControlRequest,
    ) -> Result<contracts::execution::ControlReply> {
        let reply = self
            .unary::<contracts::execution::DeleteCase>(
                contracts::execution::DeleteCase::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_schedule_case_rerun(
        &self,
        message: contracts::execution::CaseControlRequest,
    ) -> Result<contracts::execution::ControlReply> {
        let reply = self
            .unary::<contracts::execution::ScheduleCaseRerun>(
                contracts::execution::ScheduleCaseRerun::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_clear_cycle(
        &self,
        message: contracts::execution::CycleControlRequest,
    ) -> Result<contracts::execution::ControlReply> {
        let reply = self
            .unary::<contracts::execution::ClearCycle>(
                contracts::execution::ClearCycle::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_update_cycle_policy(
        &self,
        message: contracts::execution::UpdateCyclePolicyRequest,
    ) -> Result<contracts::execution::ControlReply> {
        let reply = self
            .unary::<contracts::execution::UpdateCyclePolicy>(
                contracts::execution::UpdateCyclePolicy::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn history_browse_executions(
        &self,
        message: contracts::history::BrowseExecutionsRequest,
    ) -> Result<contracts::history::ExecutionBrowsePage> {
        let reply = self
            .unary::<contracts::history::BrowseExecutions>(
                contracts::history::BrowseExecutions::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn history_get_execution_checkpoint(
        &self,
        message: contracts::history::GetExecutionCheckpointRequest,
    ) -> Result<contracts::checkpoint::ExecutionCheckpoint> {
        let reply = self
            .unary::<contracts::history::GetExecutionCheckpoint>(
                contracts::history::GetExecutionCheckpoint::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn history_delete_execution(
        &self,
        message: contracts::history::DeleteExecutionRequest,
    ) -> Result<contracts::history::DeleteExecutionResponse> {
        let reply = self
            .unary::<contracts::history::DeleteExecution>(
                contracts::history::DeleteExecution::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn history_get_checkpoint_document(
        &self,
        message: contracts::history::GetCheckpointDocumentRequest,
    ) -> Result<contracts::history::CheckpointDocument> {
        let reply = self
            .unary::<contracts::history::GetCheckpointDocument>(
                contracts::history::GetCheckpointDocument::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn history_get_checkpoint_evidence(
        &self,
        message: contracts::history::GetCheckpointEvidenceRequest,
    ) -> Result<contracts::history::CheckpointEvidence> {
        let reply = self
            .unary::<contracts::history::GetCheckpointEvidence>(
                contracts::history::GetCheckpointEvidence::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn history_read_checkpoint_output(
        &self,
        message: contracts::history::ReadCheckpointOutputRequest,
    ) -> Result<contracts::history::CheckpointFileWindow> {
        let reply = self
            .unary::<contracts::history::ReadCheckpointOutput>(
                contracts::history::ReadCheckpointOutput::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn history_download_checkpoint_artifact(
        &self,
        message: contracts::history::DownloadCheckpointArtifactRequest,
    ) -> Result<ResponseStream<contracts::content::ArtifactFrame>> {
        self.download::<contracts::history::DownloadCheckpointArtifact>(
            contracts::history::DownloadCheckpointArtifact::PATH,
            message,
        )
        .await
    }

    pub async fn history_list_checkpoint_records(
        &self,
        message: contracts::history::ListCheckpointRecordsRequest,
    ) -> Result<contracts::history::CheckpointRecordPage> {
        let reply = self
            .unary::<contracts::history::ListCheckpointRecords>(
                contracts::history::ListCheckpointRecords::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn history_list_checkpoint_telemetry(
        &self,
        message: contracts::history::ListCheckpointTelemetryRequest,
    ) -> Result<contracts::history::CheckpointTelemetryPage> {
        let reply = self
            .unary::<contracts::history::ListCheckpointTelemetry>(
                contracts::history::ListCheckpointTelemetry::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn history_get_checkpoint_context(
        &self,
        message: contracts::history::GetCheckpointContextRequest,
    ) -> Result<contracts::history::CheckpointContext> {
        let reply = self
            .unary::<contracts::history::GetCheckpointContext>(
                contracts::history::GetCheckpointContext::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn notes_list_notes(
        &self,
        message: contracts::notes::ListAnnotationsRequest,
    ) -> Result<contracts::notes::ListNotesResponse> {
        let reply = self
            .unary::<contracts::notes::ListNotes>(contracts::notes::ListNotes::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn notes_create_note(
        &self,
        message: contracts::notes::CreateNoteRequest,
    ) -> Result<contracts::notes::NoteReceipt> {
        let reply = self
            .unary::<contracts::notes::CreateNote>(contracts::notes::CreateNote::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn health_list_health(
        &self,
        message: contracts::notes::ListAnnotationsRequest,
    ) -> Result<contracts::notes::ListHealthResponse> {
        let reply = self
            .unary::<contracts::notes::ListHealth>(contracts::notes::ListHealth::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn observation_watch_runtime(
        &self,
        message: contracts::observation::WatchRuntimeRequest,
    ) -> Result<ResponseStream<contracts::observation::RuntimeStateFrame>> {
        self.download::<contracts::observation::WatchRuntime>(
            contracts::observation::WatchRuntime::PATH,
            message,
        )
        .await
    }

    pub async fn observation_watch_execution_progress(
        &self,
        message: contracts::observation::WatchExecutionProgressRequest,
    ) -> Result<ResponseStream<contracts::observation::ExecutionProgressFrame>> {
        self.download::<contracts::observation::WatchExecutionProgress>(
            contracts::observation::WatchExecutionProgress::PATH,
            message,
        )
        .await
    }

    pub async fn observation_get_runtime_shell(
        &self,
        message: contracts::observation::GetRuntimeShellRequest,
    ) -> Result<contracts::observation::RuntimeShellSnapshot> {
        let reply = self
            .unary::<contracts::observation::GetRuntimeShell>(
                contracts::observation::GetRuntimeShell::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn observation_get_traceability(
        &self,
        message: contracts::observation::GetTraceabilityRequest,
    ) -> Result<contracts::observation::TraceabilityPage> {
        let reply = self
            .unary::<contracts::observation::GetTraceability>(
                contracts::observation::GetTraceability::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn planning_register_selection(
        &self,
        message: contracts::planning::RegisterSelectionRequest,
    ) -> Result<contracts::planning::PreviewSelectionReply> {
        let reply = self
            .unary::<contracts::planning::RegisterSelection>(
                contracts::planning::RegisterSelection::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn planning_preview_selection(
        &self,
        message: contracts::planning::PreviewSelectionRequest,
    ) -> Result<contracts::planning::PreviewSelectionReply> {
        let reply = self
            .unary::<contracts::planning::PreviewSelection>(
                contracts::planning::PreviewSelection::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn plots_list_analyses(
        &self,
        message: contracts::plots::ListAnalysesRequest,
    ) -> Result<contracts::plots::ListAnalysesReply> {
        let reply = self
            .unary::<contracts::plots::ListAnalyses>(contracts::plots::ListAnalyses::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn plots_query_snapshots(
        &self,
        message: contracts::plots::PlotSnapshotsRequest,
    ) -> Result<contracts::plots::PlotSnapshotsReply> {
        let reply = self
            .unary::<contracts::plots::QuerySnapshots>(
                contracts::plots::QuerySnapshots::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn plots_watch(
        &self,
        message: contracts::plots::PlotWatchRequest,
    ) -> Result<ResponseStream<contracts::plots::PlotStreamFrame>> {
        self.download::<contracts::plots::Watch>(contracts::plots::Watch::PATH, message)
            .await
    }

    pub async fn plugins_list_installations(
        &self,
        message: contracts::plugins::ListInstallationsRequest,
    ) -> Result<contracts::plugins::InstallationPage> {
        let reply = self
            .unary::<contracts::plugins::ListInstallations>(
                contracts::plugins::ListInstallations::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn plugins_get_installation(
        &self,
        message: contracts::plugins::GetInstallationRequest,
    ) -> Result<contracts::plugins::PluginInstallation> {
        let reply = self
            .unary::<contracts::plugins::GetInstallation>(
                contracts::plugins::GetInstallation::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn plugins_stage(
        &self,
        message: impl tokio_stream::Stream<Item = contracts::plugins::StagePluginFrame> + Send + 'static,
    ) -> Result<contracts::plugins::PluginInstallation> {
        self.upload::<contracts::plugins::Stage, _>(contracts::plugins::Stage::PATH, (), message)
            .await
    }

    pub fn plugins_stage_open(
        &self,
    ) -> Result<
        RequestStream<contracts::plugins::StagePluginFrame, contracts::plugins::PluginInstallation>,
    > {
        self.require(contracts::plugins::Stage::PATH)?;
        let client = self.clone();
        RequestStream::new(self, move |stream| async move {
            client.plugins_stage(stream).await
        })
    }

    pub async fn plugins_consent(
        &self,
        message: contracts::plugins::ConsentPluginRequest,
    ) -> Result<contracts::plugins::PluginInstallation> {
        let reply = self
            .unary::<contracts::plugins::Consent>(contracts::plugins::Consent::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn plugins_set_enabled(
        &self,
        message: contracts::plugins::SetPluginEnabledRequest,
    ) -> Result<contracts::plugins::PluginInstallation> {
        let reply = self
            .unary::<contracts::plugins::SetEnabled>(contracts::plugins::SetEnabled::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn plugins_remove(
        &self,
        message: contracts::plugins::RemovePluginRequest,
    ) -> Result<contracts::plugins::RemovePluginReply> {
        let reply = self
            .unary::<contracts::plugins::Remove>(contracts::plugins::Remove::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn plugins_list_admitted_plugins(
        &self,
        message: contracts::plugins::ListAdmittedPluginsRequest,
    ) -> Result<contracts::plugins::AdmittedPluginPage> {
        let reply = self
            .unary::<contracts::plugins::ListAdmittedPlugins>(
                contracts::plugins::ListAdmittedPlugins::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn prompts_list_definitions(
        &self,
        message: contracts::prompts::ListPromptDefinitionsRequest,
    ) -> Result<contracts::prompts::ListPromptDefinitionsReply> {
        let reply = self
            .unary::<contracts::prompts::ListDefinitions>(
                contracts::prompts::ListDefinitions::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn prompts_get_definition(
        &self,
        message: contracts::prompts::GetPromptDefinitionRequest,
    ) -> Result<contracts::prompts::PromptDefinition> {
        let reply = self
            .unary::<contracts::prompts::GetDefinition>(
                contracts::prompts::GetDefinition::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn prompts_list_pending(
        &self,
        message: contracts::prompts::ListPendingPromptsRequest,
    ) -> Result<contracts::prompts::ListPendingPromptsReply> {
        let reply = self
            .unary::<contracts::prompts::ListPending>(
                contracts::prompts::ListPending::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn prompts_get_prompt(
        &self,
        message: contracts::prompts::GetPromptRequest,
    ) -> Result<contracts::prompts::PromptView> {
        let reply = self
            .unary::<contracts::prompts::GetPrompt>(contracts::prompts::GetPrompt::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn prompts_respond(
        &self,
        message: contracts::prompts::RespondToPromptRequest,
    ) -> Result<contracts::prompts::PromptResponseReceipt> {
        let reply = self
            .unary::<contracts::prompts::Respond>(contracts::prompts::Respond::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn query_aggregate(
        &self,
        message: contracts::query::AggregateRequest,
    ) -> Result<contracts::query::AggregateReply> {
        let reply = self
            .unary::<contracts::query::Aggregate>(contracts::query::Aggregate::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn reports_download_cycle_report(
        &self,
        message: contracts::reports::DownloadCycleReportRequest,
    ) -> Result<ResponseStream<contracts::reports::ReportHtmlFrame>> {
        self.download::<contracts::reports::DownloadCycleReport>(
            contracts::reports::DownloadCycleReport::PATH,
            message,
        )
        .await
    }

    pub async fn reports_render_checkpoint_report(
        &self,
        message: contracts::reports::RenderCheckpointReportRequest,
    ) -> Result<contracts::reports::RenderedReport> {
        let reply = self
            .unary::<contracts::reports::RenderCheckpointReport>(
                contracts::reports::RenderCheckpointReport::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn settings_read_storage_policy(
        &self,
        message: contracts::settings::ReadStoragePolicyRequest,
    ) -> Result<contracts::settings::StoragePolicyView> {
        let reply = self
            .unary::<contracts::settings::ReadStoragePolicy>(
                contracts::settings::ReadStoragePolicy::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn settings_update_storage_policy(
        &self,
        message: contracts::settings::UpdateStoragePolicyRequest,
    ) -> Result<contracts::settings::StoragePolicyView> {
        let reply = self
            .unary::<contracts::settings::UpdateStoragePolicy>(
                contracts::settings::UpdateStoragePolicy::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn settings_get_settings(
        &self,
        message: contracts::settings::GetSettingsRequest,
    ) -> Result<contracts::settings::SettingsFrame> {
        let reply = self
            .unary::<contracts::settings::GetSettings>(
                contracts::settings::GetSettings::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn settings_list_storage_sources(
        &self,
        message: contracts::settings::ListStorageSourcesRequest,
    ) -> Result<contracts::settings::StorageSources> {
        let reply = self
            .unary::<contracts::settings::ListStorageSources>(
                contracts::settings::ListStorageSources::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn settings_reorder_storage_sources(
        &self,
        message: contracts::settings::ReorderStorageSourcesRequest,
    ) -> Result<contracts::settings::StorageSources> {
        let reply = self
            .unary::<contracts::settings::ReorderStorageSources>(
                contracts::settings::ReorderStorageSources::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn settings_set_storage_source_enabled(
        &self,
        message: contracts::settings::SetStorageSourceEnabledRequest,
    ) -> Result<contracts::settings::StorageSources> {
        let reply = self
            .unary::<contracts::settings::SetStorageSourceEnabled>(
                contracts::settings::SetStorageSourceEnabled::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn settings_get_plugin_settings(
        &self,
        message: contracts::settings::GetPluginSettingsRequest,
    ) -> Result<contracts::settings::PluginRuntimeSettings> {
        let reply = self
            .unary::<contracts::settings::GetPluginSettings>(
                contracts::settings::GetPluginSettings::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn settings_update_plugin_settings(
        &self,
        message: contracts::settings::UpdatePluginSettingsRequest,
    ) -> Result<contracts::settings::PluginRuntimeSettings> {
        let reply = self
            .unary::<contracts::settings::UpdatePluginSettings>(
                contracts::settings::UpdatePluginSettings::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn settings_update_plugin_credential(
        &self,
        message: contracts::settings::UpdatePluginCredentialRequest,
    ) -> Result<contracts::settings::PluginRuntimeSettings> {
        let reply = self
            .unary::<contracts::settings::UpdatePluginCredential>(
                contracts::settings::UpdatePluginCredential::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn settings_read_application_settings(
        &self,
        message: contracts::settings::ReadApplicationSettingsRequest,
    ) -> Result<contracts::settings::ApplicationSettings> {
        let reply = self
            .unary::<contracts::settings::ReadApplicationSettings>(
                contracts::settings::ReadApplicationSettings::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn settings_update_appearance(
        &self,
        message: contracts::settings::UpdateAppearanceRequest,
    ) -> Result<contracts::settings::ApplicationSettings> {
        let reply = self
            .unary::<contracts::settings::UpdateAppearance>(
                contracts::settings::UpdateAppearance::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn settings_update_data_root(
        &self,
        message: contracts::settings::UpdateDataRootRequest,
    ) -> Result<contracts::settings::ApplicationSettings> {
        let reply = self
            .unary::<contracts::settings::UpdateDataRoot>(
                contracts::settings::UpdateDataRoot::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }

    pub async fn telemetry_list_samples(
        &self,
        message: contracts::telemetry::ListTelemetrySamplesRequest,
    ) -> Result<contracts::telemetry::TelemetrySamplesReply> {
        let reply = self
            .unary::<contracts::telemetry::ListSamples>(
                contracts::telemetry::ListSamples::PATH,
                message,
            )
            .await?;
        super::transport::value(reply)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
