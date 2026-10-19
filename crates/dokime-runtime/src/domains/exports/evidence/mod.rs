/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Build And Read Complete Saved Evidence Archives

/* === Modules === */

mod archive;
mod database;
mod error;
mod snapshot;

/* === Re-Exports === */

pub use error::{Error, Result};

/* === Imports === */

// Standard Library Imports
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    num::NonZeroUsize,
    path::PathBuf,
    sync::{Arc, Mutex},
};

// Third Party Imports
use harmos::storage::{ByteRange, operations::ReadFile};
use serde::Serialize;
use tempfile::TempDir;
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    task::JoinHandle,
};
use tokio_util::{sync::CancellationToken, task::task_tracker::TaskTrackerToken};

// Local Imports
use crate::runtime::RuntimeHandle;
use dokime_domain::model::{
    content::wire::ArtifactChunk,
    execution::saved::ExecutionCheckpoint,
    exports::{EVIDENCE_EXPORT_SCHEMA_VERSION, EvidenceExportMetadata, EvidenceSubject},
    identity::{NodeId, TestCycleId},
    time::Timestamp,
};

/* === Constants === */

const MAX_INVENTORY_ENTRIES: usize = 65_536;
const MAX_MANIFEST_BYTES: usize = 16 * 1024 * 1024;
const MAX_CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct Manager {
    root: PathBuf,
    admission: Arc<Semaphore>,
}

pub struct Bundle {
    source: Arc<Source>,
    offset: u64,
    pending: Option<Pending>,
}

struct Source {
    file: Mutex<File>,
    metadata: EvidenceExportMetadata,
    _directory: TempDir,
    _permit: OwnedSemaphorePermit,
    _work: TaskTrackerToken,
}

struct Pending {
    maximum: NonZeroUsize,
    task: JoinHandle<Result<ArtifactChunk>>,
}

#[derive(Serialize)]
struct Manifest<'a> {
    schema_version: u32,
    complete: bool,
    subject: &'a EvidenceSubject,
    node_id: &'a NodeId,
    cycle_id: &'a TestCycleId,
    cycle_revision: &'a Option<snapshot::CycleRevision>,
    executions: Vec<ExecutionInventory>,
    streams: Vec<database::StreamInventory>,
}

#[derive(Serialize)]
struct ExecutionInventory {
    revision: ExecutionCheckpoint,
    captured_at: Timestamp,
    counts: database::Counts,
}

struct InventoryBudget {
    remaining: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl InventoryBudget {
    fn reserve(&mut self, value: &impl Serialize) -> Result<()> {
        serde_json::to_writer(self, value)?;
        Ok(())
    }
}

impl Write for InventoryBudget {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.remaining =
            self.remaining.checked_sub(bytes.len()).ok_or_else(|| io::Error::other("evidence inventory exceeds its byte limit"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Manager {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self { root, admission: Arc::new(Semaphore::new(1)) }
    }

    async fn export(&self, runtime: &RuntimeHandle, subject: EvidenceSubject) -> Result<Bundle> {
        let work = runtime.admit_owned_work().ok_or(Error::Cancelled)?;
        let permit = self.admission.clone().try_acquire_owned().map_err(|_| Error::Busy)?;
        let cancellation = CancellationToken::new();
        let _cancel_on_drop = cancellation.clone().drop_guard();
        let root = self.root.clone();
        let storage = runtime.storage().clone();
        let executor = tokio::runtime::Handle::current();
        // SQLite And ZIP I/O Stay On The Blocking Worker; Native Reads Remain Async.
        tokio::task::spawn_blocking(move || executor.block_on(build(root, storage, subject, cancellation, permit, work))).await?
    }
}

impl RuntimeHandle {
    pub async fn export_evidence(&self, subject: EvidenceSubject) -> Result<Bundle> {
        self.evidence_exports.export(self, subject).await
    }
}

impl Bundle {
    pub fn metadata(&self) -> &EvidenceExportMetadata {
        &self.source.metadata
    }

    pub async fn next_chunk(&mut self, maximum: NonZeroUsize) -> Result<Option<ArtifactChunk>> {
        if maximum.get() > MAX_CHUNK_BYTES {
            return Err(Error::Invalid("export transfer chunk exceeds 64 KiB"));
        }
        if let Some(pending) = &self.pending {
            if pending.maximum != maximum {
                return Err(Error::Invalid("pending export read has another maximum"));
            }
        } else {
            if self.offset == self.source.metadata.size_bytes {
                return Ok(None);
            }
            let source = self.source.clone();
            let offset = self.offset;
            let length = (source.metadata.size_bytes - offset).min(maximum.get() as u64) as usize;
            let task = tokio::task::spawn_blocking(move || {
                let mut bytes = vec![0; length];
                let mut file = source.file.lock().map_err(|_| Error::Invalid("export file lock poisoned"))?;
                file.seek(SeekFrom::Start(offset))?;
                file.read_exact(&mut bytes)?;
                Ok(ArtifactChunk { offset, data: bytes })
            });
            self.pending = Some(Pending { maximum, task });
        }

        let pending = self.pending.as_mut().expect("accepted export read retained");
        let result = (&mut pending.task).await;
        self.pending = None;
        let chunk = result??;
        self.offset = chunk.offset.checked_add(chunk.data.len() as u64).ok_or(Error::TooLarge)?;
        Ok(Some(chunk))
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn build(
    root: PathBuf, storage: crate::storage::Manager, subject: EvidenceSubject, cancellation: CancellationToken,
    permit: OwnedSemaphorePermit, work: TaskTrackerToken,
) -> Result<Bundle> {
    let mut directory = std::fs::DirBuilder::new();
    directory.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        directory.mode(0o700);
    }
    directory.create(&root)?;
    let directory = tempfile::Builder::new().prefix("evidence-").tempdir_in(root)?;
    let snapshot = snapshot::Snapshot::open(&storage, subject).await?;
    if snapshot.executions.len() > MAX_INVENTORY_ENTRIES {
        return Err(Error::TooLarge);
    }
    let path = directory.path().join("evidence.sqlite");
    let mut database = database::Writer::open(&path, &snapshot)?;
    let mut archive = archive::Writer::open(&directory.path().join("evidence.zip"), &cancellation)?;
    let mut executions = Vec::new();
    let mut budget = InventoryBudget { remaining: MAX_MANIFEST_BYTES };

    for selected in &snapshot.executions {
        if cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let view = snapshot.execution(selected).await?;
        let counts = database.execution(&view, &cancellation, &mut budget).await?;
        let id = selected.execution_id.as_str();
        for (index, output) in view.outputs()?.iter().enumerate() {
            let archive_path = archive::output_path(id, index, &output.relative_path);
            let reader = view.read_file(&output.file, ByteRange { offset: 0, limit: None }).await?;
            archive.stored_file(&archive_path, Some(&output.relative_path), reader, &mut budget).await?;
            database.output(id, index, &archive_path, output)?;
        }
        let report = &view.execution().saved.as_ref().ok_or(Error::Unsealed("execution"))?.report;
        let reader = view.read_file(report, ByteRange { offset: 0, limit: None }).await?;
        archive.stored_file(&format!("reports/execution-{id}.html"), None, reader, &mut budget).await?;
        let inventory = ExecutionInventory { revision: view.checkpoint()?, captured_at: view.captured_at()?.clone(), counts };
        budget.reserve(&inventory)?;
        executions.push(inventory);
    }

    if let Some(report) = &snapshot.cycle_report {
        let reader = snapshot.storage().read_file(ReadFile { file: report.clone(), range: ByteRange { offset: 0, limit: None } }).await?;
        archive.stored_file(&format!("reports/cycle-{}.html", snapshot.cycle_id), None, reader, &mut budget).await?;
    }
    let streams = database.finish()?;
    archive.database(&path, &mut budget)?;
    let manifest = Manifest {
        schema_version: EVIDENCE_EXPORT_SCHEMA_VERSION,
        complete: true,
        subject: &snapshot.subject,
        node_id: &snapshot.node_id,
        cycle_id: &snapshot.cycle_id,
        cycle_revision: &snapshot.cycle_revision,
        executions,
        streams,
    };
    let (file, size_bytes, sha256) = archive.finish(&manifest)?;
    let metadata = EvidenceExportMetadata {
        subject: Some(snapshot.subject),
        filename: snapshot.filename,
        schema_version: EVIDENCE_EXPORT_SCHEMA_VERSION,
        size_bytes,
        sha256,
    };
    let source = Source { file: Mutex::new(file), metadata, _directory: directory, _permit: permit, _work: work };
    Ok(Bundle { source: Arc::new(source), offset: 0, pending: None })
}

/* === Tests === */
