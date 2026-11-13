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

pub mod analyses_list;
pub mod backend_appearance;
pub mod backend_overview_get;
pub mod backend_overview_watch;
pub mod cases_create;
pub mod cases_delete;
pub mod cases_get;
pub mod cases_list;
pub mod cases_parameters;
pub mod cases_watch;
mod catalog_inventory;
pub mod catalog_prompt_definitions_get;
pub mod catalog_prompt_definitions_list;
pub mod catalog_record_schemas_get;
pub mod catalog_record_schemas_list;
pub mod catalog_stream_schemas_get;
pub mod catalog_stream_schemas_list;
pub mod catalog_suites_get;
pub mod catalog_suites_list;
pub mod content_artifacts_create;
pub mod content_artifacts_get;
pub mod content_artifacts_list;
pub mod content_artifacts_watch;
pub mod content_execution_files_get;
pub mod content_execution_files_list;
pub mod content_execution_files_watch;
mod content_transfer;
pub mod cycles_create;
pub mod cycles_get;
pub mod cycles_list;
pub mod cycles_seal;
pub mod cycles_set;
pub mod cycles_watch;
pub mod document_expansion;
pub mod document_plot;
pub mod documents_get;
pub mod documents_watch;
pub mod evidence_archive;
pub mod evidence_health_events_list;
pub mod evidence_health_events_watch;
pub mod evidence_notes_create;
pub mod evidence_notes_get;
pub mod evidence_notes_list;
pub mod evidence_notes_set;
pub mod evidence_notes_watch;
pub mod evidence_records_list;
pub mod evidence_records_watch;
pub mod evidence_samples_list;
pub mod evidence_samples_watch;
mod evidence_selection;
pub mod evidence_traceability_get;
pub mod evidence_traceability_watch;
mod execution_state;
pub mod executions_cancel;
pub mod executions_commands_create;
pub mod executions_commands_get;
pub mod executions_create;
pub mod executions_delete;
pub mod executions_get;
pub mod executions_list;
pub mod executions_parameters;
pub mod executions_progress_get;
pub mod executions_progress_watch;
pub mod executions_set;
pub mod executions_start;
pub mod executions_steps;
pub mod executions_stop;
pub mod executions_watch;
mod paging;
mod planning;
mod plot_analysis;
pub mod plots_settings_get;
pub mod plots_settings_set;
pub mod plots_snapshots_list;
pub mod plots_snapshots_watch;
pub mod plugins_add;
pub mod plugins_get_installation;
pub mod plugins_install;
pub mod plugins_list;
pub mod plugins_list_admitted_plugins;
pub mod plugins_list_installations;
pub mod plugins_reload;
pub mod plugins_remove;
pub mod plugins_set_enabled;
pub mod plugins_settings_get;
pub mod plugins_settings_set;
mod projection;
mod prompt_catalog;
mod prompt_validation;
pub mod prompts_answers_create;
pub mod prompts_get;
pub mod prompts_list;
pub mod prompts_watch;
pub mod reports_create;
pub mod reports_get;
pub mod reports_publish;
pub mod secrets_delete;
pub mod secrets_export;
pub mod secrets_get;
pub mod secrets_import;
pub mod secrets_list;
pub mod secrets_set;
pub(crate) mod secrets_store;
pub mod secrets_watch;
pub mod settings_get;
pub(crate) mod settings_preferences;
pub mod settings_schema;
pub mod settings_set;
pub mod storage_sources_get;
pub mod storage_sources_set;
mod views;

/* === Re-Exports === */

pub use secrets_store::{register_redactions, remove_root_secrets};

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
    "/dokime.client.v1.Aggregates/Get",
    "/dokime.client.v1.Backend/Appearance",
    "/dokime.client.v1.Cases/Create",
    "/dokime.client.v1.Cases/Delete",
    "/dokime.client.v1.Cases/Get",
    "/dokime.client.v1.Cases/List",
    "/dokime.client.v1.Cases/Parameters",
    "/dokime.client.v1.Cases/Watch",
    "/dokime.client.v1.Analyses/List",
    "/dokime.client.v1.Content/DownloadAppearanceAsset",
    "/dokime.client.v1.Cycles/Create",
    "/dokime.client.v1.Cycles/Get",
    "/dokime.client.v1.Cycles/List",
    "/dokime.client.v1.Cycles/Seal",
    "/dokime.client.v1.Cycles/Set",
    "/dokime.client.v1.Cycles/Watch",
    "/dokime.client.v1.Documents/Get",
    "/dokime.client.v1.Documents/Watch",
    "/dokime.client.v1.Evidence/Archive",
    "/dokime.client.v1.Evidence/Statistics",
    "/dokime.client.v1.Executions/Cancel",
    "/dokime.client.v1.Executions/Create",
    "/dokime.client.v1.Executions/Delete",
    "/dokime.client.v1.Executions/Get",
    "/dokime.client.v1.Executions/List",
    "/dokime.client.v1.Executions/Parameters",
    "/dokime.client.v1.Executions/Set",
    "/dokime.client.v1.Executions/Start",
    "/dokime.client.v1.Executions/Steps",
    "/dokime.client.v1.Executions/Stop",
    "/dokime.client.v1.Executions/Watch",
    "/dokime.client.v1.Plugins/Add",
    "/dokime.client.v1.Plugins/GetInstallation",
    "/dokime.client.v1.Plugins/Install",
    "/dokime.client.v1.Plugins/List",
    "/dokime.client.v1.Plugins/ListAdmittedPlugins",
    "/dokime.client.v1.Plugins/ListInstallations",
    "/dokime.client.v1.Plugins/Reload",
    "/dokime.client.v1.Plugins/Remove",
    "/dokime.client.v1.Plugins/SetEnabled",
    "/dokime.client.v1.Prompts/Get",
    "/dokime.client.v1.Prompts/List",
    "/dokime.client.v1.Prompts/Watch",
    "/dokime.client.v1.Reports/Create",
    "/dokime.client.v1.Reports/Get",
    "/dokime.client.v1.Reports/Publish",
    "/dokime.client.v1.Secrets/Delete",
    "/dokime.client.v1.Secrets/Export",
    "/dokime.client.v1.Secrets/Get",
    "/dokime.client.v1.Secrets/Import",
    "/dokime.client.v1.Secrets/List",
    "/dokime.client.v1.Secrets/Set",
    "/dokime.client.v1.Secrets/Watch",
    "/dokime.client.v1.Settings/Get",
    "/dokime.client.v1.Settings/Schema",
    "/dokime.client.v1.Settings/Set",
    "/dokime.client.v1.backend.Overview/Get",
    "/dokime.client.v1.backend.Overview/Watch",
    "/dokime.client.v1.PromptDefinitions/Get",
    "/dokime.client.v1.PromptDefinitions/List",
    "/dokime.client.v1.RecordSchemas/Get",
    "/dokime.client.v1.RecordSchemas/List",
    "/dokime.client.v1.StreamSchemas/Get",
    "/dokime.client.v1.StreamSchemas/List",
    "/dokime.client.v1.Suites/Get",
    "/dokime.client.v1.Suites/List",
    "/dokime.client.v1.content.Artifacts/Create",
    "/dokime.client.v1.content.Artifacts/Get",
    "/dokime.client.v1.content.Artifacts/List",
    "/dokime.client.v1.content.Artifacts/Watch",
    "/dokime.client.v1.content.ExecutionFiles/Get",
    "/dokime.client.v1.content.ExecutionFiles/List",
    "/dokime.client.v1.content.ExecutionFiles/Watch",
    "/dokime.client.v1.evidence.HealthEvents/List",
    "/dokime.client.v1.evidence.HealthEvents/Watch",
    "/dokime.client.v1.evidence.Notes/Create",
    "/dokime.client.v1.evidence.Notes/Get",
    "/dokime.client.v1.evidence.Notes/List",
    "/dokime.client.v1.evidence.Notes/Set",
    "/dokime.client.v1.evidence.Notes/Watch",
    "/dokime.client.v1.evidence.Records/List",
    "/dokime.client.v1.evidence.Records/Watch",
    "/dokime.client.v1.evidence.Samples/List",
    "/dokime.client.v1.evidence.Samples/Watch",
    "/dokime.client.v1.evidence.Traceability/Get",
    "/dokime.client.v1.evidence.Traceability/Watch",
    "/dokime.client.v1.executions.Commands/Create",
    "/dokime.client.v1.executions.Commands/Get",
    "/dokime.client.v1.executions.Progress/Get",
    "/dokime.client.v1.executions.Progress/Watch",
    "/dokime.client.v1.media.Captures/Create",
    "/dokime.client.v1.media.Captures/Get",
    "/dokime.client.v1.media.Captures/Stop",
    "/dokime.client.v1.media.Captures/Watch",
    "/dokime.client.v1.plots.Settings/Get",
    "/dokime.client.v1.plots.Settings/Set",
    "/dokime.client.v1.plots.Snapshots/List",
    "/dokime.client.v1.plots.Snapshots/Watch",
    "/dokime.client.v1.plugins.Settings/Get",
    "/dokime.client.v1.plugins.Settings/Set",
    "/dokime.client.v1.prompts.Answers/Create",
    "/dokime.client.v1.storage.Sources/Get",
    "/dokime.client.v1.storage.Sources/Set",
];

/* === Macros === */

/* === Type Aliases === */

pub type Output<T> = tokio_stream::wrappers::ReceiverStream<std::result::Result<T, tonic::Status>>;

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
#[derive(Clone)]
pub struct HostIdentity {
    pub backend_id: String,
    pub epoch: String,
    pub product_version: String,
    pub target: String,
    pub started_at: Option<prost_types::Timestamp>,
    pub display_name: String,
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
    async fn start_existing(&self, execution: &str, mode: dokime_protocol::client::ExecutionMode) -> Result<StartedCycle>;
    fn control_execution(&self, execution: &str, action: RunAction) -> Result<()>;
    fn submit_command(
        &self, request: dokime_protocol::client::executions::CommandsCreateRequest,
    ) -> Result<dokime_protocol::client::executions::ExecutionCommandReceipt>;
    fn now_ns(&self) -> i64;
    fn identity(&self) -> HostIdentity;
    fn bootstrap_paths(&self) -> dokime_protocol::client::BootstrapPaths;
    fn register_sensitive(&self, value: &str) -> Result<()>;
    fn controls(&self, execution: &str) -> Result<dokime_protocol::client::ExecutionControls>;
    fn current_document(&self, scope: &v1::DocumentScope, identifier: &str) -> Result<(v1::Document, dokime_storage::ExecutionCut)>;
    fn render_report(
        &self, subject: &dokime_protocol::client::ReportSubject, document_id: &str, at: Option<&harmos_protocol::v1::ReadCut>,
    ) -> Result<(std::fs::File, dokime_protocol::client::ReportMetadata)>;
    fn retain_report(&self, file: std::fs::File, reference: harmos_protocol::storage::FileReference) -> Result<()>;
    fn temporary_report(&self, identifier: &str) -> Result<Option<(std::fs::File, harmos_protocol::storage::FileReference)>>;
    fn publish_report(&self, prepared: dokime_storage::Report) -> Result<dokime_storage::Report>;
    async fn analyze(&self, plugin: &str, operation: &str, request: v1::AnalysisRequest) -> Result<v1::AnalysisResponse>;
    async fn add_remote(&self, url: &str) -> Result<dokime_protocol::client::Plugin>;
    fn plugins(&self) -> Result<Vec<dokime_protocol::client::Plugin>>;
    async fn refresh_plugin_states(&self) -> Result<()>;
    async fn restore_managed_plugins(&self) -> Result<()>;
    fn installations(&self) -> Result<Vec<dokime_protocol::client::PluginInstallation>>;
    async fn install_plugin_bundle(
        &self, bytes: Vec<u8>, cancelled: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<dokime_protocol::client::PluginInstallation>;
    async fn set_plugin_enabled(&self, identifier: &str, enabled: bool) -> Result<dokime_protocol::client::PluginInstallation>;
    async fn remove_plugin_entry(&self, identifier: &str) -> Result<()>;
    async fn reload_plugin_entry(&self, identifier: &str) -> Result<()>;
    fn suites(&self) -> Result<Vec<(String, v1::Suite)>>;
    async fn materialize(&self, plugin: &str, template: &str, parameters: HashMap<String, v1::Value>) -> Result<v1::Origin>;
    async fn start(&self, origins: Vec<v1::Origin>) -> Result<StartedCycle>;
    fn progress(&self, execution: &str) -> Result<watch::Receiver<Progress>>;
    fn pending(&self) -> Result<Vec<(String, dokime_protocol::context::PromptsCreateRequest)>>;
    fn answer(&self, execution: &str, answer: dokime_protocol::context::PromptsCreateResponse) -> Result<()>;
    fn record_answer(
        &self, execution: &str, request_id: &str, fingerprint: &[u8], answer: dokime_protocol::context::PromptsCreateResponse,
    ) -> Result<harmos_protocol::v1::Receipt>;
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

/* === Private Functions === */

/* === Tests === */
