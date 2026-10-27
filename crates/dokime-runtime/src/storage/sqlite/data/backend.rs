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
use std::result::Result;
use std::{
    num::NonZeroUsize,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

// Third Party Imports
use harmos::{
    BackendFailure,
    storage::query::{FileSelection, ObjectSelection, ObjectVersions, RecordSelection, SchemaSelection, StreamSelection},
    storage::*,
};
use tokio::sync::{Mutex, Notify};

// Local Imports
use super::super::{Error, State};
use super::{Entry, Scan, entry::key};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Backend<C: ObjectCatalog> {
    catalog: std::marker::PhantomData<fn() -> C>,
    root: PathBuf,
    history: StoreHistory,
    state: State,
    snapshots: std::sync::Mutex<Box<[Arc<Pin>]>>,
    retention_through: Mutex<u64>,
}

// One Pinned Cut: Reads Filter Retired Rows Newer Than Its Generation, And
// Pruning Keeps Rows This Generation Can Still See Until Release Settles.
struct Pin {
    token: [u8; 16],
    at: u64,
    generation: u64,
    released: AtomicBool,
    reads: AtomicUsize,
    settled: Notify,
}

struct ActiveRead(Arc<Pin>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for ActiveRead {
    fn drop(&mut self) {
        if self.0.reads.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.0.settled.notify_waiters();
        }
    }
}

impl<C: ObjectCatalog> Backend<C> {
    pub async fn primary(root: PathBuf) -> harmos::storage::Result<Self> {
        C::validate()?;
        let state = State::open(&root).await.map_err(|error| failure("open SQLite", error))?;
        let history = state.data_history().await.map_err(|error| failure("initialize SQLite identity", error))?;
        let root = root
            .canonicalize()
            .map_err(|error| harmos::storage::Error::backend("resolve SQLite root", BackendFailure::Permanent, error))?;
        Ok(Self {
            catalog: std::marker::PhantomData,
            root,
            history,
            state,
            snapshots: std::sync::Mutex::new(Box::default()),
            retention_through: Mutex::new(0),
        })
    }

    pub fn local_state(&self) -> State {
        self.state.clone()
    }

    fn read(&self, snapshot: &BackendSnapshot) -> harmos::storage::Result<ActiveRead> {
        self.history.check(&snapshot.at.history)?;
        let snapshots = self.snapshots.lock().unwrap();
        let pin = snapshots
            .iter()
            .find(|pin| pin.token.as_slice() == snapshot.token.as_slice())
            .filter(|pin| !pin.released.load(Ordering::Acquire) && pin.at == snapshot.at.sequence)
            .ok_or(harmos::storage::Error::NotReady { domain: "SQLite snapshot view" })?
            .clone();
        pin.reads.fetch_add(1, Ordering::AcqRel);
        Ok(ActiveRead(pin))
    }

    async fn scan<T>(
        &self, snapshot: &BackendSnapshot, kind: &'static str, page: ScanPage, select: impl FnOnce(Scan) -> Result<Scan, Error>,
        convert: fn(Entry) -> Option<T>,
    ) -> harmos::storage::Result<Vec<T>> {
        let read = self.read(snapshot)?;
        let scan = select(Scan::new(kind, read.0.at, read.0.generation, page)).map_err(|error| failure("select SQLite scan", error))?;
        let entries = self.state.data_scan(scan).await.map_err(|error| failure("scan SQLite entries", error))?;
        drop(read);
        entries.into_iter().map(|entry| convert(entry).ok_or_else(|| invalid("stored collection differs from its index"))).collect()
    }
}

impl<C: ObjectCatalog> StorageBackend for Backend<C> {
    fn history(&self) -> &StoreHistory {
        &self.history
    }

    fn open(&self) -> StorageFuture<'_, StorageCursor> {
        Box::pin(async move {
            let history = self.history.clone();
            let cut = self.state.data_open(Some(history)).await.map_err(|error| failure("open SQLite", error))?;
            Ok(StorageCursor { history: cut.history, sequence: cut.sequence })
        })
    }

    fn cursor_saved(&self) -> StorageFuture<'_, StorageCursor> {
        Box::pin(async move {
            let cut = self.state.data_saved().await.map_err(|error| failure("read SQLite saved frontier", error))?;
            Ok(StorageCursor { history: cut.history, sequence: cut.sequence })
        })
    }

    fn close(&self) -> StorageFuture<'_, ()> {
        Box::pin(async move { self.state.close().await.map_err(|error| failure("close SQLite", error)) })
    }

    fn write_schemas(&self, batch: StorageBatch<StorageSchema>) -> StorageFuture<'_, StorageCursor> {
        Box::pin(async move {
            let root = self.root.clone();
            let cut = self.state.data_write(root, batch).await.map_err(|error| failure("write SQLite schemas", error))?;
            Ok(StorageCursor { history: cut.history, sequence: cut.sequence })
        })
    }

    fn write_objects(&self, batch: StorageBatch<ObjectEntry>) -> StorageFuture<'_, StorageCursor> {
        Box::pin(async move {
            let root = self.root.clone();
            let cut = self.state.data_write(root, batch).await.map_err(|error| failure("write SQLite objects", error))?;
            Ok(StorageCursor { history: cut.history, sequence: cut.sequence })
        })
    }

    fn write_records(&self, batch: StorageBatch<StoredRecord>) -> StorageFuture<'_, StorageCursor> {
        Box::pin(async move {
            let root = self.root.clone();
            let cut = self.state.data_write(root, batch).await.map_err(|error| failure("write SQLite records", error))?;
            Ok(StorageCursor { history: cut.history, sequence: cut.sequence })
        })
    }

    fn write_stream(&self, batch: StorageBatch<StoredRow>) -> StorageFuture<'_, StorageCursor> {
        Box::pin(async move {
            let root = self.root.clone();
            let cut = self.state.data_write(root, batch).await.map_err(|error| failure("write SQLite stream", error))?;
            Ok(StorageCursor { history: cut.history, sequence: cut.sequence })
        })
    }

    fn write_files(&self, batch: StorageBatch<FileReference>) -> StorageFuture<'_, StorageCursor> {
        Box::pin(async move {
            let root = self.root.clone();
            let cut = self.state.data_write(root, batch).await.map_err(|error| failure("write SQLite files", error))?;
            Ok(StorageCursor { history: cut.history, sequence: cut.sequence })
        })
    }

    fn open_snapshot<'a>(&'a self, at: &'a StorageCursor) -> StorageFuture<'a, BackendSnapshot> {
        Box::pin(async move {
            self.history.check(&at.history)?;
            let cut = self.state.data_saved().await.map_err(|error| failure("pin SQLite snapshot", error))?;
            if at.sequence > cut.sequence {
                return Err(invalid("snapshot exceeds saved frontier"));
            }
            let generation = cut.retention_generation;
            drop(cut);
            let token = *uuid::Uuid::now_v7().as_bytes();
            let mut snapshots = self.snapshots.lock().unwrap();
            let count = snapshots.len().checked_add(1).ok_or_else(|| invalid("snapshot count overflows"))?;
            let mut next = Vec::new();
            next.try_reserve_exact(count).map_err(|error| failure("allocate snapshot index", std::io::Error::other(error).into()))?;
            next.extend(snapshots.iter().cloned());
            next.push(Arc::new(Pin {
                token,
                at: at.sequence,
                generation,
                released: AtomicBool::new(false),
                reads: AtomicUsize::new(0),
                settled: Notify::new(),
            }));
            drop(std::mem::take(&mut *snapshots));
            *snapshots = next.into_boxed_slice();
            Ok(BackendSnapshot { at: at.clone(), token: token.to_vec() })
        })
    }

    fn release_snapshot<'a>(&'a self, snapshot: &'a BackendSnapshot) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            let Some(pin) = self.snapshots.lock().unwrap().iter().find(|pin| pin.token.as_slice() == snapshot.token.as_slice()).cloned()
            else {
                return Ok(());
            };
            // Close Read Admission, Then Let Accepted Reads Settle Before Pruning May Reclaim.
            pin.released.store(true, Ordering::Release);
            loop {
                let settled = pin.settled.notified();
                tokio::pin!(settled);
                settled.as_mut().enable();
                if pin.reads.load(Ordering::Acquire) == 0 {
                    break;
                }
                settled.await;
            }
            let mut snapshots = self.snapshots.lock().unwrap();
            let count = snapshots.iter().filter(|entry| !Arc::ptr_eq(entry, &pin)).count();
            let mut next = Vec::new();
            next.try_reserve_exact(count).map_err(|error| failure("release snapshot index", std::io::Error::other(error).into()))?;
            next.extend(snapshots.iter().filter(|entry| !Arc::ptr_eq(entry, &pin)).cloned());
            drop(std::mem::take(&mut *snapshots));
            *snapshots = next.into_boxed_slice();
            Ok(())
        })
    }

    fn read_schemas<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a SchemaSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<StorageSchema>> {
        Box::pin(self.scan(
            snapshot,
            "schema",
            page,
            |scan| Ok(scan.filter("schema_key", selection.schema.as_ref().map(serde_json::to_string).transpose()?)),
            |entry| match entry {
                Entry::Schema(value) => Some(value),
                _ => None,
            },
        ))
    }

    fn read_objects<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a ObjectSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<ObjectEntry>> {
        Box::pin(self.scan(
            snapshot,
            "object",
            page,
            |scan| {
                let mut scan = scan
                    .filter("schema_key", selection.schema.as_ref().map(serde_json::to_string).transpose()?)
                    .filter("item", selection.key.as_ref().map(|value| key(value.as_str())).transpose()?);
                scan.latest_objects = selection.versions == ObjectVersions::Latest;
                Ok(scan)
            },
            |entry| match entry {
                Entry::Object(value) => Some(value),
                _ => None,
            },
        ))
    }

    fn read_records<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a RecordSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<StoredRecord>> {
        Box::pin(self.scan(
            snapshot,
            "record",
            page,
            |scan| {
                let mut scan = scan
                    .filter("schema_key", Some(serde_json::to_string(&selection.schema)?))
                    .filter("scope", selection.scope.as_ref().map(|value| key(value.as_str())).transpose()?)
                    .filter("item", selection.item.as_ref().map(|value| key(value.as_str())).transpose()?);
                scan.source_window = selection.source_start.zip(selection.source_end);
                Ok(scan)
            },
            |entry| match entry {
                Entry::Record(value) => Some(value),
                _ => None,
            },
        ))
    }

    fn read_stream<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a StreamSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<StoredRow>> {
        Box::pin(self.scan(
            snapshot,
            "stream",
            page,
            |scan| {
                let mut scan = scan
                    .filter("schema_key", Some(serde_json::to_string(&selection.schema)?))
                    .filter("scope", selection.scope.as_ref().map(|value| key(value.as_str())).transpose()?)
                    .filter("item", selection.item.as_ref().map(|value| key(value.as_str())).transpose()?)
                    .filter("stream", selection.stream.as_ref().map(|value| key(value.as_str())).transpose()?);
                scan.source_window = selection.source_start.zip(selection.source_end);
                Ok(scan)
            },
            |entry| match entry {
                Entry::Stream(value) => Some(value),
                _ => None,
            },
        ))
    }

    fn read_files<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a FileSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<FileReference>> {
        Box::pin(self.scan(
            snapshot,
            "file",
            page,
            |scan| {
                Ok(scan
                    .filter("scope", selection.scope.as_ref().map(|value| key(value.as_str())).transpose()?)
                    .filter("item", selection.item.as_ref().map(|value| key(value.as_str())).transpose()?))
            },
            |entry| match entry {
                Entry::File(value) => Some(value),
                _ => None,
            },
        ))
    }

    fn read_object<'a>(&'a self, snapshot: &'a BackendSnapshot, point: &'a ObjectPoint) -> StorageFuture<'a, Option<ObjectEntry>> {
        Box::pin(async move {
            let read = self.read(snapshot)?;
            let result = self
                .state
                .data_object(read.0.at, read.0.generation, point.clone())
                .await
                .map_err(|error| failure("read SQLite object", error));
            drop(read);
            result
        })
    }

    fn read_file<'a>(
        &'a self, snapshot: &'a BackendSnapshot, file: &'a FileReference, offset: u64, max_bytes: NonZeroUsize,
    ) -> StorageFuture<'a, Vec<u8>> {
        Box::pin(async move {
            if !snapshot.at.covers(&file.position)? {
                return Err(invalid("file exceeds snapshot frontier"));
            }
            let read = self.read(snapshot)?;
            let result = self
                .state
                .data_read_file(self.root.clone(), file.clone(), read.0.at, offset, max_bytes)
                .await
                .map_err(|error| failure("read SQLite file", error));
            drop(read);
            result
        })
    }

    fn release_object_history<'a>(
        &'a self, schema: &'a SchemaKey, item: &'a ObjectKey, through: &'a StorageCursor,
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.history.check(&through.history)?;
            let mut retention = self.retention_through.lock().await;
            self.state
                .data_release_object_history(schema.clone(), item.clone(), through.clone())
                .await
                .map_err(|error| failure("release SQLite object history", error))?;
            // Revisit Previously Saved Revisions After A New Lifecycle Release.
            *retention = 0;
            Ok(())
        })
    }

    fn prune_objects<'a>(
        &'a self, through: &'a StorageCursor, max_entries: NonZeroUsize,
        retention_policy: fn(&SchemaKey) -> harmos::domain::Result<ObjectRetention>,
    ) -> StorageFuture<'a, bool> {
        Box::pin(async move {
            self.history.check(&through.history)?;
            let mut retention = self.retention_through.lock().await;
            let oldest = self.snapshots.lock().unwrap().iter().map(|pin| pin.generation).min();
            let after = if *retention <= through.sequence { *retention } else { 0 };
            let (logical, complete) = self
                .state
                .data_prune::<C>(through.clone(), after, oldest, max_entries, retention_policy)
                .await
                .map_err(|error| failure("prune SQLite objects", error))?;
            if logical {
                *retention = through.sequence;
            }
            Ok(complete)
        })
    }

    fn create_file<'a>(&'a self, metadata: &'a FileMetadata) -> StorageFuture<'a, StagedFile> {
        Box::pin(async move {
            self.state.data_create_file(self.root.clone(), metadata.clone()).await.map_err(|error| failure("create SQLite file", error))
        })
    }

    fn write_file<'a>(&'a self, staging: &'a StagedFile, offset: u64, bytes: FileChunk) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            let root = self.root.clone();
            let staging = staging.clone();
            self.state.data_write_file(root, staging, offset, bytes).await.map_err(|error| failure("write SQLite file", error))
        })
    }

    fn finish_file<'a>(
        &'a self, staging: &'a StagedFile, position: &'a StorageCursor, length: u64, sha256: [u8; 32],
    ) -> StorageFuture<'a, FileReference> {
        Box::pin(async move {
            self.state
                .data_finish_file(self.root.clone(), staging.clone(), position.clone(), length, sha256)
                .await
                .map_err(|error| failure("finish SQLite file", error))
        })
    }

    fn abort_file<'a>(&'a self, staging: &'a StagedFile) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.state.data_abort_file(self.root.clone(), staging.clone()).await.map_err(|error| failure("abort SQLite file", error))
        })
    }

    fn supports_file_sessions(&self) -> bool {
        true
    }

    fn create_production<'a>(&'a self, metadata: &'a FileMetadata) -> StorageFuture<'a, StagedFile> {
        Box::pin(async move {
            self.state.production_create(self.root.clone(), metadata.clone()).await.map_err(|error| failure("create production", error))
        })
    }

    fn fence_production<'a>(&'a self, file: &'a StagedFile, expected: u64, next: u64) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.history.check(&file.history)?;
            self.state.production_fence(file.clone(), expected, next).await.map_err(|error| failure("fence production", error))
        })
    }

    fn save_production(&self, checkpoint: StorageBatch<FileCheckpoint>) -> StorageFuture<'_, ()> {
        Box::pin(async move {
            let root = self.root.clone();
            self.state.production_save(root, checkpoint).await.map_err(|error| failure("save production", error))
        })
    }

    fn recover_productions(&self, limits: ReadLimits) -> StorageFuture<'_, Vec<FileCheckpoint>> {
        Box::pin(async move {
            self.state.production_recover(self.root.clone(), limits).await.map_err(|error| failure("recover productions", error))
        })
    }

    fn file_progress<'a>(&'a self, file: &'a StagedFile) -> StorageFuture<'a, FileProgress> {
        Box::pin(async move {
            self.state.production_progress(self.root.clone(), file.clone()).await.map_err(|error| failure("read file progress", error))
        })
    }

    fn read_production<'a>(&'a self, file: &'a StagedFile, offset: u64, max_bytes: NonZeroUsize) -> StorageFuture<'a, Vec<u8>> {
        Box::pin(async move {
            self.state
                .production_read(self.root.clone(), file.clone(), offset, max_bytes)
                .await
                .map_err(|error| failure("read production", error))
        })
    }

    fn abandon_file<'a>(&'a self, file: &'a StagedFile) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.state.production_abandon(self.root.clone(), file.clone()).await.map_err(|error| failure("abandon production", error))
        })
    }

    fn supports_file_deletion(&self) -> bool {
        true
    }

    fn delete_file<'a>(&'a self, file: &'a StagedFile) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.state.production_delete(self.root.clone(), file.clone()).await.map_err(|error| failure("delete file", error))
        })
    }

    fn supports_mutation_receipts(&self) -> bool {
        true
    }

    fn save_mutations(&self, receipts: StorageBatch<MutationReceipt>) -> StorageFuture<'_, ()> {
        Box::pin(async move { self.state.mutations_save(receipts).await.map_err(|error| failure("save mutation receipts", error)) })
    }

    fn recover_mutations(&self, limits: ReadLimits) -> StorageFuture<'_, Vec<MutationReceipt>> {
        Box::pin(async move { self.state.mutations_recover(limits).await.map_err(|error| failure("recover mutation receipts", error)) })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn invalid(reason: &'static str) -> harmos::storage::Error {
    harmos::storage::Error::InvalidInput { field: "SQLite storage", reason }
}

fn failure(operation: &'static str, error: Error) -> harmos::storage::Error {
    match error {
        Error::HarmosStorage(error) => error,
        Error::HarmosDomain(error) => error.into(),
        Error::Data(reason) => invalid(reason),
        other => {
            let kind = match &other {
                Error::Owned | Error::Conflict(_) | Error::Incompatible(_) => BackendFailure::Conflict,
                Error::Json(_) | Error::Corrupt(_) => BackendFailure::Corrupt,
                Error::Io(_) | Error::Sqlite(_) | Error::Publication(_) => BackendFailure::Transient,
                _ => BackendFailure::Permanent,
            };
            harmos::storage::Error::backend(operation, kind, other)
        }
    }
}

/* === Tests === */
