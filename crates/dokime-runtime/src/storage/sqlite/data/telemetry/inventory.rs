/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finalized Telemetry Files And Their Portable Catalog

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports
use rusqlite::Connection;
use serde::Serialize;

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

pub(super) fn files(connection: &Connection, root: &std::path::Path, execution: &str) -> Result<Vec<StreamFile>> {
    let mut statement = connection.prepare(
        "SELECT s.file_id,s.stream,s.contract,f.size_bytes,f.sha256,s.rows,s.first_at,s.last_at,f.path,s.finalized
        FROM streams s JOIN files f ON s.file_id=f.id WHERE s.execution=?1 ORDER BY s.stream",
    )?;
    let mut rows = statement.query([execution])?;
    let mut files = Vec::new();
    while let Some(row) = rows.next()? {
        if !row.get::<_, bool>(9)? {
            return Err(Error::Invalid("telemetry must be finalized before export"));
        }
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
        let digest: Vec<u8> = row.get(4)?;
        let sha256 = digest.try_into().map_err(|_| Error::Corrupt("telemetry file digest length differs"))?;
        files.push(StreamFile {
            id,
            stream: row.get(1)?,
            contract: serde_json::from_str(&row.get::<_, String>(2)?)?,
            size_bytes: catalog::unsigned(row, 3)?,
            sha256,
            rows: catalog::unsigned(row, 5)?,
            first_at: row.get(6)?,
            last_at: row.get(7)?,
            batches,
            summaries,
            path: root.join(row.get::<_, String>(8)?),
        });
    }
    Ok(files)
}

/* === Private Functions === */

/* === Tests === */
