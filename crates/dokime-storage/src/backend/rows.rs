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
    collections::BTreeMap,
    num::NonZeroUsize,
    time::{SystemTime, UNIX_EPOCH},
};

// Third Party Imports
use rusqlite::{Connection, OptionalExtension, params, types::Value};

// Local Imports
use super::*;
use harmos_query::columnar::ColumnKind;
use harmos_query::columnar::ColumnSource;
use harmos_storage::query::ObjectVersions;
use sha2::Digest;

use harmos_domain::Error as DomainError;
use harmos_storage::DefinitionOutcome;
use harmos_storage::DefinitionRefusal;
use harmos_storage::ObjectRetention;
use harmos_storage::ObjectVersion;
use harmos_storage::SchemaExchange;
use harmos_storage::SchemaWriteReceipt;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct WriteRow<'a> {
    pub position: &'a StorageCursor,
    pub kind: i64,
    pub schema: Option<&'a SchemaKey>,
    pub item: Option<&'a str>,
    pub scope: Option<&'a str>,
    pub stream: Option<&'a str>,
    pub time: Option<SystemTime>,
    pub data: Vec<u8>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn insert(connection: &Connection, history: &StoreHistory, value: WriteRow<'_>) -> Result<bool> {
    history.check(&value.position.history)?;
    if value.position.sequence == 0 {
        return Err(invalid("write position must be nonzero"));
    }
    let position = integer(value.position.sequence)?;
    let prior: Option<(i64, Vec<u8>)> = connection
        .query_row("SELECT kind,data FROM storage_rows WHERE position=?1", [position], |row| Ok((row.get(0)?, row.get(1)?)))
        .optional()
        .map_err(db)?;
    if let Some((kind, data)) = prior {
        if kind != value.kind || data != value.data {
            return Err(conflict("replayed position has different content"));
        }
        return Ok(false);
    }
    let last: u64 = connection.query_row("SELECT max(coalesce((SELECT max(position) FROM storage_rows),0),CAST((SELECT value FROM storage_meta WHERE name='saved') AS INTEGER))", [], |row| unsigned(row,0)).map_err(db)?;
    if value.position.sequence <= last {
        return Err(conflict("new writes must advance the committed source order"));
    }
    let key = value.schema.map(text).transpose()?;
    connection
        .execute(
            "INSERT INTO storage_rows VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                position,
                value.kind,
                key,
                value.schema.map(|schema| schema.id.as_str()),
                value.item,
                value.scope,
                value.stream,
                value.time.map(timestamp).transpose()?,
                value.data
            ],
        )
        .map_err(db)?;
    Ok(true)
}

pub(super) fn schema(connection: &Connection, key: &SchemaKey, at: u64) -> Result<StorageSchema> {
    let data: Option<Vec<u8>> = connection
        .query_row(
            "SELECT data FROM storage_rows WHERE kind=0 AND schema_key=?1 AND position<=?2 ORDER BY position DESC LIMIT 1",
            params![text(key)?, integer(at)?],
            |row| row.get(0),
        )
        .optional()
        .map_err(db)?;
    decode(&data.ok_or_else(|| invalid("schema is unavailable at requested cut"))?)
}

pub(super) fn save(connection: &Connection, through: u64) -> Result<()> {
    set_meta(connection, "saved", meta(connection, "saved")?.max(through))
}

pub(super) fn put_schema(connection: &Connection, history: &StoreHistory, value: &StorageSchema) -> Result<()> {
    let prior = connection
        .query_row("SELECT data FROM storage_rows WHERE kind=0 AND schema_key=?1 LIMIT 1", [text(&value.key)?], |row| {
            row.get::<_, Vec<u8>>(0)
        })
        .optional()
        .map_err(db)?;
    if let Some(prior) = prior
        && decode::<StorageSchema>(&prior)?.schema != value.schema
    {
        return Err(conflict("schema definition changed without a new version"));
    }
    value.encoded_bytes()?;
    insert(
        connection,
        history,
        WriteRow {
            position: &value.position,
            kind: 0,
            schema: Some(&value.key),
            item: None,
            scope: None,
            stream: None,
            time: None,
            data: encode(value)?,
        },
    )?;
    Ok(())
}

pub(super) fn set_schema(state: &mut State, exchange: &SchemaExchange) -> Result<SchemaWriteReceipt> {
    state.check()?;
    state.history.check(&exchange.value.position.history)?;
    exchange.value.encoded_bytes()?;
    let tx = state.connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(db)?;
    let prior: Option<Vec<u8>> = tx
        .query_row("SELECT data FROM storage_mutations WHERE id=?1", [text(&exchange.mutation().mutation)?], |row| row.get(0))
        .optional()
        .map_err(db)?;
    if let Some(data) = prior {
        let receipt = decode(&data)?;
        exchange.replay(&receipt)?;
        drop(tx);
        return Ok(SchemaWriteReceipt {
            saved: state.cursor()?,
            mutation: Some(receipt),
        });
    }
    let prior: Option<Vec<u8>> = tx
        .query_row(
            "SELECT data FROM storage_rows WHERE kind=0 AND schema_key=?1 ORDER BY position DESC LIMIT 1",
            [text(&exchange.value.key)?],
            |row| row.get(0),
        )
        .optional()
        .map_err(db)?;
    let prior: Option<StorageSchema> = prior.as_deref().map(decode).transpose()?;
    let current = prior.as_ref().map(|value| &value.position);
    let unchanged = prior.as_ref().is_none_or(|value| value.schema == exchange.value.schema);
    let outcome = if current == exchange.expected.as_ref() && unchanged {
        put_schema(&tx, &state.history, &exchange.value)?;
        DefinitionOutcome::Applied {
            revision: exchange.value.position.sequence,
        }
    } else {
        DefinitionOutcome::Refused {
            kind: DefinitionRefusal::Conflict,
            reason: if !unchanged {
                "schema definitions are immutable; use a new version; get again and retry"
            } else {
                "schema revision changed; get again and retry"
            }
            .to_owned(),
        }
    };
    let receipt = exchange.receipt(outcome);
    save(&tx, exchange.value.position.sequence)?;
    tx.execute("INSERT INTO storage_mutations VALUES(?1,?2)", params![text(&receipt.mutation)?, encode(&receipt)?]).map_err(db)?;
    tx.commit().map_err(db)?;
    Ok(SchemaWriteReceipt {
        saved: state.cursor()?,
        mutation: Some(receipt),
    })
}

pub(super) fn append_schemas(state: &mut State, values: &[StorageSchema]) -> Result<StorageCursor> {
    state.check()?;
    let through = values.last().ok_or_else(|| invalid("empty schema batch"))?.position.sequence;
    let tx = state.connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(db)?;
    for value in values {
        put_schema(&tx, &state.history, value)?;
    }
    save(&tx, through)?;
    tx.commit().map_err(db)?;
    state.cursor()
}

pub(super) fn append_records(state: &mut State, values: &[StoredRecord]) -> Result<StorageCursor> {
    state.check()?;
    let through = values.last().ok_or_else(|| invalid("empty record batch"))?.position.sequence;
    let writing = std::time::Instant::now();
    let tx = state.connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(db)?;
    for chunk in values.chunks(state.settings.chunk_rows.get()) {
        let mut groups: BTreeMap<String, Vec<StoredRecord>> = BTreeMap::new();
        for value in chunk {
            let inserted = insert(
                &tx,
                &state.history,
                WriteRow {
                    position: &value.position,
                    kind: 2,
                    schema: Some(&value.schema),
                    item: Some(value.item.as_str()),
                    scope: value.scope.as_ref().map(|value| value.as_str()),
                    stream: None,
                    time: value.source_time,
                    data: encode(value)?,
                },
            )?;
            if inserted {
                groups.entry(text(&value.schema)?).or_default().push(value.clone());
            }
        }
        for values in groups.values() {
            let first = &values[0];
            let schema = schema(&tx, &first.schema, first.position.sequence)?;
            let batch = harmos_query::source::records(&schema, values.iter())?;
            columns::store(
                &tx,
                &ColumnSource {
                    kind: ColumnKind::Records,
                    schema: first.schema.clone(),
                },
                &batch,
                state.settings,
            )?;
        }
    }
    save(&tx, through)?;
    let writing_ns = u64::try_from(writing.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let commit_started = std::time::Instant::now();
    tx.commit().map_err(db)?;
    let mut work = state.work.get();
    work.synchronous_commits += 1;
    let commit_ns = u64::try_from(commit_started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    work.synchronous_commit_ns += commit_ns;
    work.maximum_synchronous_commit_ns = work.maximum_synchronous_commit_ns.max(commit_ns);
    work.writing_ns += writing_ns;
    work.maximum_writing_ns = work.maximum_writing_ns.max(writing_ns);
    state.work.set(work);
    state.cursor()
}

pub(super) fn append_samples(state: &mut State, values: &[StoredRow]) -> Result<StorageCursor> {
    state.check()?;
    let through = values.last().ok_or_else(|| invalid("empty stream batch"))?.record.position.sequence;
    let writing = std::time::Instant::now();
    let tx = state.connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(db)?;
    for chunk in values.chunks(state.settings.chunk_rows.get()) {
        let mut groups: BTreeMap<String, Vec<StoredRow>> = BTreeMap::new();
        for value in chunk {
            let row = &value.record;
            let inserted = insert(
                &tx,
                &state.history,
                WriteRow {
                    position: &row.position,
                    kind: 3,
                    schema: Some(&row.schema),
                    item: Some(row.item.as_str()),
                    scope: row.scope.as_ref().map(|value| value.as_str()),
                    stream: Some(value.stream.as_str()),
                    time: row.source_time,
                    data: sha2::Sha256::digest(encode(value)?).to_vec(),
                },
            )?;
            if inserted {
                groups.entry(text(&row.schema)?).or_default().push(value.clone());
            }
        }
        for values in groups.values() {
            let first = &values[0].record;
            let schema = schema(&tx, &first.schema, first.position.sequence)?;
            let batch = harmos_query::source::stream(&schema, values)?;
            columns::store(
                &tx,
                &ColumnSource {
                    kind: ColumnKind::Stream,
                    schema: first.schema.clone(),
                },
                &batch,
                state.settings,
            )?;
        }
    }
    save(&tx, through)?;
    let writing_ns = u64::try_from(writing.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let commit_started = std::time::Instant::now();
    tx.commit().map_err(db)?;
    let mut work = state.work.get();
    work.synchronous_commits += 1;
    let commit_ns = u64::try_from(commit_started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    work.synchronous_commit_ns += commit_ns;
    work.maximum_synchronous_commit_ns = work.maximum_synchronous_commit_ns.max(commit_ns);
    work.writing_ns += writing_ns;
    work.maximum_writing_ns = work.maximum_writing_ns.max(writing_ns);
    state.work.set(work);
    state.cursor()
}

pub(super) fn append_object_revisions(state: &mut State, values: &[ObjectEntry]) -> Result<StorageCursor> {
    state.check()?;
    let through = values.last().ok_or_else(|| invalid("empty object batch"))?.position.sequence;
    let tx = state.connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(db)?;
    for chunk in values.chunks(state.settings.chunk_rows.get()) {
        let mut groups: BTreeMap<String, Vec<ObjectEntry>> = BTreeMap::new();
        for value in chunk {
            value.version.check_position(&value.position)?;
            let inserted = insert(
                &tx,
                &state.history,
                WriteRow {
                    position: &value.position,
                    kind: 1,
                    schema: Some(&value.value.schema),
                    item: Some(value.value.item.as_str()),
                    scope: None,
                    stream: None,
                    time: None,
                    data: encode(value)?,
                },
            )?;
            if inserted {
                groups.entry(text(&value.value.schema)?).or_default().push(value.clone());
            }
        }
        for values in groups.values() {
            let first = &values[0];
            let schema = schema(&tx, &first.value.schema, first.position.sequence)?;
            let batch = harmos_query::source::objects(&schema, values)?;
            columns::store(
                &tx,
                &ColumnSource {
                    kind: ColumnKind::Objects,
                    schema: first.value.schema.clone(),
                },
                &batch,
                state.settings,
            )?;
        }
    }
    save(&tx, through)?;
    tx.commit().map_err(db)?;
    state.cursor()
}

pub(super) fn append_file_references(state: &mut State, values: &[FileReference]) -> Result<StorageCursor> {
    state.check()?;
    let through = values.last().ok_or_else(|| invalid("empty file batch"))?.position.sequence;
    let tx = state.connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(db)?;
    for value in values {
        let finished: Option<Vec<u8>> = tx
            .query_row("SELECT finished FROM storage_files WHERE id=?1", [value.metadata.id.as_str()], |row| row.get(0))
            .optional()
            .map_err(db)?
            .flatten();
        if finished.as_deref() != Some(encode(value)?.as_slice()) {
            return Err(conflict("file is not durably finished"));
        }
        insert(
            &tx,
            &state.history,
            WriteRow {
                position: &value.position,
                kind: 4,
                schema: None,
                item: Some(value.metadata.id.as_str()),
                scope: value.metadata.scope.as_ref().map(|value| value.as_str()),
                stream: None,
                time: None,
                data: encode(value)?,
            },
        )?;
    }
    save(&tx, through)?;
    tx.commit().map_err(db)?;
    state.cursor()
}

pub(super) fn list_schemas(
    state: &State, snapshot: &BackendSnapshot, selection: &SchemaSelection, page: ScanPage,
) -> Result<Vec<StorageSchema>> {
    state.snapshot(snapshot)?;
    query(
        &state.connection,
        snapshot,
        page,
        0,
        "AND (?4 IS NULL OR schema_key=?4)",
        vec![optional(selection.schema.as_ref().map(text).transpose()?)],
    )
}

pub(super) fn list_object_revisions(
    state: &State, snapshot: &BackendSnapshot, selection: &ObjectSelection, page: ScanPage,
) -> Result<Vec<ObjectEntry>> {
    state.snapshot(snapshot)?;
    if selection.versions == ObjectVersions::Revisions {
        let missing:Option<i64>=state.connection.query_row("SELECT max(position) FROM storage_object_gaps WHERE position>?1 AND position<=?2 AND (?3 IS NULL OR schema_key=?3) AND (?4 IS NULL OR item=?4)",params![integer(page.after)?,integer(snapshot.at.sequence)?,selection.schema.as_ref().map(text).transpose()?,selection.key.as_ref().map(|key|key.as_str())],|row|row.get(0)).map_err(db)?;
        if let Some(position) = missing {
            return Err(Error::Gap {
                requested: page.after.saturating_add(1),
                available: position as u64 + 1,
            });
        }
    }
    let latest = if selection.versions == ObjectVersions::Latest {
        "AND position=(SELECT max(o.position) FROM storage_rows o WHERE o.kind=1 AND o.schema_id=storage_rows.schema_id AND o.item=storage_rows.item AND o.position<=?3 AND (?4 IS NULL OR o.schema_key=?4))"
    } else {
        ""
    };
    let condition = format!("AND (?4 IS NULL OR schema_key=?4) AND (?5 IS NULL OR item=?5) {latest}");
    query(
        &state.connection,
        snapshot,
        page,
        1,
        &condition,
        vec![
            optional(selection.schema.as_ref().map(text).transpose()?),
            optional(selection.key.as_ref().map(|key| key.as_str().to_owned())),
        ],
    )
}

pub(super) fn list_records(
    state: &State, snapshot: &BackendSnapshot, selection: &RecordSelection, page: ScanPage,
) -> Result<Vec<StoredRecord>> {
    state.snapshot(snapshot)?;
    query(
        &state.connection,
        snapshot,
        page,
        2,
        "AND schema_key=?4 AND (?5 IS NULL OR scope=?5) AND (?6 IS NULL OR item=?6) AND (?7 IS NULL OR time_ns>=?7) AND (?8 IS NULL OR time_ns<?8)",
        vec![
            Value::Text(text(&selection.schema)?),
            optional(selection.scope.as_ref().map(ToString::to_string)),
            optional(selection.item.as_ref().map(ToString::to_string)),
            optional_time(selection.source_start)?,
            optional_time(selection.source_end)?,
        ],
    )
}

pub(super) fn list_identities(
    state: &State, snapshot: &BackendSnapshot, selection: &IdentitySelection, page: ScanPage,
) -> Result<Vec<StoredIdentity>> {
    state.snapshot(snapshot)?;
    let kind = match selection.kind {
        IdentityKind::Streams => 3,
        IdentityKind::Objects => 1,
    };
    let mut query = state.connection.prepare("SELECT schema_key,scope,item,stream,position FROM storage_rows WHERE position IN (SELECT MIN(position) FROM storage_rows WHERE kind=?1 AND position<=?2 GROUP BY schema_key,scope,item,stream) AND position>?3 AND (?4 IS NULL OR schema_id=?4) ORDER BY position LIMIT ?5").map_err(db)?;
    let mut rows = query
        .query(params![
            kind,
            integer(snapshot.at.sequence)?,
            integer(page.after)?,
            selection.schema.as_ref().map(Identifier::as_str),
            integer(page.rows.get() as u64)?
        ])
        .map_err(db)?;
    let mut output = Vec::new();
    let mut retained = 0;
    while let Some(row) = rows.next().map_err(db)? {
        let schema: String = row.get(0).map_err(db)?;
        let schema = decode(schema.as_bytes())?;
        let item: String = row.get(2).map_err(db)?;
        let first = StorageCursor {
            history: state.history.clone(),
            sequence: unsigned(row, 4).map_err(db)?,
        };
        let identity = match selection.kind {
            IdentityKind::Streams => {
                let scope: Option<String> = row.get(1).map_err(db)?;
                let stream: String = row.get(3).map_err(db)?;
                StoredIdentity::Stream {
                    schema,
                    scope: scope.map(Identifier::new).transpose().map_err(DomainError::from)?,
                    item: Identifier::new(item).map_err(DomainError::from)?,
                    stream: Identifier::new(stream).map_err(DomainError::from)?,
                    first,
                }
            }
            IdentityKind::Objects => StoredIdentity::Object {
                schema,
                key: ObjectKey::from_owned(item)?,
                first,
            },
        };
        retained += identity.memory_size();
        if retained > page.bytes.get() {
            return Err(invalid("list identities: response exceeds byte budget"));
        }
        output.push(identity);
    }
    Ok(output)
}

pub(super) fn list_samples(
    state: &State, snapshot: &BackendSnapshot, selection: &StreamSelection, page: ScanPage,
) -> Result<Vec<StoredRow>> {
    use arrow_array::{Array, StringArray, StructArray, TimestampNanosecondArray, UInt64Array};
    state.snapshot(snapshot)?;
    let schema = schema(&state.connection, &selection.schema, snapshot.at.sequence)?;
    let empty = harmos_query::source::stream(&schema, &[])?;
    let fields = empty.column(0).as_any().downcast_ref::<StructArray>().ok_or_else(incompatible)?;
    let metadata = fields.fields().iter().map(|field| field.name().as_str()).collect::<Vec<_>>();
    let values = empty
        .column(1)
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(incompatible)?
        .fields()
        .iter()
        .map(|field| field.name().clone())
        .collect::<Vec<_>>();
    let source = ColumnSource {
        kind: ColumnKind::Stream,
        schema: selection.schema.clone(),
    };
    let mut statement =
        state.connection.prepare("SELECT id FROM arrow_chunks WHERE source=?1 AND last>?2 AND first<=?3 ORDER BY first").map_err(db)?;
    let mut chunks = statement.query(params![text(&source)?, integer(page.after)?, integer(snapshot.at.sequence)?]).map_err(db)?;
    let mut output = Vec::new();
    let mut bytes = 0;
    while let Some(chunk) = chunks.next().map_err(db)? {
        let batch =
            columns::read_chunk(&state.connection, chunk.get(0).map_err(db)?, &metadata, &values, state.settings.chunk_bytes.get())?;
        let storage = batch.column(0).as_any().downcast_ref::<StructArray>().ok_or_else(incompatible)?;
        let payload = batch.column(1).as_any().downcast_ref::<StructArray>().ok_or_else(incompatible)?;
        let positions =
            storage.column_by_name("position").and_then(|array| array.as_any().downcast_ref::<UInt64Array>()).ok_or_else(incompatible)?;
        let sequences =
            storage.column_by_name("sequence").and_then(|array| array.as_any().downcast_ref::<UInt64Array>()).ok_or_else(incompatible)?;
        let times = storage
            .column_by_name("source_time")
            .and_then(|array| array.as_any().downcast_ref::<TimestampNanosecondArray>())
            .ok_or_else(incompatible)?;
        for row in 0..batch.num_rows() {
            if positions.value(row) <= page.after || positions.value(row) > snapshot.at.sequence || sequences.value(row) <= selection.after
            {
                continue;
            }
            let text = |name: &str| -> Result<Option<Identifier>> {
                let array =
                    storage.column_by_name(name).and_then(|array| array.as_any().downcast_ref::<StringArray>()).ok_or_else(incompatible)?;
                if array.is_null(row) {
                    return Ok(None);
                }
                Identifier::new(array.value(row))
                    .map(Some)
                    .map_err(|source| Error::backend("Read Native Arrow Identity", BackendFailure::Corrupt, source))
            };
            let scope = text("scope")?;
            let item = text("item")?.ok_or_else(incompatible)?;
            let stream = text("stream")?.ok_or_else(incompatible)?;
            let source_time = if times.is_null(row) {
                None
            } else {
                let time = times.value(row);
                Some(if time < 0 {
                    UNIX_EPOCH - std::time::Duration::from_nanos(time.unsigned_abs())
                } else {
                    UNIX_EPOCH + std::time::Duration::from_nanos(time as u64)
                })
            };
            if selection.scope.as_ref().is_some_and(|value| scope.as_ref() != Some(value))
                || selection.item.as_ref().is_some_and(|value| value != &item)
                || selection.stream.as_ref().is_some_and(|value| value != &stream)
                || selection.source_start.is_some_and(|start| source_time.is_none_or(|time| time < start))
                || selection.source_end.is_some_and(|end| source_time.is_none_or(|time| time >= end))
            {
                continue;
            }
            let value = StoredRow {
                record: StoredRecord {
                    position: StorageCursor {
                        history: state.history.clone(),
                        sequence: positions.value(row),
                    },
                    schema: selection.schema.clone(),
                    scope,
                    item,
                    source: text("source")?,
                    source_time,
                    payload: super::payload::encode(&schema.schema, payload, row)?,
                },
                stream,
                sequence: sequences.value(row),
            };
            let size = encode(&value)?.len();
            if output.len() == page.rows.get() || (!output.is_empty() && bytes + size > page.bytes.get()) {
                return Ok(output);
            }
            if size > page.bytes.get() {
                return Err(exhausted());
            }
            bytes += size;
            output.push(value);
        }
    }
    Ok(output)
}

pub(super) fn list_files(
    state: &State, snapshot: &BackendSnapshot, selection: &FileSelection, page: ScanPage,
) -> Result<Vec<FileReference>> {
    state.snapshot(snapshot)?;
    query(
        &state.connection,
        snapshot,
        page,
        4,
        "AND (?4 IS NULL OR scope=?4) AND (?5 IS NULL OR item=?5) AND NOT EXISTS(SELECT 1 FROM deleted_files d WHERE d.id=storage_rows.item AND d.generation<=?6)",
        vec![
            optional(selection.scope.as_ref().map(ToString::to_string)),
            optional(selection.item.as_ref().map(ToString::to_string)),
            Value::Integer(integer(*state.snapshot_generations.get(&snapshot.token).ok_or_else(incompatible)?)?),
        ],
    )
}

pub(super) fn get_object(state: &State, snapshot: &BackendSnapshot, point: &ObjectPoint) -> Result<Option<ObjectEntry>> {
    state.snapshot(snapshot)?;
    let position = match &point.version {
        ObjectVersion::Latest => None,
        ObjectVersion::Exact(token) => {
            state.history.check(token.history())?;
            Some(integer(token.revision().get())?)
        }
    };
    let bytes = state.connection.query_row("SELECT data FROM storage_rows WHERE kind=1 AND schema_id=?1 AND item=?2 AND position<=?3 AND (?4 IS NULL OR position=?4) ORDER BY position DESC LIMIT 1", params![point.schema.as_str(), point.item.as_str(), integer(snapshot.at.sequence)?, position], |row| row.get::<_, Vec<u8>>(0)).optional().map_err(db)?;
    if bytes.is_none()
        && let Some(position) = position
    {
        let lost: bool = state
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM storage_object_gaps WHERE position=?1 AND schema_id=?2 AND item=?3)",
                params![position, point.schema.as_str(), point.item.as_str()],
                |row| row.get(0),
            )
            .map_err(db)?;
        if lost {
            return Err(Error::Gap {
                requested: position as u64,
                available: position as u64 + 1,
            });
        }
    }
    bytes.as_deref().map(decode).transpose()
}

pub(super) fn prune(state: &mut State, through: &StorageCursor, maximum: NonZeroUsize, retention: &ObjectCatalog) -> Result<bool> {
    state.check()?;
    state.history.check(&through.history)?;
    if through.sequence > state.cursor()?.sequence {
        return Err(invalid("pruning is ahead of durable data"));
    }
    let protected = state.snapshots.values().map(|cursor| cursor.sequence).max().unwrap_or(0);
    let tx = state.connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(db)?;
    let mut remove = Vec::new();
    let mut done = true;
    {
        let mut statement =
            tx.prepare("SELECT data FROM storage_rows WHERE kind=1 AND position>?1 AND position<=?2 ORDER BY position").map_err(db)?;
        let mut rows = statement.query(params![integer(protected)?, integer(through.sequence)?]).map_err(db)?;
        while let Some(row) = rows.next().map_err(db)? {
            let object: ObjectEntry = decode(&row.get::<_, Vec<u8>>(0).map_err(db)?)?;
            let policy = retention.retention(&object.value.schema)?;
            if policy == ObjectRetention::All {
                continue;
            }
            if policy == ObjectRetention::UntilReleased {
                let released = tx
                    .query_row(
                        "SELECT through FROM object_releases WHERE schema_key=?1 AND item=?2",
                        params![text(&object.value.schema)?, object.value.item.as_str()],
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()
                    .map_err(db)?;
                if released.is_none_or(|released| released < object.position.sequence as i64) {
                    continue;
                }
            }
            let newer: i64 = tx
                .query_row(
                    "SELECT count(*) FROM storage_rows WHERE kind=1 AND schema_id=?1 AND item=?2 AND position>?3",
                    params![object.value.schema.id.as_str(), object.value.item.as_str(), integer(object.position.sequence)?],
                    |row| row.get(0),
                )
                .map_err(db)?;
            if newer < policy.versions().get() as i64 {
                continue;
            }
            if remove.len() == maximum.get() {
                done = false;
                break;
            }
            remove.push((
                object.position.sequence,
                text(&object.value.schema)?,
                object.value.schema.id.to_string(),
                object.value.item.as_str().to_owned(),
            ));
        }
    }
    for (position, schema, schema_id, item) in &remove {
        tx.execute("INSERT OR IGNORE INTO storage_object_gaps VALUES(?1,?2,?3,?4)", params![integer(*position)?, schema, schema_id, item])
            .map_err(db)?;
        tx.execute("DELETE FROM storage_rows WHERE position=?1", [integer(*position)?]).map_err(db)?;
    }
    tx.execute("DELETE FROM arrow_columns WHERE chunk IN(SELECT id FROM arrow_chunks c WHERE json_extract(source,'$.kind')='Objects' AND NOT EXISTS(SELECT 1 FROM storage_rows r WHERE r.kind=1 AND r.position>=c.first AND r.position<=c.last))",[]).map_err(db)?;
    tx.execute("DELETE FROM arrow_chunks WHERE json_extract(source,'$.kind')='Objects' AND NOT EXISTS(SELECT 1 FROM storage_rows r WHERE r.kind=1 AND r.position>=arrow_chunks.first AND r.position<=arrow_chunks.last)",[]).map_err(db)?;
    if !remove.is_empty() {
        set_meta(&tx, "floor", meta(&tx, "floor")?.max(through.sequence))?;
    }
    tx.commit().map_err(db)?;
    Ok(done)
}

/* === Private Functions === */

fn query<T: serde::de::DeserializeOwned>(
    connection: &Connection, snapshot: &BackendSnapshot, page: ScanPage, kind: i64, condition: &str, extra: Vec<Value>,
) -> Result<Vec<T>> {
    let sql = format!(
        "SELECT data FROM storage_rows WHERE kind=?1 AND position>?2 AND position<=?3 {condition} ORDER BY position LIMIT {}",
        page.rows.get().saturating_add(1)
    );
    let mut arguments = vec![Value::Integer(kind), Value::Integer(integer(page.after)?), Value::Integer(integer(snapshot.at.sequence)?)];
    arguments.extend(extra);
    let mut statement = connection.prepare(&sql).map_err(db)?;
    let mut rows = statement.query(rusqlite::params_from_iter(arguments)).map_err(db)?;
    let mut output = Vec::new();
    let mut bytes = 0;
    while let Some(row) = rows.next().map_err(db)? {
        if output.len() == page.rows.get() {
            break;
        }
        let data = row.get_ref(0).map_err(db)?.as_blob().map_err(|_| incompatible())?;
        if bytes + data.len() > page.bytes.get() {
            if output.is_empty() {
                return Err(exhausted());
            }
            break;
        }
        bytes += data.len();
        output.push(decode(data)?);
    }
    Ok(output)
}
fn optional(value: Option<String>) -> Value {
    value.map_or(Value::Null, Value::Text)
}
fn optional_time(value: Option<SystemTime>) -> Result<Value> {
    Ok(value.map(timestamp).transpose()?.map_or(Value::Null, Value::Integer))
}
fn timestamp(value: SystemTime) -> Result<i64> {
    let nanos = match value.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos() as i128,
        Err(error) => -(error.duration().as_nanos() as i128),
    };
    nanos.try_into().map_err(|_| invalid("timestamp exceeds nanosecond range"))
}

/* === Tests === */
