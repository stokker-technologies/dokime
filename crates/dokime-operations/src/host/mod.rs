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

pub mod analyses;
pub mod backend;
pub mod cases;
pub mod catalog_inventory;
pub mod content;
pub mod cycles;
pub mod documents;
pub mod evidence;
pub mod execution_state;
pub mod executions;
pub mod files;
pub mod paging;
pub mod planning;
pub mod plots;
pub mod projection;
pub mod prompt_definitions;
pub mod prompts;
pub mod record_schemas;
pub mod reports;
pub mod secrets;
pub mod settings;
pub mod storage;
pub mod stream_schemas;
pub mod suites;
pub mod views;

/* === Re-Exports === */

pub use secrets::store::{register_redactions, remove_root_secrets};

/* === Imports === */

// Standard Library Imports
use std::{collections::HashMap, future::Future, sync::Arc, time::Duration};

// Third Party Imports
use async_trait::async_trait;
use harmos_protocol::model as h;
use prost::Message;
use tokio::sync::watch;

// Local Imports
use crate::{Error, Result};
use dokime_protocol::FILE_DESCRIPTOR_SET;
use dokime_protocol::host::BootstrapPaths;
use dokime_protocol::host::DocumentScope;
use dokime_protocol::host::ExecutionControls;
use dokime_protocol::host::ExecutionMode;
use dokime_protocol::host::Plugin;
use dokime_protocol::host::PluginInstallation;
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_protocol::host::ReportMetadata;
use dokime_protocol::host::ReportSubject;
use dokime_protocol::host::executions::CommandsCreateRequest;
use dokime_protocol::host::executions::ExecutionCommandReceipt;
use dokime_protocol::model;
use dokime_protocol::model::Document;
use dokime_storage::ExecutionCut;
use dokime_storage::Report;
use dokime_storage::Store;
use harmos_protocol::model::FileReference;
use harmos_protocol::model::OperationContract;
use harmos_protocol::model::ReadCut;
use harmos_protocol::model::Receipt;
use prost_reflect::ReflectMessage;
use prost_types::FileDescriptorSet;
use prost_types::Timestamp;
use std::fs::File;
use std::result::Result as StandardResult;
use tokio::sync::Mutex;
use tokio::sync::oneshot::Receiver;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

/* === Constants === */

const DEFAULT_PAGE_ITEMS: usize = 50;
const MAX_PAGE_ITEMS: usize = 500;

pub const METHODS: &[&str] = &[
    "/dokime.host.v1.Files/Get",
    "/dokime.host.v1.Files/Read",
    "/dokime.host.v1.Files/Delete",
    "/dokime.host.v1.files.Uploads/Create",
    "/dokime.host.v1.files.Uploads/Get",
    "/dokime.host.v1.files.Uploads/Resume",
    "/dokime.host.v1.files.Uploads/Append",
    "/dokime.host.v1.files.Uploads/Publish",
    "/dokime.host.v1.files.Uploads/Delete",
    "/dokime.host.v1.Files/Watch",
    "/dokime.host.v1.Aggregates/Get",
    "/dokime.host.v1.Backend/Appearance",
    "/dokime.host.v1.Cases/Create",
    "/dokime.host.v1.Cases/Delete",
    "/dokime.host.v1.Cases/Get",
    "/dokime.host.v1.Cases/List",
    "/dokime.host.v1.Cases/Parameters",
    "/dokime.host.v1.Cases/Watch",
    "/dokime.host.v1.Analyses/List",
    "/dokime.host.v1.Cycles/Create",
    "/dokime.host.v1.Cycles/Get",
    "/dokime.host.v1.Cycles/List",
    "/dokime.host.v1.Cycles/Seal",
    "/dokime.host.v1.Cycles/Set",
    "/dokime.host.v1.Cycles/Watch",
    "/dokime.host.v1.Documents/Get",
    "/dokime.host.v1.Documents/Watch",
    "/dokime.host.v1.Evidence/Archive",
    "/dokime.host.v1.Evidence/Statistics",
    "/dokime.host.v1.Executions/Cancel",
    "/dokime.host.v1.Executions/Create",
    "/dokime.host.v1.Executions/Delete",
    "/dokime.host.v1.Executions/Get",
    "/dokime.host.v1.executions.Schemas/Get",
    "/dokime.host.v1.Executions/List",
    "/dokime.host.v1.Executions/Parameters",
    "/dokime.host.v1.Executions/Set",
    "/dokime.host.v1.Executions/Start",
    "/dokime.host.v1.Executions/Steps",
    "/dokime.host.v1.Executions/Stop",
    "/dokime.host.v1.Executions/Watch",
    "/dokime.host.v1.Plugins/Add",
    "/dokime.host.v1.Plugins/GetInstallation",
    "/dokime.host.v1.Plugins/Install",
    "/dokime.host.v1.Plugins/List",
    "/dokime.host.v1.Plugins/ListAdmittedPlugins",
    "/dokime.host.v1.Plugins/ListInstallations",
    "/dokime.host.v1.Plugins/Reload",
    "/dokime.host.v1.Plugins/Remove",
    "/dokime.host.v1.Plugins/SetEnabled",
    "/dokime.host.v1.Prompts/Get",
    "/dokime.host.v1.Prompts/List",
    "/dokime.host.v1.Prompts/Watch",
    "/dokime.host.v1.Reports/Create",
    "/dokime.host.v1.Reports/Get",
    "/dokime.host.v1.Reports/Publish",
    "/dokime.host.v1.Secrets/Delete",
    "/dokime.host.v1.Secrets/Export",
    "/dokime.host.v1.Secrets/Get",
    "/dokime.host.v1.Secrets/Import",
    "/dokime.host.v1.Secrets/List",
    "/dokime.host.v1.Secrets/Set",
    "/dokime.host.v1.Secrets/Watch",
    "/dokime.host.v1.Settings/Get",
    "/dokime.host.v1.Settings/Schema",
    "/dokime.host.v1.Settings/Set",
    "/dokime.host.v1.backend.Overview/Get",
    "/dokime.host.v1.backend.Overview/Watch",
    "/dokime.host.v1.PromptDefinitions/Get",
    "/dokime.host.v1.PromptDefinitions/List",
    "/dokime.host.v1.RecordSchemas/Get",
    "/dokime.host.v1.RecordSchemas/List",
    "/dokime.host.v1.StreamSchemas/Get",
    "/dokime.host.v1.StreamSchemas/List",
    "/dokime.host.v1.Suites/Get",
    "/dokime.host.v1.Suites/List",
    "/dokime.host.v1.content.Artifacts/Create",
    "/dokime.host.v1.content.Artifacts/Get",
    "/dokime.host.v1.content.Artifacts/List",
    "/dokime.host.v1.content.Artifacts/Watch",
    "/dokime.host.v1.content.ExecutionFiles/Get",
    "/dokime.host.v1.content.ExecutionFiles/List",
    "/dokime.host.v1.content.ExecutionFiles/Watch",
    "/dokime.host.v1.evidence.HealthEvents/List",
    "/dokime.host.v1.evidence.HealthEvents/Watch",
    "/dokime.host.v1.evidence.Notes/Create",
    "/dokime.host.v1.evidence.Notes/Get",
    "/dokime.host.v1.evidence.Notes/List",
    "/dokime.host.v1.evidence.Notes/Set",
    "/dokime.host.v1.evidence.Notes/Watch",
    "/dokime.host.v1.evidence.Records/List",
    "/dokime.host.v1.evidence.Records/Watch",
    "/dokime.host.v1.evidence.Samples/List",
    "/dokime.host.v1.evidence.Samples/Watch",
    "/dokime.host.v1.evidence.Traceability/Get",
    "/dokime.host.v1.evidence.Traceability/Watch",
    "/dokime.host.v1.executions.Commands/Create",
    "/dokime.host.v1.executions.Commands/Get",
    "/dokime.host.v1.executions.Progress/Get",
    "/dokime.host.v1.executions.Progress/Watch",
    "/dokime.host.v1.media.Captures/Create",
    "/dokime.host.v1.media.Captures/Get",
    "/dokime.host.v1.media.Captures/Stop",
    "/dokime.host.v1.media.Captures/Watch",
    "/dokime.host.v1.plots.Settings/Get",
    "/dokime.host.v1.plots.Settings/Set",
    "/dokime.host.v1.plots.Snapshots/List",
    "/dokime.host.v1.plots.Snapshots/Watch",
    "/dokime.host.v1.plugins.Settings/Get",
    "/dokime.host.v1.plugins.Settings/Set",
    "/dokime.host.v1.prompts.Answers/Create",
    "/dokime.host.v1.storage.Sources/Get",
    "/dokime.host.v1.storage.Sources/Set",
];

/* === Macros === */

/* === Type Aliases === */

pub type Output<T> = ReceiverStream<StandardResult<T, Status>>;

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
    pub administration: Arc<Mutex<()>>,
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
    pub started_at: Option<Timestamp>,
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
    async fn start_existing(&self, execution: &str, mode: ExecutionMode) -> Result<StartedCycle>;
    fn control_execution(&self, execution: &str, action: RunAction) -> Result<()>;
    fn submit_command(&self, request: CommandsCreateRequest) -> Result<ExecutionCommandReceipt>;
    fn now_ns(&self) -> i64;
    fn identity(&self) -> HostIdentity;
    fn bootstrap_paths(&self) -> BootstrapPaths;
    fn register_sensitive(&self, value: &str) -> Result<()>;
    fn controls(&self, execution: &str) -> Result<ExecutionControls>;
    fn current_document(&self, scope: &DocumentScope, identifier: &str) -> Result<(Document, ExecutionCut)>;
    fn render_report(&self, subject: &ReportSubject, document_id: &str, at: Option<&ReadCut>) -> Result<(File, ReportMetadata)>;
    fn retain_report(&self, file: File, reference: FileReference) -> Result<()>;
    fn temporary_report(&self, identifier: &str) -> Result<Option<(File, FileReference)>>;
    fn publish_report(&self, prepared: Report) -> Result<Report>;
    async fn analyze(
        &self, plugin: &str, operation: &str, request: dokime_protocol::plugin::AnalysesComputeRequest,
    ) -> Result<dokime_protocol::plugin::AnalysesComputeResponse>;
    async fn add_remote(&self, url: &str) -> Result<Plugin>;
    fn plugins(&self) -> Result<Vec<Plugin>>;
    async fn refresh_plugin_states(&self) -> Result<()>;
    async fn restore_managed_plugins(&self) -> Result<()>;
    fn installations(&self) -> Result<Vec<PluginInstallation>>;
    async fn install_plugin_bundle(&self, bytes: Vec<u8>, cancelled: Receiver<()>) -> Result<PluginInstallation>;
    async fn set_plugin_enabled(&self, identifier: &str, enabled: bool) -> Result<PluginInstallation>;
    async fn remove_plugin_entry(&self, identifier: &str) -> Result<()>;
    async fn reload_plugin_entry(&self, identifier: &str) -> Result<()>;
    fn suites(&self) -> Result<Vec<(String, dokime_protocol::model::Suite)>>;
    async fn materialize(
        &self, plugin: &str, template: &str, parameters: HashMap<String, dokime_protocol::model::Value>,
    ) -> Result<dokime_protocol::model::Origin>;
    async fn start(&self, origins: Vec<dokime_protocol::model::Origin>) -> Result<StartedCycle>;
    fn progress(&self, execution: &str) -> Result<watch::Receiver<Progress>>;
    fn pending(&self) -> Result<Vec<(String, PromptsCreateRequest)>>;
    fn answer(&self, execution: &str, answer: PromptsCreateResponse) -> Result<()>;
    fn record_answer(&self, execution: &str, request_id: &str, fingerprint: &[u8], answer: PromptsCreateResponse) -> Result<Receipt>;
}

/* === Implementations === */

/* === Public Functions === */

pub fn declaration<Q: ReflectMessage + Default, R: ReflectMessage + Default>(id: &str, timeout: Duration) -> Result<OperationContract> {
    let types = FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)?;
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
pub(crate) fn page_size(requested: u32, configured: usize) -> Result<usize> {
    let maximum = MAX_PAGE_ITEMS.min(configured);
    if requested as usize > maximum {
        return Err(Error::Invalid(format!("List: Request at Most {maximum} Items and Follow the Returned Cursor")));
    }
    Ok(if requested == 0 { DEFAULT_PAGE_ITEMS.min(maximum) } else { requested as usize })
}
pub fn status(error: Error) -> Status {
    match error {
        Error::Permission(message) => Status::permission_denied(message),
        Error::Deadline(action) => Status::deadline_exceeded(format!("{action}: Operation Deadline Expired")),
        Error::NotFound(message) => Status::not_found(message),
        Error::Precondition(message) => Status::failed_precondition(message),
        Error::Bound(message) => Status::resource_exhausted(message),
        Error::Conflict(message) | Error::Storage(dokime_storage::Error::Conflict(message)) => Status::aborted(message),
        Error::Unavailable(message) => Status::unavailable(message),
        Error::Cancelled(message) => Status::cancelled(message),
        Error::Stale(message) => Status::failed_precondition(message),
        Error::Storage(dokime_storage::Error::NotFound(message)) => Status::not_found(message),
        Error::Storage(dokime_storage::Error::AlreadyExists(message)) => Status::already_exists(message),
        Error::Storage(dokime_storage::Error::Permission(message)) => Status::permission_denied(message),
        Error::Storage(_) => Status::failed_precondition(error.to_string()),
        _ => Status::invalid_argument(error.to_string()),
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
