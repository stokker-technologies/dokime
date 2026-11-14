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
use dokime_protocol::FILE_DESCRIPTOR_SET;
use dokime_protocol::client::BootstrapPaths;
use dokime_protocol::client::ExecutionControls;
use dokime_protocol::client::ExecutionMode;
use dokime_protocol::client::Plugin;
use dokime_protocol::client::PluginInstallation;
use dokime_protocol::client::ReportMetadata;
use dokime_protocol::client::ReportSubject;
use dokime_protocol::client::executions::CommandsCreateRequest;
use dokime_protocol::client::executions::ExecutionCommandReceipt;
use dokime_protocol::context::PromptsCreateRequest;
use dokime_protocol::context::PromptsCreateResponse;
use dokime_protocol::v1;
use dokime_protocol::v1::{Document, DocumentScope};
use dokime_storage::ExecutionCut;
use dokime_storage::Report;
use dokime_storage::Store;
use harmos_protocol::storage::FileReference;
use harmos_protocol::v1::OperationContract;
use harmos_protocol::v1::ReadCut;
use harmos_protocol::v1::Receipt;
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
    "/dokime.client.v1.Files/Get",
    "/dokime.client.v1.Files/Read",
    "/dokime.client.v1.Files/Delete",
    "/dokime.client.v1.files.Uploads/Create",
    "/dokime.client.v1.files.Uploads/Get",
    "/dokime.client.v1.files.Uploads/Resume",
    "/dokime.client.v1.files.Uploads/Append",
    "/dokime.client.v1.files.Uploads/Publish",
    "/dokime.client.v1.files.Uploads/Delete",
    "/dokime.client.v1.Files/Watch",
    "/dokime.client.v1.Aggregates/Get",
    "/dokime.client.v1.Backend/Appearance",
    "/dokime.client.v1.Cases/Create",
    "/dokime.client.v1.Cases/Delete",
    "/dokime.client.v1.Cases/Get",
    "/dokime.client.v1.Cases/List",
    "/dokime.client.v1.Cases/Parameters",
    "/dokime.client.v1.Cases/Watch",
    "/dokime.client.v1.Analyses/List",
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
    "/dokime.client.v1.executions.Schemas/Get",
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
    async fn analyze(&self, plugin: &str, operation: &str, request: v1::AnalysisRequest) -> Result<v1::AnalysisResponse>;
    async fn add_remote(&self, url: &str) -> Result<Plugin>;
    fn plugins(&self) -> Result<Vec<Plugin>>;
    async fn refresh_plugin_states(&self) -> Result<()>;
    async fn restore_managed_plugins(&self) -> Result<()>;
    fn installations(&self) -> Result<Vec<PluginInstallation>>;
    async fn install_plugin_bundle(&self, bytes: Vec<u8>, cancelled: Receiver<()>) -> Result<PluginInstallation>;
    async fn set_plugin_enabled(&self, identifier: &str, enabled: bool) -> Result<PluginInstallation>;
    async fn remove_plugin_entry(&self, identifier: &str) -> Result<()>;
    async fn reload_plugin_entry(&self, identifier: &str) -> Result<()>;
    fn suites(&self) -> Result<Vec<(String, v1::Suite)>>;
    async fn materialize(&self, plugin: &str, template: &str, parameters: HashMap<String, v1::Value>) -> Result<v1::Origin>;
    async fn start(&self, origins: Vec<v1::Origin>) -> Result<StartedCycle>;
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
pub(super) fn page_size(requested: u32, configured: usize) -> Result<usize> {
    let maximum = MAX_PAGE_ITEMS.min(configured);
    if requested as usize > maximum {
        return Err(Error::Invalid(format!("List: Request at Most {maximum} Items and Follow the Returned Cursor")));
    }
    Ok(if requested == 0 { DEFAULT_PAGE_ITEMS.min(maximum) } else { requested as usize })
}
pub fn status(error: Error) -> Status {
    match error {
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
