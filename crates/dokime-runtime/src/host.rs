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

mod installation;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard, Weak},
};

// Third Party Imports
use harmos_operations::plugins::{Create, Delete, Watch};
use harmos_protocol::host::{
    PluginDescription, PluginEntry, PluginManaged, PluginPlacement, PluginRemote, PluginState, PluginValue, PluginsCreateRequest,
    PluginsDeleteRequest, PluginsWatchRequest, plugin_placement,
};
use harmos_protocol::model::{FileMetadata, FileReference, Receipt};
use harmos_runtime::bundle::Identity;
use harmos_runtime::operations::StreamSettings;
use prost::Message;
use sha2::{Digest, Sha256};
use tokio::sync::watch::{self, Receiver, Sender};
use uuid::Uuid;

// Local Imports
use crate::execution_manager::ExecutionManager;
use crate::prompts::PromptManager;
use crate::services::{self, Assembly, ClientSettings};
use crate::{CaptureSettings, Cycle, Error, ExecutionCapabilities, PlacementSettings, Result, SchedulerClock, await_plugin_ready, process};
use dokime_operations::host::content;
use dokime_operations::host::credentials::CredentialNamespace;
use dokime_operations::host::evidence::archive;
use dokime_operations::host::files::read;
use dokime_operations::host::files::uploads::{append, create, publish};
use dokime_packaging::bundle;
use dokime_protocol::host::content::ExecutionFileMetadata;
use dokime_protocol::host::executions::{CommandsCreateRequest, ExecutionCommandReceipt};
use dokime_protocol::host::files::{UploadsAppendRequest, UploadsCreateRequest, UploadsPublishRequest};
use dokime_protocol::host::{
    EvidenceArchiveRequest, EvidenceSubject, ExecutionControls, ExecutionMode, FilesReadRequest, PromptsCreateRequest,
    PromptsCreateResponse, evidence_subject,
};
use dokime_protocol::model::{Origin, Suites, Value};
use dokime_storage::{self, ExecutionSnapshot, OutcomeCounts, StorageSettings, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/// The Exclusive Lease on a Storage Root, Released at Shutdown or With Its Last Owner.
pub(crate) type RootLease = Arc<Mutex<Option<File>>>;

/* === Structs === */

#[derive(Clone)]
pub struct HostSettings {
    /// Installation Folder Override; Unset Resolves Beside the Running Product Executable.
    pub installation: Option<PathBuf>,

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
    pub(crate) plugin_admission_failures: Arc<Mutex<Vec<(String, String)>>>,
    pub(crate) executions: ExecutionManager,
    pub(crate) prompts: Arc<PromptManager>,
    changed: Sender<u64>,
    root_lease: RootLease,
    pub(crate) assembly: Option<Arc<Assembly>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Host {
    pub async fn open(root: &Path, settings: HostSettings, operations: ClientSettings) -> Result<Self> {
        // Every Host Entry Point Owns Its Descendants Before Any Plugin Starts.
        process::own_descendants()?;
        let mut host = Self::open_store(root, settings)?;
        host.assembly = Some(Arc::new(Box::pin(services::assemble(&host, root, &operations)).await?));
        host.reconcile_installation_plugins().await;
        Ok(host)
    }
    fn open_store(root: &Path, settings: HostSettings) -> Result<Self> {
        if settings.installation.as_ref().is_some_and(|directory| !directory.is_absolute()) {
            return Err(Error::Invalid("Configure Installation Plugins: Supply an Absolute Installation Folder".into()));
        }
        let root_error = |source| Error::RootIo { path: root.to_path_buf(), source };
        fs::create_dir_all(root).map_err(root_error)?;
        let lease =
            OpenOptions::new().create(true).truncate(false).read(true).write(true).open(root.join("local.lock")).map_err(root_error)?;
        match lease.try_lock() {
            Ok(()) => (),
            Err(fs::TryLockError::WouldBlock) => return Err(Error::OccupiedRoot(root.to_path_buf())),
            Err(fs::TryLockError::Error(error)) => return Err(root_error(error)),
        }
        let root_lease: RootLease = Arc::new(Mutex::new(Some(lease)));
        let store = Store::open_with(root, settings.storage)?;
        content::validate_store(&store)?;
        let changed = watch::channel(0).0;
        let plugins = Arc::new(Mutex::new(HashMap::new()));
        let prompts = Arc::new(PromptManager::new(store.clone(), settings.clock.clone(), changed.clone()));
        let executions = ExecutionManager::new(store.clone(), settings.clone(), prompts.clone(), changed.clone(), root_lease.clone())?;
        Ok(Self {
            store,
            settings,
            plugins,
            plugin_admission_failures: Arc::new(Mutex::new(Vec::new())),
            executions,
            prompts,
            changed,
            root_lease,
            assembly: None,
        })
    }
    pub async fn add_remote(&self, name: &str, endpoint: &str) -> Result<String> {
        let placement = PluginPlacement {
            kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: endpoint.into() })),
        };
        self.create_plugin(name.into(), placement, None).await
    }
    pub async fn add_managed(&self, name: &str, executable: &Path, assets: &[PathBuf]) -> Result<String> {
        let identity = Identity {
            name: name.into(),
            version: env!("CARGO_PKG_VERSION").parse().map_err(|source| Error::Invalid(format!("Create Plugin Bundle: {source}")))?,
            description: String::new(),
            package: Default::default(),
        };
        let bytes = bundle::create(executable, assets, identity, None)
            .map_err(|source| Error::Invalid(format!("Create Plugin Bundle: {source}")))?;
        let content = self
            .upload_plugin_bundle(
                &bytes,
                FileMetadata {
                    media_type: "application/x-tar".into(),
                    ..Default::default()
                },
            )
            .await?;
        let placement = PluginPlacement {
            kind: Some(plugin_placement::Kind::Managed(PluginManaged {
                sha256: content.sha256.clone(),
                content: Some(content),
            })),
        };
        self.create_plugin(name.into(), placement, None).await
    }
    async fn upload_plugin_bundle(&self, bytes: &[u8], metadata: FileMetadata) -> Result<FileReference> {
        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Add Plugin: Assemble the Host Context".into()))?;
        let created = create::handle(
            &context,
            UploadsCreateRequest {
                request_id: Uuid::now_v7().to_string(),
                metadata: Some(metadata),
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
                    request_id: Uuid::now_v7().to_string(),
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
                sha256: Sha256::digest(bytes).to_vec(),
                request_id: Uuid::now_v7().to_string(),
            },
        )
        .await?
        .upload
        .ok_or_else(|| Error::Invalid("Add Plugin: Publish Its Bundle".into()))?;
        Ok(content)
    }
    async fn create_plugin(&self, name: String, placement: PluginPlacement, origin: Option<String>) -> Result<String> {
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
                request_id: Uuid::now_v7().to_string(),
                origin,
            })
            .await?;
        self.await_plugin_ready(&name).await?;
        Ok(name)
    }
    pub async fn await_plugin_ready(&self, name: &str) -> Result<()> {
        let context = self
            .executions
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Await Plugin Ready: Assemble the Host Context".into()))?;
        let watch = context.operations.download_protobuf::<Watch>(
            PluginsWatchRequest { name: Some(name.into()), at: None },
            StreamSettings {
                items: self.settings.placement.stream_window,
                bytes: self.settings.placement.message_bytes,
            },
        )?;
        let mut readiness = (context, self.plugins.clone(), name.to_owned(), watch);
        let result = await_plugin_ready(
            &mut readiness,
            |(context, plugins, name, _)| {
                Box::pin(async move {
                    context.host.refresh_plugin_states().await?;
                    Ok(lock(plugins)?.get(name).cloned())
                })
            },
            |(_, _, _, watch)| Box::pin(async move { Ok(watch.next().await?.is_some()) }),
            |entry| {
                if entry.state == PluginState::Stopped as i32 {
                    return None;
                }
                let failure = entry.failure.as_ref().map_or_else(
                    || "The Plugin Became Unavailable; Restart It".to_owned(),
                    |failure| format!("{}; {}", failure.message, failure.action),
                );
                Some(Error::Invalid(format!("Add Plugin {name}: {failure}")))
            },
            || Error::Invalid(format!("Add Plugin {name}: Plugin Watch Closed")),
        )
        .await
        .map(|_| ());
        readiness.3.request_cancel();
        let _ = readiness.3.join().await;
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
        self.start_execution_mode(execution, ExecutionMode::Automatic).await
    }
    pub async fn submit_execution_command(&self, request: CommandsCreateRequest) -> Result<ExecutionCommandReceipt> {
        self.executions.submit_execution_command(request).await
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
        let response = archive::handle(
            &context,
            EvidenceArchiveRequest {
                subject: Some(EvidenceSubject {
                    kind: Some(evidence_subject::Kind::Execution(execution.into())),
                }),
                at: None,
            },
        )
        .await?;
        let file = response.file.ok_or_else(|| Error::Invalid("Export Execution: Restore Its Native Archive Reference".into()))?;
        let mut bytes = Vec::new();
        while (bytes.len() as u64) < file.length {
            let response = read::handle(
                &context,
                FilesReadRequest {
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
        content::attach_execution_file(&context, execution, metadata, file).await.map_err(|source| Error::Invalid(source.to_string()))
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
