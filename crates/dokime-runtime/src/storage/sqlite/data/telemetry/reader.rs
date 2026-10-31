/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Indexed Arrow Batch Reads At A Storage Frontier

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Path;

// Third Party Imports
use harmos::storage::{SchemaKey, StorageCursor, StoredRecord, StoredRow};
use rusqlite::{Connection, types::Value};

// Local Imports
use super::{
    super::{Scan, position},
    catalog,
};
use crate::storage::{
    Error, Result, ipc,
    telemetry::{TelemetryEntry, TypedCodec},
};
use dokime_domain::model::telemetry::TelemetryContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Context<'a> {
    pub root: &'a Path,
    pub metrics: &'a super::super::super::ReadMetrics,
}

pub(in crate::storage::sqlite) struct CachedBatch {
    path: std::path::PathBuf,
    index: ipc::BatchIndex,
    projection: Option<Vec<usize>>,
    batch: arrow_array::RecordBatch,
}

pub(crate) struct NumericRow {
    pub timestamp: dokime_domain::model::time::Timestamp,
    pub sequence: u64,
    pub values: Vec<Option<f64>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn indices(connection: &Connection, id: &str) -> Result<Vec<ipc::BatchIndex>> {
    let mut statement =
        connection.prepare("SELECT offset,metadata_bytes,body_bytes,rows,sha256 FROM stream_batches WHERE file_id=?1 ORDER BY batch")?;
    Ok(statement.query_map([id], index)?.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub(super) fn entries(connection: &Connection, root: &Path, scan: &Scan) -> Result<Vec<(u64, TelemetryEntry)>> {
    projected(connection, root, scan, None, |batch, row, _, contract, scopes| {
        let mut entry = ipc::telemetry_entry(batch, row, contract)?.1;
        attach_captures(&mut entry, scopes.iter().copied());
        Ok(Some(entry))
    })
}

pub(super) fn numeric(
    connection: &Connection, root: &Path, scan: &Scan, keys: &[String], scope: &crate::evidence_runtime::ScopeSelection,
) -> Result<Vec<(u64, NumericRow)>> {
    use arrow_array::{Array, Float64Array, Int64Array, StructArray, UInt64Array};
    let mut scopes = std::collections::BTreeMap::new();
    let mut metadata = vec!["position", "at", "sequence", "timestamp"];
    if *scope != Default::default() {
        metadata.push("scope");
    }
    if scan.filters.iter().any(|(column, _)| *column == "item") {
        metadata.push("sample_id");
    }
    projected(connection, root, scan, Some((&metadata, keys)), |batch, row, _, _, captures| {
        if *scope != Default::default() {
            let encoded = ipc::metadata_text(batch, row, "scope")?;
            let matches = if let Some(matches) = scopes.get(encoded) {
                *matches
            } else {
                let actual: Option<dokime_domain::model::execution::RecordScope> = serde_json::from_str(encoded)?;
                let matches = scope.matches(actual.as_ref());
                scopes.insert(encoded.to_owned(), matches);
                matches
            };
            let captured = (scope.test_execution_id.is_some() || scope.test_cycle_id.is_some())
                && captures.iter().any(|capture| scope.matches(Some(capture)));
            if !matches && !captured {
                return Ok(None);
            }
        }
        let values = batch.column(1).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("telemetry values are not a struct"))?;
        let values = keys
            .iter()
            .map(|key| {
                let Some(column) = values.column_by_name(key).filter(|column| !column.is_null(row)) else {
                    return Ok(None);
                };
                let value = if let Some(array) = column.as_any().downcast_ref::<Float64Array>() {
                    array.value(row)
                } else if let Some(array) = column.as_any().downcast_ref::<Int64Array>() {
                    array.value(row) as f64
                } else if let Some(array) = column.as_any().downcast_ref::<UInt64Array>() {
                    array.value(row) as f64
                } else {
                    return Err(Error::Invalid("plot field is not numeric"));
                };
                Ok(value.is_finite().then_some(value))
            })
            .collect::<Result<Vec<_>>>()?;
        let metadata =
            batch.column(0).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("telemetry metadata is not a struct"))?;
        let sequence = metadata
            .column_by_name("sequence")
            .and_then(|array| array.as_any().downcast_ref::<UInt64Array>())
            .ok_or(Error::Corrupt("telemetry sequence is absent"))?
            .value(row);
        Ok(Some(NumericRow {
            timestamp: dokime_domain::model::time::Timestamp::new(ipc::metadata_text(batch, row, "timestamp")?),
            sequence,
            values,
        }))
    })
}

fn projected<T>(
    connection: &Connection, root: &Path, scan: &Scan, columns: Option<(&[&str], &[String])>,
    mut project: impl FnMut(
        &arrow_array::RecordBatch,
        usize,
        u64,
        &TelemetryContract,
        &[&dokime_domain::model::execution::RecordScope],
    ) -> Result<Option<T>>,
) -> Result<Vec<(u64, T)>> {
    let (start, end) = scan.source_window.map_or(scan.source_bounds, |(start, end)| (Some(start), Some(end)));
    let start_us = start.map(catalog::micros).transpose()?;
    let end_us = end.map(catalog::micros).transpose()?;
    let mut sql = String::from("SELECT s.file_id,s.execution,s.contract FROM streams s WHERE s.first_position<=?");
    let mut args = vec![Value::Text(position(scan.at))];
    for (column, value) in &scan.filters {
        let field = match *column {
            "schema_key" => "schema_key",
            "scope" => "execution",
            "execution" | "cycle" => {
                owner_clause(&mut sql, &mut args, column, value, false);
                continue;
            }
            "stream" => "stream",
            "item" => continue,
            _ => return Err(Error::Invalid("invalid Arrow stream selection")),
        };
        sql.push_str(&format!(" AND s.{field}=?"));
        args.push(value.clone());
    }
    let mut streams = connection.prepare(&sql)?;
    let selected = streams
        .query_map(rusqlite::params_from_iter(args), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, String>(2)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut output = Vec::new();
    for (id, execution, contract) in selected {
        let contract: TelemetryContract = serde_json::from_str(&contract)?;
        let schema = ipc::contract_schema(&contract, execution.as_deref())?;
        let projection = columns.map(|(metadata, keys)| ipc::Projection::new(schema.clone(), metadata, keys)).transpose()?;
        let mut sql = String::from(
            "SELECT b.offset,b.metadata_bytes,b.body_bytes,b.rows,b.sha256,b.batch FROM stream_batches b JOIN streams s ON s.file_id=b.file_id WHERE b.file_id=? AND b.last_position>? AND b.first_position<=?",
        );
        let mut args = vec![Value::Text(id.clone()), Value::Text(position(scan.page.after)), Value::Text(position(scan.at))];
        for (column, value) in &scan.filters {
            if matches!(*column, "execution" | "cycle") {
                owner_clause(&mut sql, &mut args, column, value, true);
            }
        }
        if let Some(start) = start_us {
            // Round The Candidate Window Outwards; Original Timestamps Apply The Exact Cut Below.
            sql.push_str(" AND b.last_at>=?");
            args.push(Value::Integer(start));
        }
        if let Some(end) = end_us {
            sql.push_str(" AND b.first_at<=?");
            args.push(Value::Integer(end));
        }
        sql.push_str(" ORDER BY b.first_position");
        let mut statement = connection.prepare(&sql)?;
        let mut batches = statement.query(rusqlite::params_from_iter(args))?;
        let mut count = 0;
        while let Some(row) = batches.next()? {
            let index = index(row)?;
            let batch_number: i64 = row.get(5)?;
            let captures = capture_ranges(connection, &id, batch_number)?;
            let batch = read_batch(
                &root.join(format!("telemetry/{id}.arrow")),
                schema.clone(),
                &index,
                projection.as_ref(),
                scan.metrics.as_deref(),
            )?;
            let (positions, times) = ipc::row_keys(&batch)?;
            let first = positions.values().partition_point(|position| *position <= scan.page.after);
            let last = positions.values().partition_point(|position| *position <= scan.at);
            let owners = scan.filters.iter().filter(|(column, _)| matches!(*column, "execution" | "cycle")).collect::<Vec<_>>();
            let mut ranges = if owners.is_empty() {
                vec![(first, last)]
            } else {
                captures
                    .iter()
                    .filter(|(_, _, scope)| {
                        owners.iter().all(|(column, value)| {
                            let actual = if *column == "execution" {
                                scope.test_execution_id.as_ref().map(|id| id.as_str())
                            } else {
                                scope.test_cycle_id.as_ref().map(|id| id.as_str())
                            };
                            actual.is_some_and(|actual| matches!(value, Value::Text(value) if value == actual))
                        })
                    })
                    .map(|(start, count, _)| ((*start).max(first), start.saturating_add(*count).min(last)))
                    .filter(|(start, end)| start < end)
                    .collect::<Vec<_>>()
            };
            ranges.sort_unstable();
            let mut selected = Vec::<(usize, usize)>::new();
            for (start, end) in ranges {
                if let Some((_, through)) = selected.last_mut()
                    && start <= *through
                {
                    *through = (*through).max(end);
                } else {
                    selected.push((start, end));
                }
            }
            'rows: for row in selected.into_iter().flat_map(|(start, end)| start..end) {
                let position = positions.value(row);
                let scopes = captures
                    .iter()
                    .filter(|(first, count, _)| row >= *first && row - *first < *count)
                    .map(|(_, _, scope)| scope)
                    .collect::<Vec<_>>();
                let time = times.value(row);
                if start_us.is_some_and(|start| time < start) || end_us.is_some_and(|end| time > end) {
                    continue;
                }
                if let Some(metrics) = &scan.metrics {
                    metrics.data_scan_rows.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    super::super::super::DATA_SCAN_ROWS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
                for (column, value) in &scan.filters {
                    if *column == "item" && *value != Value::Text(ipc::metadata_text(&batch, row, "sample_id")?.to_owned()) {
                        continue 'rows;
                    }
                }
                if start_us == Some(time) || end_us == Some(time) {
                    let time: std::time::SystemTime =
                        dokime_domain::model::time::Timestamp::new(ipc::metadata_text(&batch, row, "timestamp")?).to_datetime()?.into();
                    if start.is_some_and(|start| time < start) || end.is_some_and(|end| time >= end) {
                        continue;
                    }
                }
                let Some(value) = project(&batch, row, position, &contract, &scopes)? else {
                    continue;
                };
                output.push((position, value));
                count += 1;
                if count == scan.page.rows.get() {
                    break;
                }
            }
            if count == scan.page.rows.get() {
                break;
            }
        }
    }
    output.sort_unstable_by_key(|(position, _)| *position);
    output.truncate(scan.page.rows.get());
    Ok(output)
}

pub(super) fn stored(connection: &Connection, root: &Path, scan: &Scan) -> Result<Vec<StoredRow>> {
    let history = super::super::saved(connection)?.history;
    let mut codecs = std::collections::BTreeMap::new();
    let mut bytes = 0;
    let entries = projected(connection, root, scan, None, |batch, index, position, contract, _| {
        let (_, entry) = ipc::telemetry_entry(batch, index, contract)?;
        let schema = key(&entry)?;
        let cache_key = serde_json::to_string(&schema)?;
        if !codecs.contains_key(&cache_key) {
            let composed = catalog::composed(connection, &schema)?;
            let codec = TypedCodec::from_stored(schema.clone(), composed.schema, &harmos::config::SchemaSettings::default())?;
            codecs.insert(cache_key.clone(), codec);
        }
        let encoding = ipc::native_encoding(batch, index)?;
        Ok(Some(envelope(position, &entry, &history, &codecs[&cache_key], encoding)?))
    })?;
    let mut output = Vec::new();
    for (_, row) in entries {
        let length = super::super::entry::stream_length(&row, usize::MAX)?;
        if !output.is_empty() && length > scan.page.bytes.get().saturating_sub(bytes) {
            break;
        }
        bytes += length;
        output.push(row);
    }
    Ok(output)
}

pub(super) fn positioned(connection: &Connection, root: &Path, schema: &SchemaKey, position: u64) -> Result<Option<StoredRow>> {
    let page = harmos::storage::ScanPage {
        after: position.saturating_sub(1),
        rows: std::num::NonZeroUsize::new(1).unwrap(),
        bytes: std::num::NonZeroUsize::MAX,
    };
    let scan = Scan::new("stream", position, 0, page).filter("schema_key", Some(serde_json::to_string(schema)?));
    Ok(stored(connection, root, &scan)?.pop())
}

pub(super) fn envelope(
    position: u64, entry: &TelemetryEntry, history: &harmos::storage::StoreHistory, codec: &TypedCodec, encoding: (&str, &str),
) -> Result<StoredRow> {
    let sample = &entry.sample;
    Ok(StoredRow {
        stream: sample.stream.clone(),
        sequence: entry.ordinal,
        record: StoredRecord {
            position: StorageCursor {
                history: history.clone(),
                sequence: position,
            },
            schema: codec.schema().clone(),
            scope: sample
                .scope
                .as_ref()
                .and_then(|scope| scope.test_execution_id.as_ref())
                .map(|id| harmos::Identifier::new(id.as_str()))
                .transpose()?,
            item: harmos::Identifier::new(sample.sample_id.as_str())?,
            source: Some(harmos::Identifier::new(entry.epoch.as_str())?),
            source_time: Some(sample.timestamp.to_datetime()?.into()),
            payload: codec.encode_arrow(entry, encoding.0, encoding.1)?,
        },
    })
}

pub(super) fn key(entry: &TelemetryEntry) -> Result<SchemaKey> {
    Ok(SchemaKey {
        id: harmos::Identifier::new(format!("dokime.telemetry.{}", entry.sample.stream))?,
        version: entry.sample.schema_version.try_into().map_err(|_| Error::Corrupt("telemetry schema version is zero"))?,
        payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
    })
}

pub(super) fn latest(
    connection: &Connection, context: Context<'_>, at: &StorageCursor, schema: &SchemaKey, terms: &[(&str, Value)],
    selection: &crate::evidence_runtime::TelemetrySelection,
) -> Result<Option<std::time::SystemTime>> {
    let Context { root, metrics } = context;
    let mut sql = String::from(
        "SELECT s.file_id,s.execution,s.contract,b.offset,b.metadata_bytes,b.body_bytes,b.rows,b.last_at,b.sha256,b.batch FROM streams s JOIN stream_batches b ON s.file_id=b.file_id WHERE s.schema_key=? AND b.first_position<=?",
    );
    let mut args = vec![Value::Text(serde_json::to_string(schema)?), Value::Text(position(at.sequence))];
    batch_owners(&mut sql, &mut args, selection, terms);
    if let Some(start) = &selection.start {
        sql.push_str(" AND b.last_at>=?");
        args.push(Value::Integer(catalog::micros(start.to_datetime()?.into())?));
    }
    if let Some(end) = &selection.end {
        sql.push_str(" AND b.first_at<=?");
        args.push(Value::Integer(catalog::micros(end.to_datetime()?.into())?));
    }
    sql.push_str(" ORDER BY b.last_at DESC,b.last_position DESC");
    let mut statement = connection.prepare(&sql)?;
    let mut batches = statement.query(rusqlite::params_from_iter(args))?;
    let mut latest: Option<std::time::SystemTime> = None;
    while let Some(row) = batches.next()? {
        let last_at: i64 = row.get(7)?;
        if latest.is_some_and(|latest| catalog::micros(latest).is_ok_and(|latest| last_at < latest)) {
            break;
        }
        let id: String = row.get(0)?;
        let execution: Option<String> = row.get(1)?;
        let contract: TelemetryContract = serde_json::from_str(&row.get::<_, String>(2)?)?;
        let schema = ipc::contract_schema(&contract, execution.as_deref())?;
        let index = ipc::BatchIndex {
            offset: catalog::unsigned(row, 3)?,
            metadata_bytes: row.get(4)?,
            body_bytes: catalog::unsigned(row, 5)?,
            rows: catalog::unsigned(row, 6)?,
            sha256: catalog::digest(row, 8)?,
        };
        let batch = read_batch(&root.join(format!("telemetry/{id}.arrow")), schema, &index, None, Some(metrics))?;
        let captures = capture_ranges(connection, &id, row.get(9)?)?;
        for row in (0..batch.num_rows()).rev() {
            let (position, time) = ipc::row_key(&batch, row)?;
            if position > at.sequence || latest.is_some_and(|latest| catalog::micros(latest).is_ok_and(|latest| time < latest)) {
                continue;
            }
            metrics.telemetry_anchor_rows.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            super::super::super::TELEMETRY_ANCHOR_ROWS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let (position, mut entry) = ipc::telemetry_entry(&batch, row, &contract)?;
            attach_captures(
                &mut entry,
                captures.iter().filter(|(first, count, _)| row >= *first && row - *first < *count).map(|(_, _, scope)| scope),
            );
            if position > at.sequence || !matches(&entry, terms, selection)? {
                continue;
            }
            let time: std::time::SystemTime = entry.sample.timestamp.to_datetime()?.into();
            if latest.is_none_or(|prior| prior < time) {
                latest = Some(time);
            }
        }
    }
    Ok(latest)
}

pub(super) fn predecessors(
    connection: &Connection, context: Context<'_>, at: &StorageCursor, schema: &SchemaKey, terms: &[(&str, Value)],
    selection: &crate::evidence_runtime::TelemetrySelection, boundary: (std::time::SystemTime, &str),
) -> Result<(String, Vec<(u64, TelemetryEntry)>)> {
    let Context { root, metrics } = context;
    let (start, after) = boundary;
    let mut sql = String::from(
        "SELECT r.scope,s.file_id,s.execution,s.contract,b.offset,b.metadata_bytes,b.body_bytes,b.rows,b.sha256,r.last_at,b.batch
        FROM stream_batch_scopes r JOIN stream_batches b ON b.file_id=r.file_id AND b.batch=r.batch
        JOIN streams s ON s.file_id=b.file_id WHERE s.schema_key=? AND b.first_position<=? AND r.first_at<=? AND r.scope>?",
    );
    let mut args = vec![
        Value::Text(serde_json::to_string(schema)?),
        Value::Text(position(at.sequence)),
        Value::Integer(catalog::micros(start)?),
        Value::Text(after.to_owned()),
    ];
    batch_owners(&mut sql, &mut args, selection, terms);
    sql.push_str(" ORDER BY r.scope,r.last_at DESC,b.last_position DESC");
    let mut statement = connection.prepare(&sql)?;
    let mut batches = statement.query(rusqlite::params_from_iter(args))?;
    let mut groups = std::collections::BTreeMap::new();
    let mut selection = selection.clone();
    selection.start = None;
    while let Some(row) = batches.next()? {
        let scope: String = row.get(0)?;
        let last: i64 = row.get(9)?;
        if groups
            .get(&scope)
            .is_some_and(|(time, _, _): &(std::time::SystemTime, u64, TelemetryEntry)| catalog::micros(*time).is_ok_and(|time| time > last))
        {
            continue;
        }
        let id: String = row.get(1)?;
        let execution: Option<String> = row.get(2)?;
        let contract: TelemetryContract = serde_json::from_str(&row.get::<_, String>(3)?)?;
        let schema = ipc::contract_schema(&contract, execution.as_deref())?;
        let index = ipc::BatchIndex {
            offset: catalog::unsigned(row, 4)?,
            metadata_bytes: row.get(5)?,
            body_bytes: catalog::unsigned(row, 6)?,
            rows: catalog::unsigned(row, 7)?,
            sha256: catalog::digest(row, 8)?,
        };
        let batch = read_batch(&root.join(format!("telemetry/{id}.arrow")), schema, &index, None, Some(metrics))?;
        let captures = capture_ranges(connection, &id, row.get(10)?)?;
        for row in (0..batch.num_rows()).rev() {
            let (position, time) = ipc::row_key(&batch, row)?;
            if position > at.sequence
                || time > catalog::micros(start)?
                || groups.get(&scope).is_some_and(|(prior, _, _)| catalog::micros(*prior).is_ok_and(|prior| time < prior))
            {
                continue;
            }
            let (_, mut entry) = ipc::telemetry_entry(&batch, row, &contract)?;
            attach_captures(
                &mut entry,
                captures.iter().filter(|(first, count, _)| row >= *first && row - *first < *count).map(|(_, _, scope)| scope),
            );
            let time: std::time::SystemTime = entry.sample.timestamp.to_datetime()?.into();
            if time >= start
                || !matches(&entry, terms, &selection)?
                || serde_json::to_string(&entry.sample.scope.as_ref().map(crate::storage::records::ScopeRow::from))? != scope
            {
                continue;
            }
            if groups.get(&scope).is_none_or(|(prior, prior_position, _)| (time, position) > (*prior, *prior_position)) {
                groups.insert(scope.clone(), (time, position, entry));
            }
        }
    }
    let mut cursor = after.to_owned();
    let mut output = Vec::new();
    for (scope, (_, position, entry)) in groups.into_iter().take(256) {
        cursor = scope;
        output.push((position, entry));
    }
    Ok((cursor, output))
}

pub(super) fn selected(entry: &TelemetryEntry, source: &harmos::storage::query::StreamSelection) -> Result<bool> {
    if key(entry)? != source.schema
        || source.stream.as_ref().is_some_and(|stream| stream != &entry.sample.stream)
        || source.item.as_ref().is_some_and(|item| item.as_str() != entry.sample.sample_id.as_str())
        || source.scope.as_ref().is_some_and(|scope| {
            !entry
                .sample
                .scope
                .iter()
                .chain(&entry.sample.captured_scopes)
                .any(|actual| actual.test_execution_id.as_ref().is_some_and(|id| id.as_str() == scope.as_str()))
        })
    {
        return Ok(false);
    }
    let time: std::time::SystemTime = entry.sample.timestamp.to_datetime()?.into();
    Ok(source.source_start.is_none_or(|start| time >= start) && source.source_end.is_none_or(|end| time < end))
}

/* === Private Functions === */

fn attach_captures<'a>(entry: &mut TelemetryEntry, scopes: impl Iterator<Item = &'a dokime_domain::model::execution::RecordScope>) {
    entry.sample.captured_scopes = scopes.filter(|scope| Some(*scope) != entry.sample.scope.as_ref()).cloned().collect();
}

fn batch_owners(sql: &mut String, args: &mut Vec<Value>, selection: &crate::evidence_runtime::TelemetrySelection, terms: &[(&str, Value)]) {
    if let Some(execution) = &selection.scope.test_execution_id {
        owner_clause(sql, args, "execution", &Value::Text(execution.to_string()), true);
    }
    if let Some(cycle) = &selection.scope.test_cycle_id {
        owner_clause(sql, args, "cycle", &Value::Text(cycle.to_string()), true);
    }
    for (path, value) in terms {
        let column = match *path {
            "payload.metadata.scope.test_execution_id" if selection.scope.test_execution_id.is_none() => "execution",
            "payload.metadata.scope.test_cycle_id" if selection.scope.test_cycle_id.is_none() => "cycle",
            _ => continue,
        };
        owner_clause(sql, args, column, value, true);
    }
}

fn owner_clause(sql: &mut String, args: &mut Vec<Value>, column: &str, value: &Value, batch: bool) {
    if batch {
        sql.push_str(&format!(" AND (b.file_id,b.batch) IN (SELECT DISTINCT file_id,batch FROM stream_captures WHERE {column}=?)"));
    } else {
        sql.push_str(&format!(" AND s.file_id IN (SELECT DISTINCT file_id FROM stream_captures WHERE {column}=?)"));
    }
    args.push(value.clone());
}

fn capture_ranges(
    connection: &Connection, file: &str, batch: i64,
) -> Result<Vec<(usize, usize, dokime_domain::model::execution::RecordScope)>> {
    let mut statement = connection.prepare_cached("SELECT first_row,rows,scope FROM stream_captures WHERE file_id=?1 AND batch=?2")?;
    let rows = statement.query_map(rusqlite::params![file, batch], |row| {
        Ok((catalog::unsigned(row, 0)?, catalog::unsigned(row, 1)?, row.get::<_, String>(2)?))
    })?;
    rows.map(|row| {
        let (first, count, scope) = row?;
        Ok((
            usize::try_from(first).map_err(|_| Error::Corrupt("capture row exceeds address space"))?,
            usize::try_from(count).map_err(|_| Error::Corrupt("capture range exceeds address space"))?,
            serde_json::from_str(&scope)?,
        ))
    })
    .collect()
}

fn read_batch(
    path: &Path, schema: arrow_schema::SchemaRef, index: &ipc::BatchIndex, projection: Option<&ipc::Projection>,
    metrics: Option<&super::super::super::ReadMetrics>,
) -> Result<arrow_array::RecordBatch> {
    if let Some(metrics) = metrics {
        let mut cached = metrics.telemetry_batch.lock().unwrap();
        if let Some(prior) = cached.as_ref()
            && prior.path == path
            && prior.index == *index
            && prior.projection.as_deref() == projection.map(ipc::Projection::columns)
        {
            return Ok(prior.batch.clone());
        }
        let batch = match projection {
            Some(projection) => ipc::read_projected_batch(path, index, projection)?,
            None => ipc::read_batch(path, schema, index)?,
        };
        *cached = Some(CachedBatch {
            path: path.to_owned(),
            index: index.clone(),
            projection: projection.map(|projection| projection.columns().to_vec()),
            batch: batch.clone(),
        });
        Ok(batch)
    } else {
        match projection {
            Some(projection) => ipc::read_projected_batch(path, index, projection),
            None => ipc::read_batch(path, schema, index),
        }
    }
}

pub(super) fn matches(
    entry: &TelemetryEntry, terms: &[(&str, Value)], selection: &crate::evidence_runtime::TelemetrySelection,
) -> Result<bool> {
    let sample = &entry.sample;
    if !selection.scope.matches_evidence(sample.scope.as_ref(), &sample.captured_scopes)
        || !selection
            .signals
            .iter()
            .all(|signal| sample.values.get(signal).is_some_and(|value| !matches!(value, dokime_domain::model::values::Value::Null)))
        || !selection.sources.is_empty() && !selection.sources.contains(&sample.source)
    {
        return Ok(false);
    }
    let time = sample.timestamp.to_datetime()?;
    if selection.start.as_ref().is_some_and(|start| start.to_datetime().is_ok_and(|start| time < start))
        || selection.end.as_ref().is_some_and(|end| end.to_datetime().is_ok_and(|end| time >= end))
    {
        return Ok(false);
    }
    let captures = selection.scope.test_execution_id.is_some()
        || selection.scope.test_cycle_id.is_some()
        || terms
            .iter()
            .any(|(path, _)| matches!(*path, "payload.metadata.scope.test_execution_id" | "payload.metadata.scope.test_cycle_id"));
    for scope in std::iter::once(sample.scope.as_ref()).chain(sample.captured_scopes.iter().filter(|_| captures).map(Some)) {
        if !selection.scope.matches(scope) {
            continue;
        }
        if scope_terms(scope, terms)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn scope_terms(scope: Option<&dokime_domain::model::execution::RecordScope>, terms: &[(&str, Value)]) -> Result<bool> {
    let scope = serde_json::to_value(scope.map(crate::storage::records::ScopeRow::from))?;
    for (path, value) in terms {
        if *path == "$scope" {
            if *value != Value::Text(serde_json::to_string(&scope)?) {
                return Ok(false);
            }
            continue;
        }
        let path = path.strip_prefix("payload.metadata.scope.").ok_or(Error::Invalid("invalid telemetry scope path"))?;
        let mut field = &scope;
        for name in path.split('.') {
            field = field.get(name).unwrap_or(&serde_json::Value::Null);
        }
        let matches = match value {
            Value::Text(text) => field.as_str() == Some(text.as_str()),
            Value::Integer(number) => field.as_i64() == Some(*number),
            _ => false,
        };
        if !matches {
            return Ok(false);
        }
    }
    Ok(true)
}

fn index(row: &rusqlite::Row<'_>) -> rusqlite::Result<ipc::BatchIndex> {
    Ok(ipc::BatchIndex {
        offset: catalog::unsigned(row, 0)?,
        metadata_bytes: row.get(1)?,
        body_bytes: catalog::unsigned(row, 2)?,
        rows: catalog::unsigned(row, 3)?,
        sha256: catalog::digest(row, 4)?,
    })
}

/* === Tests === */
