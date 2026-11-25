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

pub mod aggregates;
pub mod backend;
pub mod cases;
pub mod catalog_inventory;
pub mod content;
pub mod credentials;
pub mod cycles;
pub mod documents;
pub mod evidence;
pub mod execution_state;
pub mod executions;
pub mod files;
pub mod media;
pub mod paging;
pub mod planning;
pub mod plots;
pub mod plugins {
    pub mod secrets;
}
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
use std::{collections::HashMap, future::Future, pin::Pin, result, sync::Arc, time::Duration};

// Third Party Imports
use async_trait::async_trait;
use harmos_protocol::host::PluginEntry;
use harmos_protocol::model;
use harmos_protocol::model::Receipt;
use harmos_runtime::operations::{NativeOperation, OperationManager};
use prost::Message;
use prost_types::Timestamp;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Status};

// Local Imports
use crate::{Error, Result};
use dokime_protocol::host::BootstrapPaths;
use dokime_protocol::host::DocumentScope;
use dokime_protocol::host::ExecutionControls;
use dokime_protocol::host::ExecutionMode;
use dokime_protocol::host::HostBuild;
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_protocol::host::executions::CommandsCreateRequest;
use dokime_protocol::host::executions::ExecutionCommandReceipt;
use dokime_protocol::model::{Document, Origin, Suite, Value};
use dokime_storage::ExecutionCut;
use dokime_storage::Store;

/* === Constants === */

const DEFAULT_PAGE_ITEMS: usize = 50;
const MAX_PAGE_ITEMS: usize = 500;

/* === Macros === */

/* === Type Aliases === */

pub type Output<T> = ReceiverStream<result::Result<T, Status>>;

/* === Structs === */

#[derive(Clone, serde::Deserialize)]
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
    /// Admits One Frozen Report Render at a Time, So a Download Joins the Seal's.
    pub reports: Arc<Mutex<()>>,
    /// Admits One Evidence Archive Build at a Time, So an Export Joins the Seal's.
    pub archives: Arc<Mutex<()>>,
    pub host: Arc<dyn Host>,
    pub settings: ClientSettings,
    pub operations: OperationManager,
    pub caller: Option<crate::Resources>,
    pub captures: Arc<files::captures::Captures>,
}
#[derive(Clone, Debug)]
pub struct Progress {
    pub revision: u64,
    pub phase: String,
    pub sealed: bool,
    pub failure: Option<String>,
}
#[derive(Clone)]
pub struct HostIdentity {
    pub backend_id: String,
    pub epoch: String,
    pub build: HostBuild,
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
    fn spawn_work(&self, work: Pin<Box<dyn Future<Output = ()> + Send>>) -> Result<()>;
    async fn start_existing(&self, execution: &str, mode: ExecutionMode) -> Result<StartedCycle>;
    fn control_execution(&self, execution: &str, action: RunAction) -> Result<()>;
    async fn submit_command(&self, request: CommandsCreateRequest) -> Result<ExecutionCommandReceipt>;
    fn now_ns(&self) -> i64;
    fn identity(&self) -> HostIdentity;
    fn credential_namespace(&self) -> credentials::CredentialNamespace;
    fn bootstrap_paths(&self) -> BootstrapPaths;
    fn register_sensitive(&self, value: &str) -> Result<()>;
    fn controls(&self, execution: &str) -> Result<ExecutionControls>;
    fn current_document(&self, scope: &DocumentScope, identifier: &str) -> Result<(Document, ExecutionCut)>;
    fn report_template(&self) -> std::borrow::Cow<'static, [u8]>;
    /// The Appearance the Host Resolved From Its Installation When It Opened.
    fn appearance(&self) -> dokime_protocol::host::AppearanceSnapshot;
    async fn plugins(&self) -> Result<Vec<PluginEntry>>;
    fn plugin_admission_failures(&self) -> Result<Vec<(String, String)>>;
    async fn refresh_plugin_states(&self) -> Result<()>;
    fn suites(&self) -> Result<Vec<(String, Suite)>>;
    async fn materialize(&self, plugin: &str, template: &str, parameters: HashMap<String, Value>) -> Result<Origin>;
    async fn start(&self, origins: Vec<Origin>) -> Result<StartedCycle>;
    fn progress(&self, execution: &str) -> Result<watch::Receiver<Progress>>;
    fn pending(&self) -> Result<Vec<(String, PromptsCreateRequest)>>;
    fn answer(&self, execution: &str, answer: PromptsCreateResponse) -> Result<()>;
    fn record_answer(&self, execution: &str, request_id: &str, fingerprint: &[u8], answer: PromptsCreateResponse) -> Result<Receipt>;
}

/* === Implementations === */

impl Context {
    pub fn resolve_execution<'a>(&'a self, selected: Option<&'a str>, action: &str) -> Result<&'a str> {
        let trusted = self.caller.as_ref().map(|caller| caller.execution.as_str());
        crate::resources::resolve_execution(trusted, selected, action)
    }

    pub async fn storage<O: NativeOperation<Endpoints = ()>>(&self, request: O::Request) -> Result<O::Response> {
        if let Some(caller) = &self.caller {
            let operations =
                self.operations.with_scope(caller.execution.clone()).map_err(|source| Error::Permission(format!("{}: {source}", O::ID)))?;
            return operations.call_native::<O>(request).await.map_err(|source| Error::Native { action: O::ID, source });
        }
        self.operations.call_native::<O>(request).await.map_err(|source| Error::Native { action: O::ID, source })
    }
}

/* === Public Functions === */

pub async fn complete<T>(action: &str, timeout: Duration, work: impl Future<Output = Result<T>>) -> Result<T> {
    match tokio::time::timeout(timeout, work).await {
        Ok(Ok(reply)) => Ok(reply),
        Ok(Err(Error::Invalid(message))) => Err(Error::Invalid(format!("{action}: {message}"))),
        Ok(Err(error)) => Err(error),
        Err(_) => Err(Error::Deadline(action.into())),
    }
}
/// Admit Background Work the Host Settles on Shutdown, Dropping It Once Shutdown Begins So the Host Stops Promptly.
pub(crate) fn stoppable_work<T: Send + 'static>(
    context: &Context, action: &'static str, work: impl Future<Output = Result<T>> + Send + 'static,
) -> Result<oneshot::Receiver<Result<T>>> {
    let (sender, receiver) = oneshot::channel();
    let host = context.host.clone();
    let mut changes = host.changes();
    context.host.spawn_work(Box::pin(async move {
        tokio::pin!(work);
        let result = loop {
            if host.shutting_down() {
                break Err(Error::Cancelled(format!("{action}: The Host Stopped; the Next Request Creates It")));
            }
            tokio::select! {
                result = &mut work => break result,
                changed = changes.changed() => if changed.is_err() {
                    break Err(Error::Cancelled(format!("{action}: The Host Stopped; the Next Request Creates It")));
                },
            }
        };
        let _ = sender.send(result);
    }))?;
    Ok(receiver)
}
pub(crate) fn page_size(requested: u32, configured: usize) -> Result<usize> {
    let maximum = MAX_PAGE_ITEMS.min(configured);
    if requested as usize > maximum {
        return Err(Error::Invalid(format!("List: Request at Most {maximum} Items and Follow the Returned Cursor")));
    }
    Ok(if requested == 0 { DEFAULT_PAGE_ITEMS.min(maximum) } else { requested as usize })
}
pub fn status(error: Error) -> Status {
    if let Error::Caller(failure) = &error {
        let wire = model::Failure::from(failure);
        let mut status = Status::with_details(Code::PermissionDenied, failure.to_string(), wire.encode_to_vec().into());
        status.set_source(Arc::new(error));
        return status;
    }
    let code = match &error {
        Error::Unimplemented(_) => Code::Unimplemented,
        Error::Permission(_) | Error::Storage(dokime_storage::Error::Permission(_)) => Code::PermissionDenied,
        Error::Deadline(_) => Code::DeadlineExceeded,
        Error::NotFound(_) | Error::Storage(dokime_storage::Error::NotFound(_)) => Code::NotFound,
        Error::Precondition(_) | Error::Stale(_) => Code::FailedPrecondition,
        Error::Bound(_) => Code::ResourceExhausted,
        Error::Conflict(_) | Error::Storage(dokime_storage::Error::Conflict(_)) => Code::Aborted,
        Error::Unavailable(_) => Code::Unavailable,
        Error::Native { .. } => Code::Internal,
        Error::Cancelled(_) => Code::Cancelled,
        Error::Storage(dokime_storage::Error::AlreadyExists(_)) => Code::AlreadyExists,
        Error::Storage(_) => Code::FailedPrecondition,
        _ => Code::InvalidArgument,
    };
    let mut status = Status::new(code, error.to_string());
    status.set_source(Arc::new(error));
    status
}
pub fn bounded_reply<T: prost::Message>(context: &Context, reply: T) -> Result<T> {
    if reply.encoded_len() > context.settings.message_bytes {
        return Err(Error::Bound("Receive Reply: Reduce the Selection or Page Limit to Fit the Configured Message Bound".into()));
    }
    Ok(reply)
}

/* === Private Functions === */

/* === Tests === */
