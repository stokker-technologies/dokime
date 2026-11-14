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

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

// Third Party Imports
use async_trait::async_trait;
use tokio::sync::watch;

// Local Imports
use crate::Host;
use dokime_operations::{
    Error, Result,
    host::{Host as ClientHost, Progress, RunAction, StartedCycle},
};
use dokime_storage::Store;

use dokime_operations::host::HostIdentity;
use dokime_protocol::host::BootstrapPaths;
use dokime_protocol::host::DocumentScope;
use dokime_protocol::host::ExecutionControls;
use dokime_protocol::host::ExecutionMode;
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_protocol::host::ReportMetadata;
use dokime_protocol::host::ReportSubject;
use dokime_protocol::host::executions::CommandsCreateRequest;
use dokime_protocol::host::executions::ExecutionCommandReceipt;
use dokime_protocol::model::Document;
use dokime_protocol::model::Origin;
use dokime_protocol::model::Suite;
use dokime_protocol::model::Suites;
use dokime_protocol::model::Value;
use dokime_protocol::plugin::AnalysesComputeRequest;
use dokime_protocol::plugin::AnalysesComputeResponse;
use harmos_protocol::host::PluginEntry;
use harmos_protocol::model::FileReference;
use harmos_protocol::model::ReadCut;
use harmos_protocol::model::Receipt;
use harmos_runtime::operations::OperationManager;
use harmos_runtime::services::status::Status;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct State {
    pub host: Host,
    pub operations: OperationManager,
    pub catalogs: Arc<RwLock<Vec<PluginEntry>>>,
    pub status: Arc<Status>,
    pub root: PathBuf,
    pub render_settings: crate::RenderSettings,
    pub identity: HostIdentity,
    pub temporary_report_limit: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[async_trait]
impl ClientHost for State {
    fn store(&self) -> Store {
        self.host.store.clone()
    }
    fn changes(&self) -> watch::Receiver<u64> {
        self.host.changes()
    }
    fn shutting_down(&self) -> bool {
        self.host.is_shutting_down()
    }
    fn signal_change(&self) {
        self.host.signal_change();
    }
    async fn start_existing(&self, execution: &str, mode: ExecutionMode) -> Result<StartedCycle> {
        let cycle = self.host.start_execution_mode(execution, mode).await.map_err(convert)?;
        Ok(StartedCycle {
            identifier: cycle.identifier,
            executions: cycle.executions,
        })
    }
    fn control_execution(&self, execution: &str, action: RunAction) -> Result<()> {
        match action {
            RunAction::Cancel => self.host.cancel(execution),
            RunAction::Pause => self.host.pause_execution(execution, true),
            RunAction::Resume => self.host.pause_execution(execution, false),
            RunAction::Stop => self.host.stop_execution(execution),
        }
        .map_err(|error| Error::Precondition(error.to_string()))?;
        self.host.signal_change();
        Ok(())
    }
    fn submit_command(&self, request: CommandsCreateRequest) -> Result<ExecutionCommandReceipt> {
        self.host.submit_execution_command(request).map_err(|error| Error::Precondition(error.to_string()))
    }
    fn now_ns(&self) -> i64 {
        self.host.now_ns()
    }
    fn register_sensitive(&self, value: &str) -> Result<()> {
        self.status.redaction.register(value.to_owned()).map_err(|error| Error::Unavailable(format!("Register Secret Redaction: {error}")))
    }
    fn bootstrap_paths(&self) -> BootstrapPaths {
        let root = self.root.to_string_lossy().into_owned();
        BootstrapPaths {
            application: root.clone(),
            storage: root,
            plugins: self.root.join("plugins").to_string_lossy().into_owned(),
            logs: self.root.join("logs").to_string_lossy().into_owned(),
        }
    }
    fn identity(&self) -> HostIdentity {
        self.identity.clone()
    }
    fn controls(&self, execution: &str) -> Result<ExecutionControls> {
        self.host.execution_controls(execution).map_err(convert)
    }
    fn current_document(&self, scope: &DocumentScope, identifier: &str) -> Result<(Document, dokime_storage::ExecutionCut)> {
        self.host.current_document(scope, identifier).map_err(convert)
    }
    fn render_report(&self, subject: &ReportSubject, document_id: &str, at: Option<&ReadCut>) -> Result<(File, ReportMetadata)> {
        self.host.render_report(subject, document_id, at, &self.identity, self.render_settings)
    }
    fn retain_report(&self, file: File, reference: FileReference) -> Result<()> {
        self.host.reports.retain(file, reference, self.temporary_report_limit).map_err(convert)
    }
    fn temporary_report(&self, identifier: &str) -> Result<Option<(File, FileReference)>> {
        self.host.reports.temporary(identifier).map_err(convert)
    }
    fn publish_report(&self, prepared: dokime_storage::Report) -> Result<dokime_storage::Report> {
        self.host.reports.publish_prepared(prepared).map_err(convert)
    }
    async fn analyze(&self, plugin: &str, operation: &str, request: AnalysesComputeRequest) -> Result<AnalysesComputeResponse> {
        self.host.analyze(plugin, operation, request).await.map_err(convert)
    }
    async fn plugins(&self) -> Result<Vec<PluginEntry>> {
        use harmos_protocol::host::PluginsListRequest;
        use harmos_runtime::builtins::plugins::List;
        let reply = self
            .operations
            .call_protobuf::<List>(PluginsListRequest { at: None })
            .await
            .map_err(|source| Error::Unavailable(format!("List Plugins: {source}")))?;
        *self.catalogs.write().map_err(|source| Error::Unavailable(format!("List Plugins: {source}")))? = reply.entries.clone();
        Ok(reply.entries)
    }
    async fn refresh_plugin_states(&self) -> Result<()> {
        self.plugins().await?;
        Ok(())
    }
    fn suites(&self) -> Result<Vec<(String, Suite)>> {
        use harmos_protocol::host::PluginState;
        use prost::Message;
        let plugins = self.catalogs.read().map_err(|source| Error::Unavailable(format!("Read Suites: {source}")))?;
        let mut suites = Vec::new();
        for plugin in plugins.iter().filter(|entry| entry.state == PluginState::Ready as i32) {
            let Some(catalog) = plugin.description.as_ref().and_then(|description| description.declarations.as_ref()) else {
                continue;
            };
            for section in catalog.sections.iter().filter(|section| section.r#type == "dokime.model.v1.Suites") {
                let declarations = Suites::decode(section.payload.as_slice())?;
                suites.extend(declarations.suites.into_iter().map(|suite| (plugin.name.clone(), suite)));
            }
        }
        suites.sort_by(|left, right| {
            left.1
                .metadata
                .as_ref()
                .map(|metadata| &metadata.identifier)
                .cmp(&right.1.metadata.as_ref().map(|metadata| &metadata.identifier))
        });
        Ok(suites)
    }
    async fn materialize(&self, plugin: &str, template: &str, parameters: HashMap<String, Value>) -> Result<Origin> {
        self.host.materialize(plugin, template, parameters).await.map_err(convert)
    }
    async fn start(&self, origins: Vec<Origin>) -> Result<StartedCycle> {
        let cycle = self.host.start_cycle(origins).await.map_err(convert)?;
        Ok(StartedCycle {
            identifier: cycle.identifier,
            executions: cycle.executions,
        })
    }
    fn progress(&self, execution: &str) -> Result<watch::Receiver<Progress>> {
        self.host.executions.progress(execution).map_err(convert)
    }
    fn pending(&self) -> Result<Vec<(String, PromptsCreateRequest)>> {
        self.host.pending_prompts().map_err(convert)
    }
    fn answer(&self, execution: &str, answer: PromptsCreateResponse) -> Result<()> {
        self.host.answer(execution, answer).map_err(convert)
    }
    fn record_answer(&self, execution: &str, request_id: &str, fingerprint: &[u8], answer: PromptsCreateResponse) -> Result<Receipt> {
        self.host.record_answer(execution, request_id, fingerprint, answer).map_err(convert)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn convert(error: crate::Error) -> Error {
    Error::Invalid(error.to_string())
}

/* === Tests === */
