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
use prost::Message;

// Local Imports
use crate::{Cycle, Error, PlacementSettings, PluginProcess, Result, SchedulerClock, plugin::Entry};
use dokime_protocol::v1;
use dokime_storage::{ExecutionSnapshot, Store};

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
    pub(crate) store: Store,
    pub(crate) settings: HostSettings,
    pub(crate) plugins: Arc<Mutex<HashMap<String, Arc<Entry>>>>,
    pub(crate) executions: crate::execution_manager::ExecutionManager,
    pub(crate) prompts: Arc<crate::prompts::PromptManager>,
    pub(crate) reports: crate::reports::ReportManager,
    changed: tokio::sync::watch::Sender<u64>,
    _root_lease: Arc<std::fs::File>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Host {
    pub fn open(root: &Path, settings: HostSettings) -> Result<Self> {
        let root_error = |source| Error::RootIo { path: root.to_path_buf(), source };
        std::fs::create_dir_all(root).map_err(root_error)?;
        let lease = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join("local.lock"))
            .map_err(root_error)?;
        match lease.try_lock() {
            Ok(()) => (),
            Err(std::fs::TryLockError::WouldBlock) => return Err(Error::OccupiedRoot(root.to_path_buf())),
            Err(std::fs::TryLockError::Error(error)) => return Err(root_error(error)),
        }
        let root_lease = Arc::new(lease);
        let store = Store::open(root)?;
        let changed = tokio::sync::watch::channel(0).0;
        let plugins = Arc::new(Mutex::new(HashMap::new()));
        let prompts = Arc::new(crate::prompts::PromptManager::new(store.clone(), settings.clock.clone(), changed.clone()));
        let executions = crate::execution_manager::ExecutionManager::new(
            store.clone(),
            settings.clone(),
            plugins.clone(),
            prompts.clone(),
            changed.clone(),
            root_lease.clone(),
        )?;
        let reports = crate::reports::ReportManager::new(store.clone());
        Ok(Self {
            store,
            settings,
            plugins,
            executions,
            prompts,
            reports,
            changed,
            _root_lease: root_lease,
        })
    }
    pub async fn add_remote(&self, endpoint: &str) -> Result<String> {
        let entry = Entry::admit(endpoint, None, &self.settings.placement).await?;
        self.insert(entry)
    }
    pub async fn add_managed(&self, executable: &Path) -> Result<String> {
        let process = PluginProcess::start(executable, false, &self.settings.placement).await?;
        let endpoint = process.endpoint().to_owned();
        let entry = Entry::admit(&endpoint, Some(process), &self.settings.placement).await?;
        self.insert(entry)
    }
    pub async fn add_remote_configured(&self, endpoint: &str, placement: &PlacementSettings) -> Result<String> {
        let entry = Entry::admit(endpoint, None, placement).await?;
        self.insert(entry)
    }
    pub async fn add_managed_configured(&self, executable: &Path, placement: &PlacementSettings) -> Result<String> {
        let process = PluginProcess::start(executable, false, placement).await?;
        let endpoint = process.endpoint().to_owned();
        let entry = Entry::admit(&endpoint, Some(process), placement).await?;
        self.insert(entry)
    }
    fn insert(&self, entry: Entry) -> Result<String> {
        self.validate_plugin_admission(&entry)?;
        let identifier = entry.description.id.clone();
        let mut plugins = lock(&self.plugins)?;
        if plugins.contains_key(&identifier) {
            return Err(Error::Invalid(format!("Remove the Existing Plugin Entry {identifier} Before Adding It Again")));
        }
        tracing::info!(plugin = %identifier, "Admitted Plugin Entry");
        self.store.save_plugin(&identifier, &entry.description.encode_to_vec())?;
        plugins.insert(identifier.clone(), Arc::new(entry));
        Ok(identifier)
    }
    pub(crate) fn entry(&self, identifier: &str) -> Result<Arc<Entry>> {
        lock(&self.plugins)?
            .get(identifier)
            .cloned()
            .ok_or_else(|| Error::Invalid(format!("Connect the Plugin {identifier} Before Starting Its Test Cases")))
    }
    pub fn catalog(&self, identifier: &str) -> Result<v1::Suites> {
        if let Some(entry) = lock(&self.plugins)?.get(identifier) {
            return Ok(entry.suites.clone());
        }
        let description = self.description(identifier)?;
        let catalog = description.catalog.ok_or_else(|| Error::Invalid("Inspect Plugin Catalog: Reconnect the Plugin".into()))?;
        let section = catalog
            .sections
            .iter()
            .find(|section| section.r#type == "dokime.v1.Suites")
            .ok_or_else(|| Error::Invalid("Inspect Plugin Catalog: Reconnect the Plugin with Its Suites".into()))?;
        Ok(v1::Suites::decode(section.payload.as_slice())?)
    }
    pub fn description(&self, identifier: &str) -> Result<harmos_protocol::plugin::InstanceGetResponse> {
        if let Some(entry) = lock(&self.plugins)?.get(identifier) {
            return Ok(entry.description.clone());
        }
        Ok(harmos_protocol::plugin::InstanceGetResponse::decode(self.store.plugin(identifier)?.as_slice())?)
    }
    pub fn entry_state(&self, identifier: &str) -> Result<crate::EntryState> {
        if let Some(entry) = lock(&self.plugins)?.get(identifier) {
            return Ok(entry.state);
        }
        self.store.plugin(identifier)?;
        Ok(crate::EntryState::Unavailable)
    }
    pub async fn remove_plugin(&self, identifier: &str) -> Result<()> {
        if self.executions.uses_plugin(identifier)? {
            return Err(Error::Invalid("Cancel the Plugin's Active Executions and Wait for Cleanup Before Removing It".into()));
        }
        let entry = self.entry(identifier)?;
        entry.close(&self.settings.placement).await?;
        lock(&self.plugins)?.remove(identifier);
        self.store.remove_plugin(identifier)?;
        Ok(())
    }
    pub async fn materialize(&self, plugin: &str, template: &str, parameters: HashMap<String, v1::Value>) -> Result<v1::Origin> {
        let entry = self.entry(plugin)?;
        crate::plugin::materialize(entry, template, parameters, &self.settings).await
    }
    pub async fn start_cycle(&self, origins: Vec<v1::Origin>) -> Result<Cycle> {
        self.executions.start_cycle(origins).await
    }
    pub fn capabilities(&self, execution: &str) -> Result<crate::ExecutionCapabilities> {
        self.executions.capabilities(execution)
    }
    pub fn execution_controls(&self, execution: &str) -> Result<v1::ExecutionControls> {
        self.executions.execution_controls(execution)
    }
    pub fn pause_execution(&self, execution: &str, paused: bool) -> Result<()> {
        self.executions.pause_execution(execution, paused)
    }
    pub fn stop_execution(&self, execution: &str) -> Result<()> {
        self.executions.stop_execution(execution)
    }
    pub async fn start_execution_mode(&self, execution: &str, mode: v1::ExecutionMode) -> Result<Cycle> {
        self.executions.start_execution_mode(execution, mode).await
    }
    pub async fn start_execution(&self, execution: &str) -> Result<Cycle> {
        self.start_execution_mode(execution, v1::ExecutionMode::Automatic).await
    }
    pub fn submit_execution_command(&self, request: v1::ExecutionSubmitExecutionCommandRequest) -> Result<v1::ExecutionCommandReceipt> {
        self.executions.submit_execution_command(request)
    }
    pub async fn rerun(&self, execution: &str) -> Result<Cycle> {
        if !self.capabilities(execution)?.rerun {
            return Err(Error::Invalid("Rerun Execution: Select a Sealed PluginRun Execution".into()));
        }
        let origin = self
            .store
            .cut(execution)?
            .execution
            .origin
            .ok_or_else(|| Error::Invalid("Rerun Execution: Supply Its Frozen Origin".into()))?;
        self.start_cycle(vec![origin]).await
    }
    pub fn export_execution(&self, execution: &str) -> Result<Vec<u8>> {
        if !self.capabilities(execution)?.export {
            return Err(Error::Invalid("Export Execution: Seal the Execution Before Exporting Evidence".into()));
        }
        Ok(self.store.export(execution)?)
    }
    pub fn now_ns(&self) -> i64 {
        self.settings.clock.now_ns()
    }
    pub fn execution(&self, identifier: &str) -> Result<ExecutionSnapshot> {
        Ok(self.store.read(identifier)?)
    }
    pub fn executions(&self) -> Result<Vec<String>> {
        Ok(self.store.identifiers()?)
    }
    pub fn cycle_counts(&self, cycle: &str) -> Result<dokime_storage::OutcomeCounts> {
        let outcomes = self.store.cycle_outcomes(cycle)?;
        Ok(dokime_storage::OutcomeCounts::from_outcomes(&outcomes))
    }
    pub fn plugin_entries(&self) -> Result<Vec<(String, crate::EntryState)>> {
        self.store
            .plugin_identifiers()?
            .into_iter()
            .map(|identifier| {
                let state = self.entry_state(&identifier)?;
                Ok((identifier, state))
            })
            .collect()
    }
    pub fn streams_closed(&self, identifier: &str) -> Result<bool> {
        Ok(self.store.streams_closed(identifier)?)
    }
    pub async fn next_prompt(&self, execution: &str) -> Result<v1::OperatorQuestion> {
        self.prompts.next(execution, &self.executions).await
    }
    pub fn pending_prompts(&self) -> Result<Vec<(String, v1::OperatorQuestion)>> {
        self.prompts.pending_prompts()
    }
    pub fn answer(&self, execution: &str, answer: v1::OperatorAnswer) -> Result<()> {
        self.prompts.answer(execution, answer)
    }
    pub fn cancel(&self, execution: &str) -> Result<()> {
        self.executions.cancel(execution)
    }
    pub fn changes(&self) -> tokio::sync::watch::Receiver<u64> {
        self.changed.subscribe()
    }
    pub fn is_shutting_down(&self) -> bool {
        self.executions.is_shutting_down()
    }
    pub fn signal_change(&self) {
        self.changed.send_modify(|revision| *revision += 1);
    }
    pub fn record_answer(&self, execution: &str, answer: v1::OperatorAnswer) -> Result<()> {
        self.prompts.record_answer(execution, answer)
    }
    pub async fn shutdown(&self) -> Result<()> {
        let mut result = self.executions.shutdown().await;
        if let Err(error) = self.prompts.shutdown() {
            result = Err(error);
        }
        if let Err(error) = self.reports.shutdown() {
            result = Err(error);
        }
        let identifiers: Vec<_> = lock(&self.plugins)?.keys().cloned().collect();
        for identifier in identifiers {
            if let Err(error) = self.entry(&identifier)?.close(&self.settings.placement).await {
                result = Err(error);
            }
            lock(&self.plugins)?.remove(&identifier);
        }
        result
    }
}
/* === Public Functions === */

/* === Private Functions === */

pub(crate) fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>> {
    mutex.lock().map_err(|_| Error::Invalid("Restart the Host After Its State Lock Failed".into()))
}

/* === Tests === */
