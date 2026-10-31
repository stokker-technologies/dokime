/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Telemetry Materialization And Its Portable Catalog

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports
use rusqlite::Connection;
use serde::Serialize;
use sha2::{Digest, Sha256};

// Local Imports
use super::catalog;
use crate::storage::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Serialize)]
pub(crate) struct StreamFile {
    pub id: String,
    pub stream: String,
    pub contract: dokime_domain::model::telemetry::TelemetryContract,
    pub size_bytes: u64,
    pub sha256: [u8; 32],
    pub rows: u64,
    pub first_at: Option<i64>,
    pub last_at: Option<i64>,
    pub batches: Vec<Batch>,
    pub summaries: Vec<Summary>,
    #[serde(skip)]
    pub path: PathBuf,

    #[serde(skip)]
    _directory: Option<std::sync::Arc<tempfile::TempDir>>,
}

#[derive(Clone, Serialize)]
pub(crate) struct Batch {
    pub offset: u64,
    pub metadata_bytes: u32,
    pub body_bytes: u64,
    pub first_row: u64,
    pub rows: u64,
    pub first_at: i64,
    pub last_at: i64,
    pub sha256: [u8; 32],
}

#[derive(Clone, Serialize)]
pub(crate) struct Summary {
    pub signal: String,
    pub count: u64,
    pub minimum: f64,
    pub maximum: f64,
    pub sum: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn files(
    connection: &Connection, root: &std::path::Path, execution: &str, at: &harmos::storage::StorageCursor,
) -> Result<Vec<StreamFile>> {
    super::super::saved(connection)?.history.check(&at.history)?;
    let mut statement = connection.prepare(
        "SELECT DISTINCT s.schema_key,s.contract FROM streams s WHERE s.file_id IN (
            SELECT file_id FROM streams WHERE execution=?1
            UNION SELECT file_id FROM stream_captures WHERE execution=?1) ORDER BY s.schema_key",
    )?;
    let mut contracts = statement.query([execution])?;
    let directory = std::sync::Arc::new(tempfile::tempdir()?);
    let mut files = Vec::new();
    while let Some(row) = contracts.next()? {
        let key: harmos::storage::SchemaKey = serde_json::from_str(&row.get::<_, String>(0)?)?;
        if let Some(file) = native_file(connection, root, execution, &key, at)? {
            files.push(file);
            continue;
        }
        let contract: dokime_domain::model::telemetry::TelemetryContract = serde_json::from_str(&row.get::<_, String>(1)?)?;
        let composed = catalog::composed(connection, &key)?;
        let codec = crate::storage::telemetry::TypedCodec::from_stored(key.clone(), composed.schema, &Default::default())?;
        let digest = Sha256::digest(format!("{execution}|{}", serde_json::to_string(&key)?));
        let id = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let path = directory.path().join(format!("{id}.arrow"));
        let schema = crate::storage::ipc::contract_schema(&contract, Some(execution))?;
        let mut writer = crate::storage::ipc::Writer::create(&path, schema.clone())?;
        let mut file = StreamFile {
            id,
            stream: contract.stream.to_string(),
            contract,
            size_bytes: 0,
            sha256: [0; 32],
            rows: 0,
            first_at: None,
            last_at: None,
            batches: Vec::new(),
            summaries: Vec::new(),
            path,
            _directory: Some(directory.clone()),
        };
        let mut summaries = std::collections::BTreeMap::<String, Summary>::new();
        let mut after = 0;
        loop {
            let page = harmos::storage::ScanPage {
                after,
                rows: std::num::NonZeroUsize::new(512).unwrap(),
                bytes: std::num::NonZeroUsize::MAX,
            };
            let scan = super::super::Scan::new("stream", at.sequence, 0, page)
                .filter("schema_key", Some(serde_json::to_string(&key)?))
                .filter("execution", Some(execution.to_owned()));
            let rows = super::reader::stored(connection, root, &scan)?;
            if rows.is_empty() {
                break;
            }
            let mut entries = Vec::with_capacity(rows.len());
            let mut times = Vec::with_capacity(rows.len());
            for row in rows {
                after = row.record.position.sequence;
                let entry = codec.decode_admission(&row)?;
                times.push(catalog::micros(entry.sample.timestamp.to_datetime()?.into())?);
                for (signal, value) in &entry.sample.values {
                    let Some(value) = dokime_domain::model::values::numeric_value(value).filter(|value| value.is_finite()) else {
                        continue;
                    };
                    summaries
                        .entry(signal.clone())
                        .and_modify(|summary| {
                            summary.count += 1;
                            summary.minimum = summary.minimum.min(value);
                            summary.maximum = summary.maximum.max(value);
                            summary.sum += value;
                        })
                        .or_insert(Summary {
                            signal: signal.clone(),
                            count: 1,
                            minimum: value,
                            maximum: value,
                            sum: value,
                        });
                }
                entries.push((after, entry));
            }
            let first = *times.iter().min().ok_or(Error::Invalid("empty export batch"))?;
            let last = *times.iter().max().unwrap();
            let batch = crate::storage::ipc::record_batch(schema.clone(), &entries)?;
            let index = writer.append(&batch)?;
            file.batches.push(Batch {
                offset: index.offset,
                metadata_bytes: index.metadata_bytes,
                body_bytes: index.body_bytes,
                first_row: file.rows,
                rows: index.rows,
                first_at: first,
                last_at: last,
                sha256: index.sha256,
            });
            file.rows += index.rows;
            file.first_at = Some(file.first_at.map_or(first, |prior| prior.min(first)));
            file.last_at = Some(file.last_at.map_or(last, |prior| prior.max(last)));
        }
        (file.size_bytes, file.sha256) = writer.finish()?;
        file.summaries = summaries.into_values().collect();
        if file.rows > 0 {
            files.push(file);
        }
    }
    Ok(files)
}
/* === Private Functions === */

// A Complete Owned File Retains Its Original Bytes And Catalog Identity.
fn native_file(
    connection: &Connection, root: &std::path::Path, execution: &str, key: &harmos::storage::SchemaKey, at: &harmos::storage::StorageCursor,
) -> Result<Option<StreamFile>> {
    let mut query = connection.prepare(
        "SELECT s.file_id,s.stream,s.contract,f.size_bytes,f.sha256,s.rows,s.first_at,s.last_at,f.path
        FROM streams s JOIN files f ON f.id=s.file_id
        WHERE s.execution=?1 AND s.schema_key=?2 AND s.finalized=1 AND s.last_position<=?3
        AND NOT EXISTS (SELECT 1 FROM stream_captures c JOIN streams other ON other.file_id=c.file_id
            WHERE c.execution=?1 AND other.schema_key=?2 AND c.file_id!=s.file_id)",
    )?;
    let mut rows = query.query(rusqlite::params![execution, serde_json::to_string(key)?, super::super::position(at.sequence)])?;
    let Some(row) = rows.next()? else { return Ok(None) };
    let id: String = row.get(0)?;
    let mut batches = connection.prepare(
        "SELECT offset,metadata_bytes,body_bytes,first_row,rows,first_at,last_at,sha256 FROM stream_batches WHERE file_id=?1 ORDER BY batch",
    )?;
    let batches = batches
        .query_map([&id], |row| {
            Ok(Batch {
                offset: catalog::unsigned(row, 0)?,
                metadata_bytes: row.get(1)?,
                body_bytes: catalog::unsigned(row, 2)?,
                first_row: catalog::unsigned(row, 3)?,
                rows: catalog::unsigned(row, 4)?,
                first_at: row.get(5)?,
                last_at: row.get(6)?,
                sha256: catalog::digest(row, 7)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut summaries =
        connection.prepare("SELECT signal,count,minimum,maximum,sum FROM stream_summaries WHERE file_id=?1 ORDER BY signal")?;
    let summaries = summaries
        .query_map([&id], |row| {
            Ok(Summary {
                signal: row.get(0)?,
                count: catalog::unsigned(row, 1)?,
                minimum: row.get(2)?,
                maximum: row.get(3)?,
                sum: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Some(StreamFile {
        id,
        stream: row.get(1)?,
        contract: serde_json::from_str(&row.get::<_, String>(2)?)?,
        size_bytes: catalog::unsigned(row, 3)?,
        sha256: catalog::digest(row, 4)?,
        rows: catalog::unsigned(row, 5)?,
        first_at: row.get(6)?,
        last_at: row.get(7)?,
        batches,
        summaries,
        path: root.join(row.get::<_, String>(8)?),
        _directory: None,
    }))
}

/* === Tests === */
