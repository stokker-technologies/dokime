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
use dokime_domain::operations::Endpoint;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Client {
    pub async fn content_follow_execution_file(
        &self, message: contracts::content::FollowExecutionFile,
    ) -> Result<ResponseStream<dokime_domain::model::content::wire::ExecutionFileFollowFrame>> {
        self.download::<contracts::content::FollowExecutionFile>(contracts::content::FollowExecutionFile::PATH, message).await
    }

    pub async fn appearance_get_appearance(
        &self, message: contracts::appearance::GetAppearance,
    ) -> Result<dokime_domain::model::appearance::wire::AppearanceSnapshot> {
        let reply = self.unary::<contracts::appearance::GetAppearance>(contracts::appearance::GetAppearance::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn backend_get_identity(
        &self, message: contracts::backend::GetIdentity,
    ) -> Result<dokime_domain::model::backend::wire::BackendIdentity> {
        let reply = self.unary::<contracts::backend::GetIdentity>(contracts::backend::GetIdentity::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn backend_get_health(
        &self, message: contracts::backend::GetHealth,
    ) -> Result<dokime_domain::model::backend::wire::BackendHealth> {
        let reply = self.unary::<contracts::backend::GetHealth>(contracts::backend::GetHealth::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn backend_get_availability(
        &self, message: contracts::backend::GetAvailability,
    ) -> Result<dokime_domain::model::backend::wire::BackendAvailability> {
        let reply = self.unary::<contracts::backend::GetAvailability>(contracts::backend::GetAvailability::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn backend_get_diagnostics(
        &self, message: contracts::backend::GetDiagnostics,
    ) -> Result<dokime_domain::model::backend::wire::DiagnosticsResponse> {
        let reply = self.unary::<contracts::backend::GetDiagnostics>(contracts::backend::GetDiagnostics::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn backend_collect_diagnostics(
        &self, message: contracts::backend::CollectDiagnostics,
    ) -> Result<ResponseStream<dokime_domain::model::backend::wire::DiagnosticsBundleFrame>> {
        self.download::<contracts::backend::CollectDiagnostics>(contracts::backend::CollectDiagnostics::PATH, message).await
    }

    pub async fn backend_watch_logs(
        &self, message: contracts::backend::WatchLogs,
    ) -> Result<ResponseStream<dokime_domain::model::backend::wire::LogFrame>> {
        self.download::<contracts::backend::WatchLogs>(contracts::backend::WatchLogs::PATH, message).await
    }

    pub async fn catalog_list_suites(
        &self, message: contracts::catalog::ListSuites,
    ) -> Result<dokime_domain::model::catalog::wire::ListSuitesReply> {
        let reply = self.unary::<contracts::catalog::ListSuites>(contracts::catalog::ListSuites::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn catalog_get_suite(
        &self, message: contracts::catalog::GetSuite,
    ) -> Result<dokime_domain::model::catalog::wire::SuiteDefinition> {
        let reply = self.unary::<contracts::catalog::GetSuite>(contracts::catalog::GetSuite::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn catalog_list_record_schemas(
        &self, message: contracts::catalog::ListRecordSchemas,
    ) -> Result<dokime_domain::model::catalog::wire::ListRecordSchemasReply> {
        let reply = self.unary::<contracts::catalog::ListRecordSchemas>(contracts::catalog::ListRecordSchemas::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn catalog_get_record_schema(
        &self, message: contracts::catalog::GetRecordSchema,
    ) -> Result<dokime_domain::model::definition::wire::RecordSchema> {
        let reply = self.unary::<contracts::catalog::GetRecordSchema>(contracts::catalog::GetRecordSchema::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn catalog_list_telemetry_schemas(
        &self, message: contracts::catalog::ListTelemetrySchemas,
    ) -> Result<dokime_domain::model::catalog::wire::ListTelemetrySchemasReply> {
        let reply = self.unary::<contracts::catalog::ListTelemetrySchemas>(contracts::catalog::ListTelemetrySchemas::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn catalog_get_telemetry_schema(
        &self, message: contracts::catalog::GetTelemetrySchema,
    ) -> Result<dokime_domain::model::definition::wire::TelemetrySchema> {
        let reply = self.unary::<contracts::catalog::GetTelemetrySchema>(contracts::catalog::GetTelemetrySchema::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn content_download_appearance_asset(
        &self, message: contracts::content::DownloadAppearanceAsset,
    ) -> Result<ResponseStream<dokime_domain::model::content::wire::AppearanceAssetFrame>> {
        self.download::<contracts::content::DownloadAppearanceAsset>(contracts::content::DownloadAppearanceAsset::PATH, message).await
    }

    pub async fn content_list_execution_files(
        &self, message: contracts::content::ListExecutionFiles,
    ) -> Result<dokime_domain::model::content::wire::ListExecutionFilesReply> {
        let reply = self.unary::<contracts::content::ListExecutionFiles>(contracts::content::ListExecutionFiles::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn content_read_execution_file(
        &self, message: contracts::content::ReadExecutionFile,
    ) -> Result<dokime_domain::model::content::wire::ExecutionFileWindow> {
        let reply = self.unary::<contracts::content::ReadExecutionFile>(contracts::content::ReadExecutionFile::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn content_download_execution_file(
        &self, message: contracts::content::DownloadExecutionFile,
    ) -> Result<ResponseStream<dokime_domain::model::content::wire::ExecutionFileDownloadFrame>> {
        self.download::<contracts::content::DownloadExecutionFile>(contracts::content::DownloadExecutionFile::PATH, message).await
    }

    pub async fn content_list_artifacts(
        &self, message: contracts::content::ListArtifacts,
    ) -> Result<dokime_domain::model::content::wire::ListArtifactsReply> {
        let reply = self.unary::<contracts::content::ListArtifacts>(contracts::content::ListArtifacts::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn content_download_artifact(
        &self, message: contracts::content::DownloadArtifact,
    ) -> Result<ResponseStream<dokime_domain::model::content::wire::ArtifactFrame>> {
        self.download::<contracts::content::DownloadArtifact>(contracts::content::DownloadArtifact::PATH, message).await
    }

    pub async fn content_download_evidence(
        &self, message: contracts::content::DownloadEvidence,
    ) -> Result<ResponseStream<dokime_domain::model::exports::EvidenceExportFrame>> {
        self.download::<contracts::content::DownloadEvidence>(contracts::content::DownloadEvidence::PATH, message).await
    }

    pub async fn content_upload_artifact(
        &self, message: impl tokio_stream::Stream<Item = dokime_domain::model::content::wire::UploadArtifactFrame> + Send + 'static,
    ) -> Result<dokime_domain::model::content::wire::ArtifactMetadata> {
        self.upload::<contracts::content::UploadArtifact, _>(
            contracts::content::UploadArtifact::PATH,
            contracts::content::UploadArtifact {},
            message,
        )
        .await
    }

    pub fn content_upload_artifact_open(
        &self,
    ) -> Result<
        RequestStream<dokime_domain::model::content::wire::UploadArtifactFrame, dokime_domain::model::content::wire::ArtifactMetadata>,
    > {
        self.require(contracts::content::UploadArtifact::PATH)?;
        let client = self.clone();
        RequestStream::new(self, move |stream| async move { client.content_upload_artifact(stream).await })
    }

    pub async fn documents_capture_current_document(
        &self, message: contracts::documents::CaptureCurrentDocument,
    ) -> Result<ResponseStream<dokime_domain::model::documents::wire::DocumentFrame>> {
        self.download::<contracts::documents::CaptureCurrentDocument>(contracts::documents::CaptureCurrentDocument::PATH, message).await
    }

    pub async fn evidence_list_records(
        &self, message: contracts::evidence::ListRecords,
    ) -> Result<dokime_domain::model::evidence::wire::RecordPage> {
        let reply = self.unary::<contracts::evidence::ListRecords>(contracts::evidence::ListRecords::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn evidence_watch_records(
        &self, message: contracts::evidence::WatchRecords,
    ) -> Result<ResponseStream<dokime_domain::model::evidence::wire::RecordPage>> {
        self.download::<contracts::evidence::WatchRecords>(contracts::evidence::WatchRecords::PATH, message).await
    }

    pub async fn evidence_get_captured_record_schema(
        &self, message: contracts::evidence::GetCapturedRecordSchema,
    ) -> Result<dokime_domain::model::definition::wire::RecordSchema> {
        let reply =
            self.unary::<contracts::evidence::GetCapturedRecordSchema>(contracts::evidence::GetCapturedRecordSchema::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_execution_steps(
        &self, message: contracts::execution::ListExecutionSteps,
    ) -> Result<dokime_domain::model::execution::wire::ListExecutionStepsReply> {
        let reply = self.unary::<contracts::execution::ListExecutionSteps>(contracts::execution::ListExecutionSteps::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_step_criteria(
        &self, message: contracts::execution::ListStepCriteria,
    ) -> Result<dokime_domain::model::execution::wire::ListStepCriteriaReply> {
        let reply = self.unary::<contracts::execution::ListStepCriteria>(contracts::execution::ListStepCriteria::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_copy_cycle(
        &self, message: contracts::execution::CopyCycle,
    ) -> Result<dokime_domain::model::execution::wire::CreateCycleReply> {
        let reply = self.unary::<contracts::execution::CopyCycle>(contracts::execution::CopyCycle::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_create_cycle(
        &self, message: contracts::execution::CreateCycle,
    ) -> Result<dokime_domain::model::execution::wire::CreateCycleReply> {
        let reply = self.unary::<contracts::execution::CreateCycle>(contracts::execution::CreateCycle::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_get_cycle(
        &self, message: contracts::execution::GetCycle,
    ) -> Result<dokime_domain::model::execution::wire::CycleSummary> {
        let reply = self.unary::<contracts::execution::GetCycle>(contracts::execution::GetCycle::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_get_case(
        &self, message: contracts::execution::GetCase,
    ) -> Result<dokime_domain::model::execution::wire::CaseDetail> {
        let reply = self.unary::<contracts::execution::GetCase>(contracts::execution::GetCase::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_get_execution(
        &self, message: contracts::execution::GetExecution,
    ) -> Result<dokime_domain::model::execution::wire::ExecutionDetail> {
        let reply = self.unary::<contracts::execution::GetExecution>(contracts::execution::GetExecution::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_case_parameters(
        &self, message: contracts::execution::ListCaseParameters,
    ) -> Result<dokime_domain::model::execution::wire::ListCaseParametersReply> {
        let reply = self.unary::<contracts::execution::ListCaseParameters>(contracts::execution::ListCaseParameters::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_execution_parameters(
        &self, message: contracts::execution::ListExecutionParameters,
    ) -> Result<dokime_domain::model::execution::wire::ListMaterializedParametersReply> {
        let reply = self
            .unary::<contracts::execution::ListExecutionParameters>(contracts::execution::ListExecutionParameters::PATH, message)
            .await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_cycles(
        &self, message: contracts::execution::ListCycles,
    ) -> Result<dokime_domain::model::execution::wire::ListCyclesReply> {
        let reply = self.unary::<contracts::execution::ListCycles>(contracts::execution::ListCycles::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_cases(
        &self, message: contracts::execution::ListCases,
    ) -> Result<dokime_domain::model::execution::wire::ListCasesReply> {
        let reply = self.unary::<contracts::execution::ListCases>(contracts::execution::ListCases::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_list_executions(
        &self, message: contracts::execution::ListExecutions,
    ) -> Result<dokime_domain::model::execution::wire::ListExecutionsReply> {
        let reply = self.unary::<contracts::execution::ListExecutions>(contracts::execution::ListExecutions::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_append_planned_cases(
        &self, message: contracts::execution::AppendPlannedCases,
    ) -> Result<dokime_domain::model::execution::wire::AppendCasesReply> {
        let reply = self.unary::<contracts::execution::AppendPlannedCases>(contracts::execution::AppendPlannedCases::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_run_cycle(
        &self, message: contracts::execution::RunCycle,
    ) -> Result<dokime_domain::model::execution::wire::RunReply> {
        let reply = self.unary::<contracts::execution::RunCycle>(contracts::execution::RunCycle::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_run_case(
        &self, message: contracts::execution::RunCase,
    ) -> Result<dokime_domain::model::execution::wire::RunReply> {
        let reply = self.unary::<contracts::execution::RunCase>(contracts::execution::RunCase::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_run_execution(
        &self, message: contracts::execution::RunExecution,
    ) -> Result<dokime_domain::model::execution::wire::RunReply> {
        let reply = self.unary::<contracts::execution::RunExecution>(contracts::execution::RunExecution::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_pause_execution(
        &self, message: contracts::execution::PauseExecution,
    ) -> Result<dokime_domain::model::execution::wire::ControlReply> {
        let reply = self.unary::<contracts::execution::PauseExecution>(contracts::execution::PauseExecution::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_resume_execution(
        &self, message: contracts::execution::ResumeExecution,
    ) -> Result<dokime_domain::model::execution::wire::ControlReply> {
        let reply = self.unary::<contracts::execution::ResumeExecution>(contracts::execution::ResumeExecution::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_stop_execution(
        &self, message: contracts::execution::StopExecution,
    ) -> Result<dokime_domain::model::execution::wire::ControlReply> {
        let reply = self.unary::<contracts::execution::StopExecution>(contracts::execution::StopExecution::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_get_run_request(
        &self, message: contracts::execution::GetRunRequest,
    ) -> Result<dokime_domain::model::execution::wire::RunRequestReceipt> {
        let reply = self.unary::<contracts::execution::GetRunRequest>(contracts::execution::GetRunRequest::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_submit_execution_command(
        &self, message: contracts::execution::SubmitExecutionCommand,
    ) -> Result<dokime_domain::model::execution::wire::ExecutionCommandReceipt> {
        let reply =
            self.unary::<contracts::execution::SubmitExecutionCommand>(contracts::execution::SubmitExecutionCommand::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_get_execution_command(
        &self, message: contracts::execution::GetExecutionCommand,
    ) -> Result<dokime_domain::model::execution::wire::ExecutionCommandReceipt> {
        let reply =
            self.unary::<contracts::execution::GetExecutionCommand>(contracts::execution::GetExecutionCommand::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_pause_cycle(
        &self, message: contracts::execution::PauseCycle,
    ) -> Result<dokime_domain::model::execution::wire::ControlReply> {
        let reply = self.unary::<contracts::execution::PauseCycle>(contracts::execution::PauseCycle::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_resume_cycle(
        &self, message: contracts::execution::ResumeCycle,
    ) -> Result<dokime_domain::model::execution::wire::ControlReply> {
        let reply = self.unary::<contracts::execution::ResumeCycle>(contracts::execution::ResumeCycle::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_stop_cycle(
        &self, message: contracts::execution::StopCycle,
    ) -> Result<dokime_domain::model::execution::wire::ControlReply> {
        let reply = self.unary::<contracts::execution::StopCycle>(contracts::execution::StopCycle::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_delete_case(
        &self, message: contracts::execution::DeleteCase,
    ) -> Result<dokime_domain::model::execution::wire::ControlReply> {
        let reply = self.unary::<contracts::execution::DeleteCase>(contracts::execution::DeleteCase::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_schedule_case_rerun(
        &self, message: contracts::execution::ScheduleCaseRerun,
    ) -> Result<dokime_domain::model::execution::wire::ControlReply> {
        let reply = self.unary::<contracts::execution::ScheduleCaseRerun>(contracts::execution::ScheduleCaseRerun::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_set_cycle_archived(
        &self, message: contracts::execution::SetCycleArchived,
    ) -> Result<dokime_domain::model::execution::wire::ArchiveReply> {
        let reply = self.unary::<contracts::execution::SetCycleArchived>(contracts::execution::SetCycleArchived::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_set_execution_archived(
        &self, message: contracts::execution::SetExecutionArchived,
    ) -> Result<dokime_domain::model::execution::wire::ArchiveReply> {
        let reply =
            self.unary::<contracts::execution::SetExecutionArchived>(contracts::execution::SetExecutionArchived::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_rename_cycle(
        &self, message: contracts::execution::RenameCycle,
    ) -> Result<dokime_domain::model::execution::wire::RenameReply> {
        let reply = self.unary::<contracts::execution::RenameCycle>(contracts::execution::RenameCycle::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_rename_execution(
        &self, message: contracts::execution::RenameExecution,
    ) -> Result<dokime_domain::model::execution::wire::RenameReply> {
        let reply = self.unary::<contracts::execution::RenameExecution>(contracts::execution::RenameExecution::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn execution_update_cycle_policy(
        &self, message: contracts::execution::UpdateCyclePolicy,
    ) -> Result<dokime_domain::model::execution::wire::ControlReply> {
        let reply = self.unary::<contracts::execution::UpdateCyclePolicy>(contracts::execution::UpdateCyclePolicy::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn history_browse_executions(
        &self, message: contracts::history::BrowseExecutions,
    ) -> Result<dokime_domain::model::history::wire::ExecutionBrowsePage> {
        let reply = self.unary::<contracts::history::BrowseExecutions>(contracts::history::BrowseExecutions::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn history_get_execution_checkpoint(
        &self, message: contracts::history::GetExecutionCheckpoint,
    ) -> Result<dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint> {
        let reply =
            self.unary::<contracts::history::GetExecutionCheckpoint>(contracts::history::GetExecutionCheckpoint::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn history_delete_execution(
        &self, message: contracts::history::DeleteExecution,
    ) -> Result<dokime_domain::model::history::wire::DeleteExecutionResponse> {
        let reply = self.unary::<contracts::history::DeleteExecution>(contracts::history::DeleteExecution::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn history_get_checkpoint_document(
        &self, message: contracts::history::GetCheckpointDocument,
    ) -> Result<dokime_domain::model::history::wire::CheckpointDocument> {
        let reply =
            self.unary::<contracts::history::GetCheckpointDocument>(contracts::history::GetCheckpointDocument::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn history_get_checkpoint_evidence(
        &self, message: contracts::history::GetCheckpointEvidence,
    ) -> Result<dokime_domain::model::history::wire::CheckpointEvidence> {
        let reply =
            self.unary::<contracts::history::GetCheckpointEvidence>(contracts::history::GetCheckpointEvidence::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn history_read_checkpoint_output(
        &self, message: contracts::history::ReadCheckpointOutput,
    ) -> Result<dokime_domain::model::history::wire::CheckpointFileWindow> {
        let reply = self.unary::<contracts::history::ReadCheckpointOutput>(contracts::history::ReadCheckpointOutput::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn history_download_checkpoint_artifact(
        &self, message: contracts::history::DownloadCheckpointArtifact,
    ) -> Result<ResponseStream<dokime_domain::model::content::wire::ArtifactFrame>> {
        self.download::<contracts::history::DownloadCheckpointArtifact>(contracts::history::DownloadCheckpointArtifact::PATH, message).await
    }

    pub async fn history_list_checkpoint_records(
        &self, message: contracts::history::ListCheckpointRecords,
    ) -> Result<dokime_domain::model::history::wire::CheckpointRecordPage> {
        let reply =
            self.unary::<contracts::history::ListCheckpointRecords>(contracts::history::ListCheckpointRecords::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn history_list_checkpoint_telemetry(
        &self, message: contracts::history::ListCheckpointTelemetry,
    ) -> Result<dokime_domain::model::history::wire::CheckpointTelemetryPage> {
        let reply =
            self.unary::<contracts::history::ListCheckpointTelemetry>(contracts::history::ListCheckpointTelemetry::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn history_get_checkpoint_context(
        &self, message: contracts::history::GetCheckpointContext,
    ) -> Result<dokime_domain::model::history::wire::CheckpointContext> {
        let reply = self.unary::<contracts::history::GetCheckpointContext>(contracts::history::GetCheckpointContext::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn notes_list_notes(
        &self, message: contracts::notes::ListNotes,
    ) -> Result<dokime_domain::model::notes::wire::ListNotesResponse> {
        let reply = self.unary::<contracts::notes::ListNotes>(contracts::notes::ListNotes::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn notes_create_note(&self, message: contracts::notes::CreateNote) -> Result<dokime_domain::model::notes::wire::NoteReceipt> {
        let reply = self.unary::<contracts::notes::CreateNote>(contracts::notes::CreateNote::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn notes_edit_note(
        &self, message: contracts::notes::EditNote,
    ) -> Result<dokime_domain::model::notes::wire::EditNoteResponse> {
        let reply = self.unary::<contracts::notes::EditNote>(contracts::notes::EditNote::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn health_list_health(
        &self, message: contracts::notes::ListHealth,
    ) -> Result<dokime_domain::model::notes::wire::ListHealthResponse> {
        let reply = self.unary::<contracts::notes::ListHealth>(contracts::notes::ListHealth::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn observation_watch_runtime(
        &self, message: contracts::observation::WatchRuntime,
    ) -> Result<ResponseStream<dokime_domain::model::observation::wire::RuntimeStateFrame>> {
        self.download::<contracts::observation::WatchRuntime>(contracts::observation::WatchRuntime::PATH, message).await
    }

    pub async fn observation_watch_execution_progress(
        &self, message: contracts::observation::WatchExecutionProgress,
    ) -> Result<ResponseStream<dokime_domain::model::observation::wire::ExecutionProgressFrame>> {
        self.download::<contracts::observation::WatchExecutionProgress>(contracts::observation::WatchExecutionProgress::PATH, message).await
    }

    pub async fn observation_get_runtime_shell(
        &self, message: contracts::observation::GetRuntimeShell,
    ) -> Result<dokime_domain::model::observation::wire::RuntimeShellSnapshot> {
        let reply = self.unary::<contracts::observation::GetRuntimeShell>(contracts::observation::GetRuntimeShell::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn observation_get_traceability(
        &self, message: contracts::observation::GetTraceability,
    ) -> Result<dokime_domain::model::observation::wire::TraceabilityPage> {
        let reply = self.unary::<contracts::observation::GetTraceability>(contracts::observation::GetTraceability::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn planning_register_selection(
        &self, message: contracts::planning::RegisterSelection,
    ) -> Result<dokime_domain::model::planning::wire::PreviewSelectionReply> {
        let reply = self.unary::<contracts::planning::RegisterSelection>(contracts::planning::RegisterSelection::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn planning_preview_selection(
        &self, message: contracts::planning::PreviewSelection,
    ) -> Result<dokime_domain::model::planning::wire::PreviewSelectionReply> {
        let reply = self.unary::<contracts::planning::PreviewSelection>(contracts::planning::PreviewSelection::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn plots_list_analyses(
        &self, message: contracts::plots::ListAnalyses,
    ) -> Result<dokime_domain::model::plots::wire::ListAnalysesReply> {
        let reply = self.unary::<contracts::plots::ListAnalyses>(contracts::plots::ListAnalyses::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn plots_query_snapshots(
        &self, message: contracts::plots::QuerySnapshots,
    ) -> Result<dokime_domain::model::plots::wire::PlotSnapshotsReply> {
        let reply = self.unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn plots_watch(
        &self, message: contracts::plots::Watch,
    ) -> Result<ResponseStream<dokime_domain::model::plots::wire::PlotStreamFrame>> {
        self.download::<contracts::plots::Watch>(contracts::plots::Watch::PATH, message).await
    }

    pub async fn plugins_list_installations(
        &self, message: contracts::plugins::ListInstallations,
    ) -> Result<dokime_domain::model::plugins::wire::InstallationPage> {
        let reply = self.unary::<contracts::plugins::ListInstallations>(contracts::plugins::ListInstallations::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn plugins_get_installation(
        &self, message: contracts::plugins::GetInstallation,
    ) -> Result<dokime_domain::model::plugins::wire::PluginInstallation> {
        let reply = self.unary::<contracts::plugins::GetInstallation>(contracts::plugins::GetInstallation::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn plugins_stage(
        &self, message: impl tokio_stream::Stream<Item = dokime_domain::model::plugins::wire::StagePluginFrame> + Send + 'static,
    ) -> Result<dokime_domain::model::plugins::wire::PluginInstallation> {
        self.upload::<contracts::plugins::Stage, _>(contracts::plugins::Stage::PATH, contracts::plugins::Stage {}, message).await
    }

    pub fn plugins_stage_open(
        &self,
    ) -> Result<RequestStream<dokime_domain::model::plugins::wire::StagePluginFrame, dokime_domain::model::plugins::wire::PluginInstallation>>
    {
        self.require(contracts::plugins::Stage::PATH)?;
        let client = self.clone();
        RequestStream::new(self, move |stream| async move { client.plugins_stage(stream).await })
    }

    pub async fn plugins_consent(
        &self, message: contracts::plugins::Consent,
    ) -> Result<dokime_domain::model::plugins::wire::PluginInstallation> {
        let reply = self.unary::<contracts::plugins::Consent>(contracts::plugins::Consent::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn plugins_set_enabled(
        &self, message: contracts::plugins::SetEnabled,
    ) -> Result<dokime_domain::model::plugins::wire::PluginInstallation> {
        let reply = self.unary::<contracts::plugins::SetEnabled>(contracts::plugins::SetEnabled::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn plugins_remove(
        &self, message: contracts::plugins::Remove,
    ) -> Result<dokime_domain::model::plugins::wire::RemovePluginReply> {
        let reply = self.unary::<contracts::plugins::Remove>(contracts::plugins::Remove::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn plugins_list_admitted_plugins(
        &self, message: contracts::plugins::ListAdmittedPlugins,
    ) -> Result<dokime_domain::model::plugins::wire::AdmittedPluginPage> {
        let reply = self.unary::<contracts::plugins::ListAdmittedPlugins>(contracts::plugins::ListAdmittedPlugins::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn prompts_list_definitions(
        &self, message: contracts::prompts::ListDefinitions,
    ) -> Result<dokime_domain::model::prompts::wire::ListPromptDefinitionsReply> {
        let reply = self.unary::<contracts::prompts::ListDefinitions>(contracts::prompts::ListDefinitions::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn prompts_get_definition(
        &self, message: contracts::prompts::GetDefinition,
    ) -> Result<dokime_domain::model::prompts::wire::PromptDefinition> {
        let reply = self.unary::<contracts::prompts::GetDefinition>(contracts::prompts::GetDefinition::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn prompts_list_pending(
        &self, message: contracts::prompts::ListPending,
    ) -> Result<dokime_domain::model::prompts::wire::ListPendingPromptsReply> {
        let reply = self.unary::<contracts::prompts::ListPending>(contracts::prompts::ListPending::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn prompts_get_prompt(
        &self, message: contracts::prompts::GetPrompt,
    ) -> Result<dokime_domain::model::prompts::wire::PromptView> {
        let reply = self.unary::<contracts::prompts::GetPrompt>(contracts::prompts::GetPrompt::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn prompts_respond(
        &self, message: contracts::prompts::Respond,
    ) -> Result<dokime_domain::model::prompts::wire::PromptResponseReceipt> {
        let reply = self.unary::<contracts::prompts::Respond>(contracts::prompts::Respond::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn query_aggregate(&self, message: contracts::query::Aggregate) -> Result<dokime_domain::model::query::wire::AggregateReply> {
        let reply = self.unary::<contracts::query::Aggregate>(contracts::query::Aggregate::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn reports_download_cycle_report(
        &self, message: contracts::reports::DownloadCycleReport,
    ) -> Result<ResponseStream<dokime_domain::model::reports::wire::ReportHtmlFrame>> {
        self.download::<contracts::reports::DownloadCycleReport>(contracts::reports::DownloadCycleReport::PATH, message).await
    }

    pub async fn reports_download_execution_report(
        &self, message: contracts::reports::DownloadExecutionReport,
    ) -> Result<ResponseStream<dokime_domain::model::reports::wire::ReportHtmlFrame>> {
        self.download::<contracts::reports::DownloadExecutionReport>(contracts::reports::DownloadExecutionReport::PATH, message).await
    }

    pub async fn settings_read_storage_policy(
        &self, message: contracts::settings::ReadStoragePolicy,
    ) -> Result<dokime_domain::model::settings::wire::StoragePolicyView> {
        let reply = self.unary::<contracts::settings::ReadStoragePolicy>(contracts::settings::ReadStoragePolicy::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn settings_update_storage_policy(
        &self, message: contracts::settings::UpdateStoragePolicy,
    ) -> Result<dokime_domain::model::settings::wire::StoragePolicyView> {
        let reply = self.unary::<contracts::settings::UpdateStoragePolicy>(contracts::settings::UpdateStoragePolicy::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn settings_get_settings(
        &self, message: contracts::settings::GetSettings,
    ) -> Result<dokime_domain::model::settings::wire::SettingsFrame> {
        let reply = self.unary::<contracts::settings::GetSettings>(contracts::settings::GetSettings::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn settings_list_storage_sources(
        &self, message: contracts::settings::ListStorageSources,
    ) -> Result<dokime_domain::model::settings::wire::StorageSources> {
        let reply = self.unary::<contracts::settings::ListStorageSources>(contracts::settings::ListStorageSources::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn settings_reorder_storage_sources(
        &self, message: contracts::settings::ReorderStorageSources,
    ) -> Result<dokime_domain::model::settings::wire::StorageSources> {
        let reply =
            self.unary::<contracts::settings::ReorderStorageSources>(contracts::settings::ReorderStorageSources::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn settings_set_storage_source_enabled(
        &self, message: contracts::settings::SetStorageSourceEnabled,
    ) -> Result<dokime_domain::model::settings::wire::StorageSources> {
        let reply =
            self.unary::<contracts::settings::SetStorageSourceEnabled>(contracts::settings::SetStorageSourceEnabled::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn settings_get_plugin_settings(
        &self, message: contracts::settings::GetPluginSettings,
    ) -> Result<dokime_domain::model::settings::wire::PluginRuntimeSettings> {
        let reply = self.unary::<contracts::settings::GetPluginSettings>(contracts::settings::GetPluginSettings::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn settings_update_plugin_settings(
        &self, message: contracts::settings::UpdatePluginSettings,
    ) -> Result<dokime_domain::model::settings::wire::PluginRuntimeSettings> {
        let reply =
            self.unary::<contracts::settings::UpdatePluginSettings>(contracts::settings::UpdatePluginSettings::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn settings_update_plugin_credential(
        &self, message: contracts::settings::UpdatePluginCredential,
    ) -> Result<dokime_domain::model::settings::wire::PluginRuntimeSettings> {
        let reply =
            self.unary::<contracts::settings::UpdatePluginCredential>(contracts::settings::UpdatePluginCredential::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn settings_read_application_settings(
        &self, message: contracts::settings::ReadApplicationSettings,
    ) -> Result<dokime_domain::model::settings::wire::ApplicationSettings> {
        let reply =
            self.unary::<contracts::settings::ReadApplicationSettings>(contracts::settings::ReadApplicationSettings::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn settings_update_appearance(
        &self, message: contracts::settings::UpdateAppearance,
    ) -> Result<dokime_domain::model::settings::wire::ApplicationSettings> {
        let reply = self.unary::<contracts::settings::UpdateAppearance>(contracts::settings::UpdateAppearance::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn settings_update_report_defaults(
        &self, message: contracts::settings::UpdateReportDefaults,
    ) -> Result<dokime_domain::model::settings::wire::ApplicationSettings> {
        let reply =
            self.unary::<contracts::settings::UpdateReportDefaults>(contracts::settings::UpdateReportDefaults::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn settings_update_data_root(
        &self, message: contracts::settings::UpdateDataRoot,
    ) -> Result<dokime_domain::model::settings::wire::ApplicationSettings> {
        let reply = self.unary::<contracts::settings::UpdateDataRoot>(contracts::settings::UpdateDataRoot::PATH, message).await?;
        super::transport::value(reply)
    }

    pub async fn telemetry_list_samples(
        &self, message: contracts::telemetry::ListSamples,
    ) -> Result<dokime_domain::model::telemetry::wire::TelemetrySamplesReply> {
        let reply = self.unary::<contracts::telemetry::ListSamples>(contracts::telemetry::ListSamples::PATH, message).await?;
        super::transport::value(reply)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
