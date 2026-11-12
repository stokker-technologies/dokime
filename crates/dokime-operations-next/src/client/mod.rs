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

pub mod appearance_get_appearance;
pub mod backend_collect_diagnostics;
pub mod backend_get_availability;
pub mod backend_get_health;
pub mod backend_get_identity;
mod backend_streams;
pub mod backend_watch_logs;
pub mod backend_watch_trace;
pub mod catalog_get_record_schema;
pub mod catalog_get_suite;
pub mod catalog_get_telemetry_schema;
mod catalog_inventory;
pub mod catalog_list_record_schemas;
pub mod catalog_list_suites;
pub mod catalog_list_telemetry_schemas;
pub mod content_download_artifact;
pub mod content_download_evidence;
pub mod content_download_execution_file;
pub mod content_follow_execution_file;
pub mod content_list_artifacts;
pub mod content_list_execution_files;
pub mod content_read_execution_file;
mod content_transfer;
pub mod content_upload_artifact;
pub mod conversation;
pub mod document_expansion;
pub mod document_plot;
pub mod documents_capture_current_document;
pub mod documents_update_plot_settings;
pub mod evidence_get_captured_record_schema;
pub mod evidence_list_records;
mod evidence_selection;
pub mod evidence_watch_records;
pub mod execution_append_executions;
pub mod execution_cancel_execution;
pub mod execution_create_cycle;
pub mod execution_delete_case;
pub mod execution_get_case;
pub mod execution_get_cycle;
pub mod execution_get_execution;
pub mod execution_get_execution_command;
pub mod execution_list_case_parameters;
pub mod execution_list_cases;
pub mod execution_list_cycles;
pub mod execution_list_execution_parameters;
pub mod execution_list_execution_steps;
pub mod execution_list_executions;
pub mod execution_pause_cycle;
pub mod execution_pause_execution;
pub mod execution_rename_cycle;
pub mod execution_rename_execution;
pub mod execution_rerun_cycle;
pub mod execution_resume_cycle;
pub mod execution_resume_execution;
pub mod execution_seal_cycle;
pub mod execution_set_cycle_archived;
pub mod execution_set_execution_archived;
pub mod execution_start_execution;
mod execution_state;
pub mod execution_stop_execution;
pub mod execution_submit_execution_command;
pub mod execution_update_cycle_policy;
pub mod health_list_health;
mod history;
pub mod history_browse_cycles;
pub mod history_browse_executions;
pub mod history_delete_execution;
pub mod history_download_checkpoint_artifact;
pub mod history_get_checkpoint_context;
pub mod history_get_checkpoint_document;
pub mod history_get_checkpoint_evidence;
pub mod history_get_execution_checkpoint;
pub mod history_list_checkpoint_records;
pub mod history_list_checkpoint_telemetry;
pub mod history_read_checkpoint_output;
pub mod notes_create_note;
pub mod notes_edit_note;
pub mod notes_list_notes;
mod notes_selection;
pub mod observation_get_runtime_shell;
pub mod observation_get_traceability;
pub mod observation_watch_execution_progress;
pub mod observation_watch_runtime;
mod paging;
mod planning;
pub mod planning_materialize_template;
pub mod planning_preview_selection;
pub mod planning_register_selection;
pub mod plot_analysis;
pub mod plots_list_analyses;
pub mod plots_query_snapshots;
pub mod plots_watch;
pub mod plugins_add;
pub mod plugins_get_installation;
pub mod plugins_install;
pub mod plugins_list;
pub mod plugins_list_admitted_plugins;
pub mod plugins_list_installations;
pub mod plugins_reload;
pub mod plugins_remove;
pub mod plugins_set_enabled;
mod projection;
mod prompt_catalog;
mod prompt_validation;
pub mod prompts_get_definition;
pub mod prompts_get_prompt;
pub mod prompts_list_definitions;
pub mod prompts_list_pending;
pub mod prompts_respond;
pub mod query_list_executions;
pub mod remote_create_pairing;
pub mod remote_read_listener;
pub mod remote_regenerate_certificate;
pub mod remote_update_listener;
pub mod reports_download_cycle_report;
pub mod reports_download_execution_report;
pub mod reports_materialize_report;
pub mod reports_publish_execution_report;
pub mod run_call;
pub mod runtime_get_health;
pub mod runtime_get_identity;
pub mod secrets_export;
pub mod secrets_import;
pub mod secrets_list;
pub mod secrets_lookup;
pub mod secrets_preview;
pub mod secrets_put;
pub mod secrets_remove;
pub(crate) mod secrets_store;
pub mod settings_get_plugin_settings;
pub mod settings_get_settings;
pub mod settings_list_storage_sources;
pub(crate) mod settings_preferences;
pub mod settings_read_application_settings;
pub mod settings_reorder_storage_sources;
pub mod settings_set_storage_source_enabled;
pub mod settings_update_application_settings;
pub mod settings_update_plugin_settings;
pub mod telemetry_list_samples;
pub mod telemetry_watch_samples;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashMap, future::Future, sync::Arc, time::Duration};

// Third Party Imports
use async_trait::async_trait;
use harmos_protocol::v1 as h;
use prost::Message;
use tokio::sync::watch;

// Local Imports
use crate::{Error, Result};
use dokime_protocol::v1;
use dokime_storage::Store;

/* === Constants === */

const DEFAULT_PAGE_ITEMS: usize = 50;
const MAX_PAGE_ITEMS: usize = 500;

pub const METHODS: &[&str] = &[
    "/dokime.v1.Appearance/GetAppearance",
    "/dokime.v1.Appearance/RunGetAppearance",
    "/dokime.v1.Backend/CollectDiagnostics",
    "/dokime.v1.Backend/GetAvailability",
    "/dokime.v1.Backend/GetHealth",
    "/dokime.v1.Backend/GetIdentity",
    "/dokime.v1.Backend/RunCollectDiagnostics",
    "/dokime.v1.Backend/RunGetAvailability",
    "/dokime.v1.Backend/RunGetHealth",
    "/dokime.v1.Backend/RunGetIdentity",
    "/dokime.v1.Backend/RunWatchLogs",
    "/dokime.v1.Backend/RunWatchTrace",
    "/dokime.v1.Backend/WatchLogs",
    "/dokime.v1.Backend/WatchTrace",
    "/dokime.v1.Catalog/GetRecordSchema",
    "/dokime.v1.Catalog/GetSuite",
    "/dokime.v1.Catalog/GetTelemetrySchema",
    "/dokime.v1.Catalog/ListRecordSchemas",
    "/dokime.v1.Catalog/ListSuites",
    "/dokime.v1.Catalog/ListTelemetrySchemas",
    "/dokime.v1.Catalog/RunGetRecordSchema",
    "/dokime.v1.Catalog/RunGetSuite",
    "/dokime.v1.Catalog/RunGetTelemetrySchema",
    "/dokime.v1.Catalog/RunListRecordSchemas",
    "/dokime.v1.Catalog/RunListSuites",
    "/dokime.v1.Catalog/RunListTelemetrySchemas",
    "/dokime.v1.Content/DownloadArtifact",
    "/dokime.v1.Content/DownloadEvidence",
    "/dokime.v1.Content/DownloadExecutionFile",
    "/dokime.v1.Content/FollowExecutionFile",
    "/dokime.v1.Content/ListArtifacts",
    "/dokime.v1.Content/ListExecutionFiles",
    "/dokime.v1.Content/ReadExecutionFile",
    "/dokime.v1.Content/RunDownloadArtifact",
    "/dokime.v1.Content/RunDownloadEvidence",
    "/dokime.v1.Content/RunDownloadExecutionFile",
    "/dokime.v1.Content/RunFollowExecutionFile",
    "/dokime.v1.Content/RunListArtifacts",
    "/dokime.v1.Content/RunListExecutionFiles",
    "/dokime.v1.Content/RunReadExecutionFile",
    "/dokime.v1.Content/RunUploadArtifact",
    "/dokime.v1.Content/UploadArtifact",
    "/dokime.v1.Documents/CaptureCurrentDocument",
    "/dokime.v1.Documents/RunCaptureCurrentDocument",
    "/dokime.v1.Documents/RunUpdatePlotSettings",
    "/dokime.v1.Documents/UpdatePlotSettings",
    "/dokime.v1.Evidence/GetCapturedRecordSchema",
    "/dokime.v1.Evidence/ListRecords",
    "/dokime.v1.Evidence/RunGetCapturedRecordSchema",
    "/dokime.v1.Evidence/RunListRecords",
    "/dokime.v1.Evidence/RunWatchRecords",
    "/dokime.v1.Evidence/WatchRecords",
    "/dokime.v1.Execution/AppendExecutions",
    "/dokime.v1.Execution/CancelExecution",
    "/dokime.v1.Execution/CreateCycle",
    "/dokime.v1.Execution/DeleteCase",
    "/dokime.v1.Execution/GetCase",
    "/dokime.v1.Execution/GetCycle",
    "/dokime.v1.Execution/GetExecution",
    "/dokime.v1.Execution/GetExecutionCommand",
    "/dokime.v1.Execution/ListCaseParameters",
    "/dokime.v1.Execution/ListCases",
    "/dokime.v1.Execution/ListCycles",
    "/dokime.v1.Execution/ListExecutionParameters",
    "/dokime.v1.Execution/ListExecutionSteps",
    "/dokime.v1.Execution/ListExecutions",
    "/dokime.v1.Execution/PauseCycle",
    "/dokime.v1.Execution/PauseExecution",
    "/dokime.v1.Execution/RenameCycle",
    "/dokime.v1.Execution/RenameExecution",
    "/dokime.v1.Execution/RerunCycle",
    "/dokime.v1.Execution/ResumeCycle",
    "/dokime.v1.Execution/ResumeExecution",
    "/dokime.v1.Execution/RunAppendExecutions",
    "/dokime.v1.Execution/RunCancelExecution",
    "/dokime.v1.Execution/RunCreateCycle",
    "/dokime.v1.Execution/RunDeleteCase",
    "/dokime.v1.Execution/RunGetCase",
    "/dokime.v1.Execution/RunGetCycle",
    "/dokime.v1.Execution/RunGetExecution",
    "/dokime.v1.Execution/RunGetExecutionCommand",
    "/dokime.v1.Execution/RunListCaseParameters",
    "/dokime.v1.Execution/RunListCases",
    "/dokime.v1.Execution/RunListCycles",
    "/dokime.v1.Execution/RunListExecutionParameters",
    "/dokime.v1.Execution/RunListExecutionSteps",
    "/dokime.v1.Execution/RunListExecutions",
    "/dokime.v1.Execution/RunPauseCycle",
    "/dokime.v1.Execution/RunPauseExecution",
    "/dokime.v1.Execution/RunRenameCycle",
    "/dokime.v1.Execution/RunRenameExecution",
    "/dokime.v1.Execution/RunRerunCycle",
    "/dokime.v1.Execution/RunResumeCycle",
    "/dokime.v1.Execution/RunResumeExecution",
    "/dokime.v1.Execution/RunSealCycle",
    "/dokime.v1.Execution/RunSetCycleArchived",
    "/dokime.v1.Execution/RunSetExecutionArchived",
    "/dokime.v1.Execution/RunStartExecution",
    "/dokime.v1.Execution/RunStopExecution",
    "/dokime.v1.Execution/RunSubmitExecutionCommand",
    "/dokime.v1.Execution/RunUpdateCyclePolicy",
    "/dokime.v1.Execution/SealCycle",
    "/dokime.v1.Execution/SetCycleArchived",
    "/dokime.v1.Execution/SetExecutionArchived",
    "/dokime.v1.Execution/StartExecution",
    "/dokime.v1.Execution/StopExecution",
    "/dokime.v1.Execution/SubmitExecutionCommand",
    "/dokime.v1.Execution/UpdateCyclePolicy",
    "/dokime.v1.Health/ListHealth",
    "/dokime.v1.Health/RunListHealth",
    "/dokime.v1.History/BrowseCycles",
    "/dokime.v1.History/BrowseExecutions",
    "/dokime.v1.History/DeleteExecution",
    "/dokime.v1.History/DownloadCheckpointArtifact",
    "/dokime.v1.History/GetCheckpointContext",
    "/dokime.v1.History/GetCheckpointDocument",
    "/dokime.v1.History/GetCheckpointEvidence",
    "/dokime.v1.History/GetExecutionCheckpoint",
    "/dokime.v1.History/ListCheckpointRecords",
    "/dokime.v1.History/ListCheckpointTelemetry",
    "/dokime.v1.History/ReadCheckpointOutput",
    "/dokime.v1.History/RunBrowseCycles",
    "/dokime.v1.History/RunBrowseExecutions",
    "/dokime.v1.History/RunDeleteExecution",
    "/dokime.v1.History/RunDownloadCheckpointArtifact",
    "/dokime.v1.History/RunGetCheckpointContext",
    "/dokime.v1.History/RunGetCheckpointDocument",
    "/dokime.v1.History/RunGetCheckpointEvidence",
    "/dokime.v1.History/RunGetExecutionCheckpoint",
    "/dokime.v1.History/RunListCheckpointRecords",
    "/dokime.v1.History/RunListCheckpointTelemetry",
    "/dokime.v1.History/RunReadCheckpointOutput",
    "/dokime.v1.Notes/CreateNote",
    "/dokime.v1.Notes/EditNote",
    "/dokime.v1.Notes/ListNotes",
    "/dokime.v1.Notes/RunCreateNote",
    "/dokime.v1.Notes/RunEditNote",
    "/dokime.v1.Notes/RunListNotes",
    "/dokime.v1.Observation/GetRuntimeShell",
    "/dokime.v1.Observation/GetTraceability",
    "/dokime.v1.Observation/RunGetRuntimeShell",
    "/dokime.v1.Observation/RunGetTraceability",
    "/dokime.v1.Observation/RunWatchExecutionProgress",
    "/dokime.v1.Observation/RunWatchRuntime",
    "/dokime.v1.Observation/WatchExecutionProgress",
    "/dokime.v1.Observation/WatchRuntime",
    "/dokime.v1.Planning/MaterializeTemplate",
    "/dokime.v1.Planning/PreviewSelection",
    "/dokime.v1.Planning/RegisterSelection",
    "/dokime.v1.Planning/RunMaterializeTemplate",
    "/dokime.v1.Planning/RunPreviewSelection",
    "/dokime.v1.Planning/RunRegisterSelection",
    "/dokime.v1.Plots/ListAnalyses",
    "/dokime.v1.Plots/QuerySnapshots",
    "/dokime.v1.Plots/RunListAnalyses",
    "/dokime.v1.Plots/RunQuerySnapshots",
    "/dokime.v1.Plots/RunWatch",
    "/dokime.v1.Plots/Watch",
    "/dokime.v1.Plugins/Add",
    "/dokime.v1.Plugins/GetInstallation",
    "/dokime.v1.Plugins/Install",
    "/dokime.v1.Plugins/List",
    "/dokime.v1.Plugins/ListAdmittedPlugins",
    "/dokime.v1.Plugins/ListInstallations",
    "/dokime.v1.Plugins/Reload",
    "/dokime.v1.Plugins/Remove",
    "/dokime.v1.Plugins/RunAdd",
    "/dokime.v1.Plugins/RunGetInstallation",
    "/dokime.v1.Plugins/RunInstall",
    "/dokime.v1.Plugins/RunList",
    "/dokime.v1.Plugins/RunListAdmittedPlugins",
    "/dokime.v1.Plugins/RunListInstallations",
    "/dokime.v1.Plugins/RunReload",
    "/dokime.v1.Plugins/RunRemove",
    "/dokime.v1.Plugins/RunSetEnabled",
    "/dokime.v1.Plugins/SetEnabled",
    "/dokime.v1.Prompts/GetDefinition",
    "/dokime.v1.Prompts/GetPrompt",
    "/dokime.v1.Prompts/ListDefinitions",
    "/dokime.v1.Prompts/ListPending",
    "/dokime.v1.Prompts/Respond",
    "/dokime.v1.Prompts/RunGetDefinition",
    "/dokime.v1.Prompts/RunGetPrompt",
    "/dokime.v1.Prompts/RunListDefinitions",
    "/dokime.v1.Prompts/RunListPending",
    "/dokime.v1.Prompts/RunRespond",
    "/dokime.v1.Query/ListExecutions",
    "/dokime.v1.Remote/CreatePairing",
    "/dokime.v1.Remote/ReadListener",
    "/dokime.v1.Remote/RegenerateCertificate",
    "/dokime.v1.Remote/RunCreatePairing",
    "/dokime.v1.Remote/RunReadListener",
    "/dokime.v1.Remote/RunRegenerateCertificate",
    "/dokime.v1.Remote/RunUpdateListener",
    "/dokime.v1.Remote/UpdateListener",
    "/dokime.v1.Reports/DownloadCycleReport",
    "/dokime.v1.Reports/DownloadExecutionReport",
    "/dokime.v1.Reports/MaterializeReport",
    "/dokime.v1.Reports/PublishExecutionReport",
    "/dokime.v1.Reports/RunDownloadCycleReport",
    "/dokime.v1.Reports/RunDownloadExecutionReport",
    "/dokime.v1.Reports/RunMaterializeReport",
    "/dokime.v1.Reports/RunPublishExecutionReport",
    "/dokime.v1.Runtime/GetHealth",
    "/dokime.v1.Runtime/GetIdentity",
    "/dokime.v1.Runtime/RunGetHealth",
    "/dokime.v1.Runtime/RunGetIdentity",
    "/dokime.v1.Secrets/Export",
    "/dokime.v1.Secrets/Import",
    "/dokime.v1.Secrets/List",
    "/dokime.v1.Secrets/Lookup",
    "/dokime.v1.Secrets/Preview",
    "/dokime.v1.Secrets/Put",
    "/dokime.v1.Secrets/Remove",
    "/dokime.v1.Secrets/RunExport",
    "/dokime.v1.Secrets/RunImport",
    "/dokime.v1.Secrets/RunList",
    "/dokime.v1.Secrets/RunLookup",
    "/dokime.v1.Secrets/RunPreview",
    "/dokime.v1.Secrets/RunPut",
    "/dokime.v1.Secrets/RunRemove",
    "/dokime.v1.Settings/GetPluginSettings",
    "/dokime.v1.Settings/GetSettings",
    "/dokime.v1.Settings/ListStorageSources",
    "/dokime.v1.Settings/ReadApplicationSettings",
    "/dokime.v1.Settings/ReorderStorageSources",
    "/dokime.v1.Settings/RunGetPluginSettings",
    "/dokime.v1.Settings/RunGetSettings",
    "/dokime.v1.Settings/RunListStorageSources",
    "/dokime.v1.Settings/RunReadApplicationSettings",
    "/dokime.v1.Settings/RunReorderStorageSources",
    "/dokime.v1.Settings/RunSetStorageSourceEnabled",
    "/dokime.v1.Settings/RunUpdateApplicationSettings",
    "/dokime.v1.Settings/RunUpdatePluginSettings",
    "/dokime.v1.Settings/SetStorageSourceEnabled",
    "/dokime.v1.Settings/UpdateApplicationSettings",
    "/dokime.v1.Settings/UpdatePluginSettings",
    "/dokime.v1.Telemetry/ListSamples",
    "/dokime.v1.Telemetry/RunListSamples",
    "/dokime.v1.Telemetry/RunWatchSamples",
    "/dokime.v1.Telemetry/WatchSamples",
];

/* === Macros === */

/* === Type Aliases === */

pub type Output<T> = tokio_stream::wrappers::ReceiverStream<std::result::Result<T, tonic::Status>>;

pub enum RunFrame<Q> {
    Request(Q),
    Finish,
    Cancel,
}

/* === Structs === */

#[derive(Clone)]
pub struct ClientSettings {
    pub read_timeout: Duration,
    pub mutation_timeout: Duration,
    pub watch_timeout: Duration,
    pub max_cases: usize,
    pub page_items: usize,
    pub message_bytes: usize,
    pub stream_buffer: usize,
}
#[derive(Clone)]
pub struct Context {
    pub administration: Arc<tokio::sync::Mutex<()>>,
    pub host: Arc<dyn Host>,
    pub settings: ClientSettings,
}
#[derive(Clone, Debug)]
pub struct Progress {
    pub revision: u64,
    pub phase: String,
    pub finished: bool,
    pub failure: Option<String>,
}
pub struct StartedCycle {
    pub identifier: String,
    pub executions: Vec<String>,
}

/* === Enums === */

pub enum RunAction {
    Cancel,
    Pause,
    Resume,
    Stop,
}

/* === Traits === */

#[async_trait]
pub trait Host: Send + Sync {
    fn store(&self) -> Store;
    fn changes(&self) -> watch::Receiver<u64>;
    fn shutting_down(&self) -> bool;
    fn signal_change(&self);
    async fn start_existing(&self, execution: &str, mode: v1::ExecutionMode) -> Result<StartedCycle>;
    fn control_execution(&self, execution: &str, action: RunAction) -> Result<()>;
    fn submit_command(&self, request: v1::ExecutionSubmitExecutionCommandRequest) -> Result<v1::ExecutionCommandReceipt>;
    fn now_ns(&self) -> i64;
    fn identity(&self) -> v1::BackendIdentity;
    fn bootstrap_paths(&self) -> v1::BootstrapPaths;
    fn diagnostic_events(&self, scope: Option<&v1::TraceScope>) -> Result<(Vec<v1::TraceEvent>, u64, u64)>;
    fn diagnostic_changes(&self) -> tokio::sync::broadcast::Receiver<()>;
    async fn read_remote_listener(&self) -> Result<v1::RemoteListenerStatus>;
    async fn update_remote_listener(&self, settings: v1::RemoteListenerSettings) -> Result<v1::RemoteListenerStatus>;
    async fn create_pairing(&self, label: String) -> Result<v1::CreatedPairing>;
    async fn regenerate_certificate(&self, names: Vec<String>) -> Result<v1::RemoteListenerStatus>;
    fn controls(&self, execution: &str) -> Result<v1::ExecutionControls>;
    fn current_document(&self, scope: &v1::DocumentScope, identifier: &str) -> Result<(v1::Document, dokime_storage::ExecutionCut)>;
    fn render_report(&self, subject: &v1::ReportSubject, at: Option<&v1::StorageCursor>) -> Result<(std::fs::File, v1::ReportMetadata)>;
    fn retain_report(&self, file: std::fs::File, reference: v1::FileReference) -> Result<()>;
    fn temporary_report(&self, identifier: &str) -> Result<Option<(std::fs::File, v1::FileReference)>>;
    fn publish_report(&self, execution: &str, document: v1::Document) -> Result<dokime_storage::Report>;
    async fn analyze(&self, plugin: &str, operation: &str, request: v1::AnalysisRequest) -> Result<v1::AnalysisResponse>;
    async fn add_remote(&self, url: &str) -> Result<v1::Plugin>;
    fn plugins(&self) -> Result<Vec<v1::Plugin>>;
    async fn refresh_plugin_states(&self) -> Result<()>;
    async fn restore_managed_plugins(&self) -> Result<()>;
    fn installations(&self) -> Result<Vec<v1::PluginInstallation>>;
    async fn install_plugin_bundle(&self, bytes: Vec<u8>, cancelled: tokio::sync::oneshot::Receiver<()>) -> Result<v1::PluginInstallation>;
    async fn set_plugin_enabled(&self, identifier: &str, enabled: bool) -> Result<v1::PluginInstallation>;
    async fn remove_plugin_entry(&self, identifier: &str) -> Result<()>;
    async fn reload_plugin_entry(&self, identifier: &str) -> Result<()>;
    fn suites(&self) -> Result<Vec<(String, v1::Suite)>>;
    async fn materialize(&self, plugin: &str, template: &str, parameters: HashMap<String, v1::Value>) -> Result<v1::Origin>;
    async fn start(&self, origins: Vec<v1::Origin>) -> Result<StartedCycle>;
    fn progress(&self, execution: &str) -> Result<watch::Receiver<Progress>>;
    fn pending(&self) -> Result<Vec<(String, v1::OperatorQuestion)>>;
    fn answer(&self, execution: &str, answer: v1::OperatorAnswer) -> Result<()>;
    fn record_answer(&self, execution: &str, answer: v1::OperatorAnswer) -> Result<()>;
}

/* === Implementations === */

/* === Public Functions === */

pub fn declaration<Q: prost_reflect::ReflectMessage + Default, R: prost_reflect::ReflectMessage + Default>(
    id: &str, timeout: Duration,
) -> Result<h::OperationContract> {
    let types = prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?;
    let mut contract = crate::declaration::<Q, R>(id, types)?;
    let timeout = prost_types::Duration::try_from(timeout).map_err(|error| Error::Invalid(error.to_string()))?;
    contract.policy = Some(h::OperationPolicy {
        timeout: Some(timeout),
        attempt_timeout: Some(timeout),
        attempts: 1,
        ..Default::default()
    });
    Ok(contract)
}
pub async fn complete<T>(action: &str, timeout: Duration, work: impl Future<Output = Result<T>>) -> Result<T> {
    match tokio::time::timeout(timeout, work).await {
        Ok(Ok(reply)) => Ok(reply),
        Ok(Err(Error::Invalid(message))) => Err(Error::Invalid(format!("{action}: {message}"))),
        Ok(Err(error)) => Err(error),
        Err(_) => Err(Error::Deadline(action.into())),
    }
}
pub(super) fn page_size(requested: u32, configured: usize) -> Result<usize> {
    let maximum = MAX_PAGE_ITEMS.min(configured);
    if requested as usize > maximum {
        return Err(Error::Invalid(format!("List: Request at Most {maximum} Items and Follow the Returned Cursor")));
    }
    Ok(if requested == 0 { DEFAULT_PAGE_ITEMS.min(maximum) } else { requested as usize })
}
pub fn status(error: Error) -> tonic::Status {
    match error {
        Error::Deadline(action) => tonic::Status::deadline_exceeded(format!("{action}: Operation Deadline Expired")),
        Error::NotFound(message) => tonic::Status::not_found(message),
        Error::Precondition(message) => tonic::Status::failed_precondition(message),
        Error::Bound(message) => tonic::Status::resource_exhausted(message),
        Error::Conflict(message) | Error::Storage(dokime_storage::Error::Conflict(message)) => tonic::Status::aborted(message),
        Error::Unavailable(message) => tonic::Status::unavailable(message),
        Error::Cancelled(message) => tonic::Status::cancelled(message),
        Error::Stale(message) => tonic::Status::failed_precondition(message),
        Error::Storage(dokime_storage::Error::NotFound(message)) => tonic::Status::not_found(message),
        Error::Storage(dokime_storage::Error::AlreadyExists(message)) => tonic::Status::already_exists(message),
        Error::Storage(dokime_storage::Error::Permission(message)) => tonic::Status::permission_denied(message),
        Error::Storage(_) => tonic::Status::failed_precondition(error.to_string()),
        _ => tonic::Status::invalid_argument(error.to_string()),
    }
}
pub fn bounded_reply<T: prost::Message>(context: &Context, reply: T) -> Result<T> {
    if reply.encoded_len() > context.settings.message_bytes {
        return Err(Error::Bound("Receive Reply: Reduce the Selection or Page Limit to Fit the Configured Message Bound".into()));
    }
    Ok(reply)
}

pub async fn run_unary<Q, I, R, O, F, Fut>(
    context: Context, mut input: tonic::Streaming<I>, timeout: Duration, decode: fn(I) -> Option<RunFrame<Q>>, encode: fn(R) -> O,
    handle: F,
) -> std::result::Result<tonic::Response<Output<O>>, tonic::Status>
where
    Q: Send + 'static,
    I: prost::Message + Default + Send + 'static,
    R: Send + 'static,
    O: Send + 'static,
    F: FnOnce(Context, Q) -> Fut + Send + 'static,
    Fut: Future<Output = Result<R>> + Send + 'static,
{
    let (send, receive) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let work = async {
            let first = input.message().await?.and_then(decode);
            let Some(RunFrame::Request(request)) = first else {
                return Err(tonic::Status::invalid_argument("Run Operation: Send One Request First"));
            };
            if let Some(frame) = input.message().await? {
                match decode(frame) {
                    Some(RunFrame::Finish) => (),
                    Some(RunFrame::Cancel) => return Err(tonic::Status::cancelled("Run Operation: Cancelled Before Admission")),
                    _ => return Err(tonic::Status::invalid_argument("Run Operation: Send Finish or Cancel After Its Request")),
                }
            }
            handle(context, request).await.map(encode).map_err(status)
        };
        let result = match tokio::time::timeout(timeout, work).await {
            Ok(result) => result,
            Err(_) => {
                Err(tonic::Status::deadline_exceeded("Run Operation: Deadline Expired; Retry the Read or Inspect the Committed Mutation"))
            }
        };
        let _ = send.send(result).await;
    });
    Ok(tonic::Response::new(Output::new(receive)))
}

/* === Private Functions === */

/* === Tests === */
