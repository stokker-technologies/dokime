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
use std::{collections::BTreeSet, io::Cursor};

// Third Party Imports
use arrow_array::{Array, Float64Array, Int64Array, RecordBatch, StringArray, TimestampNanosecondArray, UInt64Array};
use arrow_ipc::{reader::StreamReader, writer::StreamWriter};
use rusqlite::{Connection, OptionalExtension, params};

// Local Imports
use super::*;
use harmos_query::columnar::ColumnKind;
use harmos_query::columnar::ColumnSource;

use harmos_query::columnar::ScanEvidence;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn store(connection: &Connection, source: &ColumnSource, batch: &RecordBatch, settings: BackendSettings) -> Result<()> {
    let flat = harmos_query::columnar::flatten(batch)?;
    if batch.num_rows() == 0 {
        return Ok(());
    }
    if positions(&flat)?.values().windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(invalid("Write Arrow: Positions Must Increase"));
    }
    // Keep Each Physical Chunk Within One Scope and Object Identity for Reclamation.
    let scopes =
        flat.column_by_name("_storage.scope").and_then(|column| column.as_any().downcast_ref::<StringArray>()).ok_or_else(incompatible)?;
    let items = if source.kind == ColumnKind::Objects {
        Some(
            flat.column_by_name("_storage.item")
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
                .ok_or_else(incompatible)?,
        )
    } else {
        None
    };
    let scope = |row| (!scopes.is_null(row)).then(|| scopes.value(row));
    let mut pending = Vec::new();
    let mut first = 0;
    for row in 1..batch.num_rows() {
        let changed_scope = scope(row) != scope(row - 1);
        let changed_item = items.is_some_and(|items| items.value(row) != items.value(row - 1));
        if changed_scope || changed_item {
            pending.push(first..row);
            first = row;
        }
    }
    pending.push(first..batch.num_rows());
    pending.reverse();
    while let Some(range) = pending.pop() {
        let mut part = batch.slice(range.start, range.len());
        if part.get_array_memory_size() > settings.chunk_bytes.get() {
            let indices = (0..part.num_rows()).map(|row| row as u64).collect::<Vec<_>>();
            part = harmos_query::columnar::take(&part, &indices)?;
        }
        if part.num_rows() > settings.chunk_rows.get() || part.get_array_memory_size() > settings.chunk_bytes.get() {
            if range.len() == 1 {
                return Err(exhausted());
            }
            let middle = range.start + range.len() / 2;
            pending.push(middle..range.end);
            pending.push(range.start..middle);
            continue;
        }
        store_chunk(connection, source, &part)?;
    }
    Ok(())
}

fn store_chunk(connection: &Connection, source: &ColumnSource, batch: &RecordBatch) -> Result<()> {
    let flat = harmos_query::columnar::flatten(batch)?;
    let positions = positions(&flat)?;
    let directory = directory(connection)?;
    std::fs::create_dir_all(&directory).map_err(io)?;
    let filename = format!("{}.arrow", uuid::Uuid::now_v7());
    let path = directory.join(&filename);
    let mut writer = crate::ipc::Writer::create(&path, batch.schema()).map_err(product)?;
    let block = writer.append(batch).map_err(product)?;
    writer.finish().map_err(product)?;
    drop(writer);
    crate::ipc::sync_parent(&path).map_err(product)?;
    // The Compressed File Is Synced Before Its Catalog Entry Can Commit.
    connection
        .execute(
            "INSERT INTO arrow_chunks(source,first,last,rows,file,block) VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                text(source)?,
                integer(positions.value(0))?,
                integer(positions.value(batch.num_rows() - 1))?,
                batch.num_rows() as i64,
                filename,
                encode(&block)?
            ],
        )
        .map_err(db)?;
    let chunk = connection.last_insert_rowid();
    for (field, column) in flat.schema().fields().iter().zip(flat.columns()) {
        let range = column
            .as_any()
            .downcast_ref::<Float64Array>()
            .map(|column| {
                let finite = column.iter().flatten().filter(|value| value.is_finite()).collect::<Vec<_>>();
                (finite.iter().copied().reduce(f64::min), finite.iter().copied().reduce(f64::max))
            })
            .unwrap_or((None, None));
        let time = if let Some(array) = column.as_any().downcast_ref::<Int64Array>() {
            (array.iter().flatten().min(), array.iter().flatten().max())
        } else if let Some(array) = column.as_any().downcast_ref::<TimestampNanosecondArray>() {
            (array.iter().flatten().min(), array.iter().flatten().max())
        } else {
            (None, None)
        };
        connection
            .execute(
                "INSERT INTO arrow_columns VALUES(?1,?2,?3,?4,?5,?6,?7)",
                params![chunk, field.name(), column.get_array_memory_size() as i64, range.0, range.1, time.0, time.1],
            )
            .map_err(db)?;
    }
    super::index::maintain(connection, source, &flat)?;
    Ok(())
}

pub(super) fn directory(connection: &Connection) -> Result<std::path::PathBuf> {
    let path = connection.path().ok_or_else(|| invalid("Open Arrow: Use a Durable Catalog Path"))?;
    let path = std::path::Path::new(path);
    Ok(path.with_extension("arrow"))
}

pub(super) fn read_chunk(connection: &Connection, chunk: i64, metadata: &[&str], values: &[String], maximum: usize) -> Result<RecordBatch> {
    let (filename, block): (String, Vec<u8>) = connection
        .query_row("SELECT file,block FROM arrow_chunks WHERE id=?1", [chunk], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(db)?;
    let block = decode::<crate::ipc::BatchIndex>(&block)?;
    if std::path::Path::new(&filename).components().count() != 1 {
        return Err(incompatible());
    }
    let path = directory(connection)?.join(filename);
    let reader = arrow_ipc::reader::FileReader::try_new(std::fs::File::open(&path).map_err(io)?, None)
        .map_err(|source| Error::backend("Read Arrow Schema", BackendFailure::Corrupt, source))?;
    let projection = crate::ipc::Projection::new(reader.schema(), metadata, values).map_err(product)?;
    crate::ipc::read_query_batch(&path, &block, &projection, maximum).map_err(product)
}

fn io(source: std::io::Error) -> Error {
    Error::backend("Access Arrow Stream", BackendFailure::Permanent, source)
}
fn product(source: crate::Error) -> Error {
    Error::backend("Process Arrow Stream", BackendFailure::Corrupt, source)
}

pub(super) fn scan(state: &State, snapshot: &BackendSnapshot, read: &ArrowScan, cancel: &Cancellation, history: bool) -> Result<ArrowPage> {
    cancel.check()?;
    state.snapshot(snapshot)?;
    let after = read.position(snapshot)?;
    let schema = match rows::find_schema(&state.connection, &read.source.schema, snapshot.at.sequence)? {
        Some(schema) => schema,
        None => {
            let definition = state
                .catalog
                .definitions()
                .iter()
                .find(|definition| definition.schema().is_ok_and(|key| key == read.source.schema))
                .ok_or_else(|| invalid("schema is unavailable at requested cut"))?;
            let graph = definition.graph(&Default::default())?;
            let position = StorageCursor {
                history: state.history.clone(),
                sequence: 0,
            };
            StorageSchema::new(position, read.source.schema.clone(), Arc::new(graph))
        }
    };
    let empty = harmos_query::columnar::empty(&schema, read.source.kind)?;
    let required = read.required_columns();
    for name in &required {
        if empty.column_by_name(name).is_none() {
            return Err(invalid("requested column is unavailable"));
        }
    }
    let source = text(&read.source)?;
    let chunk: Option<(i64, u64, u64, usize)> = state
        .connection
        .query_row(
            "SELECT id,first,last,rows FROM arrow_chunks WHERE source=?1 AND last>?2 AND first<=?3 ORDER BY first LIMIT 1",
            params![source, integer(after)?, integer(snapshot.at.sequence)?],
            |row| Ok((row.get(0)?, unsigned(row, 1)?, unsigned(row, 2)?, size(row, 3)?)),
        )
        .optional()
        .map_err(db)?;
    let Some((chunk, _first, last, count)) = chunk else {
        return Ok(harmos_query::columnar::finish_page(
            snapshot,
            read,
            &empty,
            ScanEvidence {
                complete: true,
                bytes: 0,
                columns: Vec::new(),
                payload_rows: 0,
            },
            cancel,
        )?);
    };
    let complete = last >= snapshot.at.sequence
        || !state
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM arrow_chunks WHERE source=?1 AND last>?2 AND first<=?3)",
                params![source, integer(last)?, integer(snapshot.at.sequence)?],
                |row| row.get::<_, bool>(0),
            )
            .map_err(db)?;
    let mut pruned = false;
    if let Some(predicate) = &read.predicate {
        let (minimum, maximum): (Option<f64>, Option<f64>) = state
            .connection
            .query_row("SELECT minimum,maximum FROM arrow_columns WHERE chunk=?1 AND name=?2", params![chunk, predicate.column], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(db)?;
        pruned |= predicate.lower.zip(maximum).is_some_and(|(lower, maximum)| maximum < lower)
            || predicate.upper.zip(minimum).is_some_and(|(upper, minimum)| minimum >= upper);
    }
    if let Some(time) = &read.time {
        let (minimum, maximum): (Option<i64>, Option<i64>) = state
            .connection
            .query_row("SELECT time_min,time_max FROM arrow_columns WHERE chunk=?1 AND name=?2", params![chunk, time.column], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(db)?;
        pruned |= maximum.is_some_and(|maximum| maximum < time.lower_ns) || minimum.is_some_and(|minimum| minimum >= time.upper_ns);
    }
    if pruned {
        let mut page = harmos_query::columnar::finish_page(
            snapshot,
            read,
            &empty,
            ScanEvidence {
                complete,
                bytes: 0,
                columns: Vec::new(),
                payload_rows: 0,
            },
            cancel,
        )?;
        if let Some(next) = &mut page.next {
            next.position = last.min(snapshot.at.sequence);
        }
        return Ok(page);
    }
    if count > read.work.rows.get() {
        return Err(exhausted());
    }
    let bytes_read = required.iter().try_fold(0usize, |bytes, name| {
        let length: usize = state
            .connection
            .query_row("SELECT data FROM arrow_columns WHERE chunk=?1 AND name=?2", params![chunk, name], |row| size(row, 0))
            .map_err(db)?;
        Ok::<_, Error>(bytes.saturating_add(length))
    })?;
    if bytes_read.saturating_mul(4) > read.work.bytes.get() {
        return Err(exhausted());
    }
    let metadata = required.iter().filter_map(|name| name.strip_prefix("_storage.")).collect::<Vec<_>>();
    let payload = required
        .iter()
        .filter_map(|name| name.strip_prefix("payload.").map(|path| path.split('.').next().unwrap().to_owned()))
        .collect::<Vec<_>>();
    let projected = read_chunk(&state.connection, chunk, &metadata, &payload, read.work.bytes.get())?;
    let mut batch = harmos_query::columnar::flatten(&projected)?;
    let mut counters = state.work.get();
    counters.source_rows += count as u64;
    counters.source_bytes += bytes_read as u64;
    counters.decoded_chunks += 1;
    state.work.set(counters);
    if bytes_read.saturating_add(batch.get_array_memory_size().saturating_mul(3)) > read.work.bytes.get() {
        return Err(exhausted());
    }
    if read.source.kind == ColumnKind::Objects && !history {
        let key = text(&read.source.schema)?;
        let mut query = state.connection.prepare("SELECT position FROM storage_rows WHERE kind=1 AND schema_key=?1 AND position>=?3 AND position<=?4 AND position<=?2 AND position=(SELECT max(o.position) FROM storage_rows o WHERE o.kind=1 AND o.schema_key=?1 AND o.item=storage_rows.item AND o.position<=?2)").map_err(db)?;
        let retained = query
            .query_map(params![key, integer(snapshot.at.sequence)?, integer(_first)?, integer(last)?], |row| unsigned(row, 0))
            .map_err(db)?
            .collect::<std::result::Result<BTreeSet<_>, _>>()
            .map_err(db)?;
        let indices = positions(&batch)?
            .values()
            .iter()
            .enumerate()
            .filter_map(|(index, position)| retained.contains(position).then_some(index as u64))
            .collect::<Vec<_>>();
        batch = harmos_query::columnar::take(&batch, &indices)?;
    }
    let mut indices = Vec::new();
    let visible = compound::visible_positions(&state.connection, &read.source, _first, last, snapshot.at.sequence)?;
    for (index, position) in positions(&batch)?.values().iter().enumerate() {
        if visible.contains(position) {
            indices.push(index as u64);
        }
    }
    batch = harmos_query::columnar::take(&batch, &indices)?;
    let final_position = positions(&batch)?.values().iter().copied().filter(|position| *position <= snapshot.at.sequence).max();
    let mut page = harmos_query::columnar::finish_page(
        snapshot,
        read,
        &batch,
        ScanEvidence {
            complete,
            bytes: bytes_read,
            columns: required,
            payload_rows: 0,
        },
        cancel,
    )?;
    if let Some(next) = &mut page.next
        && final_position.is_none_or(|position| next.position >= position)
    {
        next.position = last.min(snapshot.at.sequence);
    }
    Ok(page)
}

pub(super) fn ipc(batch: &RecordBatch) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut writer = StreamWriter::try_new(&mut data, &batch.schema()).map_err(|_| invalid("Arrow IPC schema encoding failed"))?;
    writer.write(batch).map_err(|_| invalid("Arrow IPC encoding failed"))?;
    writer.finish().map_err(|_| invalid("Arrow IPC finish failed"))?;
    drop(writer);
    Ok(data)
}

pub(super) fn from_ipc(data: &[u8]) -> Result<RecordBatch> {
    let mut reader = StreamReader::try_new(Cursor::new(data), None).map_err(|_| incompatible())?;
    let batch = reader.next().transpose().map_err(|_| incompatible())?.ok_or_else(incompatible)?;
    if reader.next().is_some() {
        return Err(incompatible());
    }
    Ok(batch)
}

pub(super) fn collect_orphans(connection: &Connection) -> Result<()> {
    let directory = directory(connection)?;
    if !directory.exists() {
        return Ok(());
    }
    let mut query = connection.prepare("SELECT file FROM arrow_chunks").map_err(db)?;
    let referenced =
        query.query_map([], |row| row.get::<_, String>(0)).map_err(db)?.collect::<std::result::Result<BTreeSet<_>, _>>().map_err(db)?;
    for entry in std::fs::read_dir(&directory).map_err(io)? {
        let entry = entry.map_err(io)?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let owned = name.strip_suffix(".arrow").is_some_and(|id| uuid::Uuid::parse_str(id).is_ok());
        if !owned || referenced.contains(&name) {
            continue;
        }
        if !entry.file_type().map_err(io)?.is_file() {
            return Err(invalid("Collect Arrow Garbage: Expected an Owned Regular File"));
        }
        std::fs::remove_file(entry.path()).map_err(io)?;
    }
    Ok(())
}

/* === Private Functions === */

fn positions(batch: &RecordBatch) -> Result<&UInt64Array> {
    batch
        .column_by_name("_storage.position")
        .and_then(|array| array.as_any().downcast_ref::<UInt64Array>())
        .ok_or_else(|| invalid("native source has no positions"))
}

/* === Tests === */
