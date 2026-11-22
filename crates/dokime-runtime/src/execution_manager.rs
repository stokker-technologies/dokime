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
    future::Future,
    mem,
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, Ordering},
    },
};

// Third Party Imports
use harmos_protocol::host::PluginEntry;
use harmos_runtime::plugin::manager;
use prost_types::Timestamp;
use tokio::{
    runtime::Handle,
    sync::{
        self, oneshot,
        watch::{self, Receiver, Sender},
    },
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::run_control::RunControl;
use crate::services::composition::Application;
use crate::streams::SampleScope;
use crate::{
    Cycle, Error, HostSettings, Result,
    host::{RootLease, lock},
    prompts::PromptManager,
};
use crate::{clock, execution};
use dokime_operations::host::{Context, Progress};
use dokime_protocol::host::CyclePhase;
use dokime_protocol::host::execution_outcome::Attribution;
use dokime_protocol::host::executions::{CommandsCreateRequest, ExecutionCommandReceipt};
use dokime_protocol::host::{CycleSummary, ExecutionMode, ExecutionOutcome, UnevaluatedOutcome};
use dokime_protocol::model::origin;
use dokime_protocol::model::{Origin, Termination, Value};
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
    pub prompts: Arc<PromptManager>,
    pub active: Mutex<HashMap<String, Active>>,
    pub changed: Sender<u64>,
    pub context: OnceLock<Weak<Context>>,
    pub sample_scopes: Arc<Mutex<HashMap<String, SampleScope>>>,
    pub sample_failures: Arc<Mutex<HashMap<String, String>>>,
    pub runtime: OnceLock<Weak<harmos_runtime::Runtime<Application>>>,
    pub library_plugins: OnceLock<manager::Manager>,
    admission: sync::Mutex<()>,
    shutting_down: AtomicBool,
    tasks: Mutex<Vec<JoinHandle<()>>>,
    progress: Mutex<HashMap<String, Sender<Progress>>>,
    _root_lease: RootLease,
}
pub(crate) struct Active {
    pub plugin: String,
    pub cancellation: CancellationToken,
    pub control: Arc<RunControl>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionManager {
    pub(crate) fn new(
        store: Store, settings: HostSettings, prompts: Arc<PromptManager>, changed: Sender<u64>, root_lease: RootLease,
    ) -> Result<Self> {
        // Recover Only Executions Driven by a Plugin Run
        for identifier in store.identifiers()? {
            let snapshot = store.cut(&identifier)?;
            let is_run = snapshot.execution.origin.as_ref().is_some_and(|origin| matches!(origin.kind, Some(origin::Kind::PluginRun(_))));
            let started = snapshot.frontier != 0;
            if snapshot.state == ExecutionState::Open && is_run && started {
                store.close_streams(&identifier)?;
                store.close(
                    &identifier,
                    Termination::Interrupted,
                    ExecutionOutcome {
                        attribution: Some(Attribution::NotEvaluated(UnevaluatedOutcome {
                            termination: Termination::Interrupted as i32,
                            reason: "Host Restarted During Execution".into(),
                        })),
                    },
                )?;
                store.seal(&identifier, clock::date(settings.clock.now_ns()).into())?;
            } else if snapshot.state == ExecutionState::Closed && is_run {
                store.seal(&identifier, clock::date(settings.clock.now_ns()).into())?;
            }
        }

        for mut cycle in store.client_cycles()? {
            if !cycle.sealed && cycle.phase == CyclePhase::Sealing as i32 {
                cycle.phase = CyclePhase::SealFailed as i32;
                cycle.seal_failure = Some("Seal Cycle: Host Restarted Before Evidence Retention Completed; Retry Seal".into());
                store.save_cycle(&cycle)?;
            }
        }

        Ok(Self {
            inner: Arc::new(Runtime {
                _root_lease: root_lease,
                store,
                settings,
                prompts,
                changed,
                context: OnceLock::new(),
                sample_scopes: Arc::new(Mutex::new(HashMap::new())),
                sample_failures: Arc::new(Mutex::new(HashMap::new())),
                runtime: OnceLock::new(),
                library_plugins: OnceLock::new(),
                active: Mutex::new(HashMap::new()),
                admission: sync::Mutex::new(()),
                shutting_down: AtomicBool::new(false),
                tasks: Mutex::new(Vec::new()),
                progress: Mutex::new(HashMap::new()),
            }),
        })
    }
    pub(crate) fn bind_context(&self, context: &Arc<Context>) -> Result<()> {
        self.inner.context.set(Arc::downgrade(context)).map_err(|_| Error::Invalid("Bind Host Context: Assemble the Host Only Once".into()))
    }
    pub(crate) async fn submit_execution_command(&self, request: CommandsCreateRequest) -> Result<ExecutionCommandReceipt> {
        let (sender, receiver) = oneshot::channel();
        {
            let _admission = self.inner.admission.lock().await;
            self.admit("Submit Execution Command")?;
            self.reap_finished().await?;
            let manager = self.clone();
            self.spawn(async move {
                let response = manager.admit_execution_command(request).await;
                let _ = sender.send(response);
            })?;
        }
        receiver.await.map_err(|_| Error::Invalid("Submit Execution Command: Reconnect and Inspect Its Durable Receipt".into()))?
    }
    pub(crate) async fn start_cycle(&self, origins: Vec<Origin>) -> Result<Cycle> {
        let _admission = self.inner.admission.lock().await;
        self.admit("Start Cycle")?;
        self.reap_finished().await?;
        execution::start(self.clone(), origins).await
    }
    pub(crate) async fn start_execution_mode(&self, execution: &str, mode: ExecutionMode) -> Result<Cycle> {
        let _admission = self.inner.admission.lock().await;
        self.admit("Start Execution")?;
        self.reap_finished().await?;
        let context = self.inner.context.get().and_then(Weak::upgrade);
        let _evidence = match context.as_ref() {
            Some(context) => Some(context.administration.lock().await),
            None => None,
        };
        execution::start_existing_mode(self.clone(), execution, mode).await
    }
    fn admit(&self, action: &str) -> Result<()> {
        if self.is_shutting_down() {
            return Err(Error::Invalid(format!("{action}: Reconnect to a Running Host")));
        }
        Ok(())
    }
    pub(crate) async fn entry(&self, identifier: &str) -> Result<PluginEntry> {
        let context = self
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Start Execution: Assemble the Host Context".into()))?;
        context
            .host
            .plugins()
            .await?
            .into_iter()
            .find(|entry| entry.name == identifier)
            .ok_or_else(|| Error::Invalid(format!("Start Execution: Connect Plugin {identifier}")))
    }
    pub(crate) async fn streams(
        &self, entry: &PluginEntry, streams: &[String],
    ) -> Result<dokime_operations::host::executions::streams::Streams> {
        let context = self
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Capture Execution Streams: Assemble the Host Context".into()))?;
        let plugins = context.host.plugins().await?;
        Ok(dokime_operations::host::executions::streams::resolve(entry, &plugins, streams)?)
    }
    pub(crate) async fn materialize(&self, plugin: &str, template: &str, parameters: HashMap<String, Value>) -> Result<Origin> {
        let context = self
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Materialize Test Case: Assemble the Host Context".into()))?;
        Ok(context.host.materialize(plugin, template, parameters).await?)
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
        let tasks = mem::take(&mut *lock(&self.inner.tasks)?);
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
    pub(crate) fn spawn(&self, task: impl Future<Output = ()> + Send + 'static) -> Result<()> {
        let runtime = Handle::try_current().map_err(|_| Error::Invalid("Start Host Work: Run the Host Inside Its Tokio Runtime".into()))?;
        let mut tasks = lock(&self.inner.tasks)?;
        tasks.push(runtime.spawn(task));
        Ok(())
    }
    // An Abandoned Run Stays Active Until Its Binding Drains and Its Scope Is Released.
    pub(crate) async fn settled(&self, execution: &str) -> Result<()> {
        let mut changes = self.inner.changed.subscribe();
        while self.is_active(execution)? {
            changes.changed().await.map_err(|_| Error::Invalid("Settle Execution: Keep the Host Open While Its Run Drains".into()))?;
        }
        Ok(())
    }
    pub(crate) fn register_cycle(&self, identifier: &str) -> Result<()> {
        if self.inner.store.client_cycle(identifier)?.is_none() {
            let now = self.now_ns();
            self.inner.store.save_cycle(&CycleSummary {
                cycle_id: identifier.into(),
                continue_automatically: true,
                created_at: Some(Timestamp {
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
                sealed: false,
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
                    if sender.borrow().sealed {
                        return false;
                    }
                    let state = manager.inner.store.cut(execution);
                    let sealed = state.as_ref().is_ok_and(|cut| cut.state == ExecutionState::Sealed);
                    let pending =
                        manager.inner.prompts.pending_prompts().is_ok_and(|prompts| prompts.iter().any(|(id, _)| id == execution));
                    let phase = if sealed {
                        "sealed".into()
                    } else if pending {
                        "prompt".into()
                    } else {
                        manager.run_phase(execution).unwrap_or_else(|_| "interrupted".into())
                    };
                    sender.send_modify(|progress| {
                        progress.revision += 1;
                        progress.phase = phase;
                        progress.sealed = sealed || state.is_err();
                        if let Err(error) = state {
                            progress.failure = Some(error.to_string());
                        }
                    });
                    !sender.borrow().sealed
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
    /// Seals the execution's running file captures before its evidence closes.
    pub(crate) async fn settle_captures(&self, execution: &str) {
        if let Some(context) = self.inner.context.get().and_then(Weak::upgrade) {
            context.captures.settle(execution).await;
        }
    }
    pub(crate) fn finish_progress(&self, execution: &str, failure: Option<String>) -> Result<()> {
        // An Execution Closed Outside Its Driver Leaves No Capture Behind.
        if let Some(context) = self.inner.context.get().and_then(Weak::upgrade) {
            context.captures.discard(execution);
        }
        if let Some(sender) = lock(&self.inner.progress)?.remove(execution) {
            sender.send_modify(|progress| {
                progress.revision += 1;
                progress.phase = "sealed".into();
                progress.sealed = true;
                progress.failure = failure;
            });
        }
        self.signal_change();
        Ok(())
    }
    pub(crate) fn progress(&self, execution: &str) -> Result<Receiver<Progress>> {
        if let Some(sender) = lock(&self.inner.progress)?.get(execution) {
            return Ok(sender.subscribe());
        }
        if self.inner.store.cut(execution)?.state != ExecutionState::Sealed {
            return Err(Error::Invalid("Watch Execution Progress: Select an Admitted or Sealed Execution".into()));
        }
        Ok(watch::channel(Progress {
            revision: 1,
            phase: "sealed".into(),
            sealed: true,
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
        let tasks = mem::take(&mut *lock(&self.inner.tasks)?);
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

#[cfg(test)]
mod tests {
    use std::{error::Error, num::NonZeroUsize, sync::Arc, sync::Mutex, time::Duration};

    use tokio::sync::watch;

    use super::ExecutionManager;
    use crate::prompts::PromptManager;
    use crate::{CaptureSettings, ControlledClock, HostSettings, PlacementSettings};
    use dokime_operations::host::credentials::CredentialNamespace;
    use dokime_protocol::host::execution_outcome::Attribution;
    use dokime_protocol::host::{ExecutionOutcome, UnevaluatedOutcome};
    use dokime_protocol::model::{Origin, PluginRun, Termination, origin};
    use dokime_storage::{Evidence, Execution, ExecutionState, Store, storage_defaults};
    use tempfile::TempDir;

    fn manager() -> Result<(TempDir, ExecutionManager), Box<dyn Error>> {
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        let settings = HostSettings {
            installation: None,
            credentials: CredentialNamespace::new("dokime-seal-unit")?,
            storage: storage_defaults(),
            placement: PlacementSettings {
                reconnect_delay: Duration::from_secs(1),
                stream_window: NonZeroUsize::new(8).unwrap(),
                interval_window: crate::INTERVAL_WINDOW,
                interval_lag: crate::INTERVAL_LAG,
                message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            },
            clock: Arc::new(ControlledClock::new(0)),
            capture: CaptureSettings::default(),
        };
        let changed = watch::channel(0).0;
        let prompts = Arc::new(PromptManager::new(store.clone(), settings.clock.clone(), changed.clone()));
        let manager = ExecutionManager::new(store, settings, prompts, changed, Arc::new(Mutex::new(None)))?;
        manager.inner.store.create(&Execution {
            identifier: "execution".into(),
            cycle: "cycle".into(),
            origin: Some(Origin {
                kind: Some(origin::Kind::PluginRun(PluginRun {
                    plugin: "plugin".into(),
                    template: "template".into(),
                    digest: vec![1],
                    started_at: Some(Default::default()),
                    test_case: Some(Default::default()),
                })),
            }),
            ..Default::default()
        })?;
        Ok((root, manager))
    }

    #[tokio::test]
    async fn a_never_started_execution_remains_open_after_recovery() -> Result<(), Box<dyn Error>> {
        // Stop the Owner Before Its Frozen Execution Has Run
        let (_root, manager) = manager()?;
        let before = manager.inner.store.cut("execution")?;
        manager.shutdown().await?;

        // Recover Without Manufacturing an Interrupted Outcome or Seal
        let recovered = ExecutionManager::new(
            manager.inner.store.clone(),
            manager.inner.settings.clone(),
            manager.inner.prompts.clone(),
            manager.inner.changed.clone(),
            manager.inner._root_lease.clone(),
        )?;
        let after = recovered.inner.store.cut("execution")?;
        assert_eq!(after.state, ExecutionState::Open);
        assert!(after.execution.outcome.is_none());
        assert_eq!(after, before);
        assert!(recovered.progress("execution").is_err());
        recovered.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn a_started_execution_recovers_its_interrupted_sealed_outcome() -> Result<(), Box<dyn Error>> {
        // Retain Evidence That the Execution Started Before Its Owner Stopped
        let (_root, manager) = manager()?;
        manager.inner.store.append(
            "execution",
            Evidence {
                kind: "step:start".into(),
                step: "measure".into(),
                ..Default::default()
            },
        )?;
        manager.shutdown().await?;

        // Recover the Started Run With Its Interrupted Outcome
        let recovered = ExecutionManager::new(
            manager.inner.store.clone(),
            manager.inner.settings.clone(),
            manager.inner.prompts.clone(),
            manager.inner.changed.clone(),
            manager.inner._root_lease.clone(),
        )?;
        let cut = recovered.inner.store.cut("execution")?;
        assert_eq!(cut.state, ExecutionState::Sealed);
        let Some(Attribution::NotEvaluated(outcome)) = cut.execution.outcome.and_then(|outcome| outcome.attribution) else {
            panic!("Recover Execution: Missing Interrupted Outcome");
        };
        assert_eq!(outcome.termination, Termination::Interrupted as i32);
        assert!(recovered.progress("execution")?.borrow().sealed);
        recovered.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn execution_progress_settles_after_the_outcome_is_sealed() -> Result<(), Box<dyn Error>> {
        // Track an Admitted Execution Before Completion
        let (_root, manager) = manager()?;
        manager.track(&["execution".into()])?;
        let mut progress = manager.progress("execution")?;
        assert!(!progress.borrow_and_update().sealed);

        // Seal an Interrupted Outcome Before Publishing Terminal Progress
        let outcome = ExecutionOutcome {
            attribution: Some(Attribution::NotEvaluated(UnevaluatedOutcome {
                termination: Termination::Interrupted as i32,
                reason: String::new(),
            })),
        };
        manager.inner.store.close("execution", Termination::Interrupted, outcome.clone())?;
        manager.inner.store.seal("execution", Default::default())?;
        manager.finish_progress("execution", None)?;
        progress.changed().await?;
        assert!(progress.borrow_and_update().sealed);
        assert_eq!(manager.inner.store.cut("execution")?.execution.outcome, Some(outcome));
        manager.shutdown().await?;
        assert!(progress.changed().await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn execution_progress_reattaches_to_a_sealed_outcome() -> Result<(), Box<dyn Error>> {
        // Seal an Execution With No Live Progress Owner
        let (_root, manager) = manager()?;
        let outcome = ExecutionOutcome {
            attribution: Some(Attribution::NotEvaluated(UnevaluatedOutcome {
                termination: Termination::Completed as i32,
                reason: String::new(),
            })),
        };
        manager.inner.store.close("execution", Termination::Completed, outcome.clone())?;
        manager.inner.store.seal("execution", Default::default())?;

        // Return a Terminal Snapshot Without Waiting for a Runtime Update
        let mut progress = manager.progress("execution")?;
        assert!(progress.borrow_and_update().sealed);
        assert_eq!(manager.inner.store.cut("execution")?.execution.outcome, Some(outcome));
        assert!(progress.changed().await.is_err());
        manager.shutdown().await?;
        Ok(())
    }
}
