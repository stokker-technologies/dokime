/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Indexed Disk Ownership For Complex Plot Inputs

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Mutex};

// Third Party Imports
use rusqlite::{Connection, params};

// Local Imports
use crate::runtime::{Error, Result};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::execution::{EvidenceOrigin, RecordScope};
use dokime_domain::model::records::evidence::RecordEntry;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct Input {
    owner: Arc<Mutex<Owner>>,
}

struct Owner {
    connection: Connection,
    _directory: tempfile::TempDir,
    reverse_records: bool,
}

pub(super) struct Read<'a> {
    pub connection: &'a Connection,
    reverse_records: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Input {
    pub(crate) fn new() -> Result<Self> {
        let directory = tempfile::tempdir().map_err(failure)?;
        let connection = Connection::open(directory.path().join("projection.sqlite")).map_err(failure)?;
        connection
            .execute_batch(
                "PRAGMA journal_mode=OFF;
             PRAGMA synchronous=OFF;
             PRAGMA temp_store=FILE;
             PRAGMA cache_size=-8192;
             PRAGMA mmap_size=0;
             CREATE TABLE samples (
                ordinal INTEGER PRIMARY KEY, stream TEXT NOT NULL, capture BLOB NOT NULL,
                stamp TEXT NOT NULL, time BLOB, execution TEXT NOT NULL, attempt TEXT NOT NULL, payload BLOB NOT NULL
             );
             CREATE INDEX samples_time ON samples(stream, execution, attempt, time, capture, ordinal);
             CREATE INDEX samples_capture ON samples(capture, ordinal);
             CREATE INDEX samples_correlation ON samples(execution,attempt,time,capture,ordinal);
             CREATE INDEX samples_execution_time ON samples(execution,time,capture,ordinal);
             CREATE TABLE records (
                ordinal INTEGER PRIMARY KEY, kind TEXT NOT NULL, capture BLOB NOT NULL,
                stamp TEXT NOT NULL, time BLOB, execution TEXT NOT NULL, attempt TEXT NOT NULL, payload BLOB NOT NULL
             );
             CREATE INDEX records_previous ON records(kind, execution, attempt, capture, ordinal);
             CREATE INDEX records_previous_reverse ON records(kind, execution, attempt, capture, stamp DESC, ordinal);
             CREATE TABLE box_values(label TEXT NOT NULL,value REAL NOT NULL);
             CREATE INDEX box_order ON box_values(label,value);
             CREATE TABLE interval_values(ordinal INTEGER PRIMARY KEY,bin INTEGER NOT NULL,value REAL NOT NULL);
             CREATE INDEX interval_order ON interval_values(bin,ordinal);
             BEGIN;",
            )
            .map_err(failure)?;
        let owner = Owner {
            connection,
            _directory: directory,
            reverse_records: false,
        };
        let owner = Arc::new(Mutex::new(owner));
        Ok(Self { owner })
    }

    pub(crate) fn reverse_records(&self) -> Result<()> {
        self.owner.lock().map_err(failure)?.reverse_records = true;
        Ok(())
    }

    pub(crate) fn collect(&self, limit: usize) -> Result<Vec<TelemetrySample>> {
        self.read(|read| {
            let mut samples = Vec::new();
            read.samples(|sample| {
                if samples.len() == limit {
                    return Err(failure("telemetry scan exceeds the analysis sample budget"));
                }
                samples.push(sample);
                Ok(())
            })?;
            Ok(samples)
        })
    }

    pub(crate) fn append(&self, sample: &TelemetrySample) -> Result<()> {
        let owner = self.owner.lock().map_err(failure)?;
        let (execution, attempt) = keys(sample.scope.as_ref());
        let capture = sample.sequence.to_be_bytes();
        let time = time(sample.timestamp.as_str());
        let payload = serde_json::to_vec(sample).map_err(failure)?;
        owner
            .connection
            .execute(
                "INSERT INTO samples(stream,capture,stamp,time,execution,attempt,payload) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![
                    sample.stream.as_str(),
                    capture.as_slice(),
                    sample.timestamp.as_str(),
                    time.as_ref().map(|value| value.as_slice()),
                    execution,
                    attempt,
                    payload
                ],
            )
            .map_err(failure)?;
        Ok(())
    }

    pub(crate) fn append_record(&self, record: &RecordEntry) -> Result<()> {
        let owner = self.owner.lock().map_err(failure)?;
        let (execution, attempt) = keys(record.scope.as_ref());
        let capture = record.sequence.to_be_bytes();
        let time = time(record.timestamp.as_str());
        let payload = serde_json::to_vec(record).map_err(failure)?;
        owner
            .connection
            .execute(
                "INSERT INTO records(kind,capture,stamp,time,execution,attempt,payload) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![
                    record.record_type.as_str(),
                    capture.as_slice(),
                    record.timestamp.as_str(),
                    time.as_ref().map(|value| value.as_slice()),
                    execution,
                    attempt,
                    payload
                ],
            )
            .map_err(failure)?;
        Ok(())
    }

    pub(super) fn read<T>(&self, read: impl FnOnce(Read<'_>) -> Result<T>) -> Result<T> {
        let owner = self.owner.lock().map_err(failure)?;
        read(Read {
            connection: &owner.connection,
            reverse_records: owner.reverse_records,
        })
    }
}

impl Read<'_> {
    pub(super) fn samples(&self, mut visit: impl FnMut(TelemetrySample) -> Result<()>) -> Result<()> {
        let mut statement = self.connection.prepare("SELECT payload FROM samples ORDER BY capture,ordinal").map_err(failure)?;
        let mut rows = statement.query([]).map_err(failure)?;
        while let Some(row) = rows.next().map_err(failure)? {
            let bytes: Vec<u8> = row.get(0).map_err(failure)?;
            visit(serde_json::from_slice(&bytes).map_err(failure)?)?;
        }
        Ok(())
    }

    pub(super) fn records(&self, capture_order: bool, mut visit: impl FnMut(i64, RecordEntry) -> Result<()>) -> Result<()> {
        let query = if capture_order && self.reverse_records {
            "SELECT ordinal,payload FROM records ORDER BY capture,stamp DESC,ordinal"
        } else if capture_order {
            "SELECT ordinal,payload FROM records ORDER BY capture,ordinal"
        } else if self.reverse_records {
            "SELECT ordinal,payload FROM records ORDER BY stamp DESC,capture DESC,ordinal"
        } else {
            "SELECT ordinal,payload FROM records ORDER BY ordinal"
        };
        let mut statement = self.connection.prepare(query).map_err(failure)?;
        let mut rows = statement.query([]).map_err(failure)?;
        while let Some(row) = rows.next().map_err(failure)? {
            let bytes: Vec<u8> = row.get(1).map_err(failure)?;
            visit(row.get(0).map_err(failure)?, serde_json::from_slice(&bytes).map_err(failure)?)?;
        }
        Ok(())
    }

    pub(super) fn nearest(
        &self, stream: &str, scope: Option<&RecordScope>, timestamp: &str, strict_attempt: bool,
        mut keep: impl FnMut(&TelemetrySample) -> bool,
    ) -> Result<Option<TelemetrySample>> {
        let Some(time) = time(timestamp) else { return Ok(None) };
        let (execution, attempt) = keys(scope);
        let mut statement = self
            .connection
            .prepare(
                "SELECT payload FROM samples WHERE execution=?1 AND (?5=0 OR attempt=?2) AND time<=?3
             AND (stream=?4 OR substr(stream,-length(?4)-1)='.'||?4 OR substr(?4,-length(stream)-1)='.'||stream)
             ORDER BY time DESC,capture DESC,ordinal DESC",
            )
            .map_err(failure)?;
        let mut rows = statement.query(params![execution, attempt, time.as_slice(), stream, strict_attempt]).map_err(failure)?;
        while let Some(row) = rows.next().map_err(failure)? {
            let bytes: Vec<u8> = row.get(0).map_err(failure)?;
            let sample = serde_json::from_slice(&bytes).map_err(failure)?;
            if !keep(&sample) {
                continue;
            }
            return Ok(Some(sample));
        }
        Ok(None)
    }

    pub(super) fn previous(
        &self, kind: &str, ordinal: i64, record: &RecordEntry, visible: Option<&RecordScope>,
    ) -> Result<Option<RecordEntry>> {
        let (execution, attempt) = keys(record.scope.as_ref());
        let capture = record.sequence.to_be_bytes();
        let query = if self.reverse_records {
            "SELECT payload FROM records INDEXED BY records_previous_reverse
             WHERE kind=?1 AND execution=?2 AND attempt=?3 AND time IS NOT NULL AND capture<=?4
             AND (capture<?4 OR (capture=?4 AND (stamp>?6 OR (stamp=?6 AND ordinal<?5))))
             ORDER BY capture DESC,stamp ASC,ordinal DESC"
        } else {
            "SELECT payload FROM records INDEXED BY records_previous
             WHERE kind=?1 AND execution=?2 AND attempt=?3 AND time IS NOT NULL AND capture<=?4
             AND (capture<?4 OR (capture=?4 AND ordinal<?5)) ORDER BY capture DESC,ordinal DESC"
        };
        let mut statement = self.connection.prepare(query).map_err(failure)?;
        let mut rows = if self.reverse_records {
            statement.query(params![kind, execution, attempt, capture.as_slice(), ordinal, record.timestamp.as_str()])
        } else {
            statement.query(params![kind, execution, attempt, capture.as_slice(), ordinal])
        }
        .map_err(failure)?;
        while let Some(row) = rows.next().map_err(failure)? {
            let bytes: Vec<u8> = row.get(0).map_err(failure)?;
            let record: RecordEntry = serde_json::from_slice(&bytes).map_err(failure)?;
            if !super::super::sample_matches_scope(record.scope.as_ref(), visible) {
                continue;
            }
            return Ok(Some(record));
        }
        Ok(None)
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(super) fn failure(error: impl std::fmt::Display) -> Error {
    Error::Operation(format!("plot spill: {error}"))
}

fn keys(scope: Option<&RecordScope>) -> (&str, &str) {
    let execution = scope.and_then(|scope| scope.test_execution_id.as_ref());
    let execution = execution.map_or("", |id| id.as_str());
    let attempt = match scope.map(|scope| &scope.origin) {
        Some(EvidenceOrigin::Step { attempt_id }) => attempt_id.as_str(),
        _ => "",
    };
    (execution, attempt)
}

fn time(value: &str) -> Option<[u8; 16]> {
    let nanos = crate::plot::parse_timestamp(value)?.unix_timestamp_nanos();
    Some(((nanos as u128) ^ (1u128 << 127)).to_be_bytes())
}

/* === Tests === */
