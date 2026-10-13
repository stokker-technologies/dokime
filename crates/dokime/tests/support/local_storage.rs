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
use std::num::{NonZeroU32, NonZeroUsize};

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

/* === Private Functions === */
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
