/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Typed Rows From Storage Queries
//
// HARMOS GAP WORKAROUND: Harmos offers no bounded typed historical read; its
// `Query` answers Arrow batches and its follow subscriptions never signal the
// end of saved history. These readers rebuild the stored envelopes from a
// query's `_storage` columns and re-encode the `payload` column with Harmos'
// own `encode_payloads`, so domain codecs decode exactly the stored bytes.
// Replace this module when Harmos serves typed historical pages.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    num::NonZeroUsize,
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

// Third Party Imports
use arrow_array::{
    Array, RecordBatch, StringArray, StructArray, TimestampNanosecondArray, UInt32Array,
    UInt64Array,
};
use harmos::{
    Identifier,
    schema::{Described, graph::Schema},
    storage::{
        ObjectEntry, ObjectKey, ObjectRevision, ObjectValue, ObjectVersionToken, PayloadEncoding,
        QueryReader, SchemaKey, StorageCursor, StoreHistory, StoredObject, StoredRecord, StoredRow,
        query::{
            ObjectSelection, ObjectVersions, Query, QueryOutput, RowSlice, Source,
            transform::Transform,
        },
    },
};

// Local Imports
use super::{Error, Manager, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// Typed envelopes of one storage query, one Arrow batch at a time.
pub struct Rows<E> {
    reader: QueryReader,
    graph: Arc<Schema>,
    _entry: std::marker::PhantomData<fn() -> E>,
}

/// The `_storage` envelope columns of one Arrow batch.
pub struct Envelope<'a> {
    store: &'a StringArray,
    epoch: &'a StringArray,
    position: &'a UInt64Array,
    item: &'a StringArray,
    schema_id: &'a StringArray,
    schema_version: &'a UInt32Array,
    payload_encoding: &'a StringArray,
    columns: &'a StructArray,
}

/* === Enums === */

/* === Traits === */
/// A stored entry rebuilt from its envelope columns and exact payload bytes.
pub trait Entry: Sized {
    fn build(envelope: &Envelope<'_>, row: usize, payload: Vec<u8>) -> Result<Self>;
}

/* === Implementations === */
impl<E: Entry> Rows<E> {
    /// Query `source` at `at` and decode payloads with the declaration of `T`.
    pub async fn open<T: Described>(
        storage: &Manager,
        at: StorageCursor,
        source: Source,
    ) -> Result<Self> {
        Self::with_query(
            storage,
            Query {
                at,
                source,
                transforms: Vec::new(),
                slice: RowSlice::default(),
            },
            graph::<T>()?,
        )
        .await
    }

    /// Run a prepared query whose output keeps the `_storage` and `payload` columns.
    pub async fn with_query(storage: &Manager, query: Query, graph: Arc<Schema>) -> Result<Self> {
        Ok(Self {
            reader: storage.query(query).await?,
            graph,
            _entry: std::marker::PhantomData,
        })
    }

    /// The next nonempty batch of entries, or `None` once the query completes.
    pub async fn next(&mut self) -> Result<Option<Vec<E>>> {
        while let Some(output) = self.reader.next().await? {
            let QueryOutput::Batch(batch) = output else {
                continue;
            };
            let batch = batch.as_record_batch();
            if batch.num_rows() == 0 {
                continue;
            }
            return decode(batch, &self.graph).map(Some);
        }
        Ok(None)
    }

    /// Every remaining entry; for selections bounded by their domain.
    pub async fn collect(mut self) -> Result<Vec<E>> {
        let mut entries = Vec::new();
        while let Some(batch) = self.next().await? {
            entries.extend(batch);
        }
        Ok(entries)
    }
}

impl Envelope<'_> {
    fn cursor(&self, row: usize) -> Result<StorageCursor> {
        Ok(StorageCursor {
            history: StoreHistory {
                store: identifier(self.store.value(row))?,
                epoch: identifier(self.epoch.value(row))?,
            },
            sequence: self.position.value(row),
        })
    }

    fn schema(&self, row: usize) -> Result<SchemaKey> {
        Ok(SchemaKey {
            payload_encoding: match self.payload_encoding.value(row) {
                "postcard_v1" => PayloadEncoding::PostcardV1,
                "protobuf_v1" => PayloadEncoding::ProtobufV1,
                _ => return Err(Error::Invalid("unknown stored payload encoding")),
            },
            id: identifier(self.schema_id.value(row))?,
            version: self
                .schema_version
                .value(row)
                .try_into()
                .map_err(|_| Error::Invalid("stored schema version is zero"))?,
        })
    }

    fn text(&self, name: &str, row: usize) -> Result<Option<Identifier>> {
        let Some(column) = self.columns.column_by_name(name) else {
            return Ok(None);
        };
        let column = column
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or(Error::Invalid("stored envelope column has another type"))?;
        if column.is_null(row) {
            return Ok(None);
        }
        identifier(column.value(row)).map(Some)
    }

    fn record(&self, row: usize, payload: Vec<u8>) -> Result<StoredRecord> {
        let source_time = match self.columns.column_by_name("source_time") {
            Some(column) => {
                let column = column
                    .as_any()
                    .downcast_ref::<TimestampNanosecondArray>()
                    .ok_or(Error::Invalid("stored source time has another type"))?;
                (!column.is_null(row)).then(|| time(column.value(row)))
            }
            None => None,
        };
        Ok(StoredRecord {
            position: self.cursor(row)?,
            schema: self.schema(row)?,
            scope: self.text("scope", row)?,
            item: identifier(self.item.value(row))?,
            source: self.text("source", row)?,
            source_time,
            payload,
        })
    }
}

impl Entry for StoredRecord {
    fn build(envelope: &Envelope<'_>, row: usize, payload: Vec<u8>) -> Result<Self> {
        envelope.record(row, payload)
    }
}

impl Entry for StoredRow {
    fn build(envelope: &Envelope<'_>, row: usize, payload: Vec<u8>) -> Result<Self> {
        let sequence = envelope
            .columns
            .column_by_name("sequence")
            .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
            .ok_or(Error::Invalid("stored stream row has no sequence"))?
            .value(row);
        Ok(StoredRow {
            record: envelope.record(row, payload)?,
            stream: envelope
                .text("stream", row)?
                .ok_or(Error::Invalid("stored stream row has no stream"))?,
            sequence,
        })
    }
}

impl Entry for ObjectEntry {
    fn build(envelope: &Envelope<'_>, row: usize, payload: Vec<u8>) -> Result<Self> {
        let position = envelope.cursor(row)?;
        let revision = envelope
            .columns
            .column_by_name("revision")
            .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
            .ok_or(Error::Invalid("stored object has no revision"))?
            .value(row);
        // The Token Has No Public Constructor; Its Serialized Form Is Public.
        let version: ObjectVersionToken = serde_json::from_value(serde_json::json!({
            "history": position.history,
            "revision": revision,
        }))?;
        version.check_position(&position)?;
        Ok(ObjectEntry {
            version,
            value: ObjectValue {
                schema: envelope.schema(row)?,
                item: ObjectKey::new(envelope.item.value(row), NonZeroUsize::MAX)?,
                payload,
            },
            position,
        })
    }
}

/* === Public Functions === */

/// Every object of `T` at the saved cut: the latest revision of each key, or all revisions.
pub async fn objects<T>(
    storage: &Manager,
    key: Option<ObjectKey>,
    versions: ObjectVersions,
    transforms: Vec<Transform>,
) -> Result<Vec<ObjectRevision<T>>>
where
    T: StoredObject + Described,
{
    let (id, version) = T::definition();
    let schema = SchemaKey {
        payload_encoding: PayloadEncoding::PostcardV1,
        id: identifier(id.as_str())?,
        version: version
            .try_into()
            .map_err(|_| Error::Invalid("object schema version is zero"))?,
    };
    let rows = Rows::<ObjectEntry>::with_query(
        storage,
        Query {
            at: storage.cursor_saved(),
            source: Source::Objects(ObjectSelection {
                schema: Some(schema),
                key,
                versions,
            }),
            transforms,
            slice: RowSlice::default(),
        },
        graph::<T>()?,
    )
    .await?;
    rows.collect()
        .await?
        .into_iter()
        .map(|entry| {
            Ok(ObjectRevision {
                value: <T as harmos::Message>::decode(&entry.value.payload)?,
                position: entry.position,
                version: entry.version,
            })
        })
        .collect()
}

/// The payload graph a row reader re-encodes declared messages of `T` with.
pub fn graph<T: Described>() -> Result<Arc<Schema>> {
    Schema::from_declaration(T::DEFINITION, &harmos::config::SchemaSettings::default())
        .map(Arc::new)
        .map_err(|error| {
            harmos::Error::backend(
                "prepare row schema",
                harmos::BackendFailure::Permanent,
                error,
            )
            .into()
        })
}

/* === Private Functions === */
fn decode<E: Entry>(batch: &RecordBatch, graph: &Schema) -> Result<Vec<E>> {
    let columns = struct_column(batch, "_storage")?;
    let payload = struct_column(batch, "payload")?;
    let envelope = Envelope {
        store: string(columns, "store")?,
        epoch: string(columns, "epoch")?,
        position: column(columns, "position")?,
        item: string(columns, "item")?,
        schema_id: string(columns, "schema_id")?,
        schema_version: column(columns, "schema_version")?,
        payload_encoding: string(columns, "payload_encoding")?,
        columns,
    };
    let encoding = match batch.num_rows() {
        0 => return Ok(Vec::new()),
        _ => envelope.schema(0)?.payload_encoding,
    };
    let payloads =
        harmos::storage::arrow::encode_payloads(graph, payload, encoding, NonZeroUsize::MAX)?;
    payloads
        .into_iter()
        .enumerate()
        .map(|(row, payload)| E::build(&envelope, row, payload))
        .collect()
}

fn struct_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a StructArray> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<StructArray>())
        .ok_or(Error::Invalid("query output lost its storage columns"))
}

fn string<'a>(columns: &'a StructArray, name: &str) -> Result<&'a StringArray> {
    column(columns, name)
}

fn column<'a, A: 'static>(columns: &'a StructArray, name: &str) -> Result<&'a A> {
    columns
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<A>())
        .ok_or(Error::Invalid(
            "query output lost a storage envelope column",
        ))
}

fn identifier(value: &str) -> Result<Identifier> {
    Identifier::new(value).map_err(|_| Error::Invalid("stored identifier is invalid"))
}

fn time(nanos: i64) -> std::time::SystemTime {
    let offset = Duration::from_nanos(nanos.unsigned_abs());
    if nanos >= 0 {
        UNIX_EPOCH + offset
    } else {
        UNIX_EPOCH - offset
    }
}

/* === Tests === */
