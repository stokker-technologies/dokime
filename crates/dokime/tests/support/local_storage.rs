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
use dokime::storage::sqlite::Backend;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// One SQLite backend and the history its positions belong to.
pub(crate) struct Store<C: ObjectCatalog = ()> {
    pub backend: Backend<C>,
    pub history: StoreHistory,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<C: ObjectCatalog> Store<C> {
    /// Open and initialize the backend owning `root`.
    pub(crate) async fn open(root: &Path) -> Self {
        let backend = Backend::<C>::primary(root.to_path_buf()).await.unwrap();
        backend.open().await.unwrap();
        let history = backend.history().clone();
        Self { backend, history }
    }

    pub(crate) fn at(&self, sequence: u64) -> StorageCursor {
        StorageCursor {
            history: self.history.clone(),
            sequence,
        }
    }

    /// The fixture record schema, written at position one.
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

    /// Every fixture record visible at `through`, read in small pages.
    pub(crate) async fn records(&self, through: u64) -> Vec<StoredRecord> {
        let snapshot = self.backend.open_snapshot(&self.at(through)).await.unwrap();
        let records = read_records(&self.backend, &snapshot, rows(2))
            .await
            .unwrap();
        self.backend.release_snapshot(&snapshot).await.unwrap();
        records
    }
}

/* === Public Functions === */
/// Shipped-shape storage settings sized for scenarios.
pub fn storage_settings() -> StorageSettings {
    StorageSettings {
        budget: StorageBudget {
            read_memory_bytes: nonzero(16 * 1024 * 1024),
        },
        request: StorageRequestSettings {
            max_pending_queries: nonzero(64),
            max_active_queries: nonzero(4),
            max_pending_write_batches: nonzero(256),
            max_pending_write_bytes: nonzero(16 * 1024 * 1024),
        },
        response: StorageResponseSettings {
            batch_rows: nonzero(64),
            batch_bytes: nonzero(64 * 1024),
        },
    }
}

/// Every record of the fixture schema in `snapshot`, paged ascending.
pub(crate) async fn read_records<C: ObjectCatalog>(
    backend: &Backend<C>,
    snapshot: &BackendSnapshot,
    rows: NonZeroUsize,
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
        let entries = backend
            .read_records(snapshot, &selection, page(after, rows))
            .await?;
        let Some(last) = entries.last() else {
            return Ok(records);
        };
        after = last.position.sequence;
        records.extend(entries);
    }
}

pub(crate) fn page(after: u64, rows: NonZeroUsize) -> ScanPage {
    ScanPage {
        after,
        rows,
        bytes: nonzero(64 * 1024),
    }
}

pub(crate) fn rows(rows: usize) -> NonZeroUsize {
    nonzero(rows)
}

pub(crate) fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

pub(crate) fn key() -> SchemaKey {
    SchemaKey {
        payload_encoding: harmos::storage::PayloadEncoding::ProtobufV1,
        id: id("charger"),
        version: NonZeroU32::new(1).unwrap(),
    }
}

// A Single-Definition Row Graph For Fixture Schemas
pub(crate) fn graph(fields: Vec<Field>) -> Schema {
    Schema {
        root: 0,
        definitions: vec![fields],
    }
}

// The Prepared Row Graph Of A Described Message
pub(crate) fn declared<T: harmos::schema::Described>() -> Schema {
    Schema::from_declaration(T::DEFINITION, &harmos::config::SchemaSettings::default()).unwrap()
}

pub(crate) fn write_batch<'a>(parts: &'a [StoragePart<'a>]) -> StorageBatch<'a> {
    StorageBatch::new(parts, NonZeroUsize::new(16 * 1024 * 1024).unwrap()).unwrap()
}

/* === Private Functions === */
fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap()
}

/* === Tests === */
