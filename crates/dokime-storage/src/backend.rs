/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Pinned Library Backend Adapter

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

// Third Party Imports
use arrow_array::{BooleanArray, StructArray, UInt64Array};
use arrow_select::filter::filter_record_batch;
use harmos_storage::query::{
    FileSelection, ObjectSelection, RecordSelection, SchemaSelection, StreamSelection,
    columnar::{ArrowPage, ArrowScan, ArrowSeek, Cancellation, ColumnKind, StorageCapabilities},
    transform::{
        FieldPath, FieldPathExecution, ProjectExecution,
        project::{Project, Projection},
    },
};
use harmos_storage::{
    BackendFailure, BackendSnapshot, FileChunk, FileMetadata, FileReference, Identifier, ObjectCatalog, ObjectEntry, ObjectKey,
    ObjectPoint, ScanPage, SchemaKey, SchemaWrite, SchemaWriteReceipt, StagedFile, StorageBackend, StorageBatch, StorageCursor,
    StorageFuture, StorageSchema, StoreHistory, StoredRecord, StoredRow,
};

// Local Imports
use crate::{BatchSelection, Error, Store, scalar};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// One Product Execution Is One Library History; Callers Are Migrated In 0.H.5.
pub struct Backend {
    store: Store,
    execution: String,
    history: StoreHistory,
    snapshots: Mutex<BTreeMap<Vec<u8>, Retained>>,
}

#[derive(Clone)]
struct Retained {
    store: Store,
    _cancellation: Arc<crate::ReadCancellation>,
    _lease: Arc<Lease>,
}
struct Lease {
    pins: Arc<Mutex<BTreeMap<String, usize>>>,
    execution: String,
}
impl Drop for Lease {
    fn drop(&mut self) {
        if let Ok(mut pins) = self.pins.lock()
            && let Some(count) = pins.get_mut(&self.execution)
        {
            *count -= 1;
            if *count == 0 {
                pins.remove(&self.execution);
            }
        }
    }
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Backend {
    pub fn for_execution(store: Store, execution: &str) -> crate::Result<Self> {
        store.metadata(execution)?;
        let history = StoreHistory {
            store: Identifier::new(store.identity.to_string()).map_err(|e| Error::Invalid(e.to_string()))?,
            epoch: Identifier::new(execution).map_err(|e| Error::Invalid(e.to_string()))?,
        };
        Ok(Self {
            store,
            execution: execution.into(),
            history,
            snapshots: Mutex::new(BTreeMap::new()),
        })
    }
    fn cursor(&self) -> harmos_storage::Result<StorageCursor> {
        Ok(StorageCursor {
            history: self.history.clone(),
            sequence: self.store.metadata(&self.execution).map_err(failure)?.frontier,
        })
    }
    fn snapshot(&self, snapshot: &BackendSnapshot) -> harmos_storage::Result<Retained> {
        self.history.check(&snapshot.at.history).map_err(|e| failure(Error::Invalid(e.to_string())))?;
        let snapshots = self.snapshots.lock().map_err(|_| failure(Error::Data("restart after the backend snapshot lock failed")))?;
        snapshots.get(&snapshot.token).cloned().ok_or_else(|| failure(Error::Data("create a retained backend snapshot before reading")))
    }
}
impl StorageBackend for Backend {
    fn get_history(&self) -> &StoreHistory {
        &self.history
    }
    fn create_session(&self) -> StorageFuture<'_, StorageCursor> {
        Box::pin(async move { self.cursor() })
    }
    fn get_saved_position(&self) -> StorageFuture<'_, StorageCursor> {
        Box::pin(async move { self.cursor() })
    }
    fn delete_session(&self) -> StorageFuture<'_, ()> {
        Box::pin(async move { self.store.shutdown_samples().await.map_err(failure) })
    }
    fn create_snapshot<'a>(&'a self, at: &'a StorageCursor) -> StorageFuture<'a, BackendSnapshot> {
        Box::pin(async move {
            self.history.check(&at.history).map_err(|e| failure(Error::Invalid(e.to_string())))?;
            let (store, cancel) = self.store.read_session().map_err(failure)?;
            if at.sequence > store.metadata(&self.execution).map_err(failure)?.frontier {
                return unsupported_error("Create Snapshot: Select a Saved Position");
            }
            let connection = self.store.lock().map_err(failure)?;
            let exists: bool = connection
                .query_row("SELECT EXISTS(SELECT 1 FROM executions WHERE id=?1)", [&self.execution], |row| row.get(0))
                .map_err(|e| failure(e.into()))?;
            if !exists {
                return unsupported_error("Create Snapshot: Select an Existing Execution");
            }
            *self
                .store
                .snapshot_pins
                .lock()
                .map_err(|_| failure(Error::Data("restart after the snapshot pin lock failed")))?
                .entry(self.execution.clone())
                .or_default() += 1;
            let retained = Retained {
                store,
                _cancellation: Arc::new(cancel),
                _lease: Arc::new(Lease {
                    pins: self.store.snapshot_pins.clone(),
                    execution: self.execution.clone(),
                }),
            };
            drop(connection);
            let token = uuid::Uuid::now_v7().as_bytes().to_vec();
            self.snapshots
                .lock()
                .map_err(|_| failure(Error::Data("restart after the backend snapshot lock failed")))?
                .insert(token.clone(), retained);
            Ok(BackendSnapshot { at: at.clone(), token })
        })
    }
    fn delete_snapshot<'a>(&'a self, snapshot: &'a BackendSnapshot) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.history.check(&snapshot.at.history).map_err(|e| failure(Error::Invalid(e.to_string())))?;
            self.snapshots
                .lock()
                .map_err(|_| failure(Error::Data("restart after the backend snapshot lock failed")))?
                .remove(&snapshot.token);
            Ok(())
        })
    }
    fn capabilities(&self) -> StorageCapabilities {
        StorageCapabilities {
            native_arrow: true,
            ..StorageCapabilities::default()
        }
    }
    fn list_column_rows<'a>(
        &'a self, snapshot: &'a BackendSnapshot, read: &'a ArrowScan, cancel: &'a Cancellation,
    ) -> StorageFuture<'a, ArrowPage> {
        Box::pin(async move {
            cancel.check()?;
            let after = read.position(snapshot)?;
            let retained = self.snapshot(snapshot)?;
            let store = &retained.store;
            if read.source.kind != ColumnKind::Stream {
                return unsupported_error("Read Arrow: Select a Frozen Product Stream; Object and Record Adoption Belongs to 0.H.5");
            }
            // Native Arrow Has No Payload-Encoding Claim. The Pinned Row Interface Only Names PostcardV1.
            let stream = read.source.schema.id.to_string();
            let mut columns = read.columns.clone();
            if let Some(predicate) = &read.predicate {
                columns.push(predicate.column.clone());
            }
            if let Some(time) = &read.time {
                columns.push(time.column.clone());
            }
            let fields = columns.iter().filter_map(|name| name.strip_prefix("values.").map(str::to_owned)).collect::<Vec<_>>();
            let time = read.time.as_ref().filter(|t| t.column == "_storage.source_ns");
            let start = time.map_or(i64::MIN, |t| t.lower_ns);
            let end = time.map_or(i64::MAX, |t| t.upper_ns.saturating_sub(1));
            let project = Project {
                fields: read
                    .columns
                    .iter()
                    .map(|name| Projection {
                        field: FieldPath {
                            fields: name.split('.').map(str::to_owned).collect(),
                        },
                        name: name.clone(),
                    })
                    .collect(),
            };
            let mut output = None;
            let mut scanned = 0;
            let mut bytes_read = 0;
            let mut position = after;
            let mut complete = true;
            store
                .read_batches(
                    BatchSelection {
                        execution: &self.execution,
                        stream: &stream,
                        after,
                        through: snapshot.at.sequence,
                        start_ns: start,
                        end_ns: end,
                    },
                    &fields,
                    |batch| {
                        cancel.check()?;
                        let positions = batch
                            .column(0)
                            .as_any()
                            .downcast_ref::<StructArray>()
                            .unwrap()
                            .column_by_name("position")
                            .unwrap()
                            .as_any()
                            .downcast_ref::<UInt64Array>()
                            .unwrap();
                        let length = batch.num_rows().min(read.work.rows.get());
                        let mut keep = Vec::with_capacity(length);
                        let mut matched = Vec::new();
                        for row in 0..length {
                            scanned += 1;
                            position = positions.value(row);
                            let mut matches = true;
                            if let Some(predicate) = &read.predicate {
                                let path = FieldPath {
                                    fields: predicate.column.split('.').map(str::to_owned).collect(),
                                };
                                let value = scalar::number(&path.value(&batch, row)?);
                                matches &= value
                                    .is_some_and(|v| predicate.lower.is_none_or(|lo| v >= lo) && predicate.upper.is_none_or(|hi| v < hi));
                            }
                            if let Some(time) = &read.time {
                                let path = FieldPath {
                                    fields: time.column.split('.').map(str::to_owned).collect(),
                                };
                                let value = path.value(&batch, row)?;
                                matches &= matches!(value,scalar::ScalarValue::Signed(t) if t>=time.lower_ns&&t<time.upper_ns);
                            }
                            if matches {
                                matched.push(row);
                            }
                            keep.push(Some(matches));
                        }
                        let selected = filter_record_batch(&batch.slice(0, length), &BooleanArray::from(keep))?;
                        let selected = project.apply(&selected, &mut || Ok(()))?;
                        let available = selected.num_rows();
                        let mut length = available.min(read.limits.rows.get());
                        while length > 1 && selected.slice(0, length).get_array_memory_size() > read.limits.bytes.get() {
                            length /= 2;
                        }
                        if selected.slice(0, length).get_array_memory_size() > read.limits.bytes.get() {
                            return Err(Error::Data("increase the native Arrow response byte budget for this row"));
                        }
                        if batch.get_array_memory_size() > read.work.bytes.get() {
                            return Err(Error::Data("increase the native Arrow work byte budget for this batch"));
                        }
                        let selected = selected.slice(0, length);
                        bytes_read = batch.get_array_memory_size();
                        if length < available && length > 0 {
                            position = positions.value(matched[length - 1]);
                        }
                        complete = position >= snapshot.at.sequence;
                        if length < available || length < batch.num_rows() {
                            complete = false;
                        }
                        output = Some(selected);
                        Ok(false)
                    },
                )
                .map_err(failure)?;
            let batch = match output {
                Some(batch) => batch,
                None => {
                    complete = true;
                    position = snapshot.at.sequence;
                    let cut = store.metadata(&self.execution).map_err(failure)?;
                    let codec = store.source_codec(&cut, &(false, stream)).map_err(failure)?;
                    project.apply(&codec.batch(&[]).map_err(failure)?, &mut || Ok(()))?
                }
            };
            let next = if complete {
                None
            } else {
                Some(ArrowSeek {
                    snapshot: snapshot.token.clone(),
                    plan: read.identity()?,
                    position,
                })
            };
            Ok(ArrowPage {
                batch,
                next,
                complete,
                scanned,
                bytes_read,
                columns_read: columns,
                payload_rows: 0,
            })
        })
    }
    fn append_schemas(&self, _write: SchemaWrite) -> StorageFuture<'_, SchemaWriteReceipt> {
        unsupported(
            "append_schemas: The Pinned Backend PayloadEncoding Only Supports PostcardV1; Product Evidence Uses Frozen Protobuf. Adopt Protobuf Storage Before Moving Callers",
        )
    }
    fn append_object_revisions(&self, _batch: StorageBatch<ObjectEntry>) -> StorageFuture<'_, StorageCursor> {
        unsupported(
            "append_object_revisions: The Pinned Backend PayloadEncoding Only Supports PostcardV1; Product Evidence Uses Frozen Protobuf. Adopt Protobuf Storage Before Moving Callers",
        )
    }
    fn append_records(&self, _batch: StorageBatch<StoredRecord>) -> StorageFuture<'_, StorageCursor> {
        unsupported(
            "append_records: The Pinned Backend PayloadEncoding Only Supports PostcardV1; Product Evidence Uses Frozen Protobuf. Adopt Protobuf Storage Before Moving Callers",
        )
    }
    fn append_samples(&self, _batch: StorageBatch<StoredRow>) -> StorageFuture<'_, StorageCursor> {
        unsupported(
            "append_samples: The Pinned Backend PayloadEncoding Only Supports PostcardV1; Product Evidence Uses Frozen Protobuf. Adopt Protobuf Storage Before Moving Callers",
        )
    }
    fn append_file_references(&self, _batch: StorageBatch<FileReference>) -> StorageFuture<'_, StorageCursor> {
        unsupported(
            "append_file_references: Product File Resources Own Publication Positions and Writer Sessions; Adopt Their Mapping in 0.H.5 Before Using the Generic Port",
        )
    }
    fn list_schemas<'a>(
        &'a self, _snapshot: &'a BackendSnapshot, _selection: &'a SchemaSelection, _page: ScanPage,
    ) -> StorageFuture<'a, Vec<StorageSchema>> {
        unsupported(
            "list_schemas: The Pinned Backend PayloadEncoding Only Supports PostcardV1; Product Evidence Uses Frozen Protobuf. Adopt Protobuf Storage Before Moving Callers",
        )
    }
    fn list_object_revisions<'a>(
        &'a self, _snapshot: &'a BackendSnapshot, _selection: &'a ObjectSelection, _page: ScanPage,
    ) -> StorageFuture<'a, Vec<ObjectEntry>> {
        unsupported(
            "list_object_revisions: The Pinned Backend PayloadEncoding Only Supports PostcardV1; Product Evidence Uses Frozen Protobuf. Adopt Protobuf Storage Before Moving Callers",
        )
    }
    fn list_records<'a>(
        &'a self, _snapshot: &'a BackendSnapshot, _selection: &'a RecordSelection, _page: ScanPage,
    ) -> StorageFuture<'a, Vec<StoredRecord>> {
        unsupported(
            "list_records: The Pinned Backend PayloadEncoding Only Supports PostcardV1; Product Evidence Uses Frozen Protobuf. Adopt Protobuf Storage Before Moving Callers",
        )
    }
    fn list_samples<'a>(
        &'a self, _snapshot: &'a BackendSnapshot, _selection: &'a StreamSelection, _page: ScanPage,
    ) -> StorageFuture<'a, Vec<StoredRow>> {
        unsupported(
            "list_samples: The Pinned Backend PayloadEncoding Only Supports PostcardV1; Product Evidence Uses Frozen Protobuf. Adopt Protobuf Storage Before Moving Callers",
        )
    }
    fn list_files<'a>(
        &'a self, _snapshot: &'a BackendSnapshot, _selection: &'a FileSelection, _page: ScanPage,
    ) -> StorageFuture<'a, Vec<FileReference>> {
        unsupported(
            "list_files: Product File Resources Own Publication Positions and Writer Sessions; Adopt Their Mapping in 0.H.5 Before Using the Generic Port",
        )
    }
    fn get_object<'a>(&'a self, _snapshot: &'a BackendSnapshot, _point: &'a ObjectPoint) -> StorageFuture<'a, Option<ObjectEntry>> {
        unsupported(
            "get_object: The Pinned Backend PayloadEncoding Only Supports PostcardV1; Product Evidence Uses Frozen Protobuf. Adopt Protobuf Storage Before Moving Callers",
        )
    }
    fn get_file_bytes<'a>(
        &'a self, _snapshot: &'a BackendSnapshot, _file: &'a FileReference, _offset: u64, _max_bytes: NonZeroUsize,
    ) -> StorageFuture<'a, Vec<u8>> {
        unsupported(
            "get_file_bytes: Product File Resources Own Publication Positions and Writer Sessions; Adopt Their Mapping in 0.H.5 Before Using the Generic Port",
        )
    }
    fn delete_object_history<'a>(
        &'a self, _schema: &'a SchemaKey, _item: &'a ObjectKey, _through: &'a StorageCursor,
    ) -> StorageFuture<'a, ()> {
        unsupported(
            "delete_object_history: The Pinned Backend PayloadEncoding Only Supports PostcardV1; Product Evidence Uses Frozen Protobuf. Adopt Protobuf Storage Before Moving Callers",
        )
    }
    fn delete_expired_object_revisions<'a>(
        &'a self, _through: &'a StorageCursor, _max_entries: NonZeroUsize, _retention: &'a ObjectCatalog,
    ) -> StorageFuture<'a, bool> {
        unsupported(
            "delete_expired_object_revisions: The Pinned Backend PayloadEncoding Only Supports PostcardV1; Product Evidence Uses Frozen Protobuf. Adopt Protobuf Storage Before Moving Callers",
        )
    }
    fn create_file_staging<'a>(&'a self, _metadata: &'a FileMetadata) -> StorageFuture<'a, StagedFile> {
        unsupported(
            "create_file_staging: Product File Resources Own Publication Positions and Writer Sessions; Adopt Their Mapping in 0.H.5 Before Using the Generic Port",
        )
    }
    fn append_file_bytes<'a>(&'a self, _staging: &'a StagedFile, _offset: u64, _bytes: FileChunk) -> StorageFuture<'a, ()> {
        unsupported(
            "append_file_bytes: Product File Resources Own Publication Positions and Writer Sessions; Adopt Their Mapping in 0.H.5 Before Using the Generic Port",
        )
    }
    fn publish_file_production<'a>(
        &'a self, _staging: &'a StagedFile, _position: &'a StorageCursor, _length: u64, _sha256: [u8; 32],
    ) -> StorageFuture<'a, FileReference> {
        unsupported(
            "publish_file_production: Product File Resources Own Publication Positions and Writer Sessions; Adopt Their Mapping in 0.H.5 Before Using the Generic Port",
        )
    }
    fn delete_file_staging<'a>(&'a self, _staging: &'a StagedFile) -> StorageFuture<'a, ()> {
        unsupported(
            "delete_file_staging: Product File Resources Own Publication Positions and Writer Sessions; Adopt Their Mapping in 0.H.5 Before Using the Generic Port",
        )
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn failure(error: Error) -> harmos_storage::Error {
    harmos_storage::Error::backend("Process Product Arrow Storage", BackendFailure::Permanent, error)
}
fn unsupported_error<T>(action: &'static str) -> harmos_storage::Result<T> {
    Err(harmos_storage::Error::backend(action, BackendFailure::Unsupported, std::io::Error::new(std::io::ErrorKind::Unsupported, action)))
}
fn unsupported<'a, T: Send + 'a>(action: &'static str) -> StorageFuture<'a, T> {
    Box::pin(async move { unsupported_error(action) })
}

/* === Tests === */
