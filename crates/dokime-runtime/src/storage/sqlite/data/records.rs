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

pub(crate) mod query;

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
    cycle TEXT, execution TEXT, sequence TEXT NOT NULL, epoch TEXT
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_record_captures (
    execution TEXT NOT NULL, position TEXT NOT NULL REFERENCES storage_data_entries(position) ON DELETE CASCADE,
    record_id TEXT NOT NULL, ordinal TEXT NOT NULL, scope TEXT NOT NULL, cycle TEXT, sequence TEXT NOT NULL, epoch TEXT,
    PRIMARY KEY(execution,position), UNIQUE(execution,record_id)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS storage_capture_ordinal ON storage_record_captures(execution,ordinal);
CREATE INDEX IF NOT EXISTS storage_capture_position ON storage_record_captures(position);
CREATE INDEX IF NOT EXISTS storage_capture_cycle ON storage_record_captures(cycle,position);
CREATE INDEX IF NOT EXISTS storage_record_cycle ON storage_record_visibility(cycle,position);
CREATE INDEX IF NOT EXISTS storage_record_context ON storage_record_visibility(cycle,execution,position);
CREATE INDEX IF NOT EXISTS storage_record_execution ON storage_record_visibility(execution,position);
CREATE INDEX IF NOT EXISTS storage_record_execution_sequence ON storage_record_visibility(execution,epoch,sequence,position);
CREATE INDEX IF NOT EXISTS storage_record_cycle_sequence ON storage_record_visibility(cycle,epoch,sequence,position);
CREATE INDEX IF NOT EXISTS storage_capture_execution_sequence ON storage_record_captures(execution,epoch,sequence,position);
CREATE INDEX IF NOT EXISTS storage_capture_cycle_sequence ON storage_record_captures(cycle,epoch,sequence,position);
";
const CAPTURE_PAGE_SQL: &str = "SELECT e.body,e.digest FROM storage_record_captures c
    JOIN storage_data_entries e ON e.position=c.position
    WHERE c.execution=?1 AND c.ordinal>?2 AND c.position<=?3 ORDER BY c.ordinal LIMIT 512";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct ScopedRecord {
    pub position: u64,
    pub epoch: dokime_domain::model::identity::SessionId,
    pub record: dokime_domain::model::records::evidence::RecordEntry,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl super::State {
    pub(crate) async fn scoped_records(
        &self, at: StorageCursor, scope: crate::evidence_runtime::ScopeSelection, epoch: Option<String>, after_position: u64,
        after_sequence: u64, through_sequence: u64,
    ) -> Result<Vec<ScopedRecord>, Error> {
        self.read(move |connection| {
            Ok((|| {
                saved(connection)?.history.check(&at.history)?;
                let (sql, args) =
                    scoped_query(&scope, epoch.as_deref(), (after_position, at.sequence), (after_sequence, through_sequence), None)?;
                let mut query = connection.prepare(&sql)?;
                let mut rows = query.query(rusqlite::params_from_iter(args))?;
                let mut output = Vec::new();
                let mut bytes = 0usize;
                while let Some(row) = rows.next()? {
                    let body = blob(row, 1)?;
                    if !output.is_empty() && bytes.saturating_add(body.len()) > 8 * 1024 * 1024 {
                        break;
                    }
                    check_digest(body, blob(row, 2)?)?;
                    let Entry::Record(record) = Entry::decode("record", body)? else { unreachable!() };
                    let indexed = super::sequence(&row.get::<_, String>(0)?)?;
                    if record.position.sequence != indexed {
                        return Err(Error::Corrupt("record capture index position differs"));
                    }
                    let epoch = record.scope.as_ref().ok_or(Error::Corrupt("captured record has no process epoch"))?;
                    output.push(ScopedRecord {
                        position: indexed,
                        epoch: dokime_domain::model::identity::SessionId::from_uuid(epoch.as_str())?,
                        record: RecordRow::decode_entry(&record.payload)?,
                    });
                    bytes += body.len();
                }
                Ok(output)
            })())
        })
        .await?
    }

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
        "INSERT INTO storage_record_visibility VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            position(row.position.sequence),
            record.record_id.as_str(),
            position(record.ordinal),
            scope.and_then(|scope| scope.test_cycle_id.as_ref()).map(|id| id.as_str()),
            scope.and_then(|scope| scope.test_execution_id.as_ref()).map(|id| id.as_str()),
            position(record.sequence),
            row.scope.as_ref().map(|epoch| epoch.as_str())
        ],
    )?;
    for scope in &record.captured_scopes {
        let owner = scope.test_execution_id.as_ref().ok_or(Error::Corrupt("capture has no execution"))?;
        connection.execute(
            "INSERT INTO storage_record_captures VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                owner.as_str(),
                position(row.position.sequence),
                record.record_id.as_str(),
                position(record.ordinal),
                serde_json::to_string(scope)?,
                scope.test_cycle_id.as_ref().map(|id| id.as_str()),
                position(record.sequence),
                row.scope.as_ref().map(|epoch| epoch.as_str())
            ],
        )?;
    }
    Ok(())
}

/* === Private Functions === */

fn scoped_query(
    scope: &crate::evidence_runtime::ScopeSelection, epoch: Option<&str>, positions: (u64, u64), sequences: (u64, u64),
    query: Option<&query::Read>,
) -> Result<(String, Vec<rusqlite::types::Value>), Error> {
    use rusqlite::types::Value;
    let mut args = vec![Value::Text(position(positions.0)), Value::Text(position(positions.1))];
    let mut conditions = vec!["position>?1 AND position<=?2".to_owned()];
    for (column, value) in [
        ("execution", scope.test_execution_id.as_ref().map(ToString::to_string)),
        ("cycle", scope.test_cycle_id.as_ref().map(ToString::to_string)),
    ] {
        if let Some(value) = value {
            args.push(Value::Text(value));
            conditions.push(format!("{column}=?{}", args.len()));
        }
    }
    if conditions.len() == 1 {
        return Err(Error::Invalid("indexed record read requires an execution or cycle"));
    }
    if let Some(read) = query {
        if let Some(journal) = read.cut.journal {
            let epoch = read.record_epoch.as_ref().ok_or(Error::Corrupt("query cut has no record journal epoch"))?;
            args.push(Value::Text(epoch.clone()));
            let epoch_parameter = args.len();
            args.push(Value::Text(position(journal)));
            conditions.push(format!("(epoch IS NULL OR epoch<>?{epoch_parameter} OR sequence<=?{})", args.len()));
        }
        if let super::super::snapshots::Scope::Cycle(cycle) = &read.cut.scope {
            args.push(Value::Text(cycle.clone()));
            let cycle_parameter = args.len();
            args.push(Value::Integer(read.cut.catalog));
            let catalog_parameter = args.len();
            args.push(Value::Text(position(read.cut.metadata)));
            conditions.push(format!(
                "(execution IS NULL OR execution IN (SELECT m.execution FROM query_members m
                WHERE m.cycle=?{cycle_parameter} AND m.position=(SELECT h.position FROM query_history h
                WHERE h.kind='cycle' AND h.id=?{cycle_parameter} AND h.revision<=?{catalog_parameter}
                AND h.position<=?{} ORDER BY h.revision DESC LIMIT 1)))",
                args.len()
            ));
        }
    }
    let order = if let Some(epoch) = epoch {
        args.push(Value::Text(epoch.to_owned()));
        conditions.push(format!("epoch=?{}", args.len()));
        args.push(Value::Text(position(sequences.0)));
        conditions.push(format!("sequence>?{}", args.len()));
        args.push(Value::Text(position(sequences.1)));
        conditions.push(format!("sequence<=?{}", args.len()));
        "sequence,position"
    } else {
        "position"
    };
    let conditions = conditions.join(" AND ");
    let limit = query.map_or(256, |read| read.rows.saturating_add(1));
    let branch =
        |table| format!("SELECT position FROM (SELECT DISTINCT {order} FROM {table} WHERE {conditions} ORDER BY {order} LIMIT {limit})");
    // Bound Each Indexed Branch Before Deduplication And Payload Reads.
    let sql = format!(
        "SELECT e.position,e.body,e.digest FROM ({} UNION {}) selected
        JOIN storage_data_entries e ON e.position=selected.position ORDER BY e.position LIMIT {limit}",
        branch("storage_record_visibility"),
        branch("storage_record_captures"),
    );
    Ok((sql, args))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_pages_seek() {
        use crate::evidence_runtime::ScopeSelection;
        use dokime_domain::model::identity::{TestCycleId, TestExecutionId};
        let execution = TestExecutionId::from_seed("indexed-owner");
        let shared = TestExecutionId::from_seed("shared-owner");
        let cycle = TestCycleId::from_seed("indexed-cycle");
        for history in [4096_u64, 65536] {
            let connection = Connection::open_in_memory().unwrap();
            connection.execute_batch(super::super::SCHEMA).unwrap();
            connection.execute_batch(SCHEMA).unwrap();
            connection
                .execute_batch(&format!(
                    "WITH RECURSIVE rows(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM rows WHERE n<{})
                INSERT INTO storage_data_entries(position,kind,identity,item,scope,body,digest)
                SELECT printf('%016x',n),'record',printf('record-%d',n),printf('record-%d',n),
                    CASE WHEN n<={history} THEN 'previous' ELSE 'current' END,x'00',zeroblob(32) FROM rows;
                INSERT INTO storage_record_visibility
                SELECT position,identity,position,'{cycle}',CASE WHEN row_number() OVER (ORDER BY position)%2=1
                    THEN '{execution}' ELSE 'other' END,
                    printf('%016x',row_number() OVER (PARTITION BY scope ORDER BY position)),scope FROM storage_data_entries;
                INSERT INTO storage_record_captures
                SELECT '{execution}',position,record_id,ordinal,'{{}}',cycle,sequence,epoch FROM storage_record_visibility;
                INSERT INTO storage_record_captures
                SELECT '{shared}',position,record_id,ordinal,'{{}}',cycle,sequence,epoch FROM storage_record_visibility;",
                    history * 2,
                ))
                .unwrap();
            for scope in [
                ScopeSelection {
                    test_execution_id: Some(execution.clone()),
                    ..Default::default()
                },
                ScopeSelection {
                    test_execution_id: Some(shared.clone()),
                    ..Default::default()
                },
                ScopeSelection {
                    test_cycle_id: Some(cycle.clone()),
                    ..Default::default()
                },
            ] {
                for (epoch, physical_after, logical_after, logical_through, first, last) in [
                    (Some("current"), 0, 0, history, history + 1, history + 256),
                    (Some("current"), 0, history - 8, history, history * 2 - 7, history * 2),
                    (Some("current"), 0, history - 8, history - 3, history * 2 - 7, history * 2 - 3),
                    (None, history * 2 - 8, 0, u64::MAX, history * 2 - 7, history * 2),
                    (None, 0, 0, u64::MAX, 1, 256),
                ] {
                    let (sql, args) =
                        scoped_query(&scope, epoch, (physical_after, history * 2), (logical_after, logical_through), None).unwrap();
                    let mut query = connection.prepare(&sql).unwrap();
                    let rows = query
                        .query_map(rusqlite::params_from_iter(args), |row| row.get::<_, String>(0))
                        .unwrap()
                        .collect::<rusqlite::Result<Vec<_>>>()
                        .unwrap();
                    assert_eq!(rows, (first..=last).map(position).collect::<Vec<_>>());
                    let steps = query.get_status(rusqlite::StatementStatus::VmStep);
                    assert!(steps < 50_000, "scoped page took {steps} VM steps with {history} records per epoch");
                }
            }
            let (sql, args) = scoped_query(
                &ScopeSelection {
                    test_execution_id: Some(TestExecutionId::from_seed("unrelated-owner")),
                    ..Default::default()
                },
                Some("current"),
                (0, history * 2),
                (0, history),
                None,
            )
            .unwrap();
            assert!(!connection.prepare(&sql).unwrap().exists(rusqlite::params_from_iter(args)).unwrap());
        }
    }

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
                INSERT INTO storage_record_captures SELECT 'execution',position,identity,position,'{{}}',NULL,position,NULL FROM storage_data_entries;"
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
