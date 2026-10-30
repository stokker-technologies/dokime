/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File And Stream Catalog For Arrow Telemetry

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use rusqlite::{Connection, params};

// Local Imports
use super::super::Entry;
use crate::storage::{Error, Result};

/* === Constants === */

pub(in crate::storage::sqlite::data) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY, kind TEXT NOT NULL CHECK(kind IN ('artifact','telemetry','report')),
    path TEXT NOT NULL, size_bytes INTEGER NOT NULL CHECK(size_bytes>=0),
    sha256 BLOB NOT NULL CHECK(length(sha256)=32), media_type TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS artifacts (
    file_id TEXT PRIMARY KEY REFERENCES files(id), execution TEXT, relative_path TEXT
);
CREATE TABLE IF NOT EXISTS telemetry_contracts (
    schema_key TEXT PRIMARY KEY, contract TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS streams (
    file_id TEXT PRIMARY KEY REFERENCES files(id), execution TEXT, session TEXT NOT NULL,
    stream TEXT NOT NULL, schema_key TEXT NOT NULL, contract TEXT NOT NULL,
    rows INTEGER NOT NULL CHECK(rows>=0), first_at INTEGER, last_at INTEGER,
    first_position TEXT NOT NULL, last_position TEXT NOT NULL,
    durable_end INTEGER NOT NULL, hash_state BLOB NOT NULL, finalized INTEGER NOT NULL CHECK(finalized IN (0,1)),
    sealed INTEGER NOT NULL DEFAULT 0 CHECK(sealed IN (0,1))
);
CREATE INDEX IF NOT EXISTS stream_owner ON streams(execution,stream);
CREATE INDEX IF NOT EXISTS stream_schema ON streams(schema_key,execution);
CREATE TABLE IF NOT EXISTS stream_batches (
    file_id TEXT NOT NULL REFERENCES streams(file_id), batch INTEGER NOT NULL,
    offset INTEGER NOT NULL, metadata_bytes INTEGER NOT NULL, body_bytes INTEGER NOT NULL,
    first_row INTEGER NOT NULL, rows INTEGER NOT NULL,
    first_at INTEGER NOT NULL, last_at INTEGER NOT NULL,
    first_position TEXT NOT NULL, last_position TEXT NOT NULL,
    sha256 BLOB NOT NULL CHECK(length(sha256)=32),
    PRIMARY KEY(file_id,batch)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS stream_batch_time ON stream_batches(file_id,last_at,first_at);
CREATE INDEX IF NOT EXISTS stream_batch_position ON stream_batches(file_id,last_position,first_position);
CREATE TABLE IF NOT EXISTS stream_batch_scopes (
    file_id TEXT NOT NULL, batch INTEGER NOT NULL, scope TEXT NOT NULL,
    first_at INTEGER NOT NULL, last_at INTEGER NOT NULL,
    PRIMARY KEY(file_id,batch,scope),
    FOREIGN KEY(file_id,batch) REFERENCES stream_batches(file_id,batch)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS stream_batch_scope_time ON stream_batch_scopes(scope,last_at);
CREATE TABLE IF NOT EXISTS stream_summaries (
    file_id TEXT NOT NULL REFERENCES streams(file_id), signal TEXT NOT NULL,
    count INTEGER NOT NULL, minimum REAL NOT NULL, maximum REAL NOT NULL, sum REAL NOT NULL,
    PRIMARY KEY(file_id,signal)
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_sample_receipts (
    execution TEXT NOT NULL, stream TEXT NOT NULL, sample_id TEXT NOT NULL, position TEXT NOT NULL,
    PRIMARY KEY(execution,stream)
) WITHOUT ROWID;
";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn composed(connection: &Connection, key: &harmos::storage::SchemaKey) -> Result<harmos::storage::StorageSchema> {
    let encoded = serde_json::to_string(key)?;
    let mut statement = connection
        .prepare("SELECT body,digest FROM storage_data_entries WHERE kind='schema' AND schema_key=?1 ORDER BY position DESC LIMIT 1")?;
    let mut rows = statement.query([encoded])?;
    let row = rows.next()?.ok_or(Error::Corrupt("telemetry schema is absent"))?;
    let body = super::super::blob(row, 0)?;
    super::super::check_digest(body, super::super::blob(row, 1)?)?;
    let Entry::Schema(schema) = Entry::decode("schema", body)? else {
        return Err(Error::Corrupt("telemetry schema collection differs"));
    };
    Ok(schema)
}

pub(in crate::storage::sqlite::data) fn file(connection: &Connection, reference: &harmos::storage::FileReference) -> Result<()> {
    let scope = reference.metadata.scope.as_ref().map(harmos::Identifier::as_str);
    // Report Producers Already Reserve <Owner>.<UUID>.report/.context Identities.
    let report = scope
        .and_then(|owner| reference.metadata.id.as_str().strip_prefix(owner))
        .and_then(|suffix| suffix.strip_prefix('.'))
        .and_then(|suffix| suffix.strip_suffix(".report").or_else(|| suffix.strip_suffix(".context")))
        .is_some_and(|nonce| uuid::Uuid::parse_str(nonce).is_ok());
    let kind = if report { "report" } else { "artifact" };
    let length = i64::try_from(reference.length).map_err(|_| Error::Invalid("catalog file size exceeds SQLite range"))?;
    connection.execute(
        "INSERT INTO files VALUES (?1,?2,?3,?4,?5,?6)",
        params![
            reference.metadata.id.as_str(),
            kind,
            super::super::files::storage_key(reference),
            length,
            reference.sha256.as_slice(),
            reference.metadata.media_type
        ],
    )?;
    if !report {
        connection.execute("INSERT INTO artifacts VALUES (?1,?2,NULL)", params![reference.metadata.id.as_str(), scope])?;
    }
    Ok(())
}

pub(super) fn micros(at: std::time::SystemTime) -> Result<i64> {
    let at: time::OffsetDateTime = at.into();
    i64::try_from(at.unix_timestamp_nanos().div_euclid(1000)).map_err(|_| Error::Invalid("telemetry timestamp exceeds Arrow range"))
}

pub(super) fn integer(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| Error::Invalid("telemetry catalog number exceeds SQLite range"))
}

pub(super) fn unsigned(row: &rusqlite::Row<'_>, column: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(column)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(column, value))
}

pub(super) fn digest(row: &rusqlite::Row<'_>, column: usize) -> rusqlite::Result<[u8; 32]> {
    let bytes: Vec<u8> = row.get(column)?;
    bytes.try_into().map_err(|_| rusqlite::Error::InvalidColumnType(column, "sha256".into(), rusqlite::types::Type::Blob))
}

/* === Private Functions === */

/* === Tests === */
