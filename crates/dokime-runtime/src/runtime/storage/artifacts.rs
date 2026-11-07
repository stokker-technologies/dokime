/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

pub use super::artifact_reads::ArtifactReader;

/* === Imports === */

// Standard Library Imports
use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

// Third Party Imports
use harmos::storage::{ByteRange, FileChunk, FileMetadata, FileReadBatch};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

// Local Imports
use super::{
    execution::ExecutionView,
    executions::{Acquisition, ExecutionStorage},
    native_outputs::{FrozenFile, NativeOutput},
};
use crate::artifacts::ArtifactRecord;
use crate::artifacts::ArtifactStatus;
use crate::artifacts::ArtifactUploadInput;
use crate::{RuntimeHandle, runtime::Error, runtime::Result, runtime::progress::Operation};
use dokime_domain::model::execution::saved::ExecutionCheckpoint as Checkpoint;
use dokime_domain::model::execution::saved::ExecutionOutput;
use dokime_domain::model::execution::saved::ExecutionOutputContent;
use dokime_domain::model::execution::saved::SavedStreamState;
use dokime_domain::model::identity::ArtifactId;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::identity::UploadId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct ArtifactUpload {
    storage: Weak<ExecutionStorage>,
    execution: TestExecutionId,
    id: ArtifactId,
    progress: Arc<UploadProgress>,
    _owner: Arc<()>,
}

struct UploadProgress {
    operation: Operation,
    sent: AtomicU64,
    expected: u64,
}

struct Chunk {
    upload: ArtifactUpload,
    bytes: Bytes,
}

pub(super) struct Upload {
    pub(super) output: NativeOutput,
    pub(super) relative_path: String,
    file: tokio::fs::File,
    closing: Option<ArtifactStatus>,
    pending: Option<FrozenFile>,
    // Only The Message Is Kept: Holding A Runtime Error Here Makes `Upload` Recursive For Send Checks.
    capture_error: Option<String>,
    descriptor: Option<ExecutionOutput>,
    metadata: Metadata,
    record: ArtifactRecord,
    expected_size: Option<u64>,
    expected_digest: Option<String>,
    digest: Sha256,
    owner: Weak<()>,
}

/* === Enums === */

enum Bytes {
    Submitted(FileChunk),
    Read(FileReadBatch),
}

/* === Traits === */

/* === Implementations === */

impl Bytes {
    fn as_slice(&self) -> &[u8] {
        match self {
            Self::Submitted(bytes) => bytes.as_slice(),
            Self::Read(batch) => batch.bytes(),
        }
    }
}

impl Upload {
    pub(super) fn artifact_id(&self) -> &ArtifactId {
        &self.record.artifact_id
    }
    pub(super) fn descriptor(&self) -> Option<&ExecutionOutput> {
        self.descriptor.as_ref()
    }
}

impl ArtifactUpload {
    pub fn artifact_id(&self) -> &ArtifactId {
        &self.id
    }

    pub async fn send(&self, bytes: impl Into<FileChunk>) -> Result<()> {
        let bytes = Bytes::Submitted(bytes.into());
        self.send_chunk(Chunk { upload: self.clone(), bytes }).await
    }

    async fn send_chunk(&self, chunk: Chunk) -> Result<()> {
        let storage = self.storage.upgrade().ok_or(Error::ShuttingDown)?;
        let count = chunk.bytes.as_slice().len() as u64;
        let appended = tokio::spawn(async move {
            // Retain the Producer Token Through Admission
            let owner = storage.acquisition(&chunk.upload.execution).await.ok_or_else(|| error("execution acquisition is closed"))?;
            let mut acquisition = owner.lock().await;
            let state = acquisition.artifacts.get_mut(&chunk.upload.id).ok_or_else(|| error("artifact upload is missing"))?;
            append(state, chunk).await
        })
        .await
        .map_err(error)
        .and_then(|appended| appended);
        let sent = self.progress.sent.fetch_add(count, Ordering::Relaxed) + count;
        let operation = &self.progress.operation;
        if appended.is_ok() {
            operation.phase(format!("Receiving {sent} B"));
        }
        operation.progress(sent, self.progress.expected);
        operation.track(appended)
    }

    pub(crate) async fn send_file_range(&self, path: &std::path::Path, length: u64) -> Result<()> {
        let storage = self.storage.upgrade().ok_or(Error::ShuttingDown)?;
        let owner = storage.acquisition(&self.execution).await.ok_or(Error::ShuttingDown)?;
        let offset = owner.lock().await.artifacts.get(&self.id).ok_or(Error::ShuttingDown)?.record.size_bytes;
        let mut remaining = length.checked_sub(offset).ok_or_else(|| error("artifact spool is shorter than its saved prefix"))?;
        if remaining == 0 {
            return Ok(());
        }
        let mut file = tokio::fs::File::open(path).await.map_err(source_error)?;
        file.seek(std::io::SeekFrom::Start(offset)).await.map_err(source_error)?;
        while remaining > 0 {
            // Producer Buffers Are Bounded Independently Of Viewer Read Admission.
            let mut bytes = vec![0; remaining.min(64 * 1024) as usize];
            let count = file.read(&mut bytes).await.map_err(source_error)?;
            if count == 0 {
                return Err(error("artifact spool range was not fully read"));
            }
            bytes.truncate(count);
            self.send(bytes).await?;
            remaining -= count as u64;
        }
        Ok(())
    }

    pub async fn finish(&self) -> Result<ArtifactRecord> {
        self.close_with(true, None).await
    }
    pub async fn abandon(&self) -> Result<ArtifactRecord> {
        self.close_with(false, None).await
    }

    pub async fn finish_verified(&self, sha256: Option<String>) -> Result<ArtifactRecord> {
        if let Some(digest) = &sha256 {
            validate_digest(digest)?;
        }
        self.close_with(true, sha256).await
    }

    async fn close_with(&self, complete: bool, sha256: Option<String>) -> Result<ArtifactRecord> {
        let operation = &self.progress.operation;
        operation.phase(if complete { "Saving" } else { "Abandoning" });
        operation.indeterminate();
        let closed = self.close_once(complete, sha256).await;
        match &closed {
            Ok(record) if record.status == ArtifactStatus::Failed => {
                operation.fail(record.error.as_deref().unwrap_or("artifact capture failed"))
            }
            Ok(_) if complete => operation.finish(),
            Ok(_) => operation.fail("upload abandoned"),
            Err(error) => operation.fail(error),
        }
        closed
    }

    async fn close_once(&self, complete: bool, sha256: Option<String>) -> Result<ArtifactRecord> {
        let storage = self.storage.upgrade().ok_or(Error::ShuttingDown)?;
        let upload = self.clone();
        tokio::spawn(async move {
            if let Some(owner) = storage.acquisition(&upload.execution).await {
                let mut acquisition = owner.lock().await;
                let state = acquisition.artifacts.get_mut(&upload.id).ok_or_else(|| error("artifact upload is missing"))?;
                if state.closing.is_none()
                    && let Some(value) = sha256
                {
                    state.expected_digest = Some(value.to_ascii_lowercase());
                }
                close(state, complete).await?;
                save(&mut acquisition, &upload.id).await?;
                let record = acquisition.artifacts[&upload.id].record.clone();
                drop(acquisition);
                // Native Finish Already Confirms Immutable File Durability.
                // Save The Logical Descriptors Together At The Execution Boundary.
                return outcome(record, complete);
            }
            // A successful publication can retire its process-local owner.
            // Exact finish retries resolve the persisted identity instead.
            let stored = storage.find_artifact(&upload.execution, &upload.id).await?.ok_or_else(|| error("artifact upload is missing"))?;
            outcome(stored, complete)
        })
        .await
        .map_err(error)?
    }
}

impl RuntimeHandle {
    pub(crate) async fn artifacts_at_in(
        &self, session: &crate::storage::Manager, at: &harmos::storage::StorageCursor,
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<Vec<ArtifactRecord>>> {
        super::artifact_inventory::records_at_in(&self.execution_storage, session, at, None).await
    }

    // Stores a finished local file as the artifact `name`, a relative path.
    #[tracing::instrument(name = "storage.artifacts.upload_file", level = "debug", skip_all)]
    pub async fn upload_artifact_file(
        &self, scope: dokime_domain::model::execution::RecordScope, path: impl AsRef<std::path::Path>, filename: String,
        content_type: Option<String>,
    ) -> Result<ArtifactRecord> {
        let path = path.as_ref();
        let opened: Result<_> = async {
            let file = tokio::fs::File::open(path).await.map_err(source_error)?;
            let size_bytes = file.metadata().await.map_err(source_error)?.len();
            Ok((file, size_bytes))
        }
        .await;
        let size_bytes = opened.as_ref().map_or(0, |(_, size)| *size);
        let upload = self
            .begin_artifact_upload(ArtifactUploadInput {
                scope,
                filename,
                content_type,
                size_bytes,
                sha256: None,
            })
            .await?;

        let copied: Result<()> = async {
            let (file, _) = opened?;
            let session = self.storage().clone();
            let source = file.try_clone().await.map_err(source_error)?.into_std().await;
            let opened = session.read_opened_file(source, ByteRange { offset: 0, limit: Some(size_bytes) }).await;
            let mut reader = opened?;
            while let Some(batch) = reader.next().await? {
                // The Native Page Owner Survives The Spawned Append And Blocking Write.
                upload
                    .send_chunk(Chunk {
                        upload: upload.clone(),
                        bytes: Bytes::Read(batch),
                    })
                    .await?;
            }

            let completed = reader.completion()?;
            let observed_size = file.metadata().await.map_err(source_error)?.len();
            if completed.bytes != size_bytes || observed_size != size_bytes {
                return Err(harmos::storage::Error::InvalidInput {
                    field: "capture source",
                    reason: "artifact source length changed during upload",
                }
                .into());
            }
            Ok(())
        }
        .await;
        if let Err(source) = copied {
            if !content_capture_failure(&source) {
                return Err(source);
            }
            let storage = upload.storage.upgrade().ok_or(Error::ShuttingDown)?;
            let owner = storage.acquisition(&upload.execution).await.ok_or_else(|| error("execution acquisition is closed"))?;
            let mut acquisition = owner.lock().await;
            let state = acquisition.artifacts.get_mut(&upload.id).ok_or_else(|| error("artifact upload is missing"))?;
            state.capture_error = Some(source.to_string());
            drop(acquisition);
            return upload.close_with(false, None).await;
        }
        upload.finish().await
    }

    pub async fn open_artifact_stream(&self, id: &ArtifactId) -> Result<Option<ArtifactReader>> {
        self.execution_storage.artifact_reader(id).await
    }
    pub(crate) async fn capture_publication_scope(
        &self, scope: dokime_domain::model::execution::RecordScope,
    ) -> Result<dokime_domain::model::execution::RecordScope> {
        let _order = self.telemetry_admission.lock().await;
        self.state_manager.state().read(move |state| state.capture_scope(&scope)).await.map_err(error)?.value.map_err(error)
    }

    #[tracing::instrument(name = "storage.artifacts.begin_upload", level = "debug", skip_all)]
    pub async fn begin_artifact_upload(&self, mut input: ArtifactUploadInput) -> Result<ArtifactUpload> {
        let created_at = crate::model::time::now_utc();
        input.scope = self.capture_publication_scope(input.scope).await?;
        self.begin_captured_artifact_upload(input, created_at).await
    }

    pub(crate) async fn begin_streamed_artifact(&self, input: ArtifactUploadInput) -> Result<ArtifactUpload> {
        let upload = self.begin_artifact_upload(input).await?;
        let owner = self.execution_storage.acquisition(&upload.execution).await.ok_or(Error::ShuttingDown)?;
        owner.lock().await.artifacts.get_mut(&upload.id).ok_or(Error::ShuttingDown)?.expected_size = None;
        upload.progress.operation.phase("Capturing 0 B");
        Ok(upload)
    }

    pub(crate) async fn begin_captured_artifact_upload(&self, input: ArtifactUploadInput, created_at: Timestamp) -> Result<ArtifactUpload> {
        validate_input(&input)?;
        let id = input.scope.test_execution_id.clone().ok_or_else(|| error("artifact execution scope is required"))?;
        let (cycle, execution) = {
            let state = self.state_read();
            let execution = state.executions.get(&id).ok_or_else(|| error("artifact execution not found"))?;
            let cycle = state.cycle_view(&execution.test_cycle_id).ok_or_else(|| error("artifact cycle not found"))?;
            if input.scope.test_cycle_id.as_ref() != Some(&cycle.test_cycle_id)
                || input.scope.test_case_id.as_ref().is_some_and(|case| case != &execution.test_case_id)
            {
                return Err(error("artifact scope does not match its execution"));
            }
            (cycle.clone(), execution.clone())
        };
        let operation = self.operation_progress().begin(format!("Storing {}", input.filename));
        operation.phase("Receiving");
        let storage = self.execution_storage.clone();
        let progress = operation.clone();
        let begun = tokio::spawn(async move {
            storage.prepare_execution(&cycle, &execution).await?;
            storage.begin_artifact(id, input, created_at, progress).await
        })
        .await
        .map_err(error)
        .and_then(|begun| begun);
        operation.track(begun)
    }

    #[cfg(any(test, feature = "fault-injection", feature = "testing"))]
    pub fn inject_artifact_capture_failure(&self, execution: TestExecutionId, filename: String, kind: std::io::ErrorKind) {
        self.execution_storage.artifact_faults.lock().unwrap().insert((execution, filename), kind);
    }

    pub async fn finish_execution_for_test(&self, id: &TestExecutionId) -> Result<()> {
        let (cycle, execution) = {
            let state = self.state_read();
            let execution = state.executions.get(id).ok_or_else(|| error("artifact execution not found"))?;
            let cycle = state.cycle_view(&execution.test_cycle_id).ok_or_else(|| error("artifact cycle not found"))?;
            (cycle.clone(), execution.clone())
        };
        self.finish_saved_execution(&cycle, &execution).await
    }
    pub async fn checkpoint_artifact(&self, checkpoint: Checkpoint, id: &ArtifactId) -> Result<ArtifactReader> {
        checkpoint.validate().map_err(crate::storage::Error::from)?;
        let inspection = ExecutionView::open_checkpoint(&self.execution_storage.storage, &checkpoint)
            .await?
            .ok_or_else(|| error("artifact checkpoint is missing from its saved view"))?;
        ArtifactReader::open(&inspection, id).await
    }
}

impl ExecutionStorage {
    async fn begin_artifact(
        self: Arc<Self>, execution: TestExecutionId, input: ArtifactUploadInput, created_at: Timestamp, operation: Operation,
    ) -> Result<ArtifactUpload> {
        let acquisition_owner = self.acquisition(&execution).await.ok_or_else(|| error("execution has no open artifact acquisition"))?;
        let mut acquisition = acquisition_owner.lock().await;
        if acquisition.finish_requested
            || acquisition.outputs_sealed
            || acquisition.outputs_quiescent
            || acquisition.execution.saved.is_some()
        {
            return Err(Error::conflict("execution_sealed", "execution no longer accepts artifacts"));
        }
        let id = crate::model::identity::new_id::<ArtifactId>();
        let owner = Arc::new(());
        let requested = input.filename;
        let content_type = input.content_type;
        let prefix = if crate::artifacts::media_kind_for(&requested, content_type.as_deref()).is_some() { "media/" } else { "artifacts/" };
        let mut filename = requested.clone();
        let mut suffix = 2;
        while (prefix == "artifacts/" && filename == "execution.log") || acquisition.output(&format!("{prefix}{filename}")).is_some() {
            filename = crate::artifacts::name::collision_name(&requested, suffix);
            suffix += 1;
        }
        crate::artifacts::name::validate(&filename).map_err(error)?;
        let relative_path = format!("{prefix}{filename}");
        let output = NativeOutput::artifact(FileMetadata {
            id: Identifier::new(id.as_str()).map_err(error)?,
            scope: Some(Identifier::new(execution.as_str()).map_err(error)?),
            media_type: content_type.as_deref().unwrap_or("application/octet-stream").to_owned(),
            expected_length: None,
            expected_sha256: None,
        })?;
        let file = tokio::fs::File::from_std(output.file.reopen().map_err(error)?);
        let metadata = {
            let mut metadata =
                Metadata::definition(Identifier::new(id.as_str()).map_err(error)?, &filename, None, std::iter::empty::<String>());
            metadata.created_at = Some(created_at.to_datetime().map_err(error)?);
            metadata
        };
        let record = ArtifactRecord {
            artifact_id: id.clone(),
            upload_id: crate::model::identity::new_id::<UploadId>(),
            scope: input.scope,
            filename,
            content_type,
            size_bytes: 0,
            sha256: None,
            status: ArtifactStatus::Staged,
            error: None,
            created_at: Timestamp::from_datetime(metadata.created_at.expect("runtime artifact creation time")),
        };
        let progress = Arc::new(UploadProgress {
            operation,
            sent: AtomicU64::new(0),
            expected: input.size_bytes,
        });
        let state = Upload {
            file,
            output,
            relative_path,
            closing: None,
            pending: None,
            capture_error: None,
            descriptor: None,
            metadata,
            record,
            expected_size: Some(input.size_bytes),
            expected_digest: input.sha256.map(|value| value.to_ascii_lowercase()),
            digest: Sha256::new(),
            owner: Arc::downgrade(&owner),
        };
        #[cfg(any(test, feature = "fault-injection", feature = "testing"))]
        let state = {
            let mut state = state;
            if let Some(kind) = self.artifact_faults.lock().unwrap().remove(&(execution.clone(), state.record.filename.clone())) {
                state.capture_error = Some(source_error(std::io::Error::from(kind)).to_string());
            }
            state
        };
        acquisition.artifacts.insert(id.clone(), state);
        drop(acquisition);
        Ok(ArtifactUpload {
            storage: Arc::downgrade(&self),
            execution,
            id,
            progress,
            _owner: owner,
        })
    }

    pub(crate) async fn artifact_records(&self) -> Result<Vec<ArtifactRecord>> {
        super::artifact_inventory::records(self).await
    }

    pub(crate) async fn selected_artifact_records(
        &self, session: &crate::storage::Manager, at: &harmos::storage::StorageCursor, executions: &[TestExecutionId],
    ) -> Result<harmos::storage::ReadBatch<Vec<ArtifactRecord>>> {
        super::artifact_inventory::records_at_in(self, session, at, Some(executions)).await
    }

    pub(crate) async fn artifact_count(&self) -> Result<usize> {
        super::artifact_inventory::count(self).await
    }

    pub(crate) async fn artifact_reader(&self, id: &ArtifactId) -> Result<Option<ArtifactReader>> {
        let owners = self.active.lock().await.values().cloned().collect::<Vec<_>>();
        let mut found = None;
        for owner in owners {
            let acquisition = owner.lock().await;
            if let Some(upload) = acquisition.artifacts.get(id) {
                found = upload.descriptor().cloned();
                break;
            }
        }
        if found.is_none()
            && let Some(owner) = self.local.artifact_owner(id.to_string()).await?
        {
            let owner = TestExecutionId::from_uuid(owner).map_err(crate::storage::Error::from)?;
            if let Some(view) = ExecutionView::open(&self.storage, &owner, harmos::storage::ObjectVersion::Latest).await? {
                found = view
                    .execution()
                    .saved
                    .as_ref()
                    .and_then(|seal| {
                        seal.source.outputs.iter().find(|output| {
                    matches!(&output.content, ExecutionOutputContent::Artifact { record, .. } if &record.artifact_id == id)
                })
                    })
                    .cloned();
            }
        }
        match found {
            Some(output) => {
                if let Some(file) = &output.file {
                    self.storage.flush_until(&file.position).await?;
                }
                Ok(Some(ArtifactReader::from_output(self.storage.clone(), harmos::storage::ReadBatch::new(output)).await?))
            }
            None => Ok(None),
        }
    }

    async fn find_artifact(&self, owner: &TestExecutionId, id: &ArtifactId) -> Result<Option<ArtifactRecord>> {
        let records = self.selected_artifact_records(&self.storage, &self.storage.cursor_saved(), std::slice::from_ref(owner)).await?;
        Ok(records.iter().find(|record| &record.artifact_id == id).cloned())
    }
}

/* === Public Functions === */

pub(super) async fn abandon_dropped(acquisition: &mut Acquisition) -> Result<()> {
    for upload in acquisition.artifacts.values_mut() {
        if upload.record.status == ArtifactStatus::Staged && upload.owner.upgrade().is_none() {
            close(upload, false).await?;
        }
    }
    checkpoint(acquisition).await
}

pub(super) async fn close_incomplete(acquisition: &mut Acquisition) -> Result<()> {
    for upload in acquisition.artifacts.values_mut() {
        close(upload, false).await?;
    }
    checkpoint(acquisition).await
}

pub(super) async fn checkpoint(acquisition: &mut Acquisition) -> Result<()> {
    for upload in acquisition.artifacts.values_mut() {
        stage(&acquisition.storage, upload).await?;
    }
    Ok(())
}

pub(super) fn has_live_upload(acquisition: &Acquisition) -> bool {
    acquisition.artifacts.values().any(|upload| upload.closing.is_none() && upload.owner.upgrade().is_some())
}

/* === Private Functions === */

async fn append(upload: &mut Upload, chunk: Chunk) -> Result<()> {
    use std::io::{Seek, Write};

    if upload.closing.is_some() || upload.record.status != ArtifactStatus::Staged {
        return Err(error("artifact upload is closed"));
    }
    let size = upload.record.size_bytes.checked_add(chunk.bytes.as_slice().len() as u64).ok_or_else(|| error("artifact size overflow"))?;
    if upload.expected_size.is_some_and(|expected| size > expected) {
        return Err(error("artifact exceeds declared size"));
    }

    let acknowledged = upload.record.size_bytes;
    let mut file = upload.file.try_clone().await.map_err(error)?.into_std().await;
    let chunk = tokio::task::spawn_blocking(move || {
        // Discard Only An Unacknowledged Private Write Prefix Before Retrying.
        // The Acquisition Lock Serializes This Original Handle's Shared Offset.
        file.set_len(acknowledged).map_err(error)?;
        file.seek(std::io::SeekFrom::Start(acknowledged)).map_err(error)?;
        file.write_all(chunk.bytes.as_slice()).map_err(error)?;
        Ok::<_, Error>(chunk)
    })
    .await
    .map_err(error)??;

    // Acknowledge Only The Completed Write; Errors Preserve The Prior Digest And Offset.
    upload.digest.update(chunk.bytes.as_slice());
    upload.record.size_bytes = size;
    upload.output.acknowledge(size);
    Ok(())
}

fn outcome(record: ArtifactRecord, complete: bool) -> Result<ArtifactRecord> {
    if complete && !matches!(record.status, ArtifactStatus::Complete | ArtifactStatus::Failed) {
        return Err(error("artifact was cancelled or failed size/hash validation"));
    }
    Ok(record)
}

async fn close(upload: &mut Upload, complete: bool) -> Result<()> {
    if upload.closing.is_some() {
        return Ok(());
    }
    let digest = upload.digest.clone().finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let valid = upload.expected_size.is_none_or(|expected| upload.record.size_bytes == expected)
        && upload.expected_digest.as_ref().is_none_or(|expected| expected == &digest);
    let status = if complete && valid {
        ArtifactStatus::Complete
    } else if complete {
        ArtifactStatus::Rejected
    } else {
        ArtifactStatus::Cancelled
    };
    // Freeze The Decision Before Native Production; Later Waits Cannot Rewrite It
    upload.output.close_artifact(status == ArtifactStatus::Complete);
    upload.closing = Some(status);
    upload.record.sha256 = Some(digest);
    Ok(())
}

async fn stage(storage: &crate::storage::Manager, upload: &mut Upload) -> Result<()> {
    let Some(status) = upload.closing.as_ref() else {
        // A Mutable Upload Is Not a Saved Artifact Descriptor
        return Ok(());
    };
    if upload.descriptor.is_some() {
        return Ok(());
    }
    if upload.capture_error.is_some() && upload.output.has_production() {
        super::native_outputs::follow::interrupt(&upload.relative_path, &mut upload.output).await?;
    }
    let (reference, status) = if let Some(message) = upload.capture_error.take() {
        tracing::warn!(artifact = %upload.record.artifact_id, filename = %upload.record.filename, error = %message, "artifact capture failed");
        upload.record.error = Some(message);
        (None, ArtifactStatus::Failed)
    } else {
        let captured = async {
            if upload.output.has_production() {
                let state = if *status == ArtifactStatus::Complete { SavedStreamState::Finalized } else { SavedStreamState::Interrupted };
                super::native_outputs::follow::capture(&upload.relative_path, &mut upload.output, Some(state)).await?;
                return Ok(upload
                    .output
                    .descriptor()
                    .expect("captured artifact output")
                    .file()
                    .map_err(crate::storage::Error::from)?
                    .clone());
            }
            if upload.pending.is_none() {
                // The Private File Is Frozen by the Execution Gate and Closing Decision
                upload.file.set_len(upload.record.size_bytes).await.map_err(source_error)?;
                upload.file.flush().await.map_err(source_error)?;
                let file = upload.file.try_clone().await.map_err(source_error)?;
                let metadata = FileMetadata {
                    id: Identifier::new(upload.record.artifact_id.as_str()).map_err(error)?,
                    scope: upload
                        .record
                        .scope
                        .test_execution_id
                        .as_ref()
                        .map(|id| Identifier::new(id.as_str()))
                        .transpose()
                        .map_err(error)?,
                    media_type: upload.record.content_type.as_deref().unwrap_or("application/octet-stream").to_owned(),
                    expected_length: Some(upload.record.size_bytes),
                    expected_sha256: Some(upload.digest.clone().finalize().into()),
                };
                upload.pending = Some(FrozenFile::new(file, metadata));
            }
            upload.pending.as_mut().expect("original artifact producer retained").finish(storage).await
        }
        .await;
        match captured {
            Ok(reference) => (Some(reference), status.clone()),
            Err(source) if content_capture_failure(&source) => {
                tracing::warn!(artifact = %upload.record.artifact_id, filename = %upload.record.filename, error = %source, "artifact capture failed");
                upload.record.error = Some(source.to_string());
                (None, ArtifactStatus::Failed)
            }
            Err(source) => return Err(source),
        }
    };
    let mut record = upload.record.clone();
    record.status = status.clone();
    let relative_path = upload.relative_path.clone();
    let descriptor = ExecutionOutput {
        relative_path,
        created_at: record.created_at.clone(),
        file: reference,
        stream_state: Some(if status == ArtifactStatus::Complete { SavedStreamState::Finalized } else { SavedStreamState::Interrupted }),
        content: ExecutionOutputContent::Artifact {
            record: Box::new(record),
            metadata: upload.metadata.clone(),
        },
    };
    // The Descriptor Owns The Capture Outcome; Report And Provider Retries Never Recapture It
    upload.record.status = status.clone();
    upload.output.settle_artifact(&descriptor);
    upload.descriptor = Some(descriptor);
    upload.pending = None;
    Ok(())
}

async fn save(acquisition: &mut Acquisition, id: &ArtifactId) -> Result<()> {
    stage(&acquisition.storage, acquisition.artifacts.get_mut(id).unwrap()).await
}

fn validate_input(input: &ArtifactUploadInput) -> Result<()> {
    crate::artifacts::name::validate(&input.filename).map_err(|source| error(format!("invalid artifact name: {source}")))?;
    if input.content_type.as_ref().is_some_and(|value| value.is_empty() || value.len() > 255) || input.size_bytes > i64::MAX as u64 {
        return Err(error("invalid artifact media type or size"));
    }
    if let Some(digest) = &input.sha256 {
        validate_digest(digest)?;
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(error("artifact sha256 must contain 64 hexadecimal characters"));
    }
    Ok(())
}

fn source_error(source: std::io::Error) -> Error {
    harmos::storage::Error::backend("artifact source", harmos::storage::BackendFailure::Permanent, source).into()
}

fn content_capture_failure(source: &Error) -> bool {
    // These Native Errors Belong To Reading The Capture Source, Never To Publishing Stored Files Or Descriptors.
    matches!(
        source,
        Error::HarmosStorage(harmos::storage::Error::Backend {
            operation: "local filesystem" | "artifact source",
            ..
        }) | Error::HarmosStorage(harmos::storage::Error::InvalidInput {
            field: "opened file" | "capture source" | "local file",
            ..
        })
    )
}

fn error(problem: impl ToString) -> Error {
    Error::Operation(format!("artifact storage: {}", problem.to_string()))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn execution_log_keeps_plain_name_before_and_after_step_uploads() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!("../../report/fixtures/execution-report-input.json")).unwrap();
        let mut cycle: dokime_domain::model::execution::result::CycleView =
            serde_json::from_value(fixture["input"]["cycle"].clone()).unwrap();
        cycle.test_executions[0].saved = None;
        let execution = &cycle.test_executions[0];
        let root = tempfile::tempdir().unwrap();
        let mut owner = crate::Runtime::start(crate::RuntimeConfig::new(root.path(), cycle.metadata.clone())).await.unwrap();
        let runtime = owner.handle();
        let storage = &runtime.execution_storage;
        storage.acquire(&cycle, execution).await.unwrap();
        let id = &execution.test_execution_id;
        for expected in ["artifacts/execution.2.log", "artifacts/execution.3.log"] {
            let upload = storage
                .clone()
                .begin_artifact(
                    id.clone(),
                    ArtifactUploadInput {
                        scope: execution.scope.clone(),
                        filename: "execution.log".into(),
                        content_type: Some("text/plain".into()),
                        size_bytes: 1,
                        sha256: None,
                    },
                    crate::model::time::now_utc(),
                    runtime.operation_progress().begin("Artifact Collision"),
                )
                .await
                .unwrap();
            let entries = storage.native_entries(id).await.unwrap();
            assert!(entries.iter().any(|entry| entry.artifact_id == upload.artifact_id().as_str() && entry.relative_path == expected));
            storage.reserve_output(id, "artifacts/execution.log").await.unwrap();
            drop(upload);
        }
        drop(runtime);
        owner.shutdown().await.unwrap();
    }

    #[test]
    fn capture_failure_degrades_only_source_content_errors() {
        let denied = || std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        assert!(content_capture_failure(&source_error(denied())));
        let source = harmos::storage::Error::backend("local filesystem", harmos::storage::BackendFailure::Permanent, denied()).into();
        assert!(content_capture_failure(&source));
        let catalog = harmos::storage::Error::backend("write files", harmos::storage::BackendFailure::Permanent, denied()).into();
        assert!(!content_capture_failure(&catalog));
        assert!(!content_capture_failure(&Error::ShuttingDown));
    }
}
