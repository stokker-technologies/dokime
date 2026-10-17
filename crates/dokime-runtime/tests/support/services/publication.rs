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
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
// Third Party Imports
use std::num::NonZeroUsize;

use harmos::storage::{
    BackendSnapshot, FileCheckpoint, FileMetadata, FileProgress, FileReference, MutationReceipt,
    ObjectEntry, ObjectKey, ObjectPoint, ReadLimits, ScanPage, SchemaKey, StagedFile,
    StorageBackend, StorageBatch, StorageCursor, StorageFuture, StorageSchema, StoreHistory,
    StoredRecord, StoredRow,
    query::{FileSelection, ObjectSelection, RecordSelection, SchemaSelection, StreamSelection},
};
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct PublicationFault {
    suffix: &'static str,
    exit: Option<i32>,
    enabled: AtomicBool,
    failures: AtomicUsize,
    finished: Mutex<Vec<FileReference>>,
    released: AtomicUsize,
}
pub struct PublicationBackend {
    inner: Arc<dyn StorageBackend>,
    fault: Arc<PublicationFault>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl PublicationFault {
    pub fn new(suffix: &'static str) -> Self {
        Self {
            suffix,
            exit: None,
            enabled: AtomicBool::new(true),
            failures: AtomicUsize::new(0),
            finished: Mutex::new(Vec::new()),
            released: AtomicUsize::new(0),
        }
    }
    /// Only Child-Process Crash Stories Use This Constructor.
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
        &'a self,
        staging: &'a StagedFile,
        position: &'a StorageCursor,
        length: u64,
        sha256: [u8; 32],
    ) -> StorageFuture<'a, FileReference> {
        Box::pin(async move {
            if staging.id.as_str().ends_with(self.fault.suffix)
                && self.fault.enabled.load(Ordering::SeqCst)
            {
                if let Some(status) = self.fault.exit {
                    std::process::exit(status);
                }
                self.fault.failures.fetch_add(1, Ordering::SeqCst);
                return Err(harmos::Error::Backend {
                    operation: "fixture report publication",
                    kind: harmos::BackendFailure::Transient,
                    source: Box::new(std::io::Error::other(
                        "controlled report publication interruption",
                    )),
                });
            }
            let reference = self
                .inner
                .finish_file(staging, position, length, sha256)
                .await?;
            self.fault.finished.lock().unwrap().push(reference.clone());
            Ok(reference)
        })
    }
    fn release_object_history<'a>(
        &'a self,
        schema: &'a SchemaKey,
        item: &'a ObjectKey,
        through: &'a StorageCursor,
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.inner
                .release_object_history(schema, item, through)
                .await?;
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
    fn write<'a>(&'a self, batch: StorageBatch<'a>) -> StorageFuture<'a, StorageCursor> {
        self.inner.write(batch)
    }
    fn open_snapshot<'a>(&'a self, at: &'a StorageCursor) -> StorageFuture<'a, BackendSnapshot> {
        self.inner.open_snapshot(at)
    }
    fn release_snapshot<'a>(&'a self, snapshot: &'a BackendSnapshot) -> StorageFuture<'a, ()> {
        self.inner.release_snapshot(snapshot)
    }
    fn read_schemas<'a>(
        &'a self,
        snapshot: &'a BackendSnapshot,
        selection: &'a SchemaSelection,
        page: ScanPage,
    ) -> StorageFuture<'a, Vec<StorageSchema>> {
        self.inner.read_schemas(snapshot, selection, page)
    }
    fn read_objects<'a>(
        &'a self,
        snapshot: &'a BackendSnapshot,
        selection: &'a ObjectSelection,
        page: ScanPage,
    ) -> StorageFuture<'a, Vec<ObjectEntry>> {
        self.inner.read_objects(snapshot, selection, page)
    }
    fn read_records<'a>(
        &'a self,
        snapshot: &'a BackendSnapshot,
        selection: &'a RecordSelection,
        page: ScanPage,
    ) -> StorageFuture<'a, Vec<StoredRecord>> {
        self.inner.read_records(snapshot, selection, page)
    }
    fn read_stream<'a>(
        &'a self,
        snapshot: &'a BackendSnapshot,
        selection: &'a StreamSelection,
        page: ScanPage,
    ) -> StorageFuture<'a, Vec<StoredRow>> {
        self.inner.read_stream(snapshot, selection, page)
    }
    fn read_files<'a>(
        &'a self,
        snapshot: &'a BackendSnapshot,
        selection: &'a FileSelection,
        page: ScanPage,
    ) -> StorageFuture<'a, Vec<FileReference>> {
        self.inner.read_files(snapshot, selection, page)
    }
    fn read_object<'a>(
        &'a self,
        snapshot: &'a BackendSnapshot,
        point: &'a ObjectPoint,
    ) -> StorageFuture<'a, Option<ObjectEntry>> {
        self.inner.read_object(snapshot, point)
    }
    fn read_file<'a>(
        &'a self,
        snapshot: &'a BackendSnapshot,
        file: &'a FileReference,
        offset: u64,
        max_bytes: NonZeroUsize,
    ) -> StorageFuture<'a, Vec<u8>> {
        self.inner.read_file(snapshot, file, offset, max_bytes)
    }
    fn prune_objects<'a>(
        &'a self,
        through: &'a StorageCursor,
        max_entries: NonZeroUsize,
    ) -> StorageFuture<'a, bool> {
        self.inner.prune_objects(through, max_entries)
    }
    fn create_file<'a>(&'a self, metadata: &'a FileMetadata) -> StorageFuture<'a, StagedFile> {
        self.inner.create_file(metadata)
    }
    fn write_file<'a>(
        &'a self,
        staging: &'a StagedFile,
        offset: u64,
        bytes: &'a [u8],
    ) -> StorageFuture<'a, ()> {
        self.inner.write_file(staging, offset, bytes)
    }
    fn abort_file<'a>(&'a self, staging: &'a StagedFile) -> StorageFuture<'a, ()> {
        self.inner.abort_file(staging)
    }
    fn supports_file_sessions(&self) -> bool {
        self.inner.supports_file_sessions()
    }
    fn create_production<'a>(
        &'a self,
        metadata: &'a FileMetadata,
    ) -> StorageFuture<'a, StagedFile> {
        self.inner.create_production(metadata)
    }
    fn fence_production<'a>(
        &'a self,
        file: &'a StagedFile,
        expected: u64,
        next: u64,
    ) -> StorageFuture<'a, ()> {
        self.inner.fence_production(file, expected, next)
    }
    fn save_production<'a>(&'a self, checkpoint: &'a FileCheckpoint) -> StorageFuture<'a, ()> {
        self.inner.save_production(checkpoint)
    }
    fn recover_productions(&self, limits: ReadLimits) -> StorageFuture<'_, Vec<FileCheckpoint>> {
        self.inner.recover_productions(limits)
    }
    fn file_progress<'a>(&'a self, file: &'a StagedFile) -> StorageFuture<'a, FileProgress> {
        self.inner.file_progress(file)
    }
    fn read_production<'a>(
        &'a self,
        file: &'a StagedFile,
        offset: u64,
        max_bytes: NonZeroUsize,
    ) -> StorageFuture<'a, Vec<u8>> {
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
    fn save_mutations<'a>(&'a self, receipts: &'a [MutationReceipt]) -> StorageFuture<'a, ()> {
        self.inner.save_mutations(receipts)
    }
    fn recover_mutations(&self, limits: ReadLimits) -> StorageFuture<'_, Vec<MutationReceipt>> {
        self.inner.recover_mutations(limits)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
