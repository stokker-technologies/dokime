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
    pub remote: std::sync::Arc<super::remote_access::Control>,
    pub diagnostics: super::diagnostics::Diagnostics,
    pub root: std::path::PathBuf,
    pub render_settings: crate::RenderSettings,
    pub identity: v1::BackendIdentity,
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
    async fn start_existing(&self, execution: &str, mode: v1::ExecutionMode) -> Result<StartedCycle> {
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
    fn submit_command(&self, request: v1::ExecutionSubmitExecutionCommandRequest) -> Result<v1::ExecutionCommandReceipt> {
        self.host.submit_execution_command(request).map_err(|error| Error::Precondition(error.to_string()))
    }
    fn now_ns(&self) -> i64 {
        self.host.now_ns()
    }
    async fn read_remote_listener(&self) -> Result<v1::RemoteListenerStatus> {
        self.remote.read().await.map_err(convert)
    }
    async fn update_remote_listener(&self, settings: v1::RemoteListenerSettings) -> Result<v1::RemoteListenerStatus> {
        self.remote.update(settings).await.map_err(convert)
    }
    async fn create_pairing(&self, label: String) -> Result<v1::CreatedPairing> {
        self.remote.pair(label).await.map_err(convert)
    }
    async fn regenerate_certificate(&self, names: Vec<String>) -> Result<v1::RemoteListenerStatus> {
        self.remote.rotate(names).await.map_err(convert)
    }
    fn diagnostic_events(&self, scope: Option<&v1::TraceScope>) -> Result<(Vec<v1::TraceEvent>, u64, u64)> {
        Ok(self.diagnostics.snapshot(scope))
    }
    fn diagnostic_changes(&self) -> tokio::sync::broadcast::Receiver<()> {
        self.diagnostics.changes()
    }
    fn bootstrap_paths(&self) -> v1::BootstrapPaths {
        let root = self.root.to_string_lossy().into_owned();
        v1::BootstrapPaths {
            application: root.clone(),
            storage: root,
            plugins: self.root.join("plugins").to_string_lossy().into_owned(),
            logs: self.root.join("logs").to_string_lossy().into_owned(),
        }
    }
    fn identity(&self) -> v1::BackendIdentity {
        self.identity.clone()
    }
    fn controls(&self, execution: &str) -> Result<v1::ExecutionControls> {
        self.host.execution_controls(execution).map_err(convert)
    }
    fn current_document(&self, scope: &v1::DocumentScope, identifier: &str) -> Result<(v1::Document, dokime_storage::ExecutionCut)> {
        self.host.current_document(scope, identifier).map_err(convert)
    }
    fn render_report(&self, subject: &v1::ReportSubject, at: Option<&v1::StorageCursor>) -> Result<(std::fs::File, v1::ReportMetadata)> {
        self.host.render_report(subject, at, &self.identity, self.render_settings)
    }
    fn retain_report(&self, file: std::fs::File, reference: v1::FileReference) -> Result<()> {
        self.host.reports.retain(file, reference, self.temporary_report_limit).map_err(convert)
    }
    fn temporary_report(&self, identifier: &str) -> Result<Option<(std::fs::File, v1::FileReference)>> {
        self.host.reports.temporary(identifier).map_err(convert)
    }
    fn publish_report(&self, execution: &str, document: v1::Document) -> Result<dokime_storage::Report> {
        let snapshot = self.host.snapshot(execution).map_err(convert)?;
        if snapshot.seal().is_empty() {
            return Err(Error::Invalid("Publish Report: Seal the Execution Before Selecting Its Fixed Snapshot".into()));
        }
        let output = self.host.render_document(&snapshot, document, self.render_settings).map_err(convert)?;
        self.host.reports.publish(&output).map_err(convert)
    }
    async fn analyze(&self, plugin: &str, operation: &str, request: v1::AnalysisRequest) -> Result<v1::AnalysisResponse> {
        self.host.analyze(plugin, operation, request).await.map_err(convert)
    }
    async fn add_remote(&self, url: &str) -> Result<v1::Plugin> {
        self.add_entry(url).await
    }
    async fn refresh_plugin_states(&self) -> Result<()> {
        self.refresh_entry_states().await
    }
    fn plugins(&self) -> Result<Vec<v1::Plugin>> {
        self.list_entries()
    }
    async fn restore_managed_plugins(&self) -> Result<()> {
        self.restore_managed().await
    }
    fn installations(&self) -> Result<Vec<v1::PluginInstallation>> {
        self.list_installations()
    }
    async fn install_plugin_bundle(&self, bytes: Vec<u8>, cancelled: tokio::sync::oneshot::Receiver<()>) -> Result<v1::PluginInstallation> {
        self.install_bundle(bytes, cancelled).await
    }
    async fn set_plugin_enabled(&self, identifier: &str, enabled: bool) -> Result<v1::PluginInstallation> {
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
            .filter(|plugin| plugin.state == v1::PluginState::Ready as i32)
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
    fn pending(&self) -> Result<Vec<(String, v1::OperatorQuestion)>> {
        self.host.pending_prompts().map_err(convert)
    }
    fn answer(&self, execution: &str, answer: v1::OperatorAnswer) -> Result<()> {
        self.host.answer(execution, answer).map_err(convert)
    }
    fn record_answer(&self, execution: &str, answer: v1::OperatorAnswer) -> Result<()> {
        self.host.record_answer(execution, answer).map_err(convert)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn convert(error: crate::Error) -> Error {
    Error::Invalid(error.to_string())
}

/* === Tests === */
