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

// Third Party Imports
use async_trait::async_trait;
use tokio::sync::watch;

// Local Imports
use crate::{Host, host::lock};
use dokime_operations_next::{
    Error, Result,
    client::{Host as ClientHost, Progress, StartedCycle},
};
use dokime_protocol::v1;
use dokime_storage::Store;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct State {
    pub host: Host,
    pub administration: std::sync::Arc<tokio::sync::Mutex<()>>,
    pub status: std::sync::Arc<harmos_runtime::services::status::Status>,
    pub root: std::path::PathBuf,
    pub render_settings: crate::RenderSettings,
    pub identity: dokime_operations_next::client::HostIdentity,
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
    async fn start_existing(&self, execution: &str, mode: dokime_protocol::client::ExecutionMode) -> Result<StartedCycle> {
        let cycle = self.host.start_execution_mode(execution, mode).await.map_err(convert)?;
        Ok(StartedCycle {
            identifier: cycle.identifier,
            executions: cycle.executions,
        })
    }
    fn control_execution(&self, execution: &str, action: dokime_operations_next::client::RunAction) -> Result<()> {
        use dokime_operations_next::client::RunAction;
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
    fn submit_command(
        &self, request: dokime_protocol::client::executions::CommandsCreateRequest,
    ) -> Result<dokime_protocol::client::executions::ExecutionCommandReceipt> {
        self.host.submit_execution_command(request).map_err(|error| Error::Precondition(error.to_string()))
    }
    fn now_ns(&self) -> i64 {
        self.host.now_ns()
    }
    fn register_sensitive(&self, value: &str) -> Result<()> {
        self.status.redaction.register(value.to_owned()).map_err(|error| Error::Unavailable(format!("Register Secret Redaction: {error}")))
    }
    fn bootstrap_paths(&self) -> dokime_protocol::client::BootstrapPaths {
        let root = self.root.to_string_lossy().into_owned();
        dokime_protocol::client::BootstrapPaths {
            application: root.clone(),
            storage: root,
            plugins: self.root.join("plugins").to_string_lossy().into_owned(),
            logs: self.root.join("logs").to_string_lossy().into_owned(),
        }
    }
    fn identity(&self) -> dokime_operations_next::client::HostIdentity {
        self.identity.clone()
    }
    fn controls(&self, execution: &str) -> Result<dokime_protocol::client::ExecutionControls> {
        self.host.execution_controls(execution).map_err(convert)
    }
    fn current_document(&self, scope: &v1::DocumentScope, identifier: &str) -> Result<(v1::Document, dokime_storage::ExecutionCut)> {
        self.host.current_document(scope, identifier).map_err(convert)
    }
    fn render_report(
        &self, subject: &dokime_protocol::client::ReportSubject, document_id: &str, at: Option<&harmos_protocol::v1::ReadCut>,
    ) -> Result<(std::fs::File, dokime_protocol::client::ReportMetadata)> {
        self.host.render_report(subject, document_id, at, &self.identity, self.render_settings)
    }
    fn retain_report(&self, file: std::fs::File, reference: harmos_protocol::storage::FileReference) -> Result<()> {
        self.host.reports.retain(file, reference, self.temporary_report_limit).map_err(convert)
    }
    fn temporary_report(&self, identifier: &str) -> Result<Option<(std::fs::File, harmos_protocol::storage::FileReference)>> {
        self.host.reports.temporary(identifier).map_err(convert)
    }
    fn publish_report(&self, prepared: dokime_storage::Report) -> Result<dokime_storage::Report> {
        self.host.reports.publish_prepared(prepared).map_err(convert)
    }
    async fn analyze(&self, plugin: &str, operation: &str, request: v1::AnalysisRequest) -> Result<v1::AnalysisResponse> {
        self.host.analyze(plugin, operation, request).await.map_err(convert)
    }
    async fn add_remote(&self, url: &str) -> Result<dokime_protocol::client::Plugin> {
        self.add_entry(url).await
    }
    async fn refresh_plugin_states(&self) -> Result<()> {
        self.refresh_entry_states().await
    }
    fn plugins(&self) -> Result<Vec<dokime_protocol::client::Plugin>> {
        self.list_entries()
    }
    async fn restore_managed_plugins(&self) -> Result<()> {
        self.restore_managed().await
    }
    fn installations(&self) -> Result<Vec<dokime_protocol::client::PluginInstallation>> {
        self.list_installations()
    }
    async fn install_plugin_bundle(
        &self, bytes: Vec<u8>, cancelled: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<dokime_protocol::client::PluginInstallation> {
        self.install_bundle(bytes, cancelled).await
    }
    async fn set_plugin_enabled(&self, identifier: &str, enabled: bool) -> Result<dokime_protocol::client::PluginInstallation> {
        self.enable_managed(identifier, enabled).await
    }
    async fn remove_plugin_entry(&self, identifier: &str) -> Result<()> {
        self.remove_entry(identifier).await
    }
    async fn reload_plugin_entry(&self, identifier: &str) -> Result<()> {
        self.reload_entry(identifier).await
    }
    fn suites(&self) -> Result<Vec<(String, v1::Suite)>> {
        let ready: std::collections::HashSet<_> = self
            .list_entries()?
            .into_iter()
            .filter(|plugin| plugin.state == dokime_protocol::client::PluginState::Ready as i32)
            .filter_map(|plugin| plugin.description.map(|description| description.id))
            .collect();
        let plugins = lock(&self.host.plugins).map_err(convert)?;
        let mut suites = Vec::new();
        for (plugin, entry) in plugins.iter() {
            if !ready.contains(plugin) {
                continue;
            }
            suites.extend(entry.suites.suites.iter().cloned().map(|suite| (plugin.clone(), suite)));
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
    async fn materialize(&self, plugin: &str, template: &str, parameters: HashMap<String, v1::Value>) -> Result<v1::Origin> {
        self.host.materialize(plugin, template, parameters).await.map_err(convert)
    }
    async fn start(&self, origins: Vec<v1::Origin>) -> Result<StartedCycle> {
        let cycle = self.host.start_cycle(origins).await.map_err(convert)?;
        Ok(StartedCycle {
            identifier: cycle.identifier,
            executions: cycle.executions,
        })
    }
    fn progress(&self, execution: &str) -> Result<watch::Receiver<Progress>> {
        self.host.executions.progress(execution).map_err(convert)
    }
    fn pending(&self) -> Result<Vec<(String, dokime_protocol::context::PromptsCreateRequest)>> {
        self.host.pending_prompts().map_err(convert)
    }
    fn answer(&self, execution: &str, answer: dokime_protocol::context::PromptsCreateResponse) -> Result<()> {
        self.host.answer(execution, answer).map_err(convert)
    }
    fn record_answer(
        &self, execution: &str, request_id: &str, fingerprint: &[u8], answer: dokime_protocol::context::PromptsCreateResponse,
    ) -> Result<harmos_protocol::v1::Receipt> {
        self.host.record_answer(execution, request_id, fingerprint, answer).map_err(convert)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn convert(error: crate::Error) -> Error {
    Error::Invalid(error.to_string())
}

/* === Tests === */
