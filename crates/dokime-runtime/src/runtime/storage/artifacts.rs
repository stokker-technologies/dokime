/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub use super::artifact_reads::ArtifactReader;

use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

use harmos::storage::{ByteRange, FileChunk, FileMetadata, FileReadBatch};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use super::{
    execution::ExecutionView,
    executions::{Acquisition, ExecutionStorage},
    native_outputs::FrozenFile,
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

const UPLOAD_PREFIX: &str = "artifacts/uploads/";

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
    file: tokio::fs::File,
    closing: Option<ArtifactStatus>,
    pending: Option<FrozenFile>,
    descriptor: Option<ExecutionOutput>,
    metadata: Metadata,
    record: ArtifactRecord,
    expected_size: u64,
    expected_digest: Option<String>,
    digest: Sha256,
    owner: Weak<()>,
}

enum Bytes {
    Submitted(FileChunk),
    Read(FileReadBatch),
}

impl Bytes {
    fn as_slice(&self) -> &[u8] {
        match self {
            Self::Submitted(bytes) => bytes.as_slice(),
            Self::Read(batch) => batch.bytes(),
        }
    }
}

impl Upload {
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
            // Retain The Producer Token Through Admission
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
        operation.progress(sent, self.progress.expected);
        operation.track(appended)
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
            let stored = storage.find_artifact(&upload.id).await?.ok_or_else(|| error("artifact upload is missing"))?;
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
        super::artifact_inventory::records_at_in(&self.execution_storage, session, at).await
    }

    // Stores a finished local file as the artifact `name`, a relative path.
    pub async fn upload_artifact_file(
        &self, scope: dokime_domain::model::execution::RecordScope, path: impl AsRef<std::path::Path>, filename: String,
        content_type: Option<String>,
    ) -> Result<ArtifactRecord> {
        let path = path.as_ref();
        let file = tokio::fs::File::open(path).await.map_err(error)?;
        let size_bytes = file.metadata().await.map_err(error)?.len();
        let upload = self
            .begin_artifact_upload(ArtifactUploadInput {
                scope,
                filename,
                content_type,
                size_bytes,
                sha256: None,
            })
            .await?;

        let session = self.storage().clone();
        let source = file.try_clone().await.map_err(error)?.into_std().await;
        let opened = session.read_opened_file(source, ByteRange { offset: 0, limit: Some(size_bytes) }).await;
        let mut reader = opened.inspect_err(|error| tracing::warn!(path = %path.display(), %error, "artifact source read failed"))?;
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
        let observed_size = file.metadata().await.map_err(error)?.len();
        if completed.bytes != size_bytes || observed_size != size_bytes {
            return Err(error("artifact source length changed during upload"));
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

    pub async fn begin_artifact_upload(&self, mut input: ArtifactUploadInput) -> Result<ArtifactUpload> {
        let created_at = crate::model::time::now_utc();
        input.scope = self.capture_publication_scope(input.scope).await?;
        self.begin_captured_artifact_upload(input, created_at).await
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
        let filename = crate::exports::storage::paths::safe_artifact_path(&input.filename);
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
            content_type: input.content_type,
            size_bytes: 0,
            sha256: None,
            status: ArtifactStatus::Staged,
            created_at: Timestamp::from_datetime(metadata.created_at.expect("runtime artifact creation time")),
        };
        let progress = Arc::new(UploadProgress {
            operation,
            sent: AtomicU64::new(0),
            expected: input.size_bytes,
        });
        let state = Upload {
            file: tokio::fs::File::from_std(tempfile::NamedTempFile::new().map_err(error)?.into_file()),
            closing: None,
            pending: None,
            descriptor: None,
            metadata,
            record,
            expected_size: input.size_bytes,
            expected_digest: input.sha256.map(|value| value.to_ascii_lowercase()),
            digest: Sha256::new(),
            owner: Arc::downgrade(&owner),
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

    pub(crate) async fn artifact_count(&self) -> Result<usize> {
        super::artifact_inventory::count(self).await
    }

    pub(crate) async fn artifact_reader(&self, id: &ArtifactId) -> Result<Option<ArtifactReader>> {
        let session = self.storage.clone();
        let mut found = None;
        super::artifact_inventory::inspect(self, |_, output| {
            if matches!(&output.content, ExecutionOutputContent::Artifact { record, .. } if &record.artifact_id == id) {
                let descriptor = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, harmos::Error>(output.clone()))?;
                found = Some(descriptor);
            }
            Ok(())
        })
        .await?;
        match found {
            Some(output) => Ok(Some(ArtifactReader::from_output(session, output).await?)),
            None => Ok(None),
        }
    }

    async fn find_artifact(&self, id: &ArtifactId) -> Result<Option<ArtifactRecord>> {
        let mut found = None;
        super::artifact_inventory::inspect(self, |_, output| {
            let ExecutionOutputContent::Artifact { record, .. } = &output.content else {
                unreachable!("inventory visits only artifact descriptors");
            };
            if &record.artifact_id == id {
                found = Some(record.as_ref().clone());
            }
            Ok(())
        })
        .await?;
        Ok(found)
    }
}

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

async fn append(upload: &mut Upload, chunk: Chunk) -> Result<()> {
    use std::io::{Seek, Write};

    if upload.closing.is_some() || upload.record.status != ArtifactStatus::Staged {
        return Err(error("artifact upload is closed"));
    }
    let size = upload.record.size_bytes.checked_add(chunk.bytes.as_slice().len() as u64).ok_or_else(|| error("artifact size overflow"))?;
    if size > upload.expected_size {
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
    Ok(())
}

fn outcome(record: ArtifactRecord, complete: bool) -> Result<ArtifactRecord> {
    if complete && record.status != ArtifactStatus::Complete {
        return Err(error("artifact was cancelled or failed size/hash validation"));
    }
    Ok(record)
}

async fn close(upload: &mut Upload, complete: bool) -> Result<()> {
    if upload.closing.is_some() {
        return Ok(());
    }
    let digest = upload.digest.clone().finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let valid =
        upload.record.size_bytes == upload.expected_size && upload.expected_digest.as_ref().is_none_or(|expected| expected == &digest);
    let status = if complete && valid {
        ArtifactStatus::Complete
    } else if complete {
        ArtifactStatus::Rejected
    } else {
        ArtifactStatus::Cancelled
    };
    // Freeze The Decision Before Native Production; Later Waits Cannot Rewrite It
    upload.closing = Some(status);
    upload.record.sha256 = Some(digest);
    Ok(())
}

async fn stage(storage: &crate::storage::Manager, upload: &mut Upload) -> Result<()> {
    let Some(status) = upload.closing.as_ref() else {
        // A Mutable Upload Is Not A Saved Artifact Descriptor
        return Ok(());
    };
    if upload.descriptor.is_some() {
        return Ok(());
    }
    if upload.pending.is_none() {
        // The Private File Is Frozen By The Execution Gate And Closing Decision
        upload.file.set_len(upload.record.size_bytes).await.map_err(error)?;
        upload.file.flush().await.map_err(error)?;
        let file = upload.file.try_clone().await.map_err(error)?;
        let media_type = upload.record.content_type.as_deref().unwrap_or("application/octet-stream");
        let metadata = FileMetadata {
            id: Identifier::new(upload.record.artifact_id.as_str()).map_err(error)?,
            scope: upload.record.scope.test_execution_id.as_ref().map(|id| Identifier::new(id.as_str())).transpose().map_err(error)?,
            media_type: media_type.to_owned(),
            expected_length: Some(upload.record.size_bytes),
            expected_sha256: Some(upload.digest.clone().finalize().into()),
        };
        upload.pending = Some(FrozenFile::new(file, metadata));
    }
    // Name The Artifact Whose Capture Failed; The Storage Error Carries No Path.
    let finished = upload.pending.as_mut().expect("original artifact producer retained").finish(storage).await;
    let artifact = upload.record.artifact_id.as_str();
    let reference =
        finished.inspect_err(|error| tracing::warn!(artifact, filename = %upload.record.filename, %error, "artifact capture failed"))?;
    let mut record = upload.record.clone();
    record.status = status.clone();
    let relative_path = format!("{UPLOAD_PREFIX}{}/{}", record.artifact_id.as_str(), record.filename);
    let descriptor = ExecutionOutput {
        relative_path,
        created_at: record.created_at.clone(),
        file: reference,
        stream_state: Some(if *status == ArtifactStatus::Complete { SavedStreamState::Finalized } else { SavedStreamState::Interrupted }),
        content: ExecutionOutputContent::Artifact {
            record: Box::new(record),
            metadata: upload.metadata.clone(),
        },
    };
    // A Refused Descriptor Retains The Original Finished Handle For Retry
    upload.record.status = status.clone();
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

fn error(problem: impl ToString) -> Error {
    Error::Operation(format!("artifact storage: {}", problem.to_string()))
}
