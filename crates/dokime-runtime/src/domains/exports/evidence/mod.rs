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
    io::{self, Write},
    num::NonZeroUsize,
    path::PathBuf,
    sync::Arc,
};

// Third Party Imports
use harmos::storage::{ByteRange, FileReadBatch, FileReader, FileSource, ReadBatch, operations::ReadFile};
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

const MAX_INVENTORY_ENTRIES: u64 = 65_536;
const MAX_MANIFEST_BYTES: usize = 16 * 1024 * 1024;
const MAX_EXPORT_BYTES: u64 = 8 * 1024 * 1024 * 1024;

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
    session: crate::storage::Manager,
    reader: Option<FileReader>,
    batch: Option<FileReadBatch>,
    batch_offset: usize,
    offset: u64,
    pending: Option<Pending>,
}

struct Source {
    metadata: EvidenceExportMetadata,
    _directory: TempDir,
    _permit: OwnedSemaphorePermit,
    _work: TaskTrackerToken,
}

struct Pending {
    maximum: NonZeroUsize,
    task: JoinHandle<(FileReader, Result<Option<FileReadBatch>>)>,
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
        let admission = Arc::new(Semaphore::new(1));
        Self { root, admission }
    }

    async fn export(&self, runtime: &RuntimeHandle, subject: EvidenceSubject) -> Result<Bundle> {
        let work = runtime.admit_owned_work().ok_or(Error::Cancelled)?;
        let permit = self.admission.clone().try_acquire_owned().map_err(|_| Error::Busy)?;
        let cancellation = CancellationToken::new();
        let _cancel_on_drop = cancellation.clone().drop_guard();
        let manager = self.clone();
        let storage = runtime.storage().clone();
        let schema_settings = *runtime.schema_settings();
        let executor = tokio::runtime::Handle::current();
        // SQLite And ZIP I/O Stay On The Blocking Worker; Native Reads Remain Async.
        tokio::task::spawn_blocking(move || {
            executor.block_on(build(manager, storage, subject, cancellation, permit, work, schema_settings))
        })
        .await?
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

    pub fn session(&self) -> &crate::storage::Manager {
        &self.session
    }

    pub fn retained_metadata(&self) -> Result<ReadBatch<EvidenceExportMetadata>> {
        Ok(harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, harmos::storage::Error>(self.source.metadata.clone()))?)
    }

    pub async fn next_chunk(&mut self, maximum: NonZeroUsize) -> Result<Option<ReadBatch<ArtifactChunk>>> {
        if self.pending.as_ref().is_some_and(|pending| pending.maximum != maximum) {
            return Err(Error::Invalid("pending export read has another maximum"));
        }
        if self.offset == self.source.metadata.size_bytes {
            return Ok(None);
        }
        if self.batch.as_ref().is_none_or(|batch| self.batch_offset == batch.bytes().len()) {
            self.batch = None;
            if self.pending.is_none() {
                let reader = self.reader.take().ok_or(Error::Invalid("export archive is unavailable"))?;
                let future = read_source(reader, Arc::clone(&self.source));
                let task = tokio::spawn(future);
                self.pending = Some(Pending { maximum, task });
            }
            let result = (&mut self.pending.as_mut().expect("accepted export read retained").task).await;
            self.pending = None;
            let (reader, batch) = result?;
            self.reader = Some(reader);
            let batch = batch?.ok_or(Error::Invalid("export archive ended before declared length"))?;
            self.batch = Some(batch);
            self.batch_offset = 0;
        }
        let batch = self.batch.as_ref().expect("native export batch retained");
        let expected = batch.offset().checked_add(self.batch_offset as u64).ok_or(Error::TooLarge)?;
        if expected != self.offset {
            return Err(Error::Invalid("export batch differs from delivered offset"));
        }
        let end = batch.bytes().len().min(self.batch_offset.saturating_add(maximum.get()));
        let length = end - self.batch_offset;
        let next =
            self.offset.checked_add(length as u64).filter(|offset| *offset <= self.source.metadata.size_bytes).ok_or(Error::TooLarge)?;
        let offset = self.offset;
        let chunk = if self.batch_offset == 0 && end == batch.bytes().len() {
            self.batch
                .take()
                .expect("native export batch retained")
                .try_map(|batch| Ok::<_, harmos::storage::Error>(ArtifactChunk { offset, data: batch.bytes }))?
        } else {
            harmos::storage::ReadBatch::new(()).try_map(|_| {
                let mut data = Vec::new();
                data.try_reserve_exact(length).map_err(|_| Error::TooLarge)?;
                data.extend_from_slice(&batch.bytes()[self.batch_offset..end]);
                Ok::<_, Error>(ArtifactChunk { offset, data })
            })?
        };
        self.batch_offset = end;
        self.offset = next;
        Ok(Some(chunk))
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn read_source(mut reader: FileReader, source: Arc<Source>) -> (FileReader, Result<Option<FileReadBatch>>) {
    let _source = source;
    let result = reader.next().await.map_err(Error::from);
    (reader, result)
}

async fn build(
    manager: Manager, storage: crate::storage::Manager, subject: EvidenceSubject, cancellation: CancellationToken,
    permit: OwnedSemaphorePermit, work: TaskTrackerToken, schema_settings: harmos::config::SchemaSettings,
) -> Result<Bundle> {
    let Manager { root, .. } = manager;
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
    if snapshot.executions.len() as u64 > MAX_INVENTORY_ENTRIES {
        return Err(Error::TooLarge);
    }
    let path = directory.path().join("evidence.sqlite");
    let mut database = database::Writer::open(&path, &snapshot)?;
    let archive_path = directory.path().join("evidence.zip");
    let mut archive = archive::Writer::open(&archive_path, &cancellation)?;
    let mut executions = Vec::new();
    let mut budget = InventoryBudget { remaining: MAX_MANIFEST_BYTES };

    for selected in &snapshot.executions {
        if cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let view = snapshot.execution(selected).await?;
        let counts = database.execution(&view, &cancellation, &mut budget, &schema_settings).await?;
        let id = selected.execution_id.as_str();
        for (index, output) in view.outputs()?.iter().enumerate() {
            let archive_path = archive::output_path(id, index, &output.relative_path);
            let reader = view.read_file(&output.file, ByteRange { offset: 0, limit: None }).await?;
            archive.stored_file(&archive_path, Some(&output.relative_path), output.file.clone(), reader, &mut budget).await?;
            database.output(id, index, &archive_path, output)?;
        }
        let report = &view.execution().saved.as_ref().ok_or(Error::Unsealed("execution"))?.report;
        let reader = view.read_file(report, ByteRange { offset: 0, limit: None }).await?;
        archive.stored_file(&format!("reports/execution-{id}.html"), None, report.clone(), reader, &mut budget).await?;
        let inventory = ExecutionInventory {
            revision: view.checkpoint()?,
            captured_at: view.captured_at()?.clone(),
            counts,
        };
        budget.reserve(&inventory)?;
        executions.push(inventory);
    }

    if let Some(report) = &snapshot.cycle_report {
        let reader = snapshot
            .storage()
            .read_file(ReadFile {
                source: harmos::storage::FileSource::Stored(report.clone()),
                range: ByteRange { offset: 0, limit: None },
            })
            .await?;
        archive.stored_file(&format!("reports/cycle-{}.html", snapshot.cycle_id), None, report.clone(), reader, &mut budget).await?;
    }
    let streams = database.finish()?;
    let reader = storage
        .read_file(ReadFile {
            source: FileSource::Local(path),
            range: ByteRange { offset: 0, limit: None },
        })
        .await?;
    archive.database(reader, &mut budget).await?;
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
    let file = archive.finish(&manifest)?;
    let session = storage.clone();
    let reader = session.read_opened_file(file.try_clone()?, ByteRange { offset: 0, limit: None }).await?;
    let (size_bytes, sha256) = archive::hash(reader, &cancellation, MAX_EXPORT_BYTES).await?;
    if file.metadata()?.len() != size_bytes {
        return Err(Error::Invalid("finalized archive length changed"));
    }
    let metadata = EvidenceExportMetadata {
        subject: Some(snapshot.subject),
        filename: snapshot.filename,
        schema_version: EVIDENCE_EXPORT_SCHEMA_VERSION,
        size_bytes,
        sha256,
    };
    let reader = session.read_opened_file(file, ByteRange { offset: 0, limit: Some(size_bytes) }).await?;
    let source = Source {
        metadata,
        _directory: directory,
        _permit: permit,
        _work: work,
    };
    Ok(Bundle {
        source: Arc::new(source),
        session,
        reader: Some(reader),
        batch: None,
        batch_offset: 0,
        offset: 0,
        pending: None,
    })
}

/* === Tests === */
