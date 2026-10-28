/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use std::num::NonZeroUsize;

use harmos::storage::{
    BackendSnapshot, FileCheckpoint, FileChunk, FileMetadata, FileProgress, FileReference, MutationReceipt, ObjectEntry, ObjectKey,
    ObjectPoint, ObjectRetention, ReadLimits, ScanPage, SchemaKey, StagedFile, StorageBackend, StorageBatch, StorageCursor, StorageFuture,
    StorageSchema, StoreHistory, StoredRecord, StoredRow,
    query::{FileSelection, ObjectSelection, RecordSelection, SchemaSelection, StreamSelection},
};

pub struct PublicationFault {
    suffix: &'static str,
    exit: Option<i32>,
    enabled: AtomicBool,
    failures: AtomicUsize,
    finished: Mutex<Vec<FileReference>>,
    released: AtomicUsize,
    pub advance_after_create: AtomicBool,
    pub created: AtomicUsize,
    pub append_boundary: Mutex<Option<Arc<AppendBoundary>>>,
}
pub struct PublicationBackend {
    inner: Arc<dyn StorageBackend>,
    fault: Arc<PublicationFault>,
}

pub struct AppendBoundary {
    pub advance_after_append: AtomicBool,
    pub lose_append_reply: AtomicBool,
    pub calls: Mutex<Vec<(harmos::Identifier, u64, Vec<u8>)>>,
    pub lengths: Mutex<std::collections::BTreeMap<String, u64>>,
}

pub struct AdmissionClock;

pub struct AdmissionWorker(pub Mutex<Option<tokio::task::JoinHandle<harmos::storage::Result<()>>>>);

thread_local! {
    static STORAGE_NOW: std::cell::Cell<tokio::time::Instant> = std::cell::Cell::new(tokio::time::Instant::now());
}

impl harmos::storage::WorkerOwner for AdmissionWorker {
    fn stop(&self) -> harmos::storage::StorageFuture<'_, ()> {
        let task = self.0.lock().unwrap().take();
        Box::pin(async move {
            if let Some(task) = task {
                task.await.unwrap()?;
            }
            Ok(())
        })
    }
}

impl AdmissionClock {
    pub fn execution() -> harmos::storage::execution::Execution {
        STORAGE_NOW.with(|now| now.set(tokio::time::Instant::now()));
        harmos::storage::execution::Execution::new(Self::now, |duration| Box::pin(tokio::time::sleep(duration)))
    }

    pub fn advance(duration: std::time::Duration) {
        STORAGE_NOW.with(|now| now.set(now.get() + duration));
    }

    fn now() -> tokio::time::Instant {
        STORAGE_NOW.with(std::cell::Cell::get)
    }
}

impl PublicationFault {
    pub fn new(suffix: &'static str) -> Self {
        Self {
            suffix,
            exit: None,
            enabled: AtomicBool::new(true),
            failures: AtomicUsize::new(0),
            finished: Mutex::new(Vec::new()),
            released: AtomicUsize::new(0),
            advance_after_create: AtomicBool::new(false),
            created: AtomicUsize::new(0),
            append_boundary: Mutex::new(None),
        }
    }
    pub fn crash(suffix: &'static str, status: i32) -> Self {
        let mut fault = Self::new(suffix);
        fault.exit = Some(status);
        fault
    }
    pub fn disarm(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }
    pub fn failures(&self) -> usize {
        self.failures.load(Ordering::SeqCst)
    }
    pub fn finished(&self) -> Vec<FileReference> {
        self.finished.lock().unwrap().clone()
    }
    pub fn releases(&self) -> usize {
        self.released.load(Ordering::SeqCst)
    }
}
impl PublicationBackend {
    pub fn new(inner: Arc<dyn StorageBackend>, fault: Arc<PublicationFault>) -> Self {
        Self { inner, fault }
    }
}
impl StorageBackend for PublicationBackend {
    fn finish_file<'a>(
        &'a self, staging: &'a StagedFile, position: &'a StorageCursor, length: u64, sha256: [u8; 32],
    ) -> StorageFuture<'a, FileReference> {
        Box::pin(async move {
            if staging.id.as_str().ends_with(self.fault.suffix) && self.fault.enabled.load(Ordering::SeqCst) {
                if let Some(status) = self.fault.exit {
                    std::process::exit(status);
                }
                self.fault.failures.fetch_add(1, Ordering::SeqCst);
                return Err(harmos::storage::Error::Backend {
                    operation: "fixture report publication",
                    kind: harmos::BackendFailure::Transient,
                    source: Box::new(std::io::Error::other("controlled report publication interruption")),
                });
            }
            let reference = self.inner.finish_file(staging, position, length, sha256).await?;
            self.fault.finished.lock().unwrap().push(reference.clone());
            Ok(reference)
        })
    }
    fn release_object_history<'a>(
        &'a self, schema: &'a SchemaKey, item: &'a ObjectKey, through: &'a StorageCursor,
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.inner.release_object_history(schema, item, through).await?;
            self.fault.released.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    }
    fn history(&self) -> &StoreHistory {
        self.inner.history()
    }
    fn open(&self) -> StorageFuture<'_, StorageCursor> {
        self.inner.open()
    }
    fn cursor_saved(&self) -> StorageFuture<'_, StorageCursor> {
        self.inner.cursor_saved()
    }
    fn close(&self) -> StorageFuture<'_, ()> {
        self.inner.close()
    }
    fn write_schemas(&self, batch: StorageBatch<StorageSchema>) -> StorageFuture<'_, StorageCursor> {
        self.inner.write_schemas(batch)
    }
    fn write_objects(&self, batch: StorageBatch<ObjectEntry>) -> StorageFuture<'_, StorageCursor> {
        self.inner.write_objects(batch)
    }
    fn write_records(&self, batch: StorageBatch<StoredRecord>) -> StorageFuture<'_, StorageCursor> {
        self.inner.write_records(batch)
    }
    fn write_stream(&self, batch: StorageBatch<StoredRow>) -> StorageFuture<'_, StorageCursor> {
        self.inner.write_stream(batch)
    }
    fn write_files(&self, batch: StorageBatch<FileReference>) -> StorageFuture<'_, StorageCursor> {
        self.inner.write_files(batch)
    }
    fn open_snapshot<'a>(&'a self, at: &'a StorageCursor) -> StorageFuture<'a, BackendSnapshot> {
        self.inner.open_snapshot(at)
    }
    fn release_snapshot<'a>(&'a self, snapshot: &'a BackendSnapshot) -> StorageFuture<'a, ()> {
        self.inner.release_snapshot(snapshot)
    }
    fn read_schemas<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a SchemaSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<StorageSchema>> {
        self.inner.read_schemas(snapshot, selection, page)
    }
    fn read_objects<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a ObjectSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<ObjectEntry>> {
        self.inner.read_objects(snapshot, selection, page)
    }
    fn read_records<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a RecordSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<StoredRecord>> {
        self.inner.read_records(snapshot, selection, page)
    }
    fn read_stream<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a StreamSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<StoredRow>> {
        self.inner.read_stream(snapshot, selection, page)
    }
    fn read_files<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a FileSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<FileReference>> {
        self.inner.read_files(snapshot, selection, page)
    }
    fn read_object<'a>(&'a self, snapshot: &'a BackendSnapshot, point: &'a ObjectPoint) -> StorageFuture<'a, Option<ObjectEntry>> {
        self.inner.read_object(snapshot, point)
    }
    fn read_file<'a>(
        &'a self, snapshot: &'a BackendSnapshot, file: &'a FileReference, offset: u64, max_bytes: NonZeroUsize,
    ) -> StorageFuture<'a, Vec<u8>> {
        self.inner.read_file(snapshot, file, offset, max_bytes)
    }
    fn prune_objects<'a>(
        &'a self, through: &'a StorageCursor, max_entries: NonZeroUsize,
        retention: fn(&SchemaKey) -> harmos::domain::Result<ObjectRetention>,
    ) -> StorageFuture<'a, bool> {
        self.inner.prune_objects(through, max_entries, retention)
    }
    fn create_file<'a>(&'a self, metadata: &'a FileMetadata) -> StorageFuture<'a, StagedFile> {
        if let Some(boundary) = self.fault.append_boundary.lock().unwrap().as_ref() {
            boundary.lengths.lock().unwrap().insert(metadata.id.to_string(), metadata.expected_length.unwrap());
        }
        self.inner.create_file(metadata)
    }
    fn write_file<'a>(&'a self, staging: &'a StagedFile, offset: u64, bytes: FileChunk) -> StorageFuture<'a, ()> {
        let boundary = self.fault.append_boundary.lock().unwrap().clone();
        Box::pin(async move {
            let complete_report = boundary.as_ref().is_some_and(|boundary| {
                boundary.calls.lock().unwrap().push((staging.id.clone(), offset, bytes.as_slice().to_vec()));
                staging.id.as_str().ends_with(".report")
                    && boundary.lengths.lock().unwrap().get(staging.id.as_str()).copied() == Some(offset + bytes.len() as u64)
            });
            self.inner.write_file(staging, offset, bytes).await?;
            if complete_report {
                let boundary = boundary.unwrap();
                if boundary.advance_after_append.swap(false, Ordering::SeqCst) {
                    tokio::time::pause();
                    tokio::time::advance(std::time::Duration::from_secs(51)).await;
                    tokio::time::resume();
                }
                if boundary.lose_append_reply.load(Ordering::SeqCst) {
                    return Err(harmos::storage::Error::Backend {
                        operation: "fixture append reply",
                        kind: harmos::BackendFailure::Transient,
                        source: Box::new(std::io::Error::other("controlled append reply loss")),
                    });
                }
            }
            Ok(())
        })
    }
    fn abort_file<'a>(&'a self, staging: &'a StagedFile) -> StorageFuture<'a, ()> {
        self.inner.abort_file(staging)
    }
    fn supports_file_sessions(&self) -> bool {
        self.inner.supports_file_sessions()
    }
    fn create_production<'a>(&'a self, metadata: &'a FileMetadata) -> StorageFuture<'a, StagedFile> {
        Box::pin(async move {
            let file = self.inner.create_production(metadata).await?;
            self.fault.created.fetch_add(1, Ordering::SeqCst);
            if self.fault.advance_after_create.swap(false, Ordering::SeqCst) {
                AdmissionClock::advance(std::time::Duration::from_secs(2));
            }
            Ok(file)
        })
    }
    fn fence_production<'a>(&'a self, file: &'a StagedFile, expected: u64, next: u64) -> StorageFuture<'a, ()> {
        self.inner.fence_production(file, expected, next)
    }
    fn save_production(&self, checkpoint: StorageBatch<FileCheckpoint>) -> StorageFuture<'_, ()> {
        self.inner.save_production(checkpoint)
    }
    fn recover_productions(&self, limits: ReadLimits) -> StorageFuture<'_, Vec<FileCheckpoint>> {
        self.inner.recover_productions(limits)
    }
    fn file_progress<'a>(&'a self, file: &'a StagedFile) -> StorageFuture<'a, FileProgress> {
        self.inner.file_progress(file)
    }
    fn read_production<'a>(&'a self, file: &'a StagedFile, offset: u64, max_bytes: NonZeroUsize) -> StorageFuture<'a, Vec<u8>> {
        self.inner.read_production(file, offset, max_bytes)
    }
    fn abandon_file<'a>(&'a self, staging: &'a StagedFile) -> StorageFuture<'a, ()> {
        self.inner.abandon_file(staging)
    }
    fn supports_file_deletion(&self) -> bool {
        self.inner.supports_file_deletion()
    }
    fn delete_file<'a>(&'a self, file: &'a StagedFile) -> StorageFuture<'a, ()> {
        self.inner.delete_file(file)
    }
    fn supports_mutation_receipts(&self) -> bool {
        self.inner.supports_mutation_receipts()
    }
    fn save_mutations(&self, receipts: StorageBatch<MutationReceipt>) -> StorageFuture<'_, ()> {
        self.inner.save_mutations(receipts)
    }
    fn recover_mutations(&self, limits: ReadLimits) -> StorageFuture<'_, Vec<MutationReceipt>> {
        self.inner.recover_mutations(limits)
    }
    fn read_identities<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a harmos::domain::model::storage::IdentitySelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<harmos::domain::model::storage::StoredIdentity>> {
        self.inner.read_identities(snapshot, selection, page)
    }
}
