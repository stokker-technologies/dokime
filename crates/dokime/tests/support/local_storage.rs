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
use std::{
    num::{NonZeroU32, NonZeroU64, NonZeroUsize},
    time::Duration,
};

// Third Party Imports
use harmos::{
    Identifier,
    schema::{
        Cardinality,
        graph::{Field, Kind, Schema},
    },
    storage::*,
};

// Local Imports
use dokime::storage::sqlite::Backend;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
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
            reserved_control_batches: 0,
            reserved_control_bytes: 0,
        },
        response: StorageResponseSettings {
            batch_rows: nonzero(64),
            batch_bytes: nonzero(64 * 1024),
        },
        target_batch_bytes: NonZeroUsize::new(128 * 1024).unwrap(),
        max_batch_bytes: NonZeroUsize::new(256 * 1024).unwrap(),
        query_page: ReadLimits {
            rows: NonZeroUsize::new(64).unwrap(),
            bytes: NonZeroUsize::new(64 * 1024).unwrap(),
        },
        query_page_ceiling: ReadLimits {
            rows: NonZeroUsize::new(4096).unwrap(),
            bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        },
        query_decoded_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        query_decoded_bytes_ceiling: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        query_execution: QueryLimits {
            memory_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            admission: Admission::Guaranteed,
            spill_bytes: 0,
            timeout: Duration::from_secs(5),
        },
        query_execution_ceiling: QueryLimits {
            memory_bytes: NonZeroUsize::new(16 * 1024 * 1024).unwrap(),
            admission: Admission::Guaranteed,
            spill_bytes: 64 * 1024 * 1024,
            timeout: Duration::from_secs(30),
        },
        replication_limits: ReadLimits {
            rows: NonZeroUsize::new(64).unwrap(),
            bytes: NonZeroUsize::new(64 * 1024).unwrap(),
        },
        replica_lag_positions: NonZeroU64::new(4096).unwrap(),
        replication_shutdown: Duration::ZERO,
        max_views: NonZeroUsize::new(32).unwrap(),
        max_followers: NonZeroUsize::new(32).unwrap(),
        max_files: NonZeroUsize::new(8).unwrap(),
        file_session_timeout: Duration::from_secs(60),
        max_mutations: NonZeroUsize::new(1024).unwrap(),
        mutation_memory_bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
        mutation_retention: Duration::from_secs(300),
        view_idle_timeout: Duration::from_secs(30),
        snapshot_bytes: NonZeroUsize::new(4096).unwrap(),
        write_batch_entries: NonZeroUsize::new(64).unwrap(),
        object_prune_entries: NonZeroUsize::new(256).unwrap(),
        write_batch_bytes: NonZeroUsize::new(256 * 1024).unwrap(),
        flush_interval: Duration::from_millis(100),
        io_timeout: Duration::from_secs(5),
        max_retries: 3,
        retry_base: Duration::from_millis(10),
        retry_cap: Duration::from_secs(1),
    }
}

/* === Private Functions === */
fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap()
}

pub(crate) async fn read_all(backend: &Backend<()>, view: &ReadView) -> Vec<StoredRecord> {
    let selection = ReadSelection::new(ReadKind::Records);
    let mut cursor = None;
    let mut records = Vec::new();
    loop {
        let page = backend
            .read_records(view, &selection, cursor.as_ref(), limits(64))
            .await
            .unwrap();
        records.extend(page.entries);
        cursor = page.next;
        if cursor.is_none() {
            return records;
        }
    }
}

pub(crate) fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}
pub(crate) fn history() -> StoreHistory {
    StoreHistory {
        store: id("bench"),
        epoch: id("run"),
    }
}
pub(crate) fn position(sequence: u64) -> StorageCursor {
    StorageCursor {
        history: history(),
        sequence,
    }
}
pub(crate) fn key() -> SchemaKey {
    SchemaKey {
        payload_encoding: harmos::storage::PayloadEncoding::ProtobufV1,
        id: id("charger"),
        version: NonZeroU32::new(1).unwrap(),
    }
}
pub(crate) fn schema() -> StorageSchema {
    StorageSchema::new(
        position(1),
        key(),
        graph(vec![Field {
            name: "counter".into(),
            tag: 1,
            kind: Kind::U64,
            cardinality: Cardinality::Required,
        }]),
    )
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
pub(crate) fn record(sequence: u64) -> StoredRecord {
    StoredRecord {
        position: position(sequence),
        schema: key(),
        scope: Some(id("execution")),
        item: id(&format!("item-{sequence}")),
        source: Some(id("charger")),
        source_time: None,
        payload: vec![8, sequence as u8],
    }
}
pub(crate) fn limits(rows: usize) -> ReadLimits {
    ReadLimits {
        rows: NonZeroUsize::new(rows).unwrap(),
        bytes: NonZeroUsize::new(65536).unwrap(),
    }
}
pub(crate) async fn view<C: ObjectCatalog>(backend: &Backend<C>, through: u64) -> ReadView {
    let saved = position(through);
    ReadView::new(
        id(&format!("view-{through}")),
        position(through),
        saved.clone(),
        backend.read_view(&saved).await.unwrap(),
    )
    .unwrap()
}

pub(crate) fn write_batch<'a>(parts: &'a [StoragePart<'a>]) -> StorageBatch<'a> {
    StorageBatch::new(parts, NonZeroUsize::new(16 * 1024 * 1024).unwrap()).unwrap()
}

/* === Tests === */
