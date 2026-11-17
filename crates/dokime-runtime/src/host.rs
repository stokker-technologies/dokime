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
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard, Weak},
};

// Third Party Imports
use harmos_protocol::host::PluginDescription;
use harmos_protocol::host::plugin_placement;
use harmos_protocol::host::{PluginEntry, PluginManaged, PluginPlacement, PluginRemote, PluginState};
use harmos_protocol::model::{FileReference, Receipt};
use prost::Message;
use tokio::sync::watch::Receiver;

// Local Imports
use crate::ExecutionCapabilities;
use crate::{CaptureSettings, Cycle, Error, PlacementSettings, Result, SchedulerClock};
use dokime_operations::host::credentials::CredentialNamespace;
use dokime_protocol::host::ExecutionControls;
use dokime_protocol::host::ExecutionMode;
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_protocol::host::content::ExecutionFileMetadata;
use dokime_protocol::host::executions::CommandsCreateRequest;
use dokime_protocol::host::executions::ExecutionCommandReceipt;
use dokime_protocol::model::Origin;
use dokime_protocol::model::Suites;
use dokime_protocol::model::Value;
use dokime_storage::{ExecutionSnapshot, Store};

use dokime_storage::OutcomeCounts;
use dokime_storage::StorageSettings;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/// The Exclusive Lease on a Storage Root, Released at Shutdown or With Its Last Owner.
pub(crate) type RootLease = Arc<Mutex<Option<std::fs::File>>>;

/* === Structs === */

#[derive(Clone)]
pub struct HostSettings {
    pub placement: PlacementSettings,
    pub storage: StorageSettings,
    pub clock: Arc<dyn SchedulerClock>,
    pub capture: CaptureSettings,
    /// The Prefix of Every Native Credential the Host Owns; Default `dokime`.
    pub credentials: CredentialNamespace,
}
#[derive(Clone)]
pub struct Host {
    pub(crate) store: Store,
    pub(crate) settings: HostSettings,
    pub(crate) plugins: Arc<Mutex<HashMap<String, PluginEntry>>>,
    pub(crate) executions: crate::execution_manager::ExecutionManager,
    pub(crate) prompts: Arc<crate::prompts::PromptManager>,
    pub(crate) reports: crate::reports::ReportManager,
    changed: tokio::sync::watch::Sender<u64>,
    root_lease: RootLease,
    pub(crate) assembly: Option<Arc<crate::services::Assembly>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Host {
    pub async fn open(root: &Path, settings: HostSettings, operations: crate::services::ClientSettings) -> Result<Self> {
        // Every Host Entry Point Owns Its Descendants Before Any Plugin Starts.
        crate::process::own_descendants()?;
        let mut host = Self::open_store(root, settings)?;
        host.assembly = Some(Arc::new(Box::pin(crate::services::assemble(&host, root, &operations)).await?));
        Ok(host)
    }
    fn open_store(root: &Path, settings: HostSettings) -> Result<Self> {
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
        let root_lease: RootLease = Arc::new(Mutex::new(Some(lease)));
        let store = Store::open_with(root, settings.storage)?;
        dokime_operations::host::content::validate_store(&store)?;
        let changed = tokio::sync::watch::channel(0).0;
        let plugins = Arc::new(Mutex::new(HashMap::new()));
        let prompts = Arc::new(crate::prompts::PromptManager::new(store.clone(), settings.clock.clone(), changed.clone()));
        let executions = crate::execution_manager::ExecutionManager::new(
            store.clone(),
            settings.clone(),
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
            root_lease,
            assembly: None,
        })
    }
    pub async fn add_remote(&self, name: &str, endpoint: &str) -> Result<String> {
        let placement = PluginPlacement {
            kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: endpoint.into() })),
        };
        self.create_plugin(name.into(), placement).await
    }
    pub async fn add_managed(&self, name: &str, executable: &Path, assets: &[PathBuf]) -> Result<String> {
        use harmos_runtime::bundle::Identity;
        let identity = Identity {
            name: name.into(),
            version: env!("CARGO_PKG_VERSION").parse().map_err(|source| Error::Invalid(format!("Create Plugin Bundle: {source}")))?,
            description: String::new(),
            package: Default::default(),
        };
        let bytes = dokime_packaging::bundle::create(executable, assets, identity, None)
            .map_err(|source| Error::Invalid(format!("Create Plugin Bundle: {source}")))?;
        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Add Plugin: Assemble the Host Context".into()))?;
        use dokime_operations::host::files::uploads::{append, create, publish};
        use dokime_protocol::host::files::{UploadsAppendRequest, UploadsCreateRequest, UploadsPublishRequest};
        use harmos_protocol::model::FileMetadata;
        use sha2::{Digest, Sha256};
        let created = create::handle(
            &context,
            UploadsCreateRequest {
                request_id: uuid::Uuid::now_v7().to_string(),
                metadata: Some(FileMetadata {
                    media_type: "application/x-tar".into(),
                    ..Default::default()
                }),
            },
        )
        .await?;
        let mut upload = created.upload.ok_or_else(|| Error::Invalid("Add Plugin: Create Its Upload".into()))?;
        let mut offset = 0;
        for chunk in bytes.chunks(created.chunk_bytes as usize) {
            let response = append::handle(
                &context,
                UploadsAppendRequest {
                    upload: Some(upload),
                    offset,
                    data: chunk.to_vec(),
                    request_id: uuid::Uuid::now_v7().to_string(),
                },
            )
            .await?;
            upload = response.upload.ok_or_else(|| Error::Invalid("Add Plugin: Retain Its Upload".into()))?;
            offset += chunk.len() as u64;
        }
        let content = publish::handle(
            &context,
            UploadsPublishRequest {
                upload: Some(upload),
                length: bytes.len() as u64,
                sha256: Sha256::digest(&bytes).to_vec(),
                request_id: uuid::Uuid::now_v7().to_string(),
            },
        )
        .await?
        .upload
        .ok_or_else(|| Error::Invalid("Add Plugin: Publish Its Bundle".into()))?;
        let placement = PluginPlacement {
            kind: Some(plugin_placement::Kind::Managed(PluginManaged {
                sha256: content.sha256.clone(),
                content: Some(content),
            })),
        };
        self.create_plugin(name.into(), placement).await
    }
    async fn create_plugin(&self, name: String, placement: PluginPlacement) -> Result<String> {
        use harmos_protocol::host::{PluginValue, PluginsCreateRequest};
        use harmos_runtime::builtins::plugins::Create;
        self.admit("Add Plugin")?;
        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Add Plugin: Assemble the Host Context".into()))?;
        context
            .operations
            .call_protobuf::<Create>(PluginsCreateRequest {
                name: name.clone(),
                placement: Some(placement),
                value: Some(PluginValue { enabled: true }),
                request_id: uuid::Uuid::now_v7().to_string(),
            })
            .await?;
        self.await_plugin_ready(&name).await?;
        Ok(name)
    }
    pub async fn await_plugin_ready(&self, name: &str) -> Result<()> {
        use harmos_protocol::host::{PluginState, PluginsWatchRequest};
        use harmos_runtime::builtins::plugins::Watch;
        use harmos_runtime::operations::StreamSettings;
        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Await Plugin Ready: Assemble the Host Context".into()))?;
        let mut watch = context.operations.download_protobuf::<Watch>(
            PluginsWatchRequest { name: Some(name.into()), at: None },
            StreamSettings {
                items: self.settings.placement.stream_window,
                bytes: self.settings.placement.message_bytes,
            },
        )?;
        let result = loop {
            context.host.refresh_plugin_states().await?;
            let entry = lock(&self.plugins)?.get(name).cloned();
            if let Some(entry) = entry {
                if entry.state == PluginState::Ready as i32 {
                    break Ok(());
                }
                if matches!(PluginState::try_from(entry.state), Ok(PluginState::Unavailable)) {
                    // The Entry's Failure Names the Cause, Such as a Missed Load Deadline, and the Action.
                    let failure = entry.failure.as_ref().map_or_else(
                        || "The Plugin Became Unavailable; Restart It".to_owned(),
                        |failure| format!("{}; {}", failure.message, failure.action),
                    );
                    break Err(Error::Invalid(format!("Add Plugin {name}: {failure}")));
                }
            }
            if watch.next().await?.is_none() {
                break Err(Error::Invalid(format!("Add Plugin {name}: Plugin Watch Closed")));
            }
        };
        watch.request_cancel();
        let _ = watch.join().await;
        result
    }
    fn admit(&self, action: &str) -> Result<()> {
        if self.is_shutting_down() {
            return Err(Error::Invalid(format!("{action}: Reconnect to a Running Host")));
        }
        Ok(())
    }
    pub fn catalog(&self, identifier: &str) -> Result<Suites> {
        let description = self.description(identifier)?;
        let catalog = description.declarations.ok_or_else(|| Error::Invalid("Inspect Plugin Catalog: Reconnect the Plugin".into()))?;
        let section = catalog
            .sections
            .iter()
            .find(|section| section.r#type == "dokime.model.v1.Suites")
            .ok_or_else(|| Error::Invalid("Inspect Plugin Catalog: Declare Its Suites".into()))?;
        Ok(Suites::decode(section.payload.as_slice())?)
    }
    pub fn description(&self, identifier: &str) -> Result<PluginDescription> {
        let entry = lock(&self.plugins)?
            .get(identifier)
            .cloned()
            .ok_or_else(|| Error::Invalid(format!("Inspect Plugin {identifier}: Discover Its Entry")))?;
        let description = entry.description.ok_or_else(|| Error::Invalid(format!("Inspect Plugin {identifier}: Reconnect the Plugin")))?;
        Ok(description)
    }
    pub fn entry_state(&self, identifier: &str) -> Result<PluginState> {
        let state = lock(&self.plugins)?
            .get(identifier)
            .map(|entry| entry.state)
            .ok_or_else(|| Error::Invalid(format!("Read Plugin State {identifier}: Discover Its Entry")))?;
        PluginState::try_from(state).map_err(|source| Error::Invalid(format!("Read Plugin State: {source}")))
    }
    pub async fn remove_plugin(&self, identifier: &str) -> Result<()> {
        use harmos_protocol::host::PluginsDeleteRequest;
        use harmos_runtime::builtins::plugins::Delete;
        if self.executions.uses_plugin(identifier)? {
            return Err(Error::Invalid("Remove Plugin: Cancel Its Active Executions and Await Cleanup".into()));
        }
        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Remove Plugin: Assemble the Host Context".into()))?;
        let entry = context
            .host
            .plugins()
            .await?
            .into_iter()
            .find(|entry| entry.name == identifier)
            .ok_or_else(|| Error::Invalid(format!("Remove Plugin {identifier}: Select Its Entry")))?;
        context
            .operations
            .call_protobuf::<Delete>(PluginsDeleteRequest {
                name: identifier.into(),
                expected_revision: entry.revision,
            })
            .await?;
        context.host.refresh_plugin_states().await?;
        Ok(())
    }
    pub async fn detach_plugin(&self, identifier: &str) -> Result<()> {
        self.remove_plugin(identifier).await
    }
    pub async fn materialize(&self, plugin: &str, template: &str, parameters: HashMap<String, Value>) -> Result<Origin> {
        self.executions.materialize(plugin, template, parameters).await
    }
    pub async fn start_cycle(&self, origins: Vec<Origin>) -> Result<Cycle> {
        self.executions.start_cycle(origins).await
    }
    pub fn capabilities(&self, execution: &str) -> Result<ExecutionCapabilities> {
        self.executions.capabilities(execution)
    }
    pub fn execution_controls(&self, execution: &str) -> Result<ExecutionControls> {
        self.executions.execution_controls(execution)
    }
    pub fn pause_execution(&self, execution: &str, paused: bool) -> Result<()> {
        self.executions.pause_execution(execution, paused)
    }
    pub fn stop_execution(&self, execution: &str) -> Result<()> {
        self.executions.stop_execution(execution)
    }
    pub async fn start_execution_mode(&self, execution: &str, mode: ExecutionMode) -> Result<Cycle> {
        self.executions.start_execution_mode(execution, mode).await
    }
    pub async fn start_execution(&self, execution: &str) -> Result<Cycle> {
        self.start_execution_mode(execution, dokime_protocol::host::ExecutionMode::Automatic).await
    }
    pub fn submit_execution_command(&self, request: CommandsCreateRequest) -> Result<ExecutionCommandReceipt> {
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
    pub async fn export_execution(&self, execution: &str) -> Result<Vec<u8>> {
        if !self.capabilities(execution)?.export {
            return Err(Error::Invalid("Export Execution: Seal the Execution Before Exporting Evidence".into()));
        }
        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Export Execution: Open the Host Before Reading Evidence".into()))?;
        let response = dokime_operations::host::evidence::archive::handle(
            &context,
            dokime_protocol::host::EvidenceArchiveRequest {
                subject: Some(dokime_protocol::host::EvidenceSubject {
                    kind: Some(dokime_protocol::host::evidence_subject::Kind::Execution(execution.into())),
                }),
                at: None,
            },
        )
        .await?;
        let file = response.file.ok_or_else(|| Error::Invalid("Export Execution: Restore Its Native Archive Reference".into()))?;
        let mut bytes = Vec::new();
        while (bytes.len() as u64) < file.length {
            let response = dokime_operations::host::files::read::handle(
                &context,
                dokime_protocol::host::FilesReadRequest {
                    file: Some(file.clone()),
                    offset: bytes.len() as u64,
                    max_bytes: 0,
                    at: None,
                },
            )
            .await?;
            if response.data.is_empty() {
                return Err(Error::Invalid("Export Execution: Restore Its Complete Native Archive".into()));
            }
            bytes.extend(response.data);
        }
        Ok(bytes)
    }
    pub async fn attach_execution_file(&self, execution: &str, metadata: ExecutionFileMetadata, file: FileReference) -> Result<Receipt> {
        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Attach Execution File: Open the Host Before Registering Content".into()))?;
        dokime_operations::host::content::attach_execution_file(&context, execution, metadata, file)
            .await
            .map_err(|source| Error::Invalid(source.to_string()))
    }
    /// Replaces the folders plugins may capture from; captures opened afterwards use them.
    pub fn set_capture_roots(&self, roots: Vec<PathBuf>) {
        if let Some(assembly) = &self.assembly {
            assembly.context.captures.set_roots(roots);
        }
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
    pub fn cycle_counts(&self, cycle: &str) -> Result<OutcomeCounts> {
        let outcomes = self.store.cycle_outcomes(cycle)?;
        Ok(OutcomeCounts::from_outcomes(&outcomes))
    }
    pub fn plugin_entries(&self) -> Result<Vec<(String, PluginState)>> {
        lock(&self.plugins)?
            .iter()
            .map(|(name, entry)| {
                Ok((
                    name.clone(),
                    PluginState::try_from(entry.state).map_err(|source| Error::Invalid(format!("Read Plugin State: {source}")))?,
                ))
            })
            .collect()
    }
    pub fn streams_closed(&self, identifier: &str) -> Result<bool> {
        Ok(self.store.streams_closed(identifier)?)
    }
    pub async fn next_prompt(&self, execution: &str) -> Result<PromptsCreateRequest> {
        self.prompts.next(execution, &self.executions).await
    }
    pub fn pending_prompts(&self) -> Result<Vec<(String, PromptsCreateRequest)>> {
        self.prompts.pending_prompts()
    }
    pub fn answer(&self, execution: &str, answer: PromptsCreateResponse) -> Result<()> {
        self.prompts.answer(execution, answer)
    }
    pub fn cancel(&self, execution: &str) -> Result<()> {
        self.executions.cancel(execution)
    }
    pub fn changes(&self) -> Receiver<u64> {
        self.changed.subscribe()
    }
    pub fn is_shutting_down(&self) -> bool {
        self.executions.is_shutting_down()
    }
    pub fn signal_change(&self) {
        self.changed.send_modify(|revision| *revision += 1);
    }
    pub fn record_answer(&self, execution: &str, request_id: &str, fingerprint: &[u8], answer: PromptsCreateResponse) -> Result<Receipt> {
        self.prompts.record_answer(execution, request_id, fingerprint, answer)
    }
    pub async fn shutdown(&self) -> Result<()> {
        let mut result = self.executions.shutdown().await;
        if let Err(error) = self.prompts.shutdown() {
            result = Err(error);
        }
        if let Err(error) = self.reports.shutdown() {
            result = Err(error);
        }
        if let Err(error) = self.store.shutdown_samples().await {
            result = Err(error.into());
        }
        if let Some(assembly) = &self.assembly {
            if let Err(source) = assembly.runtime.shutdown_with_timeout(assembly.close_timeout).await {
                result = Err(source.into());
            }
            if let Err(source) = assembly.backend_lease.close() {
                result = Err(dokime_storage::Error::from(source).into());
            }
        }
        // Release the Root Once Owned Work Has Settled, Whatever Handles Remain. Unlock Before Closing: the
        // System Releases a Lock Left on a Closed Handle Only Eventually, So the Next Owner Could Find It Held.
        if let Some(lease) = lock(&self.root_lease)?.take()
            && let Err(source) = lease.unlock()
        {
            result = Err(Error::Invalid(format!("Release the Root Lease: {source}")));
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
