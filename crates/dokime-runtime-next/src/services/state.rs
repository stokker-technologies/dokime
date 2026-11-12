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
use std::{collections::HashMap, sync::Mutex};

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
use dokime_storage::{ExecutionState, Store};

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
    pub progress: Mutex<HashMap<String, watch::Receiver<Progress>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[async_trait]
impl ClientHost for State {
    fn store(&self) -> Store {
        self.host.inner.store.clone()
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
        self.track(cycle)
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

    fn publish_report(&self, execution: &str, document: v1::Document) -> Result<dokime_storage::Report> {
        let snapshot = self.host.snapshot(execution).map_err(convert)?;
        let output = self.host.render_document(&snapshot, document, self.render_settings).map_err(convert)?;
        self.host.publish_report(&output).map_err(convert)
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
    async fn install_plugin_bundle(&self, bytes: Vec<u8>) -> Result<v1::PluginInstallation> {
        self.install_bundle(bytes).await
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
        let plugins = lock(&self.host.inner.plugins).map_err(convert)?;
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
        self.track(cycle)
    }
    fn progress(&self, execution: &str) -> Result<watch::Receiver<Progress>> {
        if let Some(receiver) = lock(&self.progress).map_err(convert)?.get(execution) {
            return Ok(receiver.clone());
        }
        let snapshot = self.host.inner.store.cut(execution)?;
        if snapshot.state != ExecutionState::Sealed {
            return Err(Error::Invalid("Watch Only an Admitted or Sealed Execution".into()));
        }
        let (_, receiver) = watch::channel(Progress {
            revision: 1,
            phase: "finished".into(),
            finished: true,
            failure: None,
        });
        Ok(receiver)
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

impl State {
    fn track(&self, cycle: crate::Cycle) -> Result<StartedCycle> {
        if self.host.inner.store.client_cycle(&cycle.identifier)?.is_none() {
            self.host.inner.store.save_cycle(&v1::CycleSummary {
                cycle_id: cycle.identifier.clone(),
                continue_automatically: true,
                created_at: Some(prost_types::Timestamp {
                    seconds: self.host.now_ns().div_euclid(1_000_000_000),
                    nanos: self.host.now_ns().rem_euclid(1_000_000_000) as i32,
                }),
                ..Default::default()
            })?;
        }
        let started = StartedCycle {
            identifier: cycle.identifier.clone(),
            executions: cycle.executions.clone(),
        };
        let mut senders = Vec::new();
        for execution in &cycle.executions {
            let (sender, receiver) = watch::channel(Progress {
                revision: 1,
                phase: "queued".into(),
                finished: false,
                failure: None,
            });
            lock(&self.progress).map_err(convert)?.insert(execution.clone(), receiver);
            let host = self.host.clone();
            let identifier = execution.clone();
            let prompt_sender = sender.clone();
            let mut changes = host.changes();
            let prompt_task = tokio::spawn(async move {
                loop {
                    changes.borrow_and_update();
                    let Ok(cut) = host.inner.store.cut(&identifier) else {
                        return;
                    };
                    let finished = cut.state == ExecutionState::Sealed;
                    let pending = host.pending_prompts().is_ok_and(|prompts| prompts.iter().any(|(execution, _)| execution == &identifier));
                    let phase = if finished {
                        "finished".into()
                    } else if pending {
                        "prompt".into()
                    } else {
                        host.run_phase(&identifier).unwrap_or_else(|_| "interrupted".into())
                    };
                    prompt_sender.send_modify(|progress| {
                        progress.revision += 1;
                        progress.phase = phase;
                        progress.finished = finished;
                    });
                    if finished {
                        return;
                    }
                    if changes.changed().await.is_err() {
                        return;
                    }
                }
            });
            senders.push((sender, prompt_task));
        }
        tokio::spawn(async move {
            let result = cycle.wait().await;
            for (sender, prompt_task) in senders {
                prompt_task.abort();
                let _ = prompt_task.await;
                sender.send_modify(|progress| {
                    progress.revision += 1;
                    progress.phase = "finished".into();
                    progress.finished = true;
                    progress.failure = result.as_ref().err().map(ToString::to_string);
                });
            }
        });
        self.host.signal_change();
        Ok(started)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn convert(error: crate::Error) -> Error {
    Error::Invalid(error.to_string())
}

/* === Tests === */
