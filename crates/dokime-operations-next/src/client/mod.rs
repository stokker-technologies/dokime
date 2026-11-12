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

pub mod backend_get_availability;
pub mod backend_get_health;
pub mod backend_get_identity;
pub mod catalog_get_suite;
pub mod catalog_list_suites;
pub mod evidence_get_captured_record_schema;
pub mod evidence_list_records;
mod evidence_selection;
pub mod evidence_watch_records;
pub mod execution_create_cycle;
pub mod execution_get_execution;
pub mod health_list_health;
pub mod observation_watch_execution_progress;
pub mod planning_materialize_template;
pub mod plugins_add;
pub mod plugins_list;
mod projection;
pub mod prompts_get_prompt;
pub mod prompts_list_pending;
pub mod prompts_respond;
pub mod query_list_executions;
pub mod reports_download_execution_report;
pub mod reports_publish_execution_report;
pub mod run_call;
pub mod telemetry_list_samples;

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

pub const METHODS: &[&str] = &[
    "/dokime.v1.Plugins/List",
    "/dokime.v1.Plugins/Add",
    "/dokime.v1.Catalog/GetSuite",
    "/dokime.v1.Catalog/ListSuites",
    "/dokime.v1.Planning/MaterializeTemplate",
    "/dokime.v1.Execution/CreateCycle",
    "/dokime.v1.Execution/GetExecution",
    "/dokime.v1.Observation/WatchExecutionProgress",
    "/dokime.v1.Prompts/GetPrompt",
    "/dokime.v1.Prompts/ListPending",
    "/dokime.v1.Prompts/Respond",
    "/dokime.v1.Query/ListExecutions",
    "/dokime.v1.Evidence/GetCapturedRecordSchema",
    "/dokime.v1.Evidence/ListRecords",
    "/dokime.v1.Evidence/WatchRecords",
    "/dokime.v1.Evidence/RunGetCapturedRecordSchema",
    "/dokime.v1.Evidence/RunListRecords",
    "/dokime.v1.Evidence/RunWatchRecords",
    "/dokime.v1.Telemetry/ListSamples",
    "/dokime.v1.Reports/DownloadExecutionReport",
    "/dokime.v1.Reports/PublishExecutionReport",
    "/dokime.v1.Health/ListHealth",
    "/dokime.v1.Backend/GetAvailability",
    "/dokime.v1.Backend/GetHealth",
    "/dokime.v1.Backend/GetIdentity",
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

/* === Traits === */

#[async_trait]
pub trait Host: Send + Sync {
    fn store(&self) -> Store;
    fn now_ns(&self) -> i64;
    fn identity(&self) -> v1::BackendIdentity;
    fn controls(&self, execution: &str) -> Result<v1::ExecutionControls>;
    fn publish_report(&self, execution: &str, document: v1::Document) -> Result<dokime_storage::Report>;
    async fn add_remote(&self, url: &str) -> Result<v1::Plugin>;
    fn plugins(&self) -> Result<Vec<v1::Plugin>>;
    fn suites(&self) -> Result<Vec<(String, v1::Suite)>>;
    async fn materialize(&self, plugin: &str, template: &str, parameters: HashMap<String, v1::Value>) -> Result<v1::Origin>;
    async fn start(&self, origins: Vec<v1::Origin>) -> Result<StartedCycle>;
    fn progress(&self, execution: &str) -> Result<watch::Receiver<Progress>>;
    fn pending(&self) -> Result<Vec<(String, v1::OperatorQuestion)>>;
    fn answer(&self, execution: &str, answer: v1::OperatorAnswer) -> Result<()>;
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
        Ok(Err(error)) => Err(Error::Invalid(format!("{action}: {error}"))),
        Err(_) => Err(Error::Deadline(action.into())),
    }
}
pub fn status(error: Error) -> tonic::Status {
    match error {
        Error::Deadline(action) => tonic::Status::deadline_exceeded(format!("{action}: Operation Deadline Expired")),
        Error::Storage(_) => tonic::Status::failed_precondition(error.to_string()),
        _ => tonic::Status::invalid_argument(error.to_string()),
    }
}

/* === Private Functions === */

/* === Tests === */
