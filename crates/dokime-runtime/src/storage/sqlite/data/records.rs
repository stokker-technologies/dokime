/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Transactional Record Visibility And Capture Indexes.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::storage::{StorageCursor, StoredRecord};
use rusqlite::{Connection, params};

// Local Imports
use super::{Entry, blob, check_digest, position, saved};
use crate::storage::{Error, records::RecordRow};

/* === Constants === */

pub(super) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_record_visibility (
    position TEXT PRIMARY KEY REFERENCES storage_data_entries(position) ON DELETE CASCADE,
    record_id TEXT NOT NULL UNIQUE, ordinal TEXT NOT NULL UNIQUE,
    cycle TEXT, execution TEXT
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_record_captures (
    execution TEXT NOT NULL, position TEXT NOT NULL REFERENCES storage_data_entries(position) ON DELETE CASCADE,
    record_id TEXT NOT NULL, ordinal TEXT NOT NULL, scope TEXT NOT NULL,
    PRIMARY KEY(execution,position), UNIQUE(execution,record_id)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS storage_capture_ordinal ON storage_record_captures(execution,ordinal);
CREATE INDEX IF NOT EXISTS storage_capture_position ON storage_record_captures(position);
CREATE INDEX IF NOT EXISTS storage_record_cycle ON storage_record_visibility(cycle,position);
CREATE INDEX IF NOT EXISTS storage_record_context ON storage_record_visibility(cycle,execution,position);
CREATE INDEX IF NOT EXISTS storage_record_execution ON storage_record_visibility(execution,position);
";
const CAPTURE_PAGE_SQL: &str = "SELECT e.body,e.digest FROM storage_record_captures c
    JOIN storage_data_entries e ON e.position=c.position
    WHERE c.execution=?1 AND c.ordinal>?2 AND c.position<=?3 ORDER BY c.ordinal LIMIT 512";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl super::State {
    // The Index And Evidence Body Commit In The Same Serialized Transaction.
    pub(crate) async fn captured_record_page(&self, at: StorageCursor, execution: String, after: u64) -> Result<Vec<StoredRecord>, Error> {
        self.read(move |connection| {
            Ok((|| {
                let head = saved(connection)?;
                head.history.check(&at.history)?;
                if at.sequence > head.sequence {
                    return Err(Error::Data("record capture exceeds saved frontier"));
                }
                let mut query = connection.prepare(CAPTURE_PAGE_SQL)?;
                let mut rows = query.query(params![execution, position(after), position(at.sequence)])?;
                let mut output = Vec::new();
                let mut bytes = 0usize;
                while let Some(row) = rows.next()? {
                    let body = blob(row, 0)?;
                    if !output.is_empty() && bytes.saturating_add(body.len()) > 8 * 1024 * 1024 {
                        break;
                    }
                    check_digest(body, blob(row, 1)?)?;
                    let Entry::Record(record) = Entry::decode("record", body)? else { unreachable!() };
                    bytes = bytes.checked_add(body.len()).ok_or(Error::Corrupt("record page size overflow"))?;
                    output.push(record);
                }
                super::super::RECORD_CAPTURE_PAGES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                super::super::RECORD_CAPTURE_ROWS.fetch_add(output.len() as u64, std::sync::atomic::Ordering::Relaxed);
                Ok(output)
            })())
        })
        .await?
    }
}

/* === Public Functions === */

pub(super) fn index(connection: &Connection, row: &StoredRecord) -> Result<(), Error> {
    if row.schema != crate::storage::records::schema() {
        return Ok(());
    }
    let record = RecordRow::decode_entry(&row.payload)?;
    super::super::RECORD_CAPTURE_ADMISSIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let scope = record.scope.as_ref();
    connection.execute(
        "INSERT INTO storage_record_visibility VALUES (?1,?2,?3,?4,?5)",
        params![
            position(row.position.sequence),
            record.record_id.as_str(),
            position(record.ordinal),
            scope.and_then(|scope| scope.test_cycle_id.as_ref()).map(|id| id.as_str()),
            scope.and_then(|scope| scope.test_execution_id.as_ref()).map(|id| id.as_str())
        ],
    )?;
    for scope in &record.captured_scopes {
        let owner = scope.test_execution_id.as_ref().ok_or(Error::Corrupt("capture has no execution"))?;
        connection.execute(
            "INSERT INTO storage_record_captures VALUES (?1,?2,?3,?4,?5)",
            params![
                owner.as_str(),
                position(row.position.sequence),
                record.record_id.as_str(),
                position(record.ordinal),
                serde_json::to_string(scope)?
            ],
        )?;
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_page_seeks_by_execution_and_position() {
        for history in [4096u64, 65536] {
            let connection = Connection::open_in_memory().unwrap();
            connection.execute_batch(super::super::SCHEMA).unwrap();
            connection.execute_batch(SCHEMA).unwrap();
            connection
                .execute_batch(&format!(
                    "WITH RECURSIVE rows(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM rows WHERE n<{history})
                INSERT INTO storage_data_entries(position,kind,identity,item,body,digest)
                SELECT printf('%016x',n),'record',printf('record-%d',n),printf('record-%d',n),x'00',zeroblob(32) FROM rows;
                INSERT INTO storage_record_captures SELECT 'execution',position,identity,position,'{{}}' FROM storage_data_entries;"
                ))
                .unwrap();
            let mut query = connection.prepare(CAPTURE_PAGE_SQL).unwrap();
            let rows = query
                .query_map(params!["execution", position(history - 8), position(history)], |row| row.get::<_, Vec<u8>>(0))
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap();
            assert_eq!(rows.len(), 8);
            assert!(query.get_status(rusqlite::StatementStatus::VmStep) < 1000);
        }
    }
}
