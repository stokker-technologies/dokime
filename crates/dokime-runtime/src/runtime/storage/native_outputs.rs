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
use std::{sync::Arc, time::Instant};

// Third Party Imports
use futures_util::future::BoxFuture;
use harmos::storage::{ByteRange, CaptureConsistency, FileMetadata, FileReference, FrozenFile as FrozenSnapshot, OpenedCapture};

// Local Imports
use super::executions::{Acquisition, ExecutionStorage};
use crate::execution_runtime::{
    authoring::ExecutionFileTarget,
    outputs::{
        ExecutionFileContent, ExecutionFileContentKind, ExecutionFileEntry, ExecutionFileStreamState, media_type_for_path,
        validate_producer_path,
    },
};
use crate::storage::Manager;
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::execution::saved::ExecutionOutput;
use dokime_domain::model::execution::saved::ExecutionOutputContent;
use dokime_domain::model::execution::saved::SavedStreamState;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct NativeOutput {
    file: tempfile::NamedTempFile,
    created_at: Timestamp,
    captured: bool,
    pending: Option<FrozenFile>,
    descriptor: Option<ExecutionOutput>,
    log: Option<LogPrefix>,
    production: Option<follow::Production>,
}

struct LogPrefix {
    acknowledged: u64,
    closed: Option<SavedStreamState>,
}

pub(super) struct FrozenFile {
    file: FrozenSource,
    metadata: FileMetadata,
    stream_state: Option<SavedStreamState>,
    creating: tokio::sync::Mutex<Option<BoxFuture<'static, Result<OpenedCapture>>>>,
    capture: Option<OpenedCapture>,
    session: Option<crate::storage::Manager>,
}

/* === Enums === */

enum FrozenSource {
    Snapshot(FrozenSnapshot),
    Uploaded(tokio::fs::File),
}

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
            Self::Snapshot => Ok((0, usize::try_from(size_bytes).map_err(error)?)),
        }
    }
}

impl NativeOutput {
    fn closed_state(&self) -> Option<SavedStreamState> {
        self.log.as_ref().and_then(|log| log.closed).or(self.captured.then_some(SavedStreamState::Finalized))
    }

    fn new(path: &str) -> Result<Self> {
        let suffix =
            std::path::Path::new(path).extension().and_then(|value| value.to_str()).map(|value| format!(".{value}")).unwrap_or_default();
        Ok(Self {
            file: tempfile::Builder::new().prefix("dokime-output-").suffix(&suffix).tempfile().map_err(error)?,
            created_at: crate::model::time::now_utc(),
            captured: false,
            pending: None,
            descriptor: None,
            log: None,
            production: None,
        })
    }

    fn visible_length(&self, physical: u64) -> Result<u64> {
        match &self.log {
            Some(log) if physical < log.acknowledged => Err(error("native log lost its acknowledged prefix")),
            Some(log) => Ok(log.acknowledged),
            None => Ok(physical),
        }
    }

    pub(super) fn descriptor(&self) -> Option<&ExecutionOutput> {
        self.descriptor.as_ref()
    }
}

impl ExecutionStorage {
    pub(crate) async fn native_target(
        self: &Arc<Self>, execution: TestExecutionId, relative_path: String, text_hint: bool, streamed: bool,
    ) -> Result<ExecutionFileTarget> {
        validate_producer_path(&relative_path)?;
        let storage = self.clone();
        tokio::spawn(async move {
            let owner = storage.acquisition(&execution).await.ok_or_else(|| error("execution has no open output acquisition"))?;
            let mut acquisition = owner.lock().await;
            if acquisition.finish_requested || acquisition.outputs_sealed {
                return Err(error("execution has no open output acquisition"));
            }
            if acquisition.native_outputs.contains_key(&relative_path) {
                return Err(error("duplicate execution output path"));
            }
            let output = NativeOutput::new(&relative_path)?;
            let absolute_path = output.file.path().to_string_lossy().into_owned();
            acquisition.native_outputs.insert(relative_path.clone(), output);
            Ok(ExecutionFileTarget { relative_path, absolute_path, text_hint, streamed })
        })
        .await
        .map_err(error)?
    }

    pub(crate) async fn native_entries(self: &Arc<Self>, execution: &TestExecutionId) -> Result<Vec<ExecutionFileEntry>> {
        let Some(owner) = self.acquisition(execution).await else {
            return Ok(Vec::new());
        };
        let acquisition = owner.lock_owned().await;
        let execution = execution.clone();
        tokio::task::spawn_blocking(move || {
            let mut entries = Vec::with_capacity(acquisition.native_outputs.len());
            for (path, output) in &acquisition.native_outputs {
                #[cfg(windows)]
                let _path_access = std::fs::File::open(output.file.path()).map_err(error)?;
                let file = output.file.reopen().map_err(error)?;
                let size_bytes = output.visible_length(file.metadata().map_err(error)?.len())?;
                let content_type = media_type_for_path(path).map(str::to_owned);
                entries.push(ExecutionFileEntry {
                    artifact_id: format!("{}:{path}", execution.as_str()),
                    relative_path: path.clone(),
                    content_kind: ExecutionFileContentKind::from_metadata(path, content_type.as_deref()),
                    content_type,
                    stream_state: output_state(output.closed_state()),
                    size_bytes,
                    updated_at: output.created_at.clone(),
                });
            }
            Ok(entries)
        })
        .await
        .map_err(|source| Error::Join { source })?
    }

    pub(crate) async fn native_read(
        self: &Arc<Self>, execution: &TestExecutionId, path: &str, offset: u64, limit: usize,
    ) -> Result<Option<harmos::storage::ReadBatch<ExecutionFileContent>>> {
        if limit == 0 {
            return Err(error("execution output reads require a nonempty range"));
        }
        self.owned_native_read(execution, path, FileSelection::Window { offset, limit }).await
    }

    async fn owned_native_read(
        self: &Arc<Self>, execution: &TestExecutionId, path: &str, selection: FileSelection,
    ) -> Result<Option<harmos::storage::ReadBatch<ExecutionFileContent>>> {
        let Some(owner) = self.acquisition(execution).await else {
            return Ok(None);
        };
        let acquisition = owner.lock().await;
        let Some(output) = acquisition.native_outputs.get(path) else {
            return Ok(None);
        };
        // Reopen The Original Inode While Its Registered Owner Is Protected
        #[cfg(windows)]
        let _path_access = std::fs::File::open(output.file.path()).map_err(error)?;
        let file = output.file.reopen().map_err(error)?;
        let size_bytes = output.visible_length(file.metadata().map_err(error)?.len())?;
        let stream_state = output_state(output.closed_state());
        drop(acquisition);
        let (offset, length) = selection.extent(size_bytes)?;
        let reader = reader::Reader::new(self.storage.clone(), file, path, size_bytes, stream_state)?;
        reader.into_window(offset, length).await.map(Some)
    }

    pub(crate) async fn native_snapshot(
        self: &Arc<Self>, execution: &TestExecutionId, path: &str,
    ) -> Result<Option<harmos::storage::ReadBatch<ExecutionFileContent>>> {
        self.owned_native_read(execution, path, FileSelection::Snapshot).await
    }

    pub(crate) async fn native_stream(self: &Arc<Self>, execution: &TestExecutionId, path: &str) -> Result<Option<reader::Reader>> {
        let Some(owner) = self.acquisition(execution).await else {
            return Ok(None);
        };
        let acquisition = owner.lock().await;
        let Some(output) = acquisition.native_outputs.get(path) else {
            return Ok(None);
        };
        #[cfg(windows)]
        let _path_access = std::fs::File::open(output.file.path()).map_err(error)?;
        let file = output.file.reopen().map_err(error)?;
        let observed_size = output.visible_length(file.metadata().map_err(error)?.len())?;
        let stream_state = output_state(output.closed_state());
        drop(acquisition);
        Ok(Some(reader::Reader::new(self.storage.clone(), file, path, observed_size, stream_state)?))
    }

    pub(crate) async fn retire_outputs(self: &Arc<Self>, execution: &TestExecutionId) -> Result<()> {
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

    pub(crate) async fn finish_outputs(self: &Arc<Self>, execution: &TestExecutionId) -> Result<()> {
        // Running Captures Seal Into The Execution While It Still Admits Artifacts
        self.captures.close(execution).await;
        let storage = self.clone();
        let execution = execution.clone();
        tokio::spawn(async move {
            let owner = storage.acquisition(&execution).await.ok_or_else(|| error("output finalization has no admitted acquisition"))?;
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
        let owner = self.acquisition(execution).await.ok_or_else(|| error("output finalization has no admitted acquisition"))?;
        owner.lock().await.outputs_sealed = true;
        Ok(())
    }
}

impl FrozenFile {
    pub(super) fn new(file: tokio::fs::File, metadata: FileMetadata) -> Self {
        Self::with_state(FrozenSource::Uploaded(file), metadata, None)
    }

    fn with_state(file: FrozenSource, metadata: FileMetadata, stream_state: Option<SavedStreamState>) -> Self {
        Self { file, metadata, stream_state, creating: tokio::sync::Mutex::new(None), capture: None, session: None }
    }

    pub(super) async fn finish(&mut self, storage: &Manager) -> Result<FileReference> {
        if self.capture.is_none() && self.creating.get_mut().is_none() {
            let session = self.session.get_or_insert_with(|| storage.clone()).clone();
            let manager = storage.clone();
            let metadata = self.metadata.clone();
            let file = match &self.file {
                FrozenSource::Snapshot(file) => FrozenSource::Snapshot(file.clone()),
                FrozenSource::Uploaded(file) => FrozenSource::Uploaded(file.try_clone().await.map_err(error)?),
            };
            let future = async move {
                let producer = manager.produce_file(metadata).await?;
                match file {
                    FrozenSource::Snapshot(file) => manager.capture_frozen_file(file, producer).map_err(Error::from),
                    FrozenSource::Uploaded(file) => Ok(session.capture_opened_file(
                        file.into_std().await,
                        producer,
                        ByteRange { offset: 0, limit: None },
                        CaptureConsistency::Stable,
                    )),
                }
            };
            *self.creating.get_mut() = Some(Box::pin(future));
        }
        if self.capture.is_none() {
            let outcome = self.creating.get_mut().as_mut().expect("original capture creation retained").await;
            *self.creating.get_mut() = None;
            self.capture = Some(outcome?);
        }
        // Harmos Retains The Opened Source, Exact Pending Chunk And Producer Across Retries.
        let started = Instant::now();
        let reference = self.capture.as_mut().expect("original capture retained").finish().await?;
        tracing::debug!(file_id = %self.metadata.id, retained_bytes = reference.length,
            elapsed_ms = started.elapsed().as_millis() as u64, "native snapshot capture settled");
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
        let stream_state = source.log.as_ref().and_then(|log| log.closed).or(finalizing.then_some(SavedStreamState::Finalized));
        if finalizing && source.production.is_some() {
            if let Err(failure) = follow::capture(path, source, stream_state).await {
                follow::interrupt(path, source).await?;
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
        let session = acquisition.storage.clone();
        let frozen = match freeze(&session, source).await {
            Ok(frozen) => frozen,
            Err(failure) if finalizing => {
                interrupt_source(&acquisition.storage, &acquisition.execution_id, path, source).await?;
                capture_error.get_or_insert(failure);
                continue;
            }
            Err(failure) => return Err(failure),
        };
        let digest = *frozen.sha256();
        if source.descriptor.as_ref().is_none_or(|previous| previous.file.sha256 != digest) {
            let metadata = file_metadata(&acquisition.execution_id, path, Some(frozen.length()), Some(digest))?;
            let mut pending = FrozenFile::with_state(FrozenSource::Snapshot(frozen), metadata, stream_state);
            pending.session = Some(session);
            source.pending = Some(pending);
            finish_snapshot(&acquisition.storage, path, source).await?;
        }
        store_descriptor(path, source, stream_state)?;
        source.captured = stream_state.is_some();
    }
    capture_error.map_or(Ok(()), Err)
}

pub(super) async fn abandon(acquisition: &mut Acquisition) -> Result<()> {
    for (path, source) in &mut acquisition.native_outputs {
        if !source.captured {
            interrupt_source(&acquisition.storage, &acquisition.execution_id, path, source).await?;
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

async fn finish_snapshot(storage: &Manager, path: &str, source: &mut NativeOutput) -> Result<()> {
    let pending = source.pending.as_mut().expect("original frozen snapshot retained");
    let reference = pending.finish(storage).await?;
    let descriptor = ExecutionOutput {
        relative_path: path.to_owned(),
        created_at: source.created_at.clone(),
        file: reference,
        stream_state: pending.stream_state,
        content: ExecutionOutputContent::File,
    };
    source.captured = pending.stream_state.is_some();
    source.descriptor = Some(descriptor);
    source.pending = None;
    Ok(())
}

fn file_metadata(execution: &TestExecutionId, path: &str, length: Option<u64>, digest: Option<[u8; 32]>) -> Result<FileMetadata> {
    let media_type = media_type_for_path(path).unwrap_or("application/octet-stream");
    let id_length = "output.native.".len() + uuid::fmt::Hyphenated::LENGTH;
    let mut id = String::new();
    id.try_reserve_exact(id_length).map_err(error)?;
    id.push_str("output.native.");
    let mut nonce = [0u8; uuid::fmt::Hyphenated::LENGTH];
    id.push_str(uuid::Uuid::now_v7().hyphenated().encode_lower(&mut nonce));
    Ok(FileMetadata {
        id: Identifier::new(id).map_err(error)?,
        scope: Some(Identifier::new(execution.as_str()).map_err(error)?),
        media_type: media_type.to_owned(),
        expected_length: length,
        expected_sha256: digest,
    })
}

async fn freeze(session: &crate::storage::Manager, source: &NativeOutput) -> Result<FrozenSnapshot> {
    let started = Instant::now();
    let file = open_file(source).await?.into_std().await;
    let limit = source.log.as_ref().map(|log| log.acknowledged);
    let frozen = session.freeze_opened_file(file, ByteRange { offset: 0, limit }).await?;
    tracing::debug!(source_bytes = frozen.length(), elapsed_ms = started.elapsed().as_millis() as u64, "native snapshot freeze settled");
    Ok(frozen)
}

fn store_descriptor(path: &str, source: &mut NativeOutput, state: Option<SavedStreamState>) -> Result<()> {
    if source.descriptor.as_ref().is_some_and(|descriptor| descriptor.stream_state == state) {
        return Ok(());
    }
    let reference = &source.descriptor.as_ref().ok_or_else(|| error("native output snapshot is missing"))?.file;
    source.descriptor = Some(ExecutionOutput {
        relative_path: path.to_owned(),
        created_at: source.created_at.clone(),
        file: reference.clone(),
        stream_state: state,
        content: ExecutionOutputContent::File,
    });
    Ok(())
}

async fn open_file(output: &NativeOutput) -> Result<tokio::fs::File> {
    // Windows can reopen the original handle after its pathname is deleted.
    // Require the producer-facing path to remain readable, as Unix reopen does.
    #[cfg(windows)]
    let _path_access = tokio::fs::File::open(output.file.path()).await.map_err(error)?;

    // Read the original file rather than a replacement at its pathname.
    let file = output.file.reopen().map_err(error)?;
    Ok(tokio::fs::File::from_std(file))
}

fn error(value: impl ToString) -> Error {
    Error::Operation(value.to_string())
}

async fn interrupt_source(storage: &Manager, execution: &TestExecutionId, path: &str, source: &mut NativeOutput) -> Result<()> {
    if source.production.is_some() {
        return follow::interrupt(path, source).await;
    }
    if source.pending.is_some() {
        finish_snapshot(storage, path, source).await?;
    }
    if source.descriptor.is_none() {
        let session = storage.clone();
        let file = tempfile::NamedTempFile::new().map_err(error)?.into_file();
        let empty = session.freeze_opened_file(file, ByteRange { offset: 0, limit: None }).await?;
        let metadata = file_metadata(execution, path, Some(0), Some(*empty.sha256()))?;
        let mut pending = FrozenFile::with_state(FrozenSource::Snapshot(empty), metadata, Some(SavedStreamState::Interrupted));
        pending.session = Some(session);
        source.pending = Some(pending);
        finish_snapshot(storage, path, source).await?;
    }
    store_descriptor(path, source, Some(SavedStreamState::Interrupted))?;
    source.captured = true;
    Ok(())
}

/* === Tests === */
