/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Indexed Record Query Scans

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::MemorySize;

// Local Imports
use super::{ScopedRecord, scoped_query};
use crate::{
    evidence_runtime::ScopeSelection,
    storage::{
        Error,
        records::RecordRow,
        sqlite::{
            State,
            snapshots::{Cut, available},
        },
    },
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Read {
    pub cut: Cut,
    pub scope: ScopeSelection,
    pub record_epoch: Option<String>,
    pub after: u64,
    pub rows: u32,
    pub bytes: usize,
    pub scanned_bytes: u64,
}

pub(crate) struct Scan {
    pub rows: Vec<ScopedRecord>,
    pub bytes: u64,
    pub complete: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl State {
    pub(crate) async fn query_record_member(&self, cut: Cut, execution: String) -> Result<bool, Error> {
        self.read(move |connection| {
            Ok((|| {
                if !available(connection, &cut)? {
                    return Err(Error::QuerySnapshotUnavailable);
                }
                let crate::storage::sqlite::snapshots::Scope::Cycle(cycle) = &cut.scope else {
                    return Err(Error::Invalid("record member selection requires a cycle"));
                };
                connection
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM query_members m WHERE m.cycle=?1 AND m.execution=?2
                    AND m.position=(SELECT h.position FROM query_history h WHERE h.kind='cycle' AND h.id=?1
                    AND h.revision<=?3 AND h.position<=?4 ORDER BY h.revision DESC LIMIT 1))",
                        rusqlite::params![cycle, execution, cut.catalog, super::super::position(cut.metadata)],
                        |row| row.get(0),
                    )
                    .map_err(Error::from)
            })())
        })
        .await?
    }

    pub(crate) async fn query_records(&self, read: Read) -> Result<Scan, Error> {
        self.read(move |connection| {
            Ok((|| {
                if !available(connection, &read.cut)? {
                    return Err(Error::QuerySnapshotUnavailable);
                }
                let (sql, args) = scoped_query(&read.scope, None, (read.after, read.cut.at.sequence), (0, u64::MAX), Some(&read))?;
                let mut statement = connection.prepare(&sql)?;
                let mut rows = statement.query(rusqlite::params_from_iter(args))?;
                let mut scan = Scan {
                    rows: Vec::new(),
                    bytes: 0,
                    complete: true,
                };
                let mut working = 0usize;
                while let Some(row) = rows.next()? {
                    if scan.rows.len() == read.rows as usize {
                        scan.complete = false;
                        break;
                    }
                    let body = super::super::blob(row, 1)?;
                    if scan.bytes.saturating_add(body.len() as u64) > read.scanned_bytes {
                        return Err(Error::Data("record query exceeds scanned byte budget"));
                    }
                    // Keep The Stored Envelope, Native Decode And Retained Page Bounded.
                    let transient = body.len().saturating_mul(4);
                    if working.saturating_add(transient) > read.bytes {
                        if scan.rows.is_empty() {
                            return Err(Error::Data("record exceeds query working budget"));
                        }
                        scan.complete = false;
                        break;
                    }
                    super::super::check_digest(body, super::super::blob(row, 2)?)?;
                    let super::super::Entry::Record(stored) = super::super::Entry::decode("record", body)? else { unreachable!() };
                    let position = super::super::sequence(&row.get::<_, String>(0)?)?;
                    if stored.position.history != read.cut.at.history || stored.position.sequence != position {
                        return Err(Error::Corrupt("query record index position differs"));
                    }
                    let epoch = stored.scope.as_ref().ok_or(Error::Corrupt("query record has no journal epoch"))?;
                    let epoch = dokime_domain::model::identity::SessionId::from_uuid(epoch.as_str())?;
                    let record = RecordRow::from_bytes(&stored.payload, read.bytes)?.entry()?;
                    let memory = record.memory_size();
                    if working.saturating_add(memory).saturating_add(transient) > read.bytes {
                        if scan.rows.is_empty() {
                            return Err(Error::Data("decoded record exceeds query working budget"));
                        }
                        scan.complete = false;
                        break;
                    }
                    working = working.saturating_add(memory);
                    scan.bytes = scan.bytes.saturating_add(body.len() as u64);
                    scan.rows.push(ScopedRecord { position, epoch, record });
                }
                Ok(scan)
            })())
        })
        .await?
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
