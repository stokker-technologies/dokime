/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

// Third Party Imports
use async_trait::async_trait;
use harmos_protocol::v1 as h;
use tokio::sync::{Notify, oneshot};
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::{Cycle, Error, PlacementSettings, PluginProcess, Result, SchedulerClock, dispatch::Dispatch, execution, plugin::Entry};
use dokime_operations_next::Prompts;
use dokime_protocol::v1;
use dokime_storage::{ExecutionSnapshot, ExecutionState, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct HostSettings {
    pub placement: PlacementSettings,
    pub clock: Arc<dyn SchedulerClock>,
}
#[derive(Clone)]
pub struct Host {
    pub(crate) inner: Arc<Inner>,
}

pub(crate) struct Inner {
    pub store: Store,
    pub settings: HostSettings,
    pub plugins: Mutex<HashMap<String, Arc<Entry>>>,
    pub active: Mutex<HashMap<String, Active>>,
    pub prompts: Arc<PromptHub>,
    pub idle: Notify,
}
pub(crate) struct Active {
    pub plugin: String,
    pub cancellation: CancellationToken,
}
#[derive(Default)]
pub(crate) struct PromptHub {
    pending: Mutex<HashMap<String, Pending>>,
    changed: Notify,
}
struct Pending {
    question: v1::OperatorQuestion,
    answer: Option<oneshot::Sender<v1::OperatorAnswer>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Host {
    pub fn open(root: &Path, settings: HostSettings) -> Result<Self> {
        let store = Store::open(root)?;
        // Recover Only Executions Driven by a Plugin Run
        for identifier in store.identifiers()? {
            let snapshot = store.read(&identifier)?;
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
        Ok(Self {
            inner: Arc::new(Inner {
                store,
                settings,
                plugins: Mutex::new(HashMap::new()),
                active: Mutex::new(HashMap::new()),
                prompts: Arc::new(PromptHub::default()),
                idle: Notify::new(),
            }),
        })
    }
    pub async fn add_remote(&self, endpoint: &str) -> Result<String> {
        let entry = Entry::admit(endpoint, None, &self.inner.settings.placement).await?;
        self.insert(entry)
    }
    pub async fn add_managed(&self, executable: &Path) -> Result<String> {
        let process = PluginProcess::start(executable, false, &self.inner.settings.placement).await?;
        let endpoint = process.endpoint().to_owned();
        let entry = Entry::admit(&endpoint, Some(process), &self.inner.settings.placement).await?;
        self.insert(entry)
    }
    fn insert(&self, entry: Entry) -> Result<String> {
        let identifier = entry.description.id.clone();
        let mut plugins = lock(&self.inner.plugins)?;
        if plugins.contains_key(&identifier) {
            return Err(Error::Invalid(format!("Remove the Existing Plugin Entry {identifier} Before Adding It Again")));
        }
        tracing::info!(plugin = %identifier, "Admitted Plugin Entry");
        plugins.insert(identifier.clone(), Arc::new(entry));
        Ok(identifier)
    }
    pub(crate) fn entry(&self, identifier: &str) -> Result<Arc<Entry>> {
        lock(&self.inner.plugins)?
            .get(identifier)
            .cloned()
            .ok_or_else(|| Error::Invalid(format!("Connect the Plugin {identifier} Before Starting Its Test Cases")))
    }
    pub fn catalog(&self, identifier: &str) -> Result<v1::Suites> {
        Ok(self.entry(identifier)?.suites.clone())
    }
    pub fn description(&self, identifier: &str) -> Result<h::Description> {
        Ok(self.entry(identifier)?.description.clone())
    }
    pub fn entry_state(&self, identifier: &str) -> Result<crate::EntryState> {
        Ok(self.entry(identifier)?.state)
    }
    pub async fn remove_plugin(&self, identifier: &str) -> Result<()> {
        if lock(&self.inner.active)?.values().any(|active| active.plugin == identifier) {
            return Err(Error::Invalid("Cancel the Plugin's Active Executions and Wait for Cleanup Before Removing It".into()));
        }
        let entry = self.entry(identifier)?;
        entry.close(&self.inner.settings.placement).await?;
        lock(&self.inner.plugins)?.remove(identifier);
        Ok(())
    }
    pub async fn materialize(&self, plugin: &str, template: &str, parameters: HashMap<String, v1::Value>) -> Result<v1::Origin> {
        let entry = self.entry(plugin)?;
        let definition = entry.template(template)?;
        let dispatch = Dispatch {
            entry: entry.clone(),
            resources: None,
            cancellation: CancellationToken::new(),
            settings: self.inner.settings.placement.clone(),
            clock: self.inner.settings.clock.clone(),
        };
        let test_case: v1::TestCase = dispatch
            .invoke(
                &definition.materialize,
                v1::MaterializeRequest {
                    template: template.into(),
                    parameters: parameters.clone(),
                },
            )
            .await?;
        if test_case.parameters != parameters {
            return Err(Error::Invalid("Materialize Without Changing the Resolved Parameter Set".into()));
        }
        let catalog = entry.description.catalog.as_ref().ok_or_else(|| Error::Invalid("Reconnect the Plugin Catalog".into()))?;
        for operation in
            test_case.steps.iter().map(|step| &step.operation).chain(test_case.criteria.iter().map(|criterion| &criterion.operation))
        {
            if !catalog.operations.iter().any(|contract| &contract.id == operation && contract.visibility == h::Visibility::Private as i32)
            {
                return Err(Error::Invalid(format!("Declare the Materialized Operation {operation} as Private")));
            }
        }
        let run = v1::PluginRun {
            test_case: Some(test_case),
            plugin: plugin.into(),
            template: template.into(),
            digest: catalog.digest.clone(),
            started_at: entry.description.started_at,
        };
        run.validate().map_err(Error::Invalid)?;
        Ok(v1::Origin {
            kind: Some(v1::origin::Kind::PluginRun(run)),
        })
    }
    pub async fn start_cycle(&self, origins: Vec<v1::Origin>) -> Result<Cycle> {
        execution::start(self.clone(), origins).await
    }
    pub fn now_ns(&self) -> i64 {
        self.inner.settings.clock.now_ns()
    }
    pub fn execution(&self, identifier: &str) -> Result<ExecutionSnapshot> {
        Ok(self.inner.store.read(identifier)?)
    }
    pub fn executions(&self) -> Result<Vec<String>> {
        Ok(self.inner.store.identifiers()?)
    }
    pub fn streams_closed(&self, identifier: &str) -> Result<bool> {
        Ok(self.inner.store.streams_closed(identifier)?)
    }
    pub async fn next_prompt(&self, execution: &str) -> Result<v1::OperatorQuestion> {
        loop {
            let changed = self.inner.prompts.changed.notified();
            if let Some(pending) = lock(&self.inner.prompts.pending)?.get(execution) {
                return Ok(pending.question.clone());
            }
            if !lock(&self.inner.active)?.contains_key(execution) {
                return Err(Error::Invalid("Start an Active Execution Before Waiting for Its Prompt".into()));
            }
            changed.await;
        }
    }
    pub fn pending_prompts(&self) -> Result<Vec<(String, v1::OperatorQuestion)>> {
        Ok(lock(&self.inner.prompts.pending)?.iter().map(|(execution, pending)| (execution.clone(), pending.question.clone())).collect())
    }
    pub fn answer(&self, execution: &str, answer: v1::OperatorAnswer) -> Result<()> {
        let mut pending = lock(&self.inner.prompts.pending)?;
        let prompt = pending.get_mut(execution).ok_or_else(|| Error::Invalid("Wait for an Execution Prompt Before Answering".into()))?;
        if answer.prompt != prompt.question.prompt {
            return Err(Error::Invalid("Answer the Current Execution Prompt".into()));
        }
        prompt
            .answer
            .take()
            .ok_or_else(|| Error::Invalid("Answer Each Prompt Only Once".into()))?
            .send(answer)
            .map_err(|_| Error::Invalid("The Execution Stopped Before the Answer; Inspect Its Evidence".into()))?;
        Ok(())
    }
    pub fn cancel(&self, execution: &str) -> Result<()> {
        let active = lock(&self.inner.active)?;
        active.get(execution).ok_or_else(|| Error::Invalid("Cancel Only an Execution with an Active Run".into()))?.cancellation.cancel();
        Ok(())
    }
    pub async fn shutdown(&self) -> Result<()> {
        for active in lock(&self.inner.active)?.values() {
            active.cancellation.cancel();
        }
        loop {
            let idle = self.inner.idle.notified();
            if lock(&self.inner.active)?.is_empty() {
                break;
            }
            idle.await;
        }
        let identifiers: Vec<_> = lock(&self.inner.plugins)?.keys().cloned().collect();
        for identifier in identifiers {
            self.remove_plugin(&identifier).await?;
        }
        Ok(())
    }
}
#[async_trait]
impl Prompts for PromptHub {
    async fn ask(&self, execution: &str, question: v1::OperatorQuestion) -> dokime_operations_next::Result<v1::OperatorAnswer> {
        let (send, receive) = oneshot::channel();
        {
            let mut pending = self
                .pending
                .lock()
                .map_err(|_| dokime_operations_next::Error::Invalid("Restart the Host After Its Prompt Store Failed".into()))?;
            if pending.contains_key(execution) {
                return Err(dokime_operations_next::Error::Invalid("Answer the Current Prompt Before Asking Another".into()));
            }
            pending.insert(execution.into(), Pending { question, answer: Some(send) });
        }
        self.changed.notify_waiters();
        let answer =
            receive.await.map_err(|_| dokime_operations_next::Error::Invalid("Execution Stopped Before the Operator Answered".into()));
        self.clear(execution);
        answer
    }
}
impl PromptHub {
    pub(crate) fn clear(&self, execution: &str) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.remove(execution);
        }
        self.changed.notify_waiters();
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(crate) fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>> {
    mutex.lock().map_err(|_| Error::Invalid("Restart the Host After Its State Lock Failed".into()))
}

/* === Tests === */
