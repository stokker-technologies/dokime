/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod columns;
mod compound;
mod definition_mutations;
mod files;
mod index;
mod objects;
mod projection;
mod recovery;
mod rows;
mod snapshots;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    cell::Cell,
    fs::{File, OpenOptions},
    io,
    num::NonZeroUsize,
    path::Path,
    sync::{Arc, Mutex},
};

// Third Party Imports
use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior, params};
use tokio::sync;

// Local Imports
use harmos_query::columnar::ArrowPage;
use harmos_query::columnar::ArrowScan;
use harmos_query::columnar::Cancellation;
use harmos_storage::query::columnar::StorageCapabilities;
use harmos_storage::query::{FileSelection, ObjectSelection, RecordSelection, SchemaSelection, StreamSelection};
use harmos_storage::{
    BackendFailure, BackendSnapshot, CompoundExchange, Error, FileCheckpoint, FileChunk, FileMetadata, FilePhase, FileProgress,
    FileReference, Identifier, IdentityKind, IdentitySelection, MemorySize, MutationReceipt, ObjectCatalog, ObjectEntry, ObjectKey,
    ObjectPoint, ReadLimits, Result, ScanPage, SchemaKey, StagedFile, StorageBackend, StorageBatch, StorageCursor, StorageFuture,
    StorageSchema, StoreHistory, StoredIdentity, StoredRecord, StoredRow,
};

use harmos_query::index::IndexDefinition;
use harmos_query::index::IndexPage;
use harmos_query::index::IndexRead;
use harmos_storage::CompareExchange;
use harmos_storage::DefinitionOutcome;
use harmos_storage::MutationHeader;
use harmos_storage::ObjectExchange;
use harmos_storage::RecordView;
use harmos_storage::SchemaWrite;
use harmos_storage::SchemaWriteReceipt;
use harmos_storage::StorageBackends;
use harmos_storage::StorageWork;
use harmos_storage::query::projection::ProjectionDefinition;
use harmos_storage::query::projection::ProjectionPage;
use harmos_storage::query::projection::ProjectionPublication;
use harmos_storage::query::projection::ProjectionRead;
use harmos_storage::query::projection::ProjectionStatus;

/* === Constants === */

const FORMAT: &str = "dokime.scoped-objects.v1";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy)]
struct BackendSettings {
    pub chunk_rows: NonZeroUsize,
    pub chunk_bytes: NonZeroUsize,
}

#[derive(Clone)]
pub struct Backend {
    history: StoreHistory,
    state: Arc<sync::Mutex<State>>,
}

pub struct BackendLease(Arc<Mutex<Option<File>>>);

impl BackendLease {
    pub fn close(&self) -> Result<()> {
        self.0
            .lock()
            .map_err(|source| Error::backend("Close Storage Lease", BackendFailure::Permanent, io::Error::other(source.to_string())))?
            .take();
        Ok(())
    }
}

impl Drop for BackendLease {
    fn drop(&mut self) {
        if let Ok(mut lease) = self.0.lock() {
            lease.take();
        }
    }
}

struct State {
    connection: Connection,
    lease: Arc<Mutex<Option<File>>>,
    history: StoreHistory,
    settings: BackendSettings,
    catalog: ObjectCatalog,
    closed: bool,
    recovery_closed: bool,
    work: Cell<StorageWork>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Backend {
    pub(crate) fn validate_root(root: &Path) -> crate::Result<()> {
        let path = root.join("storage.sqlite3");
        if !path.exists() {
            return Ok(());
        }
        let incompatible = |_| crate::Error::Incompatible(root.display().to_string());
        let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(incompatible)?;
        validate_format(&connection).map_err(|_| crate::Error::Incompatible(root.display().to_string()))
    }

    pub fn open(store: &crate::Store, catalog: ObjectCatalog) -> crate::Result<(StorageBackends, BackendLease)> {
        let history = store.storage_history()?;
        let history = StoreHistory {
            store: Identifier::new(history.store).map_err(|source| Error::backend("Open Root History", BackendFailure::Corrupt, source))?,
            epoch: Identifier::new(history.epoch).map_err(|source| Error::backend("Open Root History", BackendFailure::Corrupt, source))?,
        };
        let settings = store.storage_settings()?;
        let primary = Self::new(
            store.database.with_file_name("storage.sqlite3"),
            history,
            BackendSettings {
                chunk_rows: settings.flush_rows,
                chunk_bytes: settings.pending_bytes,
            },
        )?;
        primary.state.try_lock().map_err(|source| Error::backend("Register Object Catalog", BackendFailure::Conflict, source))?.catalog =
            catalog;
        let state = recovery::RecoveryBackend(primary.clone());
        let lease = BackendLease(
            primary
                .state
                .try_lock()
                .map_err(|source| Error::backend("Open Storage Lease", BackendFailure::Conflict, source))?
                .lease
                .clone(),
        );
        Ok((
            StorageBackends {
                primary: Arc::new(primary),
                state: Arc::new(state),
            },
            lease,
        ))
    }

    fn new(path: impl AsRef<Path>, history: StoreHistory, settings: BackendSettings) -> Result<Self> {
        let path = path.as_ref();
        let mut lock_path = path.as_os_str().to_os_string();
        lock_path.push(".lock");
        let lease = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)
            .map_err(|source| Error::backend("open SQLite lease", BackendFailure::Permanent, source))?;
        lease.try_lock().map_err(|source| Error::backend("lock SQLite storage", BackendFailure::Conflict, source))?;
        Self::connect(Connection::open(path).map_err(db)?, history, settings, Some(lease))
    }

    fn connect(mut connection: Connection, history: StoreHistory, settings: BackendSettings, lease: Option<File>) -> Result<Self> {
        integer(settings.chunk_rows.get() as u64)?;
        integer(settings.chunk_bytes.get() as u64)?;
        validate_format(&connection)?;
        let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0)).map_err(db)?;
        if version == 1 {
            let stored: Option<String> = connection
                .query_row("SELECT value FROM storage_meta WHERE name='history'", [], |row| row.get(0))
                .optional()
                .map_err(|source| {
                    Error::backend("open SQLite metadata; back up the original data and reset explicitly", BackendFailure::Corrupt, source)
                })?;
            if stored.as_deref() != Some(text(&history)?.as_str()) {
                return Err(incompatible());
            }
        }
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;").map_err(db)?;
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db)?;
        tx.execute_batch("CREATE TABLE IF NOT EXISTS recovery_entries(sequence INTEGER PRIMARY KEY, data BLOB NOT NULL);
            CREATE TABLE IF NOT EXISTS recovery_snapshots(sequence INTEGER PRIMARY KEY, header BLOB NOT NULL, payload BLOB NOT NULL, published INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS storage_meta(name TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS storage_rows(position INTEGER PRIMARY KEY, kind INTEGER NOT NULL, schema_key TEXT, schema_id TEXT, item TEXT, scope TEXT, stream TEXT, time_ns INTEGER, data BLOB NOT NULL);
            CREATE INDEX IF NOT EXISTS storage_source ON storage_rows(kind,schema_key,position);
            CREATE INDEX IF NOT EXISTS storage_object ON storage_rows(kind,schema_id,item,position);
            CREATE TABLE IF NOT EXISTS arrow_chunks(id INTEGER PRIMARY KEY, source TEXT NOT NULL, first INTEGER NOT NULL, last INTEGER NOT NULL, rows INTEGER NOT NULL, file TEXT NOT NULL, block BLOB NOT NULL);
            CREATE INDEX IF NOT EXISTS arrow_source ON arrow_chunks(source,last,first);
            CREATE TABLE IF NOT EXISTS arrow_columns(chunk INTEGER NOT NULL REFERENCES arrow_chunks(id), name TEXT NOT NULL, data INTEGER NOT NULL, minimum REAL, maximum REAL, time_min INTEGER, time_max INTEGER, PRIMARY KEY(chunk,name));
            CREATE TABLE IF NOT EXISTS deleted_files(id TEXT PRIMARY KEY,generation INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS storage_files(id TEXT PRIMARY KEY, scope TEXT, metadata BLOB NOT NULL, length INTEGER NOT NULL, finished BLOB);
            CREATE INDEX IF NOT EXISTS storage_file_scope ON storage_files(scope,id);
            CREATE TABLE IF NOT EXISTS storage_file_chunks(id TEXT NOT NULL REFERENCES storage_files(id), offset INTEGER NOT NULL, data BLOB NOT NULL, PRIMARY KEY(id,offset));
            CREATE TABLE IF NOT EXISTS storage_file_sessions(id TEXT PRIMARY KEY REFERENCES storage_files(id), generation INTEGER NOT NULL, abandoned INTEGER NOT NULL, checkpoint BLOB);
            CREATE TABLE IF NOT EXISTS storage_mutations(id TEXT PRIMARY KEY, data BLOB NOT NULL);
            CREATE TABLE IF NOT EXISTS storage_object_deletions(position INTEGER PRIMARY KEY,schema_id TEXT NOT NULL,item TEXT NOT NULL);
            CREATE INDEX IF NOT EXISTS storage_object_deletion_key ON storage_object_deletions(schema_id,item,position);
            CREATE TABLE IF NOT EXISTS storage_row_deletions(position INTEGER PRIMARY KEY,deleted_at INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS storage_retained_snapshots(token BLOB PRIMARY KEY,position INTEGER NOT NULL,generation INTEGER NOT NULL,scope TEXT,owner TEXT);
            CREATE TABLE IF NOT EXISTS storage_scope_deletions(scope TEXT NOT NULL,position INTEGER PRIMARY KEY);
            CREATE TEMP TABLE storage_snapshot_leases(token BLOB PRIMARY KEY,position INTEGER NOT NULL,generation INTEGER NOT NULL,scope TEXT);
            CREATE TEMP VIEW storage_retained_cuts AS SELECT position,generation,scope FROM storage_retained_snapshots UNION ALL SELECT position,generation,scope FROM storage_snapshot_leases;
            CREATE TABLE IF NOT EXISTS storage_compound_ranges(first INTEGER PRIMARY KEY,last INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS storage_indexes(name TEXT PRIMARY KEY,revision INTEGER NOT NULL,source TEXT NOT NULL,definition BLOB NOT NULL);
            CREATE TABLE IF NOT EXISTS storage_index_entries(name TEXT NOT NULL,revision INTEGER NOT NULL,key BLOB NOT NULL,position INTEGER NOT NULL,PRIMARY KEY(name,revision,key,position)) WITHOUT ROWID;
            CREATE TABLE IF NOT EXISTS projection_definitions(name TEXT NOT NULL,revision INTEGER NOT NULL,definition BLOB NOT NULL,retired INTEGER,PRIMARY KEY(name,revision));
            CREATE TABLE IF NOT EXISTS projection_active(name TEXT PRIMARY KEY,revision INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS projection_cuts(name TEXT NOT NULL,revision INTEGER NOT NULL,frontier INTEGER NOT NULL,rows INTEGER NOT NULL,changes INTEGER NOT NULL,schema BLOB NOT NULL,PRIMARY KEY(name,revision,frontier));
            CREATE TABLE IF NOT EXISTS projection_pages(name TEXT NOT NULL,revision INTEGER NOT NULL,frontier INTEGER NOT NULL,first_row INTEGER NOT NULL,rows INTEGER NOT NULL,data BLOB NOT NULL,PRIMARY KEY(name,revision,frontier,first_row));
            CREATE TABLE IF NOT EXISTS projection_changes(name TEXT NOT NULL,revision INTEGER NOT NULL,frontier INTEGER NOT NULL,ordinal INTEGER NOT NULL,key BLOB NOT NULL,PRIMARY KEY(name,revision,frontier,ordinal));
            CREATE TABLE IF NOT EXISTS projection_bucket_state(name TEXT NOT NULL,revision INTEGER NOT NULL,frontier INTEGER NOT NULL,source_rows INTEGER NOT NULL,output_rows INTEGER NOT NULL,PRIMARY KEY(name,revision)) WITHOUT ROWID;
            CREATE TABLE IF NOT EXISTS projection_bucket_members(name TEXT NOT NULL,revision INTEGER NOT NULL,identity BLOB NOT NULL,bucket_key BLOB NOT NULL,data BLOB NOT NULL,PRIMARY KEY(name,revision,bucket_key,identity),UNIQUE(name,revision,identity)) WITHOUT ROWID;
            CREATE TABLE IF NOT EXISTS projection_bucket_outputs(name TEXT NOT NULL,revision INTEGER NOT NULL,key BLOB NOT NULL,frontier INTEGER NOT NULL,data BLOB,PRIMARY KEY(name,revision,key,frontier)) WITHOUT ROWID;
            CREATE TABLE IF NOT EXISTS projection_bucket_cuts(name TEXT NOT NULL,revision INTEGER NOT NULL,frontier INTEGER NOT NULL,PRIMARY KEY(name,revision,frontier)) WITHOUT ROWID;
            CREATE TABLE IF NOT EXISTS object_receipts(position INTEGER PRIMARY KEY,fingerprint BLOB NOT NULL,outcome BLOB NOT NULL);
            CREATE TABLE IF NOT EXISTS storage_object_gaps(position INTEGER PRIMARY KEY,schema_key TEXT NOT NULL,schema_id TEXT NOT NULL,item TEXT NOT NULL,scope TEXT);
            CREATE INDEX IF NOT EXISTS storage_object_gap_selection ON storage_object_gaps(schema_key,item,position);
            CREATE TABLE IF NOT EXISTS object_releases(schema_key TEXT NOT NULL, item TEXT NOT NULL, through INTEGER NOT NULL, PRIMARY KEY(schema_key,item));").map_err(db)?;
        let encoded = text(&history)?;
        let stored: Option<String> =
            tx.query_row("SELECT value FROM storage_meta WHERE name='history'", [], |row| row.get(0)).optional().map_err(db)?;
        if stored.as_ref().is_some_and(|stored| stored != &encoded) {
            return Err(incompatible());
        }
        tx.execute("INSERT OR IGNORE INTO storage_meta VALUES('format',?1)", [FORMAT]).map_err(db)?;
        tx.execute("INSERT OR IGNORE INTO storage_meta VALUES('history',?1)", [&encoded]).map_err(db)?;
        tx.execute_batch("INSERT OR IGNORE INTO storage_meta VALUES('deletion_generation','0'); INSERT OR IGNORE INTO storage_meta VALUES('recovery_written','0'); INSERT OR IGNORE INTO storage_meta VALUES('recovery_saved','0'); INSERT OR IGNORE INTO storage_meta VALUES('saved','0'); INSERT OR IGNORE INTO storage_meta VALUES('floor','0'); INSERT OR IGNORE INTO storage_meta VALUES('object_receipt_floor','0'); PRAGMA user_version=1;").map_err(db)?;
        tx.commit().map_err(db)?;
        columns::collect_orphans(&connection)?;
        Ok(Self {
            history: history.clone(),
            state: Arc::new(sync::Mutex::new(State {
                connection,
                lease: Arc::new(Mutex::new(lease)),
                history,
                settings,
                catalog: ObjectCatalog::new(),
                closed: false,
                recovery_closed: false,
                work: Cell::default(),
            })),
        })
    }

    async fn call<T: Send + 'static>(&self, action: impl FnOnce(&mut State) -> Result<T> + Send + 'static) -> Result<T> {
        let mut state = self.state.clone().lock_owned().await;
        tokio::task::spawn_blocking(move || action(&mut state))
            .await
            .map_err(|source| Error::backend("SQLite worker", BackendFailure::Permanent, source))?
    }
}

impl State {
    fn check(&self) -> Result<()> {
        self.check_lease()?;
        if self.closed {
            return Err(Error::Closing { domain: "SQLite storage" });
        }
        Ok(())
    }

    fn check_lease(&self) -> Result<()> {
        if self
            .lease
            .lock()
            .map_err(|source| Error::backend("Read Storage Lease", BackendFailure::Permanent, io::Error::other(source.to_string())))?
            .is_none()
        {
            return Err(Error::Closing { domain: "SQLite storage" });
        }
        Ok(())
    }

    fn cursor(&self) -> Result<StorageCursor> {
        Ok(StorageCursor {
            history: self.history.clone(),
            sequence: meta(&self.connection, "saved")?,
        })
    }

    fn snapshot(&self, snapshot: &BackendSnapshot) -> Result<()> {
        self.check()?;
        self.history.check(&snapshot.at.history)?;
        snapshots::read(&self.connection, snapshot)?;
        Ok(())
    }
}

impl StorageBackend for Backend {
    fn mutate<'a>(&'a self, exchange: &'a CompoundExchange) -> StorageFuture<'a, MutationReceipt> {
        Box::pin(async move {
            exchange.validate()?;
            let position = exchange.position.clone();
            let items = exchange.items.clone();
            let header = exchange.mutation().clone();
            let receipt = self.call(move |state| compound::commit(state, &position, &items, &header)).await?;
            exchange.replay(&receipt)?;
            Ok(receipt)
        })
    }

    fn get_work(&self) -> StorageFuture<'_, StorageWork> {
        Box::pin(self.call(|state| Ok(state.work.get())))
    }

    fn get_history(&self) -> &StoreHistory {
        &self.history
    }

    fn capabilities(&self) -> StorageCapabilities {
        StorageCapabilities {
            file_sessions: true,
            file_deletion: true,
            mutation_receipts: true,
            native_arrow: true,
            columnar_append: false,
            native_index: true,
            durable_projections: true,
            conditional_objects: true,
            compound_mutations: true,
        }
    }

    fn create_session(&self) -> StorageFuture<'_, StorageCursor> {
        Box::pin(self.call(|state| {
            state.check_lease()?;
            state.closed = false;
            compound::reclaim(state)?;
            state.cursor()
        }))
    }
    fn get_saved_position(&self) -> StorageFuture<'_, StorageCursor> {
        Box::pin(self.call(|state| {
            state.check()?;
            state.cursor()
        }))
    }
    fn delete_session(&self) -> StorageFuture<'_, ()> {
        Box::pin(self.call(|state| {
            state.connection.execute("DELETE FROM storage_snapshot_leases", []).map_err(db)?;
            compound::reclaim(state)?;
            state.closed = true;
            state
                .lease
                .lock()
                .map_err(|source| Error::backend("Close Storage Lease", BackendFailure::Permanent, io::Error::other(source.to_string())))?
                .take();
            Ok(())
        }))
    }

    fn list_record_views<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a RecordSelection, page: ScanPage,
    ) -> StorageFuture<'a, Option<Vec<RecordView>>> {
        let (snapshot, selection) = (snapshot.clone(), selection.clone());
        Box::pin(self.call(move |state| {
            rows::list_records(state, &snapshot, &selection, page)?.into_iter().map(RecordView::new).collect::<Result<Vec<_>>>().map(Some)
        }))
    }
    fn get_record_position<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a RecordSelection, offset: u64,
    ) -> StorageFuture<'a, Option<u64>> {
        let (snapshot, selection) = (snapshot.clone(), selection.clone());
        Box::pin(self.call(move |state| {
            let mut after = 0;
            let mut remaining = offset;
            while remaining > 0 {
                let count = usize::try_from(remaining).unwrap_or(usize::MAX).min(state.settings.chunk_rows.get());
                let page = ScanPage {
                    after,
                    rows: NonZeroUsize::new(count).unwrap(),
                    bytes: state.settings.chunk_bytes,
                };
                let records = rows::list_records(state, &snapshot, &selection, page)?;
                if records.is_empty() {
                    return Ok(Some(snapshot.at.sequence));
                }
                remaining -= records.len() as u64;
                after = records.last().unwrap().position.sequence;
            }
            Ok(Some(after))
        }))
    }
    fn advance_projection<'a>(
        &'a self, _snapshot: &'a BackendSnapshot, _name: &'a Identifier, _expected_revision: Option<u64>, _work: ReadLimits,
        _cancel: &'a Cancellation,
    ) -> StorageFuture<'a, Option<u64>> {
        Box::pin(async { Ok(None) })
    }
    fn delete_file<'a>(&'a self, file: &'a StagedFile) -> StorageFuture<'a, ()> {
        let file = file.clone();
        Box::pin(self.call(move |state| files::delete(state, &file)))
    }

    fn append_schemas(&self, write: SchemaWrite) -> StorageFuture<'_, SchemaWriteReceipt> {
        Box::pin(self.call(move |state| match write {
            SchemaWrite::Append(values) => {
                rows::append_schemas(state, values.as_slice()).map(|saved| SchemaWriteReceipt { saved, mutation: None })
            }
            SchemaWrite::Set(exchange) => rows::set_schema(state, &exchange),
        }))
    }
    fn append_object_revisions(&self, values: StorageBatch<ObjectEntry>) -> StorageFuture<'_, StorageCursor> {
        Box::pin(self.call(move |state| rows::append_object_revisions(state, values.as_slice())))
    }
    fn append_records(&self, values: StorageBatch<StoredRecord>) -> StorageFuture<'_, StorageCursor> {
        Box::pin(self.call(move |state| rows::append_records(state, values.as_slice())))
    }
    fn append_samples(&self, values: StorageBatch<StoredRow>) -> StorageFuture<'_, StorageCursor> {
        Box::pin(self.call(move |state| rows::append_samples(state, values.as_slice())))
    }
    fn append_file_references(&self, values: StorageBatch<FileReference>) -> StorageFuture<'_, StorageCursor> {
        Box::pin(self.call(move |state| rows::append_file_references(state, values.as_slice())))
    }

    fn create_snapshot<'a>(&'a self, at: &'a StorageCursor) -> StorageFuture<'a, BackendSnapshot> {
        let at = at.clone();
        Box::pin(self.call(move |state| snapshots::create(state, &at)))
    }

    fn delete_snapshot<'a>(&'a self, snapshot: &'a BackendSnapshot) -> StorageFuture<'a, ()> {
        let snapshot = snapshot.clone();
        Box::pin(self.call(move |state| snapshots::close(state, &snapshot)))
    }

    fn retain_snapshot<'a>(&'a self, snapshot: &'a BackendSnapshot, scope: Option<&'a Identifier>) -> StorageFuture<'a, BackendSnapshot> {
        let (snapshot, scope) = (snapshot.clone(), scope.cloned());
        Box::pin(self.call(move |state| snapshots::retain(state, &snapshot, scope.as_ref())))
    }

    fn open_retained_snapshot<'a>(&'a self, retained: &'a BackendSnapshot) -> StorageFuture<'a, BackendSnapshot> {
        let retained = retained.clone();
        Box::pin(self.call(move |state| snapshots::open(state, &retained)))
    }

    fn release_retained_snapshot<'a>(&'a self, retained: &'a BackendSnapshot) -> StorageFuture<'a, ()> {
        let retained = retained.clone();
        Box::pin(self.call(move |state| snapshots::release(state, &retained)))
    }

    fn watch_frontier<'a>(&'a self, after: &'a StorageCursor, through: &'a StorageCursor) -> StorageFuture<'a, StorageCursor> {
        let (after, through) = (after.clone(), through.clone());
        Box::pin(self.call(move |state| snapshots::frontier(state, &after, &through)))
    }

    fn list_schemas<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a SchemaSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<StorageSchema>> {
        let (snapshot, selection) = (snapshot.clone(), selection.clone());
        Box::pin(self.call(move |state| rows::list_schemas(state, &snapshot, &selection, page)))
    }
    fn list_object_revisions<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a ObjectSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<ObjectEntry>> {
        let (snapshot, selection) = (snapshot.clone(), selection.clone());
        Box::pin(self.call(move |state| rows::list_object_revisions(state, &snapshot, &selection, page)))
    }
    fn list_records<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a RecordSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<StoredRecord>> {
        let (snapshot, selection) = (snapshot.clone(), selection.clone());
        Box::pin(self.call(move |state| rows::list_records(state, &snapshot, &selection, page)))
    }
    fn list_samples<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a StreamSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<StoredRow>> {
        let (snapshot, selection) = (snapshot.clone(), selection.clone());
        Box::pin(self.call(move |state| rows::list_samples(state, &snapshot, &selection, page)))
    }
    fn list_identities<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a IdentitySelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<StoredIdentity>> {
        let (snapshot, selection) = (snapshot.clone(), selection.clone());
        Box::pin(self.call(move |state| rows::list_identities(state, &snapshot, &selection, page)))
    }
    fn list_files<'a>(
        &'a self, snapshot: &'a BackendSnapshot, selection: &'a FileSelection, page: ScanPage,
    ) -> StorageFuture<'a, Vec<FileReference>> {
        let (snapshot, selection) = (snapshot.clone(), selection.clone());
        Box::pin(self.call(move |state| rows::list_files(state, &snapshot, &selection, page)))
    }
    fn get_object<'a>(&'a self, snapshot: &'a BackendSnapshot, point: &'a ObjectPoint) -> StorageFuture<'a, Option<ObjectEntry>> {
        let (snapshot, point) = (snapshot.clone(), point.clone());
        Box::pin(self.call(move |state| rows::get_object(state, &snapshot, &point)))
    }
    fn delete_object_history<'a>(
        &'a self, schema: &'a SchemaKey, item: &'a ObjectKey, through: &'a StorageCursor,
    ) -> StorageFuture<'a, ()> {
        let (schema, item, through) = (schema.clone(), item.clone(), through.clone());
        Box::pin(self.call(move |state| {
            state.check()?; state.history.check(&through.history)?;
            if through.sequence > state.cursor()?.sequence { return Err(invalid("release is ahead of durable data")); }
            state.connection.execute("INSERT INTO object_releases VALUES(?1,?2,?3) ON CONFLICT(schema_key,item) DO UPDATE SET through=max(through,excluded.through)", params![text(&schema)?, item.as_str(), integer(through.sequence)?]).map_err(db)?;
            Ok(())
        }))
    }
    fn delete_expired_object_revisions<'a>(
        &'a self, through: &'a StorageCursor, maximum: NonZeroUsize, retention: &'a ObjectCatalog,
    ) -> StorageFuture<'a, bool> {
        let (through, retention) = (through.clone(), retention.clone());
        Box::pin(self.call(move |state| rows::prune(state, &through, maximum, &retention)))
    }

    fn list_column_rows<'a>(
        &'a self, snapshot: &'a BackendSnapshot, read: &'a ArrowScan, cancel: &'a Cancellation,
    ) -> StorageFuture<'a, ArrowPage> {
        let (snapshot, read, cancel) = (snapshot.clone(), read.clone(), cancel.clone());
        Box::pin(self.call(move |state| columns::scan(state, &snapshot, &read, &cancel, false)))
    }

    fn set_index_definition<'a>(
        &'a self, definition: &'a IndexDefinition, expected: Option<u64>, rebuild: bool, work: ReadLimits, cancel: &'a Cancellation,
        mutation: Option<&'a MutationHeader>,
    ) -> StorageFuture<'a, DefinitionOutcome> {
        let mutation = mutation.cloned();
        let (definition, cancel) = (definition.clone(), cancel.clone());
        Box::pin(self.call(move |state| index::register(state, &definition, expected, rebuild, work, &cancel, mutation.as_ref())))
    }
    fn get_index_definition<'a>(&'a self, name: &'a Identifier) -> StorageFuture<'a, (IndexDefinition, u64)> {
        let name = name.clone();
        Box::pin(self.call(move |state| {
            state.check()?;
            index::definition(&state.connection, &name)
        }))
    }
    fn list_index_entries<'a>(
        &'a self, snapshot: &'a BackendSnapshot, read: &'a IndexRead, cancel: &'a Cancellation,
    ) -> StorageFuture<'a, IndexPage> {
        let (snapshot, read, cancel) = (snapshot.clone(), read.clone(), cancel.clone());
        Box::pin(self.call(move |state| index::scan(state, &snapshot, &read, &cancel)))
    }
    fn set_projection_definition<'a>(
        &'a self, definition: &'a ProjectionDefinition, expected: Option<u64>, rebuild: bool, mutation: Option<&'a MutationHeader>,
    ) -> StorageFuture<'a, DefinitionOutcome> {
        let mutation = mutation.cloned();
        let definition = definition.clone();
        Box::pin(self.call(move |state| projection::register(state, &definition, expected, rebuild, mutation.as_ref())))
    }
    fn get_projection_definition<'a>(
        &'a self, name: &'a Identifier, revision: Option<u64>,
    ) -> StorageFuture<'a, (ProjectionDefinition, u64)> {
        let name = name.clone();
        Box::pin(self.call(move |state| {
            state.check()?;
            match revision {
                Some(revision) => Ok((projection::definition_at(&state.connection, &name, revision)?, revision)),
                None => projection::definition(&state.connection, &name),
            }
        }))
    }

    fn get_projection_status<'a>(
        &'a self, name: &'a Identifier, revision: u64, frontier: Option<&'a StorageCursor>,
    ) -> StorageFuture<'a, Option<ProjectionStatus>> {
        let (name, frontier) = (name.clone(), frontier.cloned());
        Box::pin(self.call(move |state| {
            let frontier = match frontier {
                Some(frontier) => frontier,
                None => match projection::frontier(state, &name, revision)? {
                    Some(frontier) => frontier,
                    None => return Ok(None),
                },
            };
            projection::status(state, &name, revision, &frontier)
        }))
    }

    fn set_projection_output(&self, publication: ProjectionPublication) -> StorageFuture<'_, ()> {
        Box::pin(self.call(move |state| projection::publish(state, &publication)))
    }
    fn list_projection_rows<'a>(&'a self, read: &'a ProjectionRead) -> StorageFuture<'a, ProjectionPage> {
        let read = read.clone();
        Box::pin(self.call(move |state| projection::read(state, &read)))
    }
    fn delete_projection_history<'a>(&'a self, name: &'a Identifier, revision: u64, through: &'a StorageCursor) -> StorageFuture<'a, ()> {
        let (name, through) = (name.clone(), through.clone());
        Box::pin(self.call(move |state| projection::retire(state, &name, revision, &through)))
    }
    fn set_object<'a>(&'a self, exchange: &'a ObjectExchange) -> StorageFuture<'a, CompareExchange> {
        let exchange = exchange.clone();
        Box::pin(self.call(move |state| objects::exchange(state, &exchange)))
    }
    fn delete_object_receipts<'a>(&'a self, through: &'a StorageCursor) -> StorageFuture<'a, ()> {
        let through = through.clone();
        Box::pin(self.call(move |state| objects::retire(state, &through)))
    }
    fn create_file_staging<'a>(&'a self, metadata: &'a FileMetadata) -> StorageFuture<'a, StagedFile> {
        let metadata = metadata.clone();
        Box::pin(self.call(move |state| files::create(state, &metadata)))
    }
    fn create_file_production<'a>(&'a self, metadata: &'a FileMetadata) -> StorageFuture<'a, StagedFile> {
        let metadata = metadata.clone();
        Box::pin(self.call(move |state| files::production(state, &metadata)))
    }
    fn set_file_writer_generation<'a>(&'a self, file: &'a StagedFile, expected: u64, next: u64) -> StorageFuture<'a, ()> {
        let file = file.clone();
        Box::pin(self.call(move |state| files::fence(state, &file, expected, next)))
    }
    fn set_file_production_checkpoints(&self, checkpoints: StorageBatch<FileCheckpoint>) -> StorageFuture<'_, ()> {
        Box::pin(self.call(move |state| files::checkpoints(state, checkpoints.as_slice())))
    }
    fn list_file_production_checkpoints(&self, limits: ReadLimits) -> StorageFuture<'_, Vec<FileCheckpoint>> {
        Box::pin(self.call(move |state| files::recover(state, limits)))
    }
    fn get_file_production<'a>(&'a self, file: &'a StagedFile) -> StorageFuture<'a, FileProgress> {
        let file = file.clone();
        Box::pin(self.call(move |state| files::progress(state, &file)))
    }
    fn get_file_production_bytes<'a>(&'a self, file: &'a StagedFile, offset: u64, maximum: NonZeroUsize) -> StorageFuture<'a, Vec<u8>> {
        let file = file.clone();
        Box::pin(self.call(move |state| {
            let progress = files::progress(state, &file)?;
            files::bytes(state, &file, offset, maximum, progress.length)
        }))
    }
    fn delete_file_production<'a>(&'a self, file: &'a StagedFile) -> StorageFuture<'a, ()> {
        let file = file.clone();
        Box::pin(self.call(move |state| files::abandon(state, &file)))
    }
    fn append_file_bytes<'a>(&'a self, file: &'a StagedFile, offset: u64, bytes: FileChunk) -> StorageFuture<'a, ()> {
        let file = file.clone();
        Box::pin(self.call(move |state| files::write(state, &file, offset, bytes.as_slice())))
    }
    fn publish_file_production<'a>(
        &'a self, file: &'a StagedFile, position: &'a StorageCursor, length: u64, sha256: [u8; 32],
    ) -> StorageFuture<'a, FileReference> {
        let (file, position) = (file.clone(), position.clone());
        Box::pin(self.call(move |state| files::finish(state, &file, &position, length, sha256)))
    }
    fn delete_file_staging<'a>(&'a self, file: &'a StagedFile) -> StorageFuture<'a, ()> {
        let file = file.clone();
        Box::pin(self.call(move |state| files::abort(state, &file)))
    }
    fn get_file_bytes<'a>(
        &'a self, snapshot: &'a BackendSnapshot, file: &'a FileReference, offset: u64, max_bytes: NonZeroUsize,
    ) -> StorageFuture<'a, Vec<u8>> {
        let (snapshot, file) = (snapshot.clone(), file.clone());
        Box::pin(self.call(move |state| files::read(state, &snapshot, &file, offset, max_bytes)))
    }

    fn append_mutation_receipts(&self, receipts: StorageBatch<MutationReceipt>) -> StorageFuture<'_, ()> {
        Box::pin(self.call(move |state| {
            state.check()?;
            let tx = state.connection.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db)?;
            for receipt in receipts.as_slice() {
                let id = text(&receipt.mutation)?;
                let bytes = encode(receipt)?;
                let prior: Option<Vec<u8>> =
                    tx.query_row("SELECT data FROM storage_mutations WHERE id=?1", [&id], |row| row.get(0)).optional().map_err(db)?;
                if prior.as_ref().is_some_and(|prior| prior != &bytes) {
                    return Err(conflict("mutation receipt differs"));
                }
                tx.execute("INSERT OR IGNORE INTO storage_mutations VALUES(?1,?2)", params![id, bytes]).map_err(db)?;
            }
            tx.commit().map_err(db)
        }))
    }
    fn list_mutation_receipts(&self, limits: ReadLimits) -> StorageFuture<'_, Vec<MutationReceipt>> {
        Box::pin(self.call(move |state| {
            state.check()?;
            let mut query = state.connection.prepare("SELECT data FROM storage_mutations ORDER BY id").map_err(db)?;
            let mut rows = query.query([]).map_err(db)?;
            let mut output = Vec::new();
            let mut bytes = 0;
            while let Some(row) = rows.next().map_err(db)? {
                let data: Vec<u8> = row.get(0).map_err(db)?;
                bytes += data.len();
                limits.check(output.len() + 1, bytes)?;
                output.push(decode(&data)?);
            }
            Ok(output)
        }))
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn meta(connection: &Connection, name: &str) -> Result<u64> {
    let value: String = connection.query_row("SELECT value FROM storage_meta WHERE name=?1", [name], |row| row.get(0)).map_err(db)?;
    value.parse().map_err(|_| incompatible())
}
fn set_meta(connection: &Connection, name: &str, value: u64) -> Result<()> {
    connection.execute("UPDATE storage_meta SET value=?1 WHERE name=?2", params![value.to_string(), name]).map_err(db)?;
    Ok(())
}
fn unsigned(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value = row.get::<_, i64>(index)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}
fn size(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<usize> {
    let value = row.get::<_, i64>(index)?;
    usize::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}
fn integer(value: u64) -> Result<i64> {
    value.try_into().map_err(|_| invalid("value exceeds SQLite signed integer range"))
}
fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|source| Error::backend("encode SQLite value", BackendFailure::Permanent, source))
}
fn text<T: serde::Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|source| Error::backend("encode SQLite key", BackendFailure::Permanent, source))
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|source| {
        Error::backend("decode SQLite value; back up the original data and reset explicitly", BackendFailure::Corrupt, source)
    })
}
fn invalid(reason: &'static str) -> Error {
    Error::InvalidInput { field: "SQLite storage", reason }
}
fn conflict(reason: &'static str) -> Error {
    Error::backend("SQLite storage", BackendFailure::Conflict, io::Error::new(io::ErrorKind::InvalidData, reason))
}
fn incompatible() -> Error {
    Error::backend(
        "open SQLite storage",
        BackendFailure::Corrupt,
        io::Error::new(
            io::ErrorKind::InvalidData,
            "incompatible storage format or history; back up the original data and select a fresh storage root; the old folder is preserved",
        ),
    )
}

fn validate_format(connection: &Connection) -> Result<()> {
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0)).map_err(db)?;
    let tables: i64 = connection
        .query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'", [], |row| row.get(0))
        .map_err(db)?;
    if version == 0 && tables == 0 {
        return Ok(());
    }
    if version != 1 {
        return Err(incompatible());
    }
    let format: Option<String> =
        connection.query_row("SELECT value FROM storage_meta WHERE name='format'", [], |row| row.get(0)).optional().map_err(db)?;
    if format.as_deref() != Some(FORMAT) {
        return Err(incompatible());
    }
    Ok(())
}
fn db(source: rusqlite::Error) -> Error {
    let kind = match source.sqlite_error_code() {
        Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked) => BackendFailure::Transient,
        Some(rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase) => BackendFailure::Corrupt,
        Some(rusqlite::ErrorCode::ConstraintViolation) => BackendFailure::Conflict,
        _ => BackendFailure::Permanent,
    };
    Error::backend(
        if kind == BackendFailure::Corrupt { "SQLite storage; back up the original data and reset explicitly" } else { "SQLite storage" },
        kind,
        source,
    )
}

fn exhausted() -> Error {
    harmos_query::columnar::exhausted().into()
}

/* === Tests === */
