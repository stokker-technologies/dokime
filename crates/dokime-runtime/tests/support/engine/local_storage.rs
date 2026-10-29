/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Shared Fixtures For SQLite Backend And Storage Manager Scenarios

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    num::{NonZeroU32, NonZeroUsize},
    path::Path,
    sync::Arc,
};

// Third Party Imports
use harmos::{
    Identifier,
    schema::{
        Cardinality,
        graph::{Field, Kind, Schema},
    },
    storage::{query::RecordSelection, *},
};

// Local Imports
use dokime_runtime::storage::sqlite::Backend;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Store<C: ObjectCatalog = ()> {
    pub backend: Arc<Backend<C>>,
    pub history: StoreHistory,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<C: ObjectCatalog> Store<C> {
    pub(crate) async fn open(root: &Path) -> Self {
        let backend = Arc::new(Backend::<C>::primary(root.to_path_buf()).await.unwrap());
        let history = backend.history().clone();
        backend.open().await.unwrap();
        Self { backend, history }
    }

    pub(crate) fn batch<T>(&self, values: Vec<T>) -> StorageBatch<T> {
        StorageBatch::new(values)
    }

    pub(crate) fn at(&self, sequence: u64) -> StorageCursor {
        StorageCursor { history: self.history.clone(), sequence }
    }

    pub(crate) fn schema(&self) -> StorageSchema {
        StorageSchema::new(
            self.at(1),
            key(),
            graph(vec![Field {
                name: "counter".into(),
                tag: 1,
                kind: Kind::U64,
                cardinality: Cardinality::Required,
            }]),
        )
    }

    pub(crate) fn record(&self, sequence: u64) -> StoredRecord {
        StoredRecord {
            position: self.at(sequence),
            schema: key(),
            scope: Some(id("execution")),
            item: id(&format!("item-{sequence}")),
            source: Some(id("charger")),
            source_time: None,
            payload: vec![8, sequence as u8],
        }
    }

    pub(crate) async fn records(&self, through: u64) -> Vec<StoredRecord> {
        let snapshot = self.backend.open_snapshot(&self.at(through)).await.unwrap();
        let records = read_records(&self.backend, &snapshot, rows(2)).await.unwrap();
        self.backend.release_snapshot(&snapshot).await.unwrap();
        records
    }
}

/* === Public Functions === */

pub fn storage_settings() -> StorageSettings {
    StorageSettings {
        filesystem: dokime_runtime::config::storage_defaults().filesystem,
        request: StorageRequestSettings {
            max_active_file_operations: dokime_runtime::config::storage_defaults().request.max_active_file_operations,
            max_pending_queries: nonzero(64),
            max_active_queries: nonzero(4),
            max_pending_write_batches: nonzero(256),
            max_pending_write_bytes: nonzero(16 * 1024 * 1024),
        },
        response: StorageResponseSettings {
            batch_rows: nonzero(64),
            // One Response Batch Holds A Whole Report Publication Record.
            batch_bytes: nonzero(1024 * 1024),
        },
    }
}

pub(crate) async fn read_records<C: ObjectCatalog>(
    backend: &Backend<C>, snapshot: &BackendSnapshot, rows: NonZeroUsize,
) -> harmos::Result<Vec<StoredRecord>> {
    let selection = RecordSelection {
        schema: key(),
        scope: None,
        item: None,
        source_start: None,
        source_end: None,
    };
    let mut records = Vec::new();
    let mut after = 0;
    loop {
        let entries = backend.read_records(snapshot, &selection, page(after, rows)).await?;
        let Some(last) = entries.last() else {
            return Ok(records);
        };
        after = last.position.sequence;
        records.extend(entries);
    }
}

pub(crate) fn page(after: u64, rows: NonZeroUsize) -> ScanPage {
    ScanPage { after, rows, bytes: nonzero(64 * 1024) }
}

pub(crate) fn rows(rows: usize) -> NonZeroUsize {
    nonzero(rows)
}

pub(crate) fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

pub(crate) fn key() -> SchemaKey {
    SchemaKey {
        payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
        id: id("charger"),
        version: NonZeroU32::new(1).unwrap(),
    }
}

// A Single-Definition Row Graph For Fixture Schemas
pub(crate) fn graph(fields: Vec<Field>) -> Schema {
    Schema { root: 0, definitions: vec![fields] }
}

// The Prepared Row Graph Of A Described Message
pub(crate) fn declared<T: harmos::schema::Described>() -> Schema {
    Schema::from_declaration(T::DEFINITION, &harmos::config::SchemaSettings::default()).unwrap()
}

/* === Private Functions === */

fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap()
}

/* === Tests === */
