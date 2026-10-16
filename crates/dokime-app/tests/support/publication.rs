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
use harmos::storage::frame::{Scan, ScanCursor, ScanOrder, ScanPage};
use harmos::storage::{
    BackendSnapshot, BootstrapId, BootstrapManifest, FileCheckpoint, FileMetadata, FileProgress,
    FileReference, MutationReceipt, ObjectBrowseCursor, ObjectBrowsePage, ObjectBrowseRequest,
    ObjectEntry, ObjectKey, ObjectPoint, ObjectReleaseCursor, ObjectReleasePage,
    ObjectRetentionRelease, ObjectVersions, QueryLimits, ReadCursor, ReadLimits, ReadPage,
    ReadSelection, ReadView, RetainedBatch, RetainedSnapshot, SchemaKey, StagedFile,
    StorageBackend, StorageBatch, StorageCursor, StorageFuture, StorageSchema, StoreHistory,
    StoredRecord, StoredRow,
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
    inner: Arc<dyn StorageBackend<Catalog = dokime::storage::objects::Objects>>,
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
    pub fn new(
        inner: Arc<dyn StorageBackend<Catalog = dokime::storage::objects::Objects>>,
        fault: Arc<PublicationFault>,
    ) -> Self {
        Self { inner, fault }
    }
}
impl StorageBackend for PublicationBackend {
    type Catalog = dokime::storage::objects::Objects;
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
    fn read_object_releases<'a>(
        &'a self,
        _view: &'a ReadView,
        _cursor: Option<&'a ObjectReleaseCursor>,
        _limits: ReadLimits,
    ) -> StorageFuture<'a, ObjectReleasePage> {
        self.inner.read_object_releases(_view, _cursor, _limits)
    }
    fn write_bootstrap_releases<'a>(
        &'a self,
        _identity: &'a BootstrapId,
        _releases: &'a [ObjectRetentionRelease],
    ) -> StorageFuture<'a, ()> {
        self.inner.write_bootstrap_releases(_identity, _releases)
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
    fn supports_retained_snapshots(&self) -> bool {
        self.inner.supports_retained_snapshots()
    }
    fn read_retained_view<'a>(
        &'a self,
        _through: &'a StorageCursor,
        _limits: QueryLimits,
    ) -> StorageFuture<'a, RetainedSnapshot> {
        self.inner.read_retained_view(_through, _limits)
    }
    fn begin_bootstrap<'a>(
        &'a self,
        _identity: &'a BootstrapId,
        _saved: &'a StorageCursor,
        _limits: QueryLimits,
    ) -> StorageFuture<'a, ()> {
        self.inner.begin_bootstrap(_identity, _saved, _limits)
    }
    fn write_bootstrap<'a>(&'a self, _batch: RetainedBatch<'a>) -> StorageFuture<'a, ()> {
        self.inner.write_bootstrap(_batch)
    }
    fn create_bootstrap_file<'a>(
        &'a self,
        _identity: &'a BootstrapId,
        _metadata: &'a FileMetadata,
    ) -> StorageFuture<'a, StagedFile> {
        self.inner.create_bootstrap_file(_identity, _metadata)
    }
    fn write_bootstrap_file<'a>(
        &'a self,
        _identity: &'a BootstrapId,
        _staging: &'a StagedFile,
        _offset: u64,
        _bytes: &'a [u8],
    ) -> StorageFuture<'a, ()> {
        self.inner
            .write_bootstrap_file(_identity, _staging, _offset, _bytes)
    }
    fn finish_bootstrap_file<'a>(
        &'a self,
        _identity: &'a BootstrapId,
        _staging: &'a StagedFile,
        _position: &'a StorageCursor,
        _length: u64,
        _sha256: [u8; 32],
    ) -> StorageFuture<'a, FileReference> {
        self.inner
            .finish_bootstrap_file(_identity, _staging, _position, _length, _sha256)
    }
    fn abort_bootstrap_file<'a>(
        &'a self,
        _identity: &'a BootstrapId,
        _staging: &'a StagedFile,
    ) -> StorageFuture<'a, ()> {
        self.inner.abort_bootstrap_file(_identity, _staging)
    }
    fn finish_bootstrap<'a>(
        &'a self,
        _manifest: &'a BootstrapManifest,
    ) -> StorageFuture<'a, StorageCursor> {
        self.inner.finish_bootstrap(_manifest)
    }
    fn abort_bootstrap<'a>(&'a self, _identity: &'a BootstrapId) -> StorageFuture<'a, ()> {
        self.inner.abort_bootstrap(_identity)
    }
    fn write<'a>(&'a self, batch: StorageBatch<'a>) -> StorageFuture<'a, StorageCursor> {
        self.inner.write(batch)
    }
    fn read_view<'a>(&'a self, through: &'a StorageCursor) -> StorageFuture<'a, BackendSnapshot> {
        self.inner.read_view(through)
    }
    fn release_view<'a>(&'a self, snapshot: &'a BackendSnapshot) -> StorageFuture<'a, ()> {
        self.inner.release_view(snapshot)
    }
    fn read_schemas<'a>(
        &'a self,
        view: &'a ReadView,
        selection: &'a ReadSelection,
        cursor: Option<&'a ReadCursor>,
        limits: ReadLimits,
    ) -> StorageFuture<'a, ReadPage<StorageSchema>> {
        self.inner.read_schemas(view, selection, cursor, limits)
    }
    fn read_files<'a>(
        &'a self,
        view: &'a ReadView,
        selection: &'a ReadSelection,
        cursor: Option<&'a ReadCursor>,
        limits: ReadLimits,
    ) -> StorageFuture<'a, ReadPage<FileReference>> {
        self.inner.read_files(view, selection, cursor, limits)
    }
    fn read_object<'a>(
        &'a self,
        view: &'a ReadView,
        query: &'a ObjectPoint,
        max_bytes: std::num::NonZeroUsize,
    ) -> StorageFuture<'a, Option<ObjectEntry>> {
        self.inner.read_object(view, query, max_bytes)
    }
    fn read_objects<'a>(
        &'a self,
        view: &'a ReadView,
        selection: &'a ReadSelection,
        cursor: Option<&'a ReadCursor>,
        limits: ReadLimits,
    ) -> StorageFuture<'a, ReadPage<ObjectEntry>> {
        self.inner.read_objects(view, selection, cursor, limits)
    }
    fn validate_record_filter(
        &self,
        _selection: &ReadSelection,
        _limits: QueryLimits,
    ) -> harmos::Result<()> {
        self.inner.validate_record_filter(_selection, _limits)
    }
    fn read_records<'a>(
        &'a self,
        view: &'a ReadView,
        selection: &'a ReadSelection,
        cursor: Option<&'a ReadCursor>,
        limits: ReadLimits,
    ) -> StorageFuture<'a, ReadPage<StoredRecord>> {
        self.inner.read_records(view, selection, cursor, limits)
    }
    fn read_stream<'a>(
        &'a self,
        view: &'a ReadView,
        selection: &'a ReadSelection,
        cursor: Option<&'a ReadCursor>,
        limits: ReadLimits,
    ) -> StorageFuture<'a, ReadPage<StoredRow>> {
        self.inner.read_stream(view, selection, cursor, limits)
    }
    fn supports_file_sessions(&self) -> bool {
        self.inner.supports_file_sessions()
    }
    fn supports_mutation_receipts(&self) -> bool {
        self.inner.supports_mutation_receipts()
    }
    fn save_mutations<'a>(&'a self, _receipts: &'a [MutationReceipt]) -> StorageFuture<'a, ()> {
        self.inner.save_mutations(_receipts)
    }
    fn recover_mutations(&self, _limits: ReadLimits) -> StorageFuture<'_, Vec<MutationReceipt>> {
        self.inner.recover_mutations(_limits)
    }
    fn create_production<'a>(
        &'a self,
        _metadata: &'a FileMetadata,
    ) -> StorageFuture<'a, StagedFile> {
        self.inner.create_production(_metadata)
    }
    fn fence_production<'a>(
        &'a self,
        _file: &'a StagedFile,
        _expected: u64,
        _next: u64,
    ) -> StorageFuture<'a, ()> {
        self.inner.fence_production(_file, _expected, _next)
    }
    fn save_production<'a>(&'a self, _checkpoint: &'a FileCheckpoint) -> StorageFuture<'a, ()> {
        self.inner.save_production(_checkpoint)
    }
    fn recover_productions(&self, _limits: ReadLimits) -> StorageFuture<'_, Vec<FileCheckpoint>> {
        self.inner.recover_productions(_limits)
    }
    fn file_progress<'a>(&'a self, _file: &'a StagedFile) -> StorageFuture<'a, FileProgress> {
        self.inner.file_progress(_file)
    }
    fn read_production<'a>(
        &'a self,
        _file: &'a StagedFile,
        _offset: u64,
        _max_bytes: std::num::NonZeroUsize,
    ) -> StorageFuture<'a, Vec<u8>> {
        self.inner.read_production(_file, _offset, _max_bytes)
    }
    fn supports_file_deletion(&self) -> bool {
        self.inner.supports_file_deletion()
    }
    fn delete_file<'a>(&'a self, _file: &'a StagedFile) -> StorageFuture<'a, ()> {
        self.inner.delete_file(_file)
    }
    fn abandon_file<'a>(&'a self, _staging: &'a StagedFile) -> StorageFuture<'a, ()> {
        self.inner.abandon_file(_staging)
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
    fn read_file<'a>(
        &'a self,
        view: &'a ReadView,
        file: &'a FileReference,
        offset: u64,
        max_bytes: std::num::NonZeroUsize,
    ) -> StorageFuture<'a, Vec<u8>> {
        self.inner.read_file(view, file, offset, max_bytes)
    }
    fn prune_objects<'a>(
        &'a self,
        through: &'a StorageCursor,
        max_entries: std::num::NonZeroUsize,
    ) -> StorageFuture<'a, bool> {
        self.inner.prune_objects(through, max_entries)
    }
    fn close(&self) -> StorageFuture<'_, ()> {
        self.inner.close()
    }
    fn validate_object_browse(
        &self,
        request: &ObjectBrowseRequest,
        limits: QueryLimits,
    ) -> harmos::Result<()> {
        self.inner.validate_object_browse(request, limits)
    }
    fn read_object_browse<'a>(
        &'a self,
        view: &'a ReadView,
        request: &'a ObjectBrowseRequest,
        cursor: Option<&'a ObjectBrowseCursor>,
        limits: ReadLimits,
    ) -> StorageFuture<'a, ObjectBrowsePage> {
        self.inner.read_object_browse(view, request, cursor, limits)
    }
    fn validate_object_versions(&self, versions: ObjectVersions) -> harmos::Result<()> {
        self.inner.validate_object_versions(versions)
    }
    fn validate_query_limits(&self, limits: QueryLimits) -> harmos::Result<()> {
        self.inner.validate_query_limits(limits)
    }
    fn read_query_view<'a>(
        &'a self,
        through: &'a StorageCursor,
        limits: QueryLimits,
    ) -> StorageFuture<'a, BackendSnapshot> {
        self.inner.read_query_view(through, limits)
    }
    fn supports_scan_order(&self, scan: &Scan, order: &ScanOrder) -> bool {
        self.inner.supports_scan_order(scan, order)
    }
    fn scan_ordered<'a>(
        &'a self,
        view: &'a ReadView,
        scan: &'a Scan,
        order: &'a ScanOrder,
        cursor: Option<&'a ScanCursor>,
        limits: ReadLimits,
    ) -> StorageFuture<'a, ScanPage> {
        self.inner.scan_ordered(view, scan, order, cursor, limits)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
