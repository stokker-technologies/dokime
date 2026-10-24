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
    BackendSnapshot, FileCheckpoint, FileMetadata, FileProgress, FileReference, MutationReceipt, ObjectEntry, ObjectKey, ObjectPoint,
    ReadLimits, ScanPage, SchemaKey, StagedFile, StorageBackend, StorageBatch, StorageCursor, StorageFuture, StorageSchema, StoreHistory,
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

pub struct MutationBoundary {
    pub advance_after_append: AtomicBool,
    pub lose_append_reply: AtomicBool,
    pub calls: Mutex<Vec<harmos::storage::MutationId>>,
    pub lengths: Mutex<std::collections::BTreeMap<String, u64>>,
}

pub struct MutationHost<H> {
    pub inner: H,
    pub storage: dokime_runtime::storage::Manager,
    pub boundary: Arc<MutationBoundary>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<H: harmos::operations::Host<Error = harmos::Error>> harmos::operations::Host for MutationHost<H> {
    type Error = harmos::Error;
    type Instance<O: harmos::operations::OperationDeclaration> = H::Instance<O>;

    fn context(&self) -> &harmos::operations::CallContext {
        self.inner.context()
    }

    fn with_value<T: harmos::operations::ContextValue>(&self, value: T) -> Result<Self, harmos::operations::ContextError> {
        Ok(Self { inner: self.inner.with_value(value)?, storage: self.storage.clone(), boundary: self.boundary.clone() })
    }

    async fn submit<O: harmos::operations::OperationDeclaration>(&self, request: O) -> Result<Self::Instance<O>, Self::Error> {
        self.inner.submit(request).await
    }

    async fn submit_with_resources<O: harmos::operations::OperationDeclaration<Response = ()>, R: Send + Sync + 'static>(
        &self, request: O, resources: Arc<R>,
    ) -> Result<Self::Instance<O>, Self::Error> {
        self.inner.submit_with_resources(request, resources).await
    }

    async fn call<O: harmos::operations::OperationDeclaration>(
        &self, request: O,
    ) -> Result<harmos::operations::Response<O::Response>, Self::Error> {
        use harmos::storage::operations::{AppendFile, CreateFile, FinishFile};
        let request_value = &request as &dyn std::any::Any;
        let create = request_value.downcast_ref::<CreateFile>();
        let append = request_value.downcast_ref::<AppendFile>();
        let finish = request_value.downcast_ref::<FinishFile>();
        let created = create.map(|value| &value.mutation);
        let appended = append.map(|value| &value.mutation);
        let finished = finish.map(|value| &value.mutation);
        let mutation = created.or(appended).or(finished);
        if let Some(mutation) = mutation {
            self.boundary.calls.lock().unwrap().push(mutation.clone());
        }
        if let Some(created) = create {
            let id = created.metadata.id.to_string();
            let length = created.metadata.expected_length.unwrap();
            self.boundary.lengths.lock().unwrap().insert(id, length);
        }
        let complete_report = append.is_some_and(|value| {
            let length = self.boundary.lengths.lock().unwrap().get(value.session.id.as_str()).copied();
            value.session.id.as_str().ends_with(".report") && length == Some(value.offset + value.bytes.len() as u64)
        });

        let response = self.inner.call(request).await?;
        if !complete_report {
            return Ok(response);
        }
        if self.boundary.advance_after_append.swap(false, Ordering::SeqCst) {
            let accepted = self.storage.info().unwrap().accepted;
            self.storage.flush_until(&accepted).await.unwrap();
            tokio::time::pause();
            tokio::time::advance(std::time::Duration::from_secs(51)).await;
            tokio::time::resume();
        }
        if self.boundary.lose_append_reply.swap(false, Ordering::SeqCst) {
            let lost = std::io::Error::other("controlled append reply loss");
            return Err(harmos::Error::backend("fixture append reply", harmos::BackendFailure::Transient, lost));
        }
        Ok(response)
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
    fn prune_objects<'a>(&'a self, through: &'a StorageCursor, max_entries: NonZeroUsize) -> StorageFuture<'a, bool> {
        self.inner.prune_objects(through, max_entries)
    }
    fn create_file<'a>(&'a self, metadata: &'a FileMetadata) -> StorageFuture<'a, StagedFile> {
        self.inner.create_file(metadata)
    }
    fn write_file<'a>(&'a self, staging: &'a StagedFile, offset: u64, bytes: &'a [u8]) -> StorageFuture<'a, ()> {
        self.inner.write_file(staging, offset, bytes)
    }
    fn abort_file<'a>(&'a self, staging: &'a StagedFile) -> StorageFuture<'a, ()> {
        self.inner.abort_file(staging)
    }
    fn supports_file_sessions(&self) -> bool {
        self.inner.supports_file_sessions()
    }
    fn create_production<'a>(&'a self, metadata: &'a FileMetadata) -> StorageFuture<'a, StagedFile> {
        self.inner.create_production(metadata)
    }
    fn fence_production<'a>(&'a self, file: &'a StagedFile, expected: u64, next: u64) -> StorageFuture<'a, ()> {
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
