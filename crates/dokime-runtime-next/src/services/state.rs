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
    pub temporary_reports: Mutex<std::collections::BTreeMap<String, (std::fs::File, v1::FileReference)>>,
    pub temporary_report_limit: usize,
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
    fn current_document(&self, scope: &v1::DocumentScope, identifier: &str) -> Result<(v1::Document, dokime_storage::ExecutionCut)> {
        self.host.current_document(scope, identifier).map_err(convert)
    }
    fn render_report(&self, subject: &v1::ReportSubject, at: Option<&v1::StorageCursor>) -> Result<(std::fs::File, v1::ReportMetadata)> {
        use std::io::{Seek, Write};
        let (store, _read_cancellation) = self.host.inner.store.read_session()?;
        let (execution, cycle) = match subject.kind.as_ref() {
            Some(v1::report_subject::Kind::Execution(id)) => (Some(id.as_str()), store.metadata(id)?.execution.cycle),
            Some(v1::report_subject::Kind::Cycle(id)) => (None, id.clone()),
            None => return Err(Error::Invalid("Render Report: Select an Execution or Cycle".into())),
        };
        let mut snapshots = Vec::new();
        let mut after = String::new();
        loop {
            let page = store.cycle_execution_page(&cycle, &after, self.render_settings.page_size)?;
            if page.is_empty() {
                break;
            }
            for id in page {
                after = id.clone();
                if execution.is_some_and(|selected| selected != id) {
                    continue;
                }
                let cut = store.metadata(&id)?;
                if cut.execution.cycle != cycle {
                    continue;
                }
                let scope = v1::DocumentScope {
                    cycle_id: cycle.clone(),
                    execution_id: id,
                };
                let (document, mut cut) = self.host.current_document_in(&store, &scope, "").map_err(convert)?;
                if let Some(at) = at {
                    let history = v1::StoreHistory {
                        store: self.identity.backend_id.clone(),
                        epoch: scope.execution_id.clone(),
                    };
                    if execution.is_none() || at.history.as_ref() != Some(&history) || at.sequence > cut.frontier {
                        return Err(Error::Invalid("Render Report: Refresh the Execution's Fixed Evidence Cursor".into()));
                    }
                    cut.frontier = at.sequence;
                }
                snapshots.push((document, self.host.snapshot_in(store.clone(), cut).map_err(convert)?));
            }
        }
        if snapshots.is_empty() {
            return Err(Error::Invalid("Render Report: Select an Existing Execution or Cycle".into()));
        }
        let file = if snapshots.len() == 1 {
            let (document, snapshot) = snapshots.remove(0);
            self.host.render_document(&snapshot, document, self.render_settings).map_err(convert)?.into_file()
        } else {
            let mut output = tempfile::tempfile().map_err(|error| Error::Invalid(error.to_string()))?;
            output
                .write_all(b"<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><title>Cycle Report</title><body>")
                .map_err(|error| Error::Invalid(error.to_string()))?;
            for (document, snapshot) in snapshots {
                let rendered = self.host.render_document(&snapshot, document, self.render_settings).map_err(convert)?;
                let mut file = rendered.into_file();
                // Each Execution Is an Independent Embedded HTML Document.
                let mut bytes = String::new();
                use std::io::Read;
                file.read_to_string(&mut bytes).map_err(|error| Error::Invalid(error.to_string()))?;
                let body = bytes.split_once("<body>").map_or(bytes.as_str(), |(_, body)| body).trim_end_matches("</body></html>");
                output.write_all(b"<article>").map_err(|error| Error::Invalid(error.to_string()))?;
                output.write_all(body.as_bytes()).map_err(|error| Error::Invalid(error.to_string()))?;
                output.write_all(b"</article>").map_err(|error| Error::Invalid(error.to_string()))?;
            }
            output.write_all(b"</body></html>").map_err(|error| Error::Invalid(error.to_string()))?;
            output.rewind().map_err(|error| Error::Invalid(error.to_string()))?;
            output
        };
        let metadata = v1::ReportMetadata {
            test_cycle_id: cycle,
            test_execution_id: execution.map(str::to_owned),
            identifier: if execution.is_some() { "execution".into() } else { "cycle".into() },
            origin: v1::ReportOrigin::Current as i32,
            ..Default::default()
        };
        Ok((file, metadata))
    }
    fn retain_report(&self, file: std::fs::File, reference: v1::FileReference) -> Result<()> {
        let identifier =
            reference.metadata.as_ref().ok_or_else(|| Error::Invalid("Render Report: Supply Its Temporary Identity".into()))?.id.clone();
        let mut reports = lock(&self.temporary_reports).map_err(convert)?;
        if reports.len() >= self.temporary_report_limit && !reports.contains_key(&identifier) {
            let first = reports.keys().next().cloned();
            if let Some(first) = first {
                reports.remove(&first);
            }
        }
        reports.insert(identifier, (file, reference));
        Ok(())
    }
    fn temporary_report(&self, identifier: &str) -> Result<Option<(std::fs::File, v1::FileReference)>> {
        let reports = lock(&self.temporary_reports).map_err(convert)?;
        reports
            .get(identifier)
            .map(|(file, reference)| Ok((file.try_clone().map_err(|error| Error::Invalid(error.to_string()))?, reference.clone())))
            .transpose()
    }
    fn publish_report(&self, execution: &str, document: v1::Document) -> Result<dokime_storage::Report> {
        let (store, _read_cancellation) = self.host.inner.store.read_session()?;
        let cut = store.cut(execution)?;
        if cut.state != ExecutionState::Sealed {
            return Err(Error::Invalid("Publish Report: Seal the Execution Before Selecting Its Fixed Snapshot".into()));
        }
        let snapshot = self.host.snapshot_in(store, cut).map_err(convert)?;
        let output = self.host.render_document(&snapshot, document, self.render_settings).map_err(convert)?;
        self.host.publish_report(&output).map_err(convert)
    }
    async fn analyze(&self, plugin: &str, operation: &str, request: v1::AnalysisRequest) -> Result<v1::AnalysisResponse> {
        self.host.analyze(plugin, operation, request).await.map_err(convert)
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
