/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod follow;
mod logs;
pub(crate) mod reader;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    io::{Read, Seek},
    num::NonZeroUsize,
    sync::Arc,
    time::Instant,
};
// Third Party Imports
use futures_util::future::BoxFuture;
use harmos::{
    MemorySize,
    storage::{FileMetadata, FileReference, QueryValue, StorageFile, StorageProductionPreparation},
};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
// Local Imports
use super::executions::{Acquisition, ExecutionStorage};
use crate::execution_runtime::{
    authoring::ExecutionFileTarget,
    outputs::{
        ExecutionFileContent, ExecutionFileContentKind, ExecutionFileEntry,
        ExecutionFileStreamState, MAX_ARTIFACT_READ_BYTES, MAX_EXECUTION_FILE_SNAPSHOT_BYTES,
        media_type_for_path, validate_producer_path,
    },
};
use crate::{Error, Result};
use dokime::core::{Identifier, TestExecutionId, Timestamp};
use dokime::execution::saved::{ExecutionOutput, ExecutionOutputContent, SavedStreamState};
use dokime::storage::Manager;

/* === Constants === */
const COPY_BYTES: usize = 64 * 1024;
const MAX_PREFIX_BYTES: usize = 2 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct NativeOutput {
    file: tempfile::NamedTempFile,
    created_at: Timestamp,
    captured: bool,
    pending: Option<FrozenFile>,
    descriptor: Option<QueryValue<ExecutionOutput>>,
    log: Option<LogPrefix>,
    production: Option<follow::Production>,
    // Original Registry Node And Relative Key Charge Lives With Its Producer
    _registry: StorageProductionPreparation,
}

struct InventorySource {
    storage: Arc<ExecutionStorage>,
    execution: TestExecutionId,
}

struct FileSource {
    storage: Arc<ExecutionStorage>,
    execution: TestExecutionId,
    path: String,
    selection: FileSelection,
}

struct LogPrefix {
    acknowledged: u64,
    closed: Option<SavedStreamState>,
}

pub(super) struct FrozenFile {
    file: tokio::fs::File,
    metadata: QueryValue<FileMetadata>,
    stream_state: Option<SavedStreamState>,
    creating: tokio::sync::Mutex<Option<BoxFuture<'static, harmos::Result<StorageFile>>>>,
    creation_memory: Option<(StorageProductionPreparation, StorageProductionPreparation)>,
    producer: Option<StorageFile>,
    written: bool,
}

/* === Enums === */
#[derive(Clone, Copy)]
enum FileSelection {
    Window { offset: u64, limit: usize },
    Snapshot,
}

/* === Traits === */

/* === Implementations === */
impl FileSelection {
    fn extent(self, size_bytes: u64) -> Result<(u64, usize)> {
        match self {
            Self::Window { offset, limit } => {
                let offset = offset.min(size_bytes);
                let length = (size_bytes - offset).min(limit as u64);
                Ok((offset, usize::try_from(length).map_err(error)?))
            }
            Self::Snapshot => {
                if size_bytes > MAX_EXECUTION_FILE_SNAPSHOT_BYTES as u64 {
                    return Err(Error::ExecutionFileTooLarge);
                }
                Ok((0, usize::try_from(size_bytes).map_err(error)?))
            }
        }
    }
}

impl MemorySize for InventorySource {
    fn memory_size(&self) -> usize {
        let identity = self.execution.memory_size();
        if identity == usize::MAX {
            return usize::MAX;
        }
        std::mem::size_of::<Self>()
            .saturating_add(identity.saturating_sub(std::mem::size_of::<TestExecutionId>()))
    }
}

impl MemorySize for FileSource {
    fn memory_size(&self) -> usize {
        let identity = self.execution.memory_size();
        if identity == usize::MAX {
            return usize::MAX;
        }
        std::mem::size_of::<Self>()
            .saturating_add(identity.saturating_sub(std::mem::size_of::<TestExecutionId>()))
            .saturating_add(self.path.capacity())
    }
}

impl NativeOutput {
    fn new(storage: &Manager, path: &str) -> Result<Self> {
        let registry_bytes = harmos::btree_map_memory::<String, Self>(1)
            .checked_add(path.len())
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| error("native output registry memory overflows"))?;
        let registry = storage.prepare_production(registry_bytes)?;
        let suffix = std::path::Path::new(path)
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| format!(".{value}"))
            .unwrap_or_default();
        Ok(Self {
            file: tempfile::Builder::new()
                .prefix("dokime-output-")
                .suffix(&suffix)
                .tempfile()
                .map_err(error)?,
            created_at: Timestamp::now_utc(),
            captured: false,
            pending: None,
            descriptor: None,
            log: None,
            production: None,
            _registry: registry,
        })
    }

    fn visible_length(&self, physical: u64) -> Result<u64> {
        match &self.log {
            Some(log) if physical < log.acknowledged => {
                Err(error("native log lost its acknowledged prefix"))
            }
            Some(log) => Ok(log.acknowledged),
            None => Ok(physical),
        }
    }

    pub(super) fn descriptor(&self) -> Option<&ExecutionOutput> {
        self.descriptor.as_deref()
    }
}

impl ExecutionStorage {
    pub(crate) async fn native_target(
        self: &Arc<Self>,
        execution: TestExecutionId,
        relative_path: String,
        text_hint: bool,
        streamed: bool,
    ) -> Result<ExecutionFileTarget> {
        validate_producer_path(&relative_path)?;
        let storage = self.clone();
        tokio::spawn(async move {
            let owner = storage
                .acquisition(&execution)
                .await
                .ok_or_else(|| error("execution has no open output acquisition"))?;
            let mut acquisition = owner.lock().await;
            if acquisition.finish_requested || acquisition.outputs_sealed {
                return Err(error("execution has no open output acquisition"));
            }
            if acquisition.native_outputs.contains_key(&relative_path) {
                return Err(error("duplicate execution output path"));
            }
            let output = NativeOutput::new(&storage.storage, &relative_path)?;
            let absolute_path = output.file.path().to_string_lossy().into_owned();
            acquisition
                .native_outputs
                .insert(relative_path.clone(), output);
            Ok(ExecutionFileTarget {
                relative_path,
                absolute_path,
                text_hint,
                streamed,
            })
        })
        .await
        .map_err(error)?
    }

    pub(crate) async fn native_entries(
        self: &Arc<Self>,
        execution: &TestExecutionId,
    ) -> Result<QueryValue<Vec<ExecutionFileEntry>>> {
        let bytes = std::mem::size_of::<InventorySource>()
            .checked_add(execution.as_str().len())
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| error("native inventory source memory overflows"))?;
        let preparation = self.storage.prepare_read_production(bytes)?;
        let computation = preparation.computation()?;
        let deadline = computation.deadline();
        let source = preparation
            .finish(InventorySource {
                storage: self.clone(),
                execution: TestExecutionId::from_uuid(prefix_text(execution.as_str())?)
                    .map_err(error)?,
            })?
            .try_into_value()?
            .try_into_shared()?;
        computation.check()?;
        drop(computation);

        let preparation = source.prepare_work()?;
        let read = async move {
            let computation = source.computation()?;
            computation.check()?;
            let owner = source.storage.acquisition(&source.execution).await;
            computation.check()?;
            let Some(owner) = owner else {
                let bytes =
                    NonZeroUsize::new(std::mem::size_of::<Vec<ExecutionFileEntry>>()).unwrap();
                return Ok(computation.project_shared(&source, bytes, |_, _| {
                    Ok(Vec::<ExecutionFileEntry>::new())
                })?);
            };
            let acquisition = owner.lock_owned().await;
            computation.check()?;
            let mut count = 0usize;
            let mut bytes = std::mem::size_of::<Vec<ExecutionFileEntry>>();
            for (path, output) in &acquisition.native_outputs {
                let fields = [
                    std::mem::size_of::<ExecutionFileEntry>(),
                    source.execution.as_str().len(),
                    1,
                    path.len(),
                    path.len(),
                    media_type_for_path(path).map_or(0, str::len),
                    output.created_at.as_str().len(),
                ];
                for field in fields {
                    bytes = bytes
                        .checked_add(field)
                        .ok_or_else(|| error("native output inventory memory overflows"))?;
                }
                count += 1;
            }
            let allowance = NonZeroUsize::new(bytes)
                .ok_or_else(|| error("empty native inventory allowance"))?;
            let work_owner = source.clone();
            let physical = move || {
                computation
                    .project_shared(&source, allowance, |source, _| {
                        let mut entries = Vec::new();
                        entries
                            .try_reserve_exact(count)
                            .map_err(|failure| prefix_error(error(failure)))?;
                        if entries.capacity() > count {
                            return Err(prefix_error(error(
                                "native output inventory slots exceed admission",
                            )));
                        }
                        for (path, output) in &acquisition.native_outputs {
                            let id_bytes = source
                                .execution
                                .as_str()
                                .len()
                                .checked_add(1)
                                .and_then(|bytes| bytes.checked_add(path.len()))
                                .ok_or_else(|| {
                                    prefix_error(error(
                                        "native inventory identity memory overflows",
                                    ))
                                })?;
                            let mut artifact_id = String::new();
                            artifact_id
                                .try_reserve_exact(id_bytes)
                                .map_err(|failure| prefix_error(error(failure)))?;
                            if artifact_id.capacity() > id_bytes {
                                return Err(prefix_error(error(
                                    "native output inventory identity exceeds admission",
                                )));
                            }
                            artifact_id.push_str(source.execution.as_str());
                            artifact_id.push(':');
                            artifact_id.push_str(path);
                            #[cfg(windows)]
                            let _path_access = std::fs::File::open(output.file.path())
                                .map_err(|failure| prefix_error(error(failure)))?;
                            let file = output
                                .file
                                .reopen()
                                .map_err(|failure| prefix_error(error(failure)))?;
                            let length = file
                                .metadata()
                                .map_err(|failure| prefix_error(error(failure)))?
                                .len();
                            let size_bytes = output.visible_length(length).map_err(prefix_error)?;
                            let content_type = media_type_for_path(path)
                                .map(prefix_text)
                                .transpose()
                                .map_err(prefix_error)?;
                            entries.push(ExecutionFileEntry {
                                artifact_id,
                                relative_path: prefix_text(path).map_err(prefix_error)?,
                                content_kind: ExecutionFileContentKind::from_metadata(
                                    path,
                                    content_type.as_deref(),
                                ),
                                content_type,
                                stream_state: output_state(
                                    output
                                        .log
                                        .as_ref()
                                        .and_then(|log| log.closed)
                                        .or(output.captured.then_some(SavedStreamState::Finalized)),
                                ),
                                size_bytes,
                                updated_at: Timestamp::new(
                                    prefix_text(output.created_at.as_str())
                                        .map_err(prefix_error)?,
                                ),
                            });
                        }
                        Ok(entries)
                    })
                    .map_err(Error::from)
            };
            let memory = work_owner
                .reserve_work(NonZeroUsize::new(std::mem::size_of_val(&physical)).unwrap())?;
            let task = tokio::task::spawn_blocking(move || {
                let _memory = memory;
                physical()
            });
            tokio::time::timeout_at(deadline, task)
                .await
                .map_err(|_| harmos::Error::ResourceExhausted {
                    resource: "query computation time",
                })?
                .map_err(|source| Error::Join { source })?
        };
        let read = preparation.boxed(read)?;
        let task = tokio::spawn(read);
        tokio::time::timeout_at(deadline, task)
            .await
            .map_err(|_| harmos::Error::ResourceExhausted {
                resource: "query computation time",
            })?
            .map_err(|source| Error::Join { source })?
    }

    pub(crate) async fn native_read(
        self: &Arc<Self>,
        execution: &TestExecutionId,
        path: &str,
        offset: u64,
        limit: usize,
    ) -> Result<Option<QueryValue<ExecutionFileContent>>> {
        if limit == 0 || limit > MAX_ARTIFACT_READ_BYTES {
            return Err(error("execution output reads require 1 byte to 1 MiB"));
        }
        self.owned_native_read(execution, path, FileSelection::Window { offset, limit })
            .await
    }

    pub(crate) async fn native_read_up_to(
        self: &Arc<Self>,
        execution: &TestExecutionId,
        path: &str,
        maximum: usize,
    ) -> Result<Option<QueryValue<ExecutionFileContent>>> {
        if maximum == 0 || maximum > MAX_PREFIX_BYTES {
            return Err(error("execution output prefixes require 1 byte to 2 MiB"));
        }
        self.owned_native_read(
            execution,
            path,
            FileSelection::Window {
                offset: 0,
                limit: maximum,
            },
        )
        .await
    }

    async fn owned_native_read(
        self: &Arc<Self>,
        execution: &TestExecutionId,
        path: &str,
        selection: FileSelection,
    ) -> Result<Option<QueryValue<ExecutionFileContent>>> {
        let source_bytes = std::mem::size_of::<FileSource>()
            .checked_add(execution.as_str().len())
            .and_then(|bytes| bytes.checked_add(path.len()))
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| error("native prefix source memory overflows"))?;
        let preparation = self.storage.prepare_read_production(source_bytes)?;
        let computation = preparation.computation()?;
        let deadline = computation.deadline();
        let source = preparation
            .finish(FileSource {
                storage: self.clone(),
                execution: TestExecutionId::from_uuid(prefix_text(execution.as_str())?)
                    .map_err(error)?,
                path: prefix_text(path)?,
                selection,
            })?
            .try_into_value()?
            .try_into_shared()?;
        computation.check()?;
        drop(computation);

        // Admit The Actual Future Before Accepting Its Source Acquisition Work
        let preparation = source.prepare_work()?;
        let read = async move {
            let computation = source.computation()?;
            computation.check()?;
            let owner = source.storage.acquisition(&source.execution).await;
            computation.check()?;
            let Some(owner) = owner else {
                return Ok(None);
            };
            let acquisition = owner.lock().await;
            computation.check()?;
            let Some(output) = acquisition.native_outputs.get(&source.path) else {
                return Ok(None);
            };
            // Reopen The Original Inode While Its Registered Owner Is Protected
            #[cfg(windows)]
            let _path_access = std::fs::File::open(output.file.path()).map_err(error)?;
            let mut file = output.file.reopen().map_err(error)?;
            let size_bytes = output.visible_length(file.metadata().map_err(error)?.len())?;
            let stream_state = output_state(
                output
                    .log
                    .as_ref()
                    .and_then(|log| log.closed)
                    .or(output.captured.then_some(SavedStreamState::Finalized)),
            );
            drop(acquisition);
            computation.check()?;
            let (offset, length) = source.selection.extent(size_bytes)?;
            let allowance = content_memory(&source.path, length)?;
            let work_owner = source.clone();
            let physical = move || {
                // Cancellation Releases The Waiter; Accepted Physical Work Keeps Its Owners

                computation
                    .project_shared(&source, allowance, |source, _| {
                        let mut bytes = content_bytes(length).map_err(prefix_error)?;
                        file.seek(std::io::SeekFrom::Start(offset))
                            .map_err(|failure| prefix_error(error(failure)))?;
                        file.read_exact(&mut bytes)
                            .map_err(|failure| prefix_error(error(failure)))?;
                        let content_type = media_type_for_path(&source.path)
                            .map(prefix_text)
                            .transpose()
                            .map_err(prefix_error)?;
                        Ok(ExecutionFileContent {
                            relative_path: prefix_text(&source.path).map_err(prefix_error)?,
                            content_kind: ExecutionFileContentKind::from_metadata(
                                &source.path,
                                content_type.as_deref(),
                            ),
                            content_type,
                            stream_state,
                            offset,
                            size_bytes,
                            bytes,
                        })
                    })
                    .map_err(Error::from)
            };
            let memory = work_owner
                .reserve_work(NonZeroUsize::new(std::mem::size_of_val(&physical)).unwrap())?;
            let task = tokio::task::spawn_blocking(move || {
                let _memory = memory;
                physical()
            });
            tokio::time::timeout_at(deadline, task)
                .await
                .map_err(|_| harmos::Error::ResourceExhausted {
                    resource: "query computation time",
                })?
                .map_err(|source| Error::Join { source })?
                .map(Some)
        };
        let read = preparation.boxed(read)?;
        let task = tokio::spawn(read);
        tokio::time::timeout_at(deadline, task)
            .await
            .map_err(|_| harmos::Error::ResourceExhausted {
                resource: "query computation time",
            })?
            .map_err(|source| Error::Join { source })?
    }

    pub(crate) async fn native_snapshot(
        self: &Arc<Self>,
        execution: &TestExecutionId,
        path: &str,
    ) -> Result<Option<QueryValue<ExecutionFileContent>>> {
        self.owned_native_read(execution, path, FileSelection::Snapshot)
            .await
    }

    pub(crate) async fn native_stream(
        self: &Arc<Self>,
        execution: &TestExecutionId,
        path: &str,
    ) -> Result<Option<reader::Reader>> {
        let input_bytes = std::mem::size_of::<String>()
            .checked_add(path.len())
            .and_then(|bytes| bytes.checked_add(execution.memory_size()))
            .ok_or_else(|| error("native stream input memory overflows"))?;
        let preparation = reader::Reader::prepare(&self.storage, path)?;
        let computation = preparation.computation()?;
        let deadline = computation.deadline();
        let headers = std::mem::size_of::<reader::Reader>();
        let working =
            input_bytes
                .checked_add(headers)
                .ok_or_else(|| harmos::Error::ResourceExhausted {
                    resource: "native stream working memory",
                })?;
        let input_memory = self.storage.reserve_arrow_memory(working)?;
        let mut owned_path = String::new();
        owned_path
            .try_reserve_exact(path.len())
            .map_err(|_| harmos::Error::ResourceExhausted {
                resource: "native stream working memory",
            })?;
        if owned_path.capacity() > path.len() {
            return Err(harmos::Error::ResourceExhausted {
                resource: "native stream path",
            }
            .into());
        }
        owned_path.push_str(path);
        let execution = execution.clone();
        let storage = self.clone();
        let open = async move {
            let _input_memory = input_memory;
            computation.check()?;
            let owner = storage.acquisition(&execution).await;
            computation.check()?;
            let Some(owner) = owner else {
                return Ok(None);
            };
            let acquisition = owner.lock().await;
            computation.check()?;
            let Some(output) = acquisition.native_outputs.get(&owned_path) else {
                return Ok(None);
            };
            // Keep Accepted Open Work And Its Original Claims Together.
            #[cfg(windows)]
            let _path_access = std::fs::File::open(output.file.path()).map_err(error)?;
            let file = output.file.reopen().map_err(error)?;
            let observed_size = output.visible_length(file.metadata().map_err(error)?.len())?;
            let stream_state = output_state(
                output
                    .log
                    .as_ref()
                    .and_then(|log| log.closed)
                    .or(output.captured.then_some(SavedStreamState::Finalized)),
            );
            drop(acquisition);
            if observed_size > MAX_EXECUTION_FILE_SNAPSHOT_BYTES as u64 {
                return Err(Error::ExecutionFileTooLarge);
            }
            computation.check()?;
            reader::Reader::new(
                preparation,
                &computation,
                file,
                &owned_path,
                observed_size,
                stream_state,
            )
            .map(Some)
        };
        let working = std::mem::size_of_val(&open);
        let memory = self.storage.reserve_arrow_memory(working)?;
        let task = tokio::spawn(async move {
            let _memory = memory;
            open.await
        });
        tokio::time::timeout_at(deadline, task)
            .await
            .map_err(|_| harmos::Error::ResourceExhausted {
                resource: "query computation time",
            })?
            .map_err(|source| Error::Join { source })?
    }

    pub(crate) async fn retire_outputs(
        self: &Arc<Self>,
        execution: &TestExecutionId,
    ) -> Result<()> {
        let Some(owner) = self.acquisition(execution).await else {
            return Ok(());
        };
        tokio::spawn(async move {
            let mut acquisition = owner.lock().await;
            acquisition.outputs_sealed = true;
            acquisition.outputs_quiescent = true;
            abandon(&mut acquisition).await?;
            super::artifacts::close_incomplete(&mut acquisition).await
        })
        .await?
    }

    pub(crate) async fn finish_outputs(
        self: &Arc<Self>,
        execution: &TestExecutionId,
    ) -> Result<()> {
        // Running Captures Seal Into The Execution While It Still Admits Artifacts
        self.captures.close(execution).await;
        let storage = self.clone();
        let execution = execution.clone();
        tokio::spawn(async move {
            let owner = storage
                .acquisition(&execution)
                .await
                .ok_or_else(|| error("output finalization has no admitted acquisition"))?;
            let mut acquisition = owner.lock().await;
            // Provider Dispatch And File Tails Have Settled Before This Barrier.
            // The Original Owner Lock Drains Admitted Direct Log Appends As Well.
            acquisition.outputs_sealed = true;
            acquisition.outputs_quiescent = true;
            for output in acquisition.native_outputs.values_mut() {
                if let Some(log) = output.log.as_mut() {
                    log.closed.get_or_insert(SavedStreamState::Finalized);
                }
            }
            Ok(())
        })
        .await
        .map_err(error)?
    }

    pub(crate) async fn seal_output_admission(&self, execution: &TestExecutionId) -> Result<()> {
        let owner = self
            .acquisition(execution)
            .await
            .ok_or_else(|| error("output finalization has no admitted acquisition"))?;
        owner.lock().await.outputs_sealed = true;
        Ok(())
    }
}

impl FrozenFile {
    pub(super) fn new(file: tokio::fs::File, metadata: QueryValue<FileMetadata>) -> Self {
        Self {
            file,
            metadata,
            stream_state: None,
            creating: tokio::sync::Mutex::new(None),
            creation_memory: None,
            producer: None,
            written: false,
        }
    }

    pub(super) async fn finish(&mut self, storage: &Manager) -> Result<FileReference> {
        if !self.written {
            // The Frozen Source And Original Producer Survive A Failed Attempt.
            // Never Recreate A File With The Same Identity Or Replay Its Prefix.
            if self.producer.is_none() {
                if self.creating.get_mut().is_none() {
                    let metadata_bytes = self.metadata.memory_size();
                    if metadata_bytes == usize::MAX {
                        return Err(error("native file creation metadata memory overflows"));
                    }
                    let metadata_memory = storage.prepare_production(
                        NonZeroUsize::new(metadata_bytes)
                            .ok_or_else(|| error("empty native file metadata allowance"))?,
                    )?;
                    let manager = storage.clone();
                    let metadata = (*self.metadata).clone();
                    let future = async move { manager.produce_file(metadata).await };
                    let future_memory = storage.prepare_production(
                        NonZeroUsize::new(std::mem::size_of_val(&future))
                            .ok_or_else(|| error("empty native file creation future allowance"))?,
                    )?;
                    self.creation_memory = Some((metadata_memory, future_memory));
                    *self.creating.get_mut() = Some(Box::pin(future));
                }
                let outcome = self
                    .creating
                    .get_mut()
                    .as_mut()
                    .expect("original file creation retained")
                    .await;
                *self.creating.get_mut() = None;
                self.creation_memory = None;
                self.producer = Some(outcome?);
            }
            let retained = self
                .producer
                .as_ref()
                .expect("original file producer retained")
                .length();
            let frozen_length = self.file.metadata().await.map_err(error)?.len();
            if retained > frozen_length {
                return Err(error("original production exceeds frozen source"));
            }
            let started = Instant::now();
            tracing::debug!(file_id = %self.metadata.id, retained_bytes = retained,
                source_bytes = frozen_length, "native snapshot copy started");
            self.file
                .seek(std::io::SeekFrom::Start(retained))
                .await
                .map_err(error)?;
            let buffer_bytes = std::mem::size_of::<Vec<u8>>() + COPY_BYTES;
            let _buffer = storage.prepare_production(NonZeroUsize::new(buffer_bytes).unwrap())?;
            let mut buffer = copy_buffer()?;
            loop {
                let count = self.file.read(&mut buffer).await.map_err(error)?;
                if count == 0 {
                    break;
                }
                self.producer
                    .as_mut()
                    .expect("original file producer retained")
                    .write_all(&buffer[..count])
                    .await?;
            }
            self.written = true;
            tracing::debug!(file_id = %self.metadata.id, retained_bytes = frozen_length,
                elapsed_ms = started.elapsed().as_millis() as u64, "native snapshot copy settled");
        }
        // Finish Failure Retries This Same Handle And Admission Position
        let started = Instant::now();
        tracing::debug!(file_id = %self.metadata.id, "native snapshot seal started");
        let reference = self
            .producer
            .as_mut()
            .expect("written file producer retained")
            .finish()
            .await?;
        tracing::debug!(file_id = %self.metadata.id, retained_bytes = reference.length,
            elapsed_ms = started.elapsed().as_millis() as u64, "native snapshot seal settled");
        Ok(reference)
    }
}

/* === Public Functions === */
pub(super) async fn capture(acquisition: &mut Acquisition) -> Result<()> {
    if !acquisition.outputs_sealed || !acquisition.outputs_quiescent {
        return Err(error("native output capture requires producer quiescence"));
    }
    snapshot(acquisition, true).await
}

pub(super) async fn snapshot(acquisition: &mut Acquisition, finalizing: bool) -> Result<()> {
    let mut capture_error = None;
    for (path, source) in &mut acquisition.native_outputs {
        if source.captured {
            continue;
        }
        if finalizing && source.log.as_ref().is_some_and(|log| log.closed.is_none()) {
            return Err(error("native log producer has not closed"));
        }
        let stream_state = source
            .log
            .as_ref()
            .and_then(|log| log.closed)
            .or(finalizing.then_some(SavedStreamState::Finalized));
        if finalizing && source.production.is_some() {
            if let Err(failure) =
                follow::capture(&acquisition.storage, path, source, stream_state).await
            {
                follow::interrupt(&acquisition.storage, path, source).await?;
                capture_error.get_or_insert(failure);
            }
            continue;
        }
        // Complete Original Frozen Production Before Inspecting Mutable Bytes Again
        if source.pending.is_some() {
            finish_snapshot(&acquisition.storage, path, source).await?;
            if source.captured {
                continue;
            }
        }
        let (frozen, digest) = match freeze(&acquisition.storage, source).await {
            Ok(frozen) => frozen,
            Err(failure) if finalizing => {
                interrupt_source(
                    &acquisition.storage,
                    &acquisition.execution_id,
                    path,
                    source,
                )
                .await?;
                capture_error.get_or_insert(failure);
                continue;
            }
            Err(failure) => return Err(failure),
        };
        if source
            .descriptor
            .as_ref()
            .is_none_or(|previous| previous.file.sha256 != digest)
        {
            let length = frozen.metadata().await.map_err(error)?.len();
            let metadata = file_metadata(
                &acquisition.storage,
                &acquisition.execution_id,
                path,
                Some(length),
                Some(digest),
            )?;
            source.pending = Some(FrozenFile {
                file: frozen,
                metadata,
                stream_state,
                creating: tokio::sync::Mutex::new(None),
                creation_memory: None,
                producer: None,
                written: false,
            });
            finish_snapshot(&acquisition.storage, path, source).await?;
        }
        store_descriptor(&acquisition.storage, path, source, stream_state)?;
        source.captured = stream_state.is_some();
    }
    capture_error.map_or(Ok(()), Err)
}

pub(super) async fn abandon(acquisition: &mut Acquisition) -> Result<()> {
    for (path, source) in &mut acquisition.native_outputs {
        if !source.captured {
            interrupt_source(
                &acquisition.storage,
                &acquisition.execution_id,
                path,
                source,
            )
            .await?;
        }
    }
    Ok(())
}

/* === Private Functions === */
fn output_state(closed: Option<SavedStreamState>) -> ExecutionFileStreamState {
    match closed {
        Some(SavedStreamState::Finalized) => ExecutionFileStreamState::Finalized,
        Some(SavedStreamState::Interrupted) => ExecutionFileStreamState::Interrupted,
        None => ExecutionFileStreamState::Streaming,
    }
}

fn prefix_text(source: &str) -> Result<String> {
    let mut text = String::new();
    text.try_reserve_exact(source.len()).map_err(error)?;
    if text.capacity() > source.len() {
        return Err(harmos::Error::ResourceExhausted {
            resource: "native prefix text capacity",
        }
        .into());
    }
    text.push_str(source);
    Ok(text)
}

fn prefix_error(failure: Error) -> harmos::Error {
    match failure {
        Error::Harmos(source) => source,
        other => harmos::Error::backend(
            "read original native prefix",
            harmos::BackendFailure::Corrupt,
            other,
        ),
    }
}

fn content_memory(path: &str, bytes: usize) -> Result<NonZeroUsize> {
    std::mem::size_of::<ExecutionFileContent>()
        .checked_add(path.len())
        .and_then(|length| length.checked_add(media_type_for_path(path).map_or(0, str::len)))
        .and_then(|length| length.checked_add(bytes))
        .and_then(NonZeroUsize::new)
        .ok_or_else(|| error("native file content memory overflows"))
}

fn content_bytes(length: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| harmos::Error::ResourceExhausted {
            resource: "native file content allocation",
        })?;
    if bytes.capacity() > length {
        return Err(harmos::Error::ResourceExhausted {
            resource: "native file content capacity",
        }
        .into());
    }
    bytes.resize(length, 0);
    Ok(bytes)
}

async fn finish_snapshot(storage: &Manager, path: &str, source: &mut NativeOutput) -> Result<()> {
    let pending = source
        .pending
        .as_mut()
        .expect("original frozen snapshot retained");
    let reference = pending.finish(storage).await?;
    // Descriptor Refusal Retains Both The Finished Handle And Frozen Source
    let bytes = descriptor_memory(path, &reference, &source.created_at)?;
    let descriptor = storage
        .produce(bytes, |_| {
            Ok(ExecutionOutput {
                relative_path: path.to_owned(),
                created_at: source.created_at.clone(),
                file: reference.clone(),
                stream_state: pending.stream_state,
                content: ExecutionOutputContent::File,
            })
        })?
        .try_into_value()?;
    source.captured = pending.stream_state.is_some();
    source.descriptor = Some(descriptor);
    source.pending = None;
    Ok(())
}

fn file_metadata(
    storage: &Manager,
    execution: &TestExecutionId,
    path: &str,
    length: Option<u64>,
    digest: Option<[u8; 32]>,
) -> Result<QueryValue<FileMetadata>> {
    let media_type = media_type_for_path(path).unwrap_or("application/octet-stream");
    let prefix = "output.native.";
    let id_bytes = prefix.len() + uuid::fmt::Hyphenated::LENGTH;
    let bytes = std::mem::size_of::<FileMetadata>()
        .checked_add(id_bytes)
        .and_then(|bytes| bytes.checked_add(execution.as_str().len()))
        .and_then(|bytes| bytes.checked_add(media_type.len()))
        .and_then(NonZeroUsize::new)
        .ok_or_else(|| error("native file metadata memory overflows"))?;
    let metadata = storage.produce(bytes, |_| {
        let mut id = String::new();
        id.try_reserve_exact(id_bytes)
            .map_err(|_| harmos::Error::ResourceExhausted {
                resource: "native output file identity",
            })?;
        if id.capacity() > id_bytes {
            return Err(harmos::Error::ResourceExhausted {
                resource: "native output file identity capacity",
            });
        }
        id.push_str(prefix);
        let mut uuid = [0; uuid::fmt::Hyphenated::LENGTH];
        id.push_str(uuid::Uuid::now_v7().hyphenated().encode_lower(&mut uuid));
        Ok(FileMetadata {
            id: Identifier::new(id).map_err(|source| {
                harmos::Error::backend(
                    "native output file identity",
                    harmos::BackendFailure::Permanent,
                    source,
                )
            })?,
            scope: Some(Identifier::new(execution.as_str()).map_err(|source| {
                harmos::Error::backend(
                    "native output file scope",
                    harmos::BackendFailure::Permanent,
                    source,
                )
            })?),
            media_type: media_type.to_owned(),
            expected_length: length,
            expected_sha256: digest,
        })
    })?;
    Ok(metadata.try_into_value()?)
}

fn descriptor_memory(
    path: &str,
    reference: &FileReference,
    created_at: &Timestamp,
) -> Result<NonZeroUsize> {
    let reference_bytes = reference.memory_size();
    let timestamp_bytes = created_at.memory_size();
    if reference_bytes == usize::MAX || timestamp_bytes == usize::MAX {
        return Err(error("native output descriptor memory overflows"));
    }
    std::mem::size_of::<ExecutionOutput>()
        .checked_add(path.len())
        .and_then(|bytes| {
            bytes.checked_add(reference_bytes.saturating_sub(std::mem::size_of::<FileReference>()))
        })
        .and_then(|bytes| {
            bytes.checked_add(timestamp_bytes.saturating_sub(std::mem::size_of::<Timestamp>()))
        })
        .and_then(NonZeroUsize::new)
        .ok_or_else(|| error("native output descriptor memory overflows"))
}

fn copy_buffer() -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(COPY_BYTES)
        .map_err(|_| error("native file copy buffer allocation failed"))?;
    if buffer.capacity() > COPY_BYTES {
        return Err(error("native file copy buffer capacity exceeds admission"));
    }
    buffer.resize(COPY_BYTES, 0);
    Ok(buffer)
}

async fn freeze(storage: &Manager, source: &NativeOutput) -> Result<(tokio::fs::File, [u8; 32])> {
    let started = Instant::now();
    let mut file = open_file(source).await?;
    let before = file.metadata().await.map_err(error)?;
    let modified = before.modified().map_err(error)?;
    let mut remaining = source.visible_length(before.len())?;
    if remaining > MAX_EXECUTION_FILE_SNAPSHOT_BYTES as u64 {
        return Err(crate::Error::ExecutionFileTooLarge);
    }
    let source_bytes = remaining;
    tracing::debug!(source_bytes, "native snapshot freeze started");
    let mut frozen = tokio::fs::File::from_std(tempfile::tempfile().map_err(error)?);
    let mut digest = Sha256::new();
    let buffer_bytes = std::mem::size_of::<Vec<u8>>() + COPY_BYTES;
    let _buffer = storage.prepare_production(NonZeroUsize::new(buffer_bytes).unwrap())?;
    let mut buffer = copy_buffer()?;
    while remaining > 0 {
        let count = remaining.min(buffer.len() as u64) as usize;
        file.read_exact(&mut buffer[..count]).await.map_err(error)?;
        frozen.write_all(&buffer[..count]).await.map_err(error)?;
        digest.update(&buffer[..count]);
        remaining -= count as u64;
    }
    let after = open_file(source).await?.metadata().await.map_err(error)?;
    if after.len() != before.len() || after.modified().map_err(error)? != modified {
        return Err(error(
            "native output changed during snapshot; retry after the producer settles",
        ));
    }
    frozen.flush().await.map_err(error)?;
    frozen
        .seek(std::io::SeekFrom::Start(0))
        .await
        .map_err(error)?;
    tracing::debug!(
        source_bytes,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "native snapshot freeze settled"
    );
    Ok((frozen, digest.finalize().into()))
}

fn store_descriptor(
    storage: &Manager,
    path: &str,
    source: &mut NativeOutput,
    state: Option<SavedStreamState>,
) -> Result<()> {
    if source
        .descriptor
        .as_ref()
        .is_some_and(|descriptor| descriptor.stream_state == state)
    {
        return Ok(());
    }
    let reference = &source
        .descriptor
        .as_ref()
        .ok_or_else(|| error("native output snapshot is missing"))?
        .file;
    let descriptor = storage
        .produce(
            descriptor_memory(path, reference, &source.created_at)?,
            |_| {
                Ok(ExecutionOutput {
                    relative_path: path.to_owned(),
                    created_at: source.created_at.clone(),
                    file: reference.clone(),
                    stream_state: state,
                    content: ExecutionOutputContent::File,
                })
            },
        )?
        .try_into_value()?;
    source.descriptor = Some(descriptor);
    Ok(())
}

async fn open_file(output: &NativeOutput) -> Result<tokio::fs::File> {
    // Windows can reopen the original handle after its pathname is deleted.
    // Require the producer-facing path to remain readable, as Unix reopen does.
    #[cfg(windows)]
    let _path_access = tokio::fs::File::open(output.file.path())
        .await
        .map_err(error)?;

    // Read the original file rather than a replacement at its pathname.
    let file = output.file.reopen().map_err(error)?;
    Ok(tokio::fs::File::from_std(file))
}

fn error(value: impl ToString) -> Error {
    Error::Operation(value.to_string())
}

async fn interrupt_source(
    storage: &Manager,
    execution: &TestExecutionId,
    path: &str,
    source: &mut NativeOutput,
) -> Result<()> {
    if source.production.is_some() {
        return follow::interrupt(storage, path, source).await;
    }
    if source.pending.is_some() {
        finish_snapshot(storage, path, source).await?;
    }
    if source.descriptor.is_none() {
        let digest: [u8; 32] = Sha256::digest([]).into();
        source.pending = Some(FrozenFile {
            file: tokio::fs::File::from_std(tempfile::tempfile().map_err(error)?),
            metadata: file_metadata(storage, execution, path, Some(0), Some(digest))?,
            stream_state: Some(SavedStreamState::Interrupted),
            creating: tokio::sync::Mutex::new(None),
            creation_memory: None,
            producer: None,
            written: false,
        });
        finish_snapshot(storage, path, source).await?;
    }
    store_descriptor(storage, path, source, Some(SavedStreamState::Interrupted))?;
    source.captured = true;
    Ok(())
}

/* === Tests === */
