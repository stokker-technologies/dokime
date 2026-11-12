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
pub mod conversation;
pub mod evidence_list_records;
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
pub mod observation_watch_execution_progress;
mod paging;
mod planning;
pub mod planning_materialize_template;
pub mod planning_preview_selection;
pub mod planning_register_selection;
pub mod plugins_add;
pub mod plugins_list;
mod projection;
mod prompt_catalog;
mod prompt_validation;
pub mod prompts_get_definition;
pub mod prompts_get_prompt;
pub mod prompts_list_definitions;
pub mod prompts_list_pending;
pub mod prompts_respond;
pub mod query_list_executions;
pub mod reports_download_execution_report;
pub mod reports_publish_execution_report;
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
    "/dokime.v1.Backend/GetAvailability",
    "/dokime.v1.Backend/GetHealth",
    "/dokime.v1.Backend/GetIdentity",
    "/dokime.v1.Catalog/GetSuite",
    "/dokime.v1.Catalog/ListSuites",
    "/dokime.v1.Evidence/ListRecords",
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
    "/dokime.v1.Observation/WatchExecutionProgress",
    "/dokime.v1.Planning/MaterializeTemplate",
    "/dokime.v1.Planning/PreviewSelection",
    "/dokime.v1.Planning/RegisterSelection",
    "/dokime.v1.Planning/RunMaterializeTemplate",
    "/dokime.v1.Planning/RunPreviewSelection",
    "/dokime.v1.Planning/RunRegisterSelection",
    "/dokime.v1.Plugins/Add",
    "/dokime.v1.Plugins/List",
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
    "/dokime.v1.Reports/DownloadExecutionReport",
    "/dokime.v1.Reports/PublishExecutionReport",
    "/dokime.v1.Telemetry/ListSamples",
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

pub enum RunAction { Cancel, Pause, Resume, Stop }

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
        Ok(Err(error)) => Err(error),
        Err(_) => Err(Error::Deadline(action.into())),
    }
}
pub fn status(error: Error) -> tonic::Status {
    match error {
        Error::Deadline(action) => tonic::Status::deadline_exceeded(format!("{action}: Operation Deadline Expired")),
        Error::NotFound(message) => tonic::Status::not_found(message),
        Error::Conflict(message) => tonic::Status::failed_precondition(message),
        Error::Storage(_) => tonic::Status::failed_precondition(error.to_string()),
        _ => tonic::Status::invalid_argument(error.to_string()),
    }
}

/* === Private Functions === */

/* === Tests === */
