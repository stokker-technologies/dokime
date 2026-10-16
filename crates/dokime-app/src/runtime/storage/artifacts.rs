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
use std::{
    num::NonZeroUsize,
    sync::{
        Arc, Weak,
        atomic::{AtomicU64, Ordering},
    },
};
// Third Party Imports
use harmos::{
    MemorySize,
    storage::{FileMetadata, QueryValue},
};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
// Local Imports
use super::{
    execution::ExecutionView,
    executions::{Acquisition, ExecutionStorage},
    native_outputs::FrozenFile,
};
use crate::{Error, Result, RuntimeHandle, runtime::progress::Operation};
use dokime::{
    artifacts::{ArtifactRecord, ArtifactStatus, ArtifactUploadInput},
    core::{ArtifactId, Identifier, Metadata, TestExecutionId, Timestamp, UploadId},
    execution::saved::{
        ExecutionCheckpoint as Checkpoint, ExecutionOutput, ExecutionOutputContent,
        SavedStreamState,
    },
};

/* === Constants === */
const CHUNK_BYTES: usize = 1024 * 1024;
const EXECUTION_BYTES: usize = 4 * 1024 * 1024;
const DESCRIPTOR_BYTES: usize = 64 * 1024;
const UPLOAD_PREFIX: &str = "artifacts/uploads/";

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

/// How far an upload has come against the size it declared.
struct UploadProgress {
    operation: Operation,
    sent: AtomicU64,
    expected: u64,
}

struct Chunk {
    upload: ArtifactUpload,
    bytes: Vec<u8>,
}

pub(super) struct Upload {
    file: tokio::fs::File,
    closing: Option<ArtifactStatus>,
    pending: Option<FrozenFile>,
    descriptor: Option<QueryValue<ExecutionOutput>>,
    metadata: Metadata,
    record: ArtifactRecord,
    expected_size: u64,
    expected_digest: Option<String>,
    digest: Sha256,
    owner: Weak<()>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Upload {
    pub(super) fn descriptor(&self) -> Option<&ExecutionOutput> {
        self.descriptor.as_deref()
    }
}

impl MemorySize for Chunk {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(
                self.upload
                    .execution
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<TestExecutionId>()),
            )
            .saturating_add(
                self.upload
                    .id
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<ArtifactId>()),
            )
            .saturating_add(self.bytes.capacity())
    }
}

impl ArtifactUpload {
    pub fn artifact_id(&self) -> &ArtifactId {
        &self.id
    }

    pub async fn send(&self, bytes: &[u8]) -> Result<()> {
        if bytes.len() > CHUNK_BYTES {
            return Err(error("artifact chunks must not exceed 1 MiB"));
        }
        let storage = self.storage.upgrade().ok_or(Error::ShuttingDown)?;

        // Preclaim Owned Ingress Before Copying
        let allowance = std::mem::size_of::<Chunk>()
            .checked_add(self.execution.as_str().len())
            .and_then(|value| value.checked_add(self.id.as_str().len()))
            .and_then(|value| value.checked_add(bytes.len()))
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| error("artifact ingress size overflow"))?;
        let chunk = storage
            .storage
            .produce(allowance, |_allowance| {
                Ok(Chunk {
                    upload: self.clone(),
                    bytes: bytes.to_vec(),
                })
            })
            .map_err(Error::from)?;

        let appended = tokio::spawn(async move {
            // Retain The Original Charge And Producer Token Through Admission
            let ingress = chunk.value();
            let upload = &ingress.upload;
            let owner = storage
                .acquisition(&upload.execution)
                .await
                .ok_or_else(|| error("execution acquisition is closed"))?;
            let mut acquisition = owner.lock().await;
            let state = acquisition
                .artifacts
                .get_mut(&upload.id)
                .ok_or_else(|| error("artifact upload is missing"))?;
            append(state, &ingress.bytes).await
        })
        .await
        .map_err(error)
        .and_then(|appended| appended);
        let sent = self
            .progress
            .sent
            .fetch_add(bytes.len() as u64, Ordering::Relaxed)
            + bytes.len() as u64;
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
                let state = acquisition
                    .artifacts
                    .get_mut(&upload.id)
                    .ok_or_else(|| error("artifact upload is missing"))?;
                if state.closing.is_none()
                    && let Some(value) = sha256
                {
                    state.expected_digest = Some(value.to_ascii_lowercase());
                }
                close(state, complete).await?;
                save(&mut acquisition, &upload.id).await?;
                let record = acquisition.artifacts[&upload.id].record.clone();
                drop(acquisition);
                storage.storage.flush().await?;
                return outcome(record, complete);
            }
            // A successful publication can retire its process-local owner.
            // Exact finish retries resolve the persisted identity instead.
            let stored = storage
                .find_artifact(&upload.id)
                .await?
                .ok_or_else(|| error("artifact upload is missing"))?;
            outcome((*stored).clone(), complete)
        })
        .await
        .map_err(error)?
    }
}

impl RuntimeHandle {
    // Stores a finished local file as the artifact `name`, a relative path.
    pub async fn upload_artifact_file(
        &self,
        scope: dokime::core::RecordScope,
        path: impl AsRef<std::path::Path>,
        filename: String,
        content_type: Option<String>,
    ) -> Result<ArtifactRecord> {
        use tokio::io::AsyncReadExt;
        let path = path.as_ref();
        let mut file = tokio::fs::File::open(path).await.map_err(error)?;
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
        let mut buffer = vec![0; 64 * 1024];
        loop {
            let count = file.read(&mut buffer).await.map_err(error)?;
            if count == 0 {
                break;
            }
            upload.send(&buffer[..count]).await?;
        }
        upload.finish().await
    }

    pub async fn open_artifact_stream(&self, id: &ArtifactId) -> Result<Option<ArtifactReader>> {
        self.execution_storage.artifact_reader(id).await
    }
    pub(crate) async fn capture_publication_scope(
        &self,
        scope: dokime::core::RecordScope,
    ) -> Result<dokime::core::RecordScope> {
        let _order = self.telemetry_admission.lock().await;
        self.state_manager
            .state()
            .read(move |state| state.capture_scope(&scope))
            .await
            .map_err(error)?
            .value
            .map_err(error)
    }

    pub async fn begin_artifact_upload(
        &self,
        mut input: ArtifactUploadInput,
    ) -> Result<ArtifactUpload> {
        let created_at = Timestamp::now_utc();
        input.scope = self.capture_publication_scope(input.scope).await?;
        self.begin_captured_artifact_upload(input, created_at).await
    }

    pub(crate) async fn begin_captured_artifact_upload(
        &self,
        input: ArtifactUploadInput,
        created_at: Timestamp,
    ) -> Result<ArtifactUpload> {
        validate_input(&input)?;
        let id = input
            .scope
            .test_execution_id
            .clone()
            .ok_or_else(|| error("artifact execution scope is required"))?;
        let (cycle, execution) = {
            let state = self.state_read();
            let execution = state
                .executions
                .get(&id)
                .ok_or_else(|| error("artifact execution not found"))?;
            let cycle = state
                .cycle_view(&execution.test_cycle_id)
                .ok_or_else(|| error("artifact cycle not found"))?;
            if input.scope.test_cycle_id.as_ref() != Some(&cycle.test_cycle_id)
                || input
                    .scope
                    .test_case_id
                    .as_ref()
                    .is_some_and(|case| case != &execution.test_case_id)
            {
                return Err(error("artifact scope does not match its execution"));
            }
            (cycle.clone(), execution.clone())
        };
        let operation = self
            .operation_progress()
            .begin(format!("Storing {}", input.filename));
        operation.phase("Receiving");
        let storage = self.execution_storage.clone();
        let progress = operation.clone();
        let begun = tokio::spawn(async move {
            storage.prepare_execution(&cycle, &execution).await?;
            storage
                .begin_artifact(id, input, created_at, progress)
                .await
        })
        .await
        .map_err(error)
        .and_then(|begun| begun);
        operation.track(begun)
    }

    pub async fn finish_execution_for_test(&self, id: &TestExecutionId) -> Result<()> {
        let (cycle, execution) = {
            let state = self.state_read();
            let execution = state
                .executions
                .get(id)
                .ok_or_else(|| error("artifact execution not found"))?;
            let cycle = state
                .cycle_view(&execution.test_cycle_id)
                .ok_or_else(|| error("artifact cycle not found"))?;
            (cycle.clone(), execution.clone())
        };
        self.finish_saved_execution(&cycle, &execution).await
    }
    pub async fn checkpoint_artifact(
        &self,
        checkpoint: Checkpoint,
        id: &ArtifactId,
    ) -> Result<ArtifactReader> {
        checkpoint
            .validate()
            .map_err(dokime::storage::Error::from)?;
        let storage = &self.execution_storage.storage;
        let view = storage
            .view_with_limits(self.config().storage.interactive)
            .await?;
        let inspection = ExecutionView::open_checkpoint(
            storage,
            view,
            &checkpoint,
            NonZeroUsize::new(EXECUTION_BYTES).unwrap(),
        )
        .await?
        .ok_or_else(|| error("artifact checkpoint is missing from its saved view"))?;
        ArtifactReader::open(
            &inspection,
            id,
            NonZeroUsize::new(DESCRIPTOR_BYTES).unwrap(),
        )
        .await
    }
}

impl ExecutionStorage {
    async fn begin_artifact(
        self: Arc<Self>,
        execution: TestExecutionId,
        input: ArtifactUploadInput,
        created_at: Timestamp,
        operation: Operation,
    ) -> Result<ArtifactUpload> {
        let acquisition_owner = self
            .acquisition(&execution)
            .await
            .ok_or_else(|| error("execution has no open artifact acquisition"))?;
        let mut acquisition = acquisition_owner.lock().await;
        if acquisition.finish_requested
            || acquisition.outputs_sealed
            || acquisition.outputs_quiescent
            || acquisition.execution.saved.is_some()
        {
            return Err(Error::conflict(
                "execution_sealed",
                "execution no longer accepts artifacts",
            ));
        }
        let id = ArtifactId::new();
        let owner = Arc::new(());
        let filename = crate::exports::storage::paths::safe_artifact_path(&input.filename);
        let metadata = {
            let mut metadata = Metadata::definition(
                Identifier::new(id.as_str()).map_err(error)?,
                &filename,
                None,
                std::iter::empty::<String>(),
            );
            metadata.created_at = Some(created_at.to_datetime().map_err(error)?);
            metadata
        };
        let record = ArtifactRecord {
            artifact_id: id.clone(),
            upload_id: UploadId::new(),
            scope: input.scope,
            filename,
            content_type: input.content_type,
            size_bytes: 0,
            sha256: None,
            status: ArtifactStatus::Staged,
            created_at: Timestamp::from_datetime(
                metadata.created_at.expect("runtime artifact creation time"),
            ),
        };
        let progress = Arc::new(UploadProgress {
            operation,
            sent: AtomicU64::new(0),
            expected: input.size_bytes,
        });
        let state = Upload {
            file: tokio::fs::File::from_std(tempfile::tempfile().map_err(error)?),
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

    pub(crate) async fn artifact_records(
        &self,
        scan_limit: usize,
    ) -> Result<QueryValue<Vec<ArtifactRecord>>> {
        super::artifact_inventory::project_records(self, scan_limit, |records, _| Ok(records)).await
    }

    pub(crate) async fn project_artifact_records<T: MemorySize>(
        &self,
        scan_limit: usize,
        map: impl FnOnce(Vec<ArtifactRecord>, NonZeroUsize) -> harmos::Result<T>,
    ) -> Result<QueryValue<T>> {
        super::artifact_inventory::project_records(self, scan_limit, map).await
    }

    pub(crate) async fn project_artifacts<T: MemorySize>(
        &self,
        scan_limit: usize,
        map: impl FnOnce(&[ArtifactRecord], NonZeroUsize) -> harmos::Result<T>,
    ) -> Result<QueryValue<T>> {
        self.project_artifact_records(scan_limit, |records, allowance| map(&records, allowance))
            .await
    }

    /// Counts every artifact under `budget`, or the interactive default.
    pub(crate) async fn artifact_count(
        &self,
        budget: Option<harmos::storage::QueryLimits>,
    ) -> Result<usize> {
        super::artifact_inventory::count(self, budget).await
    }

    pub(crate) async fn artifact_reader(&self, id: &ArtifactId) -> Result<Option<ArtifactReader>> {
        let mut found = None;
        super::artifact_inventory::inspect(self, |_, output| {
            if matches!(&output.content, ExecutionOutputContent::Artifact { record, .. } if &record.artifact_id == id) {
                let bytes = NonZeroUsize::new(output.memory_size()).ok_or(harmos::Error::ResourceExhausted { resource: "artifact descriptor memory" })?;
                found = Some(self.storage.produce(bytes, |_| Ok(output.clone()))?.try_into_value()?);
            }
            Ok(())
        }).await?;
        match found {
            Some(output) => Ok(Some(
                ArtifactReader::from_output(&self.storage, output).await?,
            )),
            None => Ok(None),
        }
    }

    async fn find_artifact(&self, id: &ArtifactId) -> Result<Option<QueryValue<ArtifactRecord>>> {
        let mut found = None;
        super::artifact_inventory::inspect(self, |_, output| {
            let ExecutionOutputContent::Artifact { record, .. } = &output.content else {
                unreachable!("inventory visits only artifact descriptors");
            };
            if &record.artifact_id == id {
                let limit = NonZeroUsize::new(DESCRIPTOR_BYTES).unwrap();
                found = Some(
                    self.storage
                        .produce(limit, |allowance| {
                            if record.memory_size() > allowance.get() {
                                return Err(harmos::Error::ResourceExhausted {
                                    resource: "artifact record projection",
                                });
                            }
                            Ok(record.as_ref().clone())
                        })?
                        .try_into_value()?,
                );
            }
            Ok(())
        })
        .await?;
        Ok(found)
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
    acquisition
        .artifacts
        .values()
        .any(|upload| upload.closing.is_none() && upload.owner.upgrade().is_some())
}

/* === Private Functions === */
async fn append(upload: &mut Upload, bytes: &[u8]) -> Result<()> {
    if upload.closing.is_some() || upload.record.status != ArtifactStatus::Staged {
        return Err(error("artifact upload is closed"));
    }
    let size = upload
        .record
        .size_bytes
        .checked_add(bytes.len() as u64)
        .ok_or_else(|| error("artifact size overflow"))?;
    if size > upload.expected_size {
        return Err(error("artifact exceeds declared size"));
    }
    // Discard Only An Unacknowledged Private Write Prefix Before Retrying
    upload
        .file
        .set_len(upload.record.size_bytes)
        .await
        .map_err(error)?;
    upload
        .file
        .seek(std::io::SeekFrom::Start(upload.record.size_bytes))
        .await
        .map_err(error)?;
    upload.file.write_all(bytes).await.map_err(error)?;
    upload.digest.update(bytes);
    upload.record.size_bytes = size;
    Ok(())
}

fn outcome(record: ArtifactRecord, complete: bool) -> Result<ArtifactRecord> {
    if complete && record.status != ArtifactStatus::Complete {
        return Err(error(
            "artifact was cancelled or failed size/hash validation",
        ));
    }
    Ok(record)
}

async fn close(upload: &mut Upload, complete: bool) -> Result<()> {
    if upload.closing.is_some() {
        return Ok(());
    }
    let digest = upload
        .digest
        .clone()
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let valid = upload.record.size_bytes == upload.expected_size
        && upload
            .expected_digest
            .as_ref()
            .is_none_or(|expected| expected == &digest);
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

async fn stage(storage: &dokime::storage::Manager, upload: &mut Upload) -> Result<()> {
    let Some(status) = upload.closing.as_ref() else {
        // A Mutable Upload Is Not A Saved Artifact Descriptor
        return Ok(());
    };
    if upload.descriptor.is_some() {
        return Ok(());
    }
    if upload.pending.is_none() {
        // The Private File Is Frozen By The Execution Gate And Closing Decision
        upload
            .file
            .set_len(upload.record.size_bytes)
            .await
            .map_err(error)?;
        upload.file.flush().await.map_err(error)?;
        let file = upload.file.try_clone().await.map_err(error)?;
        let media_type = upload
            .record
            .content_type
            .as_deref()
            .unwrap_or("application/octet-stream");
        let bytes = std::mem::size_of::<FileMetadata>()
            .checked_add(upload.record.artifact_id.as_str().len())
            .and_then(|bytes| {
                bytes.checked_add(
                    upload
                        .record
                        .scope
                        .test_execution_id
                        .as_ref()
                        .map_or(0, |id| id.as_str().len()),
                )
            })
            .and_then(|bytes| bytes.checked_add(media_type.len()))
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| error("artifact file metadata memory overflows"))?;
        let metadata = storage
            .produce(bytes, |_| {
                Ok(FileMetadata {
                    id: Identifier::new(upload.record.artifact_id.as_str()).map_err(|source| {
                        harmos::Error::backend(
                            "artifact file identity",
                            harmos::BackendFailure::Permanent,
                            source,
                        )
                    })?,
                    scope: upload
                        .record
                        .scope
                        .test_execution_id
                        .as_ref()
                        .map(|id| Identifier::new(id.as_str()))
                        .transpose()
                        .map_err(|source| {
                            harmos::Error::backend(
                                "artifact file scope",
                                harmos::BackendFailure::Permanent,
                                source,
                            )
                        })?,
                    media_type: media_type.to_owned(),
                    expected_length: Some(upload.record.size_bytes),
                    expected_sha256: Some(upload.digest.clone().finalize().into()),
                })
            })?
            .try_into_value()?;
        upload.pending = Some(FrozenFile::new(file, metadata));
    }
    let reference = upload
        .pending
        .as_mut()
        .expect("original artifact producer retained")
        .finish(storage)
        .await?;
    let bytes = std::mem::size_of::<ExecutionOutput>()
        .checked_add(UPLOAD_PREFIX.len())
        .and_then(|bytes| bytes.checked_add(upload.record.artifact_id.as_str().len()))
        .and_then(|bytes| bytes.checked_add(1))
        .and_then(|bytes| bytes.checked_add(upload.record.filename.len()))
        .and_then(|bytes| {
            bytes.checked_add(
                upload
                    .record
                    .created_at
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<Timestamp>()),
            )
        })
        .and_then(|bytes| {
            bytes.checked_add(
                reference
                    .memory_size()
                    .saturating_sub(std::mem::size_of_val(&reference)),
            )
        })
        .and_then(|bytes| {
            bytes.checked_add(
                upload
                    .record
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<Box<ArtifactRecord>>()),
            )
        })
        .and_then(|bytes| {
            bytes.checked_add(
                upload
                    .metadata
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<Metadata>()),
            )
        })
        .and_then(NonZeroUsize::new)
        .ok_or_else(|| error("artifact descriptor memory overflows"))?;
    let descriptor = storage
        .produce(bytes, |_| {
            let mut record = upload.record.clone();
            record.status = status.clone();
            let length =
                UPLOAD_PREFIX.len() + record.artifact_id.as_str().len() + 1 + record.filename.len();
            let mut relative_path = String::new();
            relative_path.try_reserve_exact(length).map_err(|_| {
                harmos::Error::ResourceExhausted {
                    resource: "artifact output path",
                }
            })?;
            if relative_path.capacity() > length {
                return Err(harmos::Error::ResourceExhausted {
                    resource: "artifact output path capacity",
                });
            }
            relative_path.push_str(UPLOAD_PREFIX);
            relative_path.push_str(record.artifact_id.as_str());
            relative_path.push('/');
            relative_path.push_str(&record.filename);
            Ok(ExecutionOutput {
                relative_path,
                created_at: record.created_at.clone(),
                file: reference.clone(),
                stream_state: Some(if *status == ArtifactStatus::Complete {
                    SavedStreamState::Finalized
                } else {
                    SavedStreamState::Interrupted
                }),
                content: ExecutionOutputContent::Artifact {
                    record: Box::new(record),
                    metadata: upload.metadata.clone(),
                },
            })
        })?
        .try_into_value()?;
    // A Refused Descriptor Retains The Original Finished Handle For Retry
    upload.record.status = status.clone();
    upload.descriptor = Some(descriptor);
    upload.pending = None;
    Ok(())
}

async fn save(acquisition: &mut Acquisition, id: &ArtifactId) -> Result<()> {
    stage(
        &acquisition.storage,
        acquisition.artifacts.get_mut(id).unwrap(),
    )
    .await
}

fn validate_input(input: &ArtifactUploadInput) -> Result<()> {
    dokime::artifacts::name::validate(&input.filename)
        .map_err(|source| error(format!("invalid artifact name: {source}")))?;
    if input
        .content_type
        .as_ref()
        .is_some_and(|value| value.is_empty() || value.len() > 255)
        || input.size_bytes > i64::MAX as u64
    {
        return Err(error("invalid artifact media type or size"));
    }
    if let Some(digest) = &input.sha256 {
        validate_digest(digest)?;
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(error(
            "artifact sha256 must contain 64 hexadecimal characters",
        ));
    }
    Ok(())
}

fn error(problem: impl ToString) -> Error {
    Error::Operation(format!("artifact storage: {}", problem.to_string()))
}

/* === Tests === */
