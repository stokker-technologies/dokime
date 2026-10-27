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

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, VecDeque},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::Arc,
};

// Third Party Imports
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

// Local Imports
use super::{ExecutionFixtureContext, failure};
use crate::artifacts::ArtifactUploadInput;
use crate::records::RecordSeverity;
use crate::transport::{ArtifactChunk, MAX_ARTIFACT_BYTES, MAX_ARTIFACT_CHUNK_BYTES, ProviderExecutionFailure};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::records::publication::RecordPublishRequest;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct ArtifactBridge {
    state: Arc<Mutex<Transfers>>,
}

#[derive(Debug)]
pub(super) struct DiscardedArtifact {
    pub(super) transfer_id: String,
    pub(super) name: String,
    pub(super) received: u64,
}

struct Transfers<S: SpillStorage = DiskSpill> {
    failure: Option<ProviderExecutionFailure>,
    transfers: BTreeMap<String, Transfer<S>>,
    received_bytes: u64,
    accepted_chunks: usize,
    transfer_count: usize,
    incomplete_transfers: usize,
    storage: S,
}

struct DiskSpill {
    spill_root: PathBuf,
    spill: Option<tempfile::TempDir>,
}

struct TransferBody<S: SpillStorage> {
    file: Option<S::File>,
}

struct Transfer<S: SpillStorage> {
    capture_scope: Option<RecordScope>,
    created_at: Option<dokime_domain::model::time::Timestamp>,
    name: String,
    mime: String,
    next_sequence: u64,
    received: u64,
    body: TransferBody<S>,
    digest: Sha256,
    chunks: VecDeque<ChunkFingerprint>,
    fingerprint_base: u64,
    state: TransferState,
}

struct ChunkFingerprint {
    digest: [u8; 32],
    bytes: usize,
    last: bool,
    aborted: bool,
    replace: bool,
}

/* === Enums === */

#[derive(Clone, Copy, PartialEq, Eq)]
enum TransferState {
    Open,
    Complete,
    Stored,
    Aborted,
}

/* === Traits === */

trait SpillStorage {
    type File: Read + Write + Seek;

    fn prepare(&mut self) -> std::io::Result<()>;
    fn create(&self) -> std::io::Result<Self::File>;
    fn truncate(file: &mut Self::File) -> std::io::Result<()>;
    fn release(&mut self);
}

/* === Implementations === */

impl ArtifactBridge {
    pub(super) fn new(spill_root: PathBuf) -> Self {
        Self { state: Arc::new(Mutex::new(Transfers::new(spill_root))) }
    }

    pub(super) async fn accept(
        &mut self, context: &ExecutionFixtureContext, scope: &RecordScope, chunks: Vec<ArtifactChunk>,
    ) -> Result<Vec<DiscardedArtifact>, ProviderExecutionFailure> {
        let mut state = self.state.clone().lock_owned().await;
        let context = context.clone();
        let scope = scope.clone();
        tokio::spawn(async move {
            if let Some(error) = &state.failure {
                return Err(error.clone());
            }
            let result = state.accept(&context, &scope, &chunks).await;
            if let Err(error) = &result {
                state.failure = Some(error.clone());
            }
            result
        })
        .await
        .map_err(|error| artifact_failure(error.to_string()))?
    }

    pub(super) async fn finish(&self) -> Result<Vec<DiscardedArtifact>, ProviderExecutionFailure> {
        self.state.lock().await.finish()
    }
}

impl SpillStorage for DiskSpill {
    type File = tempfile::NamedTempFile;

    fn prepare(&mut self) -> std::io::Result<()> {
        if self.spill.is_none() {
            self.spill = Some(tempfile::Builder::new().prefix("dokime-sidecar-artifact-").tempdir_in(&self.spill_root)?);
        }
        Ok(())
    }

    fn create(&self) -> std::io::Result<Self::File> {
        tempfile::NamedTempFile::new_in(self.spill.as_ref().expect("prepared spill directory").path())
    }

    fn truncate(file: &mut Self::File) -> std::io::Result<()> {
        file.as_file_mut().set_len(0)
    }

    fn release(&mut self) {
        self.spill = None;
    }
}

impl Transfers {
    fn new(spill_root: PathBuf) -> Self {
        Self::with_storage(DiskSpill { spill_root, spill: None })
    }

    async fn accept(
        &mut self, context: &ExecutionFixtureContext, scope: &RecordScope, chunks: &[ArtifactChunk],
    ) -> Result<Vec<DiscardedArtifact>, ProviderExecutionFailure> {
        let mut abandoned = Vec::new();
        for chunk in chunks {
            let replay = self.transfers.get(&chunk.transfer_id).is_some_and(|transfer| chunk.sequence < transfer.next_sequence);
            let captured = if self.transfers.contains_key(&chunk.transfer_id) {
                None
            } else {
                let created_at = crate::model::time::now_utc();
                let scope = context.capture_publication_scope(scope.clone()).await.map_err(|error| artifact_failure(error.to_string()))?;
                Some((scope, created_at))
            };
            abandoned.extend(self.ingest(std::slice::from_ref(chunk))?);
            if let Some((scope, created_at)) = captured {
                let transfer = self.transfers.get_mut(&chunk.transfer_id).expect("accepted transfer");
                transfer.capture_scope = Some(scope);
                transfer.created_at = Some(created_at);
            }
            if replay || !chunk.last || chunk.aborted {
                continue;
            }
            self.store(context, &chunk.transfer_id).await?;
        }
        Ok(abandoned)
    }

    async fn store(&mut self, context: &ExecutionFixtureContext, transfer_id: &str) -> Result<(), ProviderExecutionFailure> {
        let transfer = self.transfers.get_mut(transfer_id).expect("a terminal chunk belongs to a transfer this bridge accepted");
        // The digest was already computed incrementally as bytes were spilled
        // to disk, so verifying a two-gibibyte upload costs no second pass
        // over content already read once to write it. Passing it through
        // gives end-to-end verification the host did not previously have.
        let scope = transfer.capture_scope.clone().ok_or_else(|| artifact_failure("artifact publication has no captured scope"))?;
        let created_at = transfer.created_at.clone().ok_or_else(|| artifact_failure("artifact publication has no arrival time"))?;
        let upload = context
            .begin_captured_artifact_upload(
                ArtifactUploadInput {
                    scope: scope.clone(),
                    filename: transfer.name.clone(),
                    content_type: Some(transfer.mime.clone()),
                    size_bytes: transfer.received,
                    sha256: Some(transfer.digest()),
                },
                created_at,
            )
            .await
            .map_err(|error| artifact_failure(error.to_string()))?;
        // Read the spilled content back one bounded chunk at a time: peak
        // host memory for this transfer is one chunk, on the way in and on
        // the way back out.
        transfer.body.start_reading().map_err(|error| artifact_failure(error.to_string()))?;
        let mut buffer = vec![0u8; MAX_ARTIFACT_CHUNK_BYTES];
        loop {
            let read = transfer.body.read_next(&mut buffer).map_err(|error| artifact_failure(error.to_string()))?;
            if read == 0 {
                break;
            }
            let mut bytes = std::mem::take(&mut buffer);
            bytes.truncate(read);
            upload.send(bytes).await.map_err(|error| artifact_failure(error.to_string()))?;
            buffer.resize(MAX_ARTIFACT_CHUNK_BYTES, 0);
        }
        let record = upload.finish().await.map_err(|error| artifact_failure(error.to_string()))?;
        let mut created = RecordPublishRequest::new(crate::records::BuiltinRecordType::ArtifactCreated.identifier(), RecordSeverity::Info);
        created.scope = Some(scope.clone());
        created.related_artifact_id = Some(record.artifact_id.clone());
        created.message = Some(format!("Stored execution artifact {}", record.filename));
        created.details = BTreeMap::from([
            ("artifact_id".to_owned(), Value::from(record.artifact_id.as_str())),
            ("filename".to_owned(), Value::from(record.filename.as_str())),
            ("size_bytes".to_owned(), Value::from(record.size_bytes as i64)),
            ("transfer_id".to_owned(), Value::from(transfer_id)),
        ]);
        context.publish_record(created).await.map_err(|error| {
            artifact_failure(format!("stored sidecar artifact `{transfer_id}` but could not publish its creation event: {}", error))
        })?;
        self.mark_uploaded(transfer_id);
        Ok(())
    }
}

impl<S: SpillStorage> Transfers<S> {
    fn with_storage(storage: S) -> Self {
        Self {
            failure: None,
            transfers: BTreeMap::new(),
            received_bytes: 0,
            accepted_chunks: 0,
            transfer_count: 0,
            incomplete_transfers: 0,
            storage,
        }
    }

    fn finish(&mut self) -> Result<Vec<DiscardedArtifact>, ProviderExecutionFailure> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        let mut discarded = Vec::new();
        for (transfer_id, transfer) in
            self.transfers.iter_mut().filter(|(_, transfer)| matches!(transfer.state, TransferState::Open | TransferState::Complete))
        {
            discarded.push(DiscardedArtifact {
                transfer_id: transfer_id.clone(),
                name: transfer.name.clone(),
                received: transfer.received,
            });
            transfer.body.release();
        }
        // Every transfer has been stored, aborted, or discarded above: the
        // execution's spill directory has nothing left to hold, so it is
        // reclaimed now rather than waiting for `Transfers` itself to drop.
        self.storage.release();
        Ok(discarded)
    }

    fn ingest(&mut self, chunks: &[ArtifactChunk]) -> Result<Vec<DiscardedArtifact>, ProviderExecutionFailure> {
        let mut abandoned = Vec::new();
        for chunk in chunks {
            validate_chunk(chunk)?;
            let replay = self.transfers.get(&chunk.transfer_id).is_some_and(|transfer| chunk.sequence < transfer.next_sequence);
            let admitted = if replay { 0 } else { chunk.bytes.len() as u64 };
            if !replay && self.accepted_chunks as u64 >= 262_144 {
                return Err(artifact_failure("execution exceeded its artifact chunk limit"));
            }
            if self.received_bytes.saturating_add(admitted) > 8 * 1024 * 1024 * 1024 {
                return Err(artifact_failure("sidecar execution exceeded its configured artifact byte limit"));
            }
            self.storage.prepare().map_err(|error| artifact_failure(format!("could not prepare the artifact spill directory: {error}")))?;
            match self.transfers.get_mut(&chunk.transfer_id) {
                Some(transfer) => {
                    let was_open = transfer.state == TransferState::Open;
                    transfer.ingest(&self.storage, chunk, 1024)?;
                    if was_open && transfer.state != TransferState::Open {
                        self.incomplete_transfers -= 1;
                    }
                }
                None => {
                    if chunk.sequence != 0 {
                        return Err(artifact_failure(format!(
                            "artifact `{}` started at chunk {} instead of zero",
                            chunk.transfer_id, chunk.sequence
                        )));
                    }
                    self.reserve_new_transfer(chunk)?;
                    let mut transfer = Transfer::new(chunk);
                    if let Err(error) = transfer.ingest(&self.storage, chunk, 1024) {
                        self.release_new_transfer(chunk);
                        return Err(error);
                    }
                    self.transfers.insert(chunk.transfer_id.clone(), transfer);
                }
            }
            if !replay {
                self.accepted_chunks += 1;
                self.received_bytes += admitted;
            }
            if !replay && chunk.aborted {
                abandoned.push(self.release_aborted(&chunk.transfer_id));
            }
        }
        Ok(abandoned)
    }

    fn release_aborted(&mut self, transfer_id: &str) -> DiscardedArtifact {
        let transfer = self.transfers.get(transfer_id).expect("an aborting chunk belongs to a transfer this bridge accepted");
        let received = transfer.received;
        self.received_bytes = self.received_bytes.saturating_sub(received);
        DiscardedArtifact { transfer_id: transfer_id.to_owned(), name: transfer.name.clone(), received }
    }

    fn reserve_new_transfer(&mut self, chunk: &ArtifactChunk) -> Result<(), ProviderExecutionFailure> {
        if self.transfer_count as u64 >= 32 {
            return Err(artifact_failure("sidecar execution exceeded its configured artifact transfer limit"));
        }
        if !chunk.last && self.incomplete_transfers as u64 >= 16 {
            return Err(artifact_failure("sidecar execution exceeded its configured incomplete artifact limit"));
        }
        self.transfer_count += 1;
        if !chunk.last {
            self.incomplete_transfers += 1;
        }
        Ok(())
    }

    fn release_new_transfer(&mut self, chunk: &ArtifactChunk) {
        self.transfer_count -= 1;
        if !chunk.last {
            self.incomplete_transfers -= 1;
        }
    }

    fn mark_uploaded(&mut self, transfer_id: &str) {
        if let Some(transfer) = self.transfers.get_mut(transfer_id) {
            transfer.state = TransferState::Stored;
            transfer.body.release();
        }
    }
}

impl<S: SpillStorage> TransferBody<S> {
    fn empty() -> Self {
        Self { file: None }
    }

    fn write_all(&mut self, storage: &S, bytes: &[u8]) -> std::io::Result<()> {
        let file = match &mut self.file {
            Some(file) => file,
            None => self.file.insert(storage.create()?),
        };
        file.write_all(bytes)
    }

    fn restart(&mut self) -> std::io::Result<()> {
        let Some(file) = &mut self.file else {
            return Ok(());
        };
        S::truncate(file)?;
        file.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    fn release(&mut self) {
        self.file = None;
    }

    fn start_reading(&mut self) -> std::io::Result<()> {
        let Some(file) = &mut self.file else {
            return Ok(());
        };
        file.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    fn read_next(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let Some(file) = &mut self.file else {
            return Ok(0);
        };
        file.read(buffer)
    }
}

impl<S: SpillStorage> Transfer<S> {
    fn new(first: &ArtifactChunk) -> Self {
        Self {
            capture_scope: None,
            created_at: None,
            name: first.name.clone(),
            mime: first.mime.clone(),
            next_sequence: 0,
            received: 0,
            body: TransferBody::empty(),
            digest: Sha256::new(),
            chunks: VecDeque::new(),
            fingerprint_base: 0,
            state: TransferState::Open,
        }
    }

    fn digest(&self) -> String {
        self.digest.clone().finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>()
    }

    fn ingest(&mut self, storage: &S, chunk: &ArtifactChunk, replay_chunks: u64) -> Result<(), ProviderExecutionFailure> {
        self.validate_identity(chunk)?;
        if chunk.sequence < self.next_sequence {
            return self.validate_replay(chunk);
        }
        if chunk.sequence > self.next_sequence {
            return Err(artifact_failure(format!(
                "artifact `{}` expected chunk {} but received {}",
                chunk.transfer_id, self.next_sequence, chunk.sequence
            )));
        }
        if self.state != TransferState::Open {
            return Err(artifact_failure(format!(
                "artifact `{}` sent chunk {} after its terminal chunk",
                chunk.transfer_id, chunk.sequence
            )));
        }

        // A replacing piece starts the content over, so the limit is about the
        // artifact this transfer now holds rather than about everything that
        // ever arrived for it. What the execution was charged stays charged:
        // the bytes crossed the wire, and a producer rewriting a file without
        // limit is spending the run's capacity whether or not they are stored.
        let assembled = if chunk.replace { 0 } else { self.received };
        let received = assembled + chunk.bytes.len() as u64;
        if received > MAX_ARTIFACT_BYTES {
            return Err(artifact_failure(format!(
                "artifact `{}` streamed more than its {MAX_ARTIFACT_BYTES} byte transfer limit",
                chunk.transfer_id
            )));
        }

        // The replay window slides rather than refuses: the oldest
        // fingerprint is dropped as the newest arrives, so an artifact that
        // has sent more pieces than the window holds simply cannot replay
        // that far back — see `validate_replay`.
        if self.chunks.len() as u64 >= replay_chunks {
            self.chunks.pop_front();
            self.fingerprint_base += 1;
        }
        self.chunks.push_back(fingerprint(chunk));
        self.next_sequence += 1;
        if chunk.aborted {
            // What arrived is the prefix of a file the producer has given up on,
            // so it is released here rather than assembled any further.
            self.body.release();
            self.state = TransferState::Aborted;
            return Ok(());
        }
        if chunk.replace {
            self.body.restart().map_err(|error| artifact_failure(error.to_string()))?;
            self.digest = Sha256::new();
        }
        self.body.write_all(storage, &chunk.bytes).map_err(|error| artifact_failure(error.to_string()))?;
        self.digest.update(&chunk.bytes);
        self.received = received;
        if chunk.last {
            self.state = TransferState::Complete;
        }
        Ok(())
    }

    fn validate_identity(&self, chunk: &ArtifactChunk) -> Result<(), ProviderExecutionFailure> {
        if chunk.name != self.name || chunk.mime != self.mime {
            return Err(artifact_failure(format!("artifact `{}` changed name or media type between chunks", chunk.transfer_id)));
        }
        Ok(())
    }

    fn validate_replay(&self, chunk: &ArtifactChunk) -> Result<(), ProviderExecutionFailure> {
        let offset = chunk.sequence.checked_sub(self.fingerprint_base).ok_or_else(|| artifact_failure("artifact replay window expired"))?;
        let expected = self.chunks.get(offset as usize).ok_or_else(|| artifact_failure("artifact replay referenced an unknown chunk"))?;
        let actual = fingerprint(chunk);
        if expected.digest != actual.digest
            || expected.bytes != actual.bytes
            || expected.last != actual.last
            || expected.aborted != actual.aborted
            || expected.replace != actual.replace
        {
            return Err(artifact_failure(format!(
                "artifact `{}` replayed chunk {} with different content",
                chunk.transfer_id, chunk.sequence
            )));
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn validate_chunk(chunk: &ArtifactChunk) -> Result<(), ProviderExecutionFailure> {
    if chunk.transfer_id.trim().is_empty() {
        return Err(artifact_failure("artifact transfer identity is empty"));
    }
    crate::artifacts::name::validate(&chunk.name).map_err(|error| artifact_failure(error.to_string()))?;
    if chunk.mime.trim().is_empty() {
        return Err(artifact_failure(format!("artifact `{}` has an empty media type", chunk.transfer_id)));
    }
    if chunk.bytes.is_empty() && !chunk.last {
        return Err(artifact_failure(format!("artifact `{}` has an empty non-terminal chunk", chunk.transfer_id)));
    }
    if chunk.aborted && !chunk.last {
        return Err(artifact_failure(format!(
            "artifact `{}` aborted at chunk {} without ending its transfer",
            chunk.transfer_id, chunk.sequence
        )));
    }
    if chunk.aborted && !chunk.bytes.is_empty() {
        return Err(artifact_failure(format!("artifact `{}` carried content on the chunk that aborts it", chunk.transfer_id)));
    }
    if chunk.replace && chunk.aborted {
        return Err(artifact_failure(format!(
            "artifact `{}` both replaced and abandoned its content on chunk {}",
            chunk.transfer_id, chunk.sequence
        )));
    }
    Ok(())
}

fn fingerprint(chunk: &ArtifactChunk) -> ChunkFingerprint {
    ChunkFingerprint {
        digest: Sha256::digest(&chunk.bytes).into(),
        bytes: chunk.bytes.len(),
        last: chunk.last,
        aborted: chunk.aborted,
        replace: chunk.replace,
    }
}

fn artifact_failure(message: impl Into<String>) -> ProviderExecutionFailure {
    failure("sidecar_artifact", message)
}

/* === Tests === */
