/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Orchestration

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

// Third Party Imports
use tokio::{sync::watch, task::JoinHandle};
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::{Cycle, Error, HostSettings, Result, host::lock, plugin::Entry, prompts::PromptManager};
use dokime_operations_next::client::Progress;
use dokime_protocol::v1;
use dokime_storage::{ExecutionState, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct ExecutionManager {
    pub(crate) inner: Arc<Runtime>,
}
pub(crate) struct Runtime {
    pub store: Store,
    pub settings: HostSettings,
    pub plugins: Arc<Mutex<HashMap<String, Arc<Entry>>>>,
    pub prompts: Arc<PromptManager>,
    pub active: Mutex<HashMap<String, Active>>,
    pub changed: watch::Sender<u64>,
    admission: tokio::sync::Mutex<()>,
    shutting_down: AtomicBool,
    tasks: Mutex<Vec<JoinHandle<()>>>,
    progress: Mutex<HashMap<String, watch::Sender<Progress>>>,
    _root_lease: crate::host::RootLease,
}
pub(crate) struct Active {
    pub plugin: String,
    pub cancellation: CancellationToken,
    pub control: Arc<crate::run_control::RunControl>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionManager {
    pub(crate) fn new(
        store: Store, settings: HostSettings, plugins: Arc<Mutex<HashMap<String, Arc<Entry>>>>, prompts: Arc<PromptManager>,
        changed: watch::Sender<u64>, root_lease: crate::host::RootLease,
    ) -> Result<Self> {
        // Recover Only Executions Driven by a Plugin Run
        for identifier in store.identifiers()? {
            let snapshot = store.cut(&identifier)?;
            let is_run =
                snapshot.execution.origin.as_ref().is_some_and(|origin| matches!(origin.kind, Some(v1::origin::Kind::PluginRun(_))));
            if snapshot.state == ExecutionState::Open && is_run {
                store.close_streams(&identifier)?;
                store.close(
                    &identifier,
                    v1::Termination::Interrupted,
                    v1::ExecutionOutcome {
                        attribution: Some(v1::execution_outcome::Attribution::NotEvaluated(v1::UnevaluatedOutcome {
                            termination: v1::Termination::Interrupted as i32,
                            reason: "Host Restarted During Execution".into(),
                        })),
                    },
                )?;
                store.seal(&identifier)?;
            } else if snapshot.state == ExecutionState::Closed && is_run {
                store.seal(&identifier)?;
            }
        }
        let now = settings.clock.now_ns();
        store.interrupt_commands(prost_types::Timestamp {
            seconds: now.div_euclid(1_000_000_000),
            nanos: now.rem_euclid(1_000_000_000) as i32,
        })?;

        Ok(Self {
            inner: Arc::new(Runtime {
                _root_lease: root_lease,
                store,
                settings,
                plugins,
                prompts,
                changed,
                active: Mutex::new(HashMap::new()),
                admission: tokio::sync::Mutex::new(()),
                shutting_down: AtomicBool::new(false),
                tasks: Mutex::new(Vec::new()),
                progress: Mutex::new(HashMap::new()),
            }),
        })
    }
    pub(crate) async fn start_cycle(&self, origins: Vec<v1::Origin>) -> Result<Cycle> {
        let _admission = self.inner.admission.lock().await;
        self.admit("Start Cycle")?;
        self.reap_finished().await?;
        crate::execution::start(self.clone(), origins).await
    }
    pub(crate) async fn start_execution_mode(&self, execution: &str, mode: dokime_protocol::client::ExecutionMode) -> Result<Cycle> {
        let _admission = self.inner.admission.lock().await;
        self.admit("Start Execution")?;
        self.reap_finished().await?;
        crate::execution::start_existing_mode(self.clone(), execution, mode).await
    }
    fn admit(&self, action: &str) -> Result<()> {
        if self.is_shutting_down() {
            return Err(Error::Invalid(format!("{action}: Reconnect to a Running Host")));
        }
        Ok(())
    }
    pub(crate) fn entry(&self, identifier: &str) -> Result<Arc<Entry>> {
        lock(&self.inner.plugins)?
            .get(identifier)
            .cloned()
            .ok_or_else(|| Error::Invalid(format!("Start Execution: Connect Plugin {identifier}")))
    }
    pub(crate) async fn materialize(&self, plugin: &str, template: &str, parameters: HashMap<String, v1::Value>) -> Result<v1::Origin> {
        crate::plugin::materialize(self.entry(plugin)?, template, parameters, &self.inner.settings).await
    }
    pub(crate) fn now_ns(&self) -> i64 {
        self.inner.settings.clock.now_ns()
    }
    pub(crate) fn signal_change(&self) {
        self.inner.changed.send_modify(|revision| *revision += 1);
    }
    pub(crate) fn is_shutting_down(&self) -> bool {
        self.inner.shutting_down.load(Ordering::Acquire)
    }
    pub(crate) fn is_active(&self, execution: &str) -> Result<bool> {
        Ok(lock(&self.inner.active)?.contains_key(execution))
    }
    pub(crate) fn uses_plugin(&self, plugin: &str) -> Result<bool> {
        Ok(lock(&self.inner.active)?.values().any(|active| active.plugin == plugin))
    }
    pub(crate) fn cancel(&self, execution: &str) -> Result<()> {
        let active = lock(&self.inner.active)?;
        let active = active
            .get(execution)
            .ok_or_else(|| Error::Invalid("Cancel Execution: Select a Running Execution; Queued Executions Cannot Be Cancelled".into()))?;
        active.control.cancel(&active.cancellation)?;
        self.signal_change();
        Ok(())
    }
    async fn reap_finished(&self) -> Result<()> {
        let tasks = std::mem::take(&mut *lock(&self.inner.tasks)?);
        let mut failure = None;
        for task in tasks {
            if !task.is_finished() {
                lock(&self.inner.tasks)?.push(task);
                continue;
            }
            if let Err(error) = task.await {
                failure.get_or_insert(Error::Join(error));
            }
        }
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(())
    }
    pub(crate) fn spawn(&self, task: impl std::future::Future<Output = ()> + Send + 'static) -> Result<()> {
        let mut tasks = lock(&self.inner.tasks)?;
        tasks.push(tokio::spawn(task));
        Ok(())
    }
    pub(crate) fn register_cycle(&self, identifier: &str) -> Result<()> {
        if self.inner.store.client_cycle(identifier)?.is_none() {
            let now = self.now_ns();
            self.inner.store.save_cycle(&v1::CycleSummary {
                cycle_id: identifier.into(),
                continue_automatically: true,
                created_at: Some(prost_types::Timestamp {
                    seconds: now.div_euclid(1_000_000_000),
                    nanos: now.rem_euclid(1_000_000_000) as i32,
                }),
                ..Default::default()
            })?;
        }
        Ok(())
    }
    pub(crate) fn track(&self, executions: &[String]) -> Result<()> {
        let mut senders = Vec::new();
        for execution in executions {
            let (sender, _) = watch::channel(Progress {
                revision: 1,
                phase: "queued".into(),
                finished: false,
                failure: None,
            });
            lock(&self.inner.progress)?.insert(execution.clone(), sender.clone());
            senders.push((execution.clone(), sender));
        }
        let manager = self.clone();
        let mut changes = self.inner.changed.subscribe();
        self.spawn(async move {
            loop {
                changes.borrow_and_update();
                senders.retain(|(execution, sender)| {
                    if sender.borrow().finished {
                        return false;
                    }
                    let state = manager.inner.store.cut(execution);
                    let finished = state.as_ref().is_ok_and(|cut| cut.state == ExecutionState::Sealed);
                    let pending =
                        manager.inner.prompts.pending_prompts().is_ok_and(|prompts| prompts.iter().any(|(id, _)| id == execution));
                    let phase = if finished {
                        "finished".into()
                    } else if pending {
                        "prompt".into()
                    } else {
                        manager.run_phase(execution).unwrap_or_else(|_| "interrupted".into())
                    };
                    sender.send_modify(|progress| {
                        progress.revision += 1;
                        progress.phase = phase;
                        progress.finished = finished || state.is_err();
                        if let Err(error) = state {
                            progress.failure = Some(error.to_string());
                        }
                    });
                    !sender.borrow().finished
                });
                if senders.is_empty() {
                    return;
                }
                if changes.changed().await.is_err() {
                    return;
                }
            }
        })?;
        self.signal_change();
        Ok(())
    }
    pub(crate) fn finish_progress(&self, execution: &str, failure: Option<String>) -> Result<()> {
        if let Some(sender) = lock(&self.inner.progress)?.remove(execution) {
            sender.send_modify(|progress| {
                progress.revision += 1;
                progress.phase = "finished".into();
                progress.finished = true;
                progress.failure = failure;
            });
        }
        self.signal_change();
        Ok(())
    }
    pub(crate) fn progress(&self, execution: &str) -> Result<watch::Receiver<Progress>> {
        if let Some(sender) = lock(&self.inner.progress)?.get(execution) {
            return Ok(sender.subscribe());
        }
        if self.inner.store.cut(execution)?.state != ExecutionState::Sealed {
            return Err(Error::Invalid("Watch Execution Progress: Select an Admitted or Sealed Execution".into()));
        }
        Ok(watch::channel(Progress {
            revision: 1,
            phase: "finished".into(),
            finished: true,
            failure: None,
        })
        .1)
    }
    pub(crate) async fn shutdown(&self) -> Result<()> {
        let _admission = self.inner.admission.lock().await;
        self.inner.shutting_down.store(true, Ordering::Release);
        self.signal_change();
        for active in lock(&self.inner.active)?.values() {
            active.cancellation.cancel();
        }
        // Cleanup Must Not Wait for a New Operator Question During Shutdown
        self.inner.prompts.close_admission()?;
        let tasks = std::mem::take(&mut *lock(&self.inner.tasks)?);
        let mut failure = None;
        for task in tasks {
            if let Err(error) = task.await {
                failure.get_or_insert(Error::Join(error));
            }
        }
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
