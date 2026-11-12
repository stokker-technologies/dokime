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
    pub render_settings: crate::RenderSettings,
    pub identity: v1::BackendIdentity,
    pub urls: Mutex<HashMap<String, String>>,
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
    fn now_ns(&self) -> i64 {
        self.host.now_ns()
    }
    fn identity(&self) -> v1::BackendIdentity {
        self.identity.clone()
    }
    fn controls(&self, execution: &str) -> Result<v1::ExecutionControls> {
        let capabilities = self.host.capabilities(execution).map_err(convert)?;
        Ok(v1::ExecutionControls {
            can_start: capabilities.start,
            can_rerun: capabilities.rerun,
            can_cancel: capabilities.cancel,
            can_inspect: capabilities.inspect,
            can_export: capabilities.export,
            ..Default::default()
        })
    }
    fn publish_report(&self, execution: &str, document: v1::Document) -> Result<dokime_storage::Report> {
        let snapshot = self.host.snapshot(execution).map_err(convert)?;
        let output = self.host.render_document(&snapshot, document, self.render_settings).map_err(convert)?;
        self.host.publish_report(&output).map_err(convert)
    }
    async fn add_remote(&self, url: &str) -> Result<v1::Plugin> {
        let entry = self.host.add_remote(url).await.map_err(convert)?;
        lock(&self.urls).map_err(convert)?.insert(entry.clone(), url.into());
        let description = self.host.description(&entry).map_err(convert)?;
        Ok(v1::Plugin {
            entry,
            placement: Some(v1::plugin::Placement::Remote(v1::RemotePlugin { url: url.into() })),
            state: v1::PluginState::Ready as i32,
            description: Some(description),
            ..Default::default()
        })
    }
    fn plugins(&self) -> Result<Vec<v1::Plugin>> {
        let urls = lock(&self.urls).map_err(convert)?;
        let mut entries = Vec::new();
        for (entry, state) in self.host.plugin_entries().map_err(convert)? {
            let state = match state {
                crate::EntryState::Starting => v1::PluginState::Starting,
                crate::EntryState::Connecting => v1::PluginState::Connecting,
                crate::EntryState::Ready => v1::PluginState::Ready,
                crate::EntryState::Failed => v1::PluginState::Failed,
                crate::EntryState::Unavailable => v1::PluginState::Unavailable,
            };
            entries.push(v1::Plugin {
                entry: entry.clone(),
                placement: urls.get(&entry).map(|url| v1::plugin::Placement::Remote(v1::RemotePlugin { url: url.clone() })),
                state: state as i32,
                description: Some(self.host.description(&entry).map_err(convert)?),
                ..Default::default()
            });
        }
        entries.sort_by(|left, right| left.entry.cmp(&right.entry));
        Ok(entries)
    }
    fn suites(&self) -> Result<Vec<(String, v1::Suite)>> {
        let plugins = lock(&self.host.inner.plugins).map_err(convert)?;
        let mut suites = Vec::new();
        for (plugin, entry) in plugins.iter() {
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
            let prompt_task = tokio::spawn(async move {
                if host.next_prompt(&identifier).await.is_ok() {
                    prompt_sender.send_modify(|progress| {
                        progress.revision += 1;
                        progress.phase = "prompt".into();
                    });
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
        Ok(started)
    }
    fn progress(&self, execution: &str) -> Result<watch::Receiver<Progress>> {
        if let Some(receiver) = lock(&self.progress).map_err(convert)?.get(execution) {
            return Ok(receiver.clone());
        }
        let snapshot = self.host.execution(execution).map_err(convert)?;
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
}

/* === Public Functions === */

/* === Private Functions === */

fn convert(error: crate::Error) -> Error {
    Error::Invalid(error.to_string())
}

/* === Tests === */
