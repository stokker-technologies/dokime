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
    cycle TEXT, execution TEXT, sequence TEXT NOT NULL, epoch TEXT, scope TEXT
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
CREATE INDEX IF NOT EXISTS storage_record_case ON storage_record_visibility(json_extract(scope,'$.test_case_id'),position);
CREATE INDEX IF NOT EXISTS storage_record_template ON storage_record_visibility(json_extract(scope,'$.template_id'),position);
CREATE INDEX IF NOT EXISTS storage_record_suite ON storage_record_visibility(json_extract(scope,'$.suite_id'),position);
CREATE INDEX IF NOT EXISTS storage_capture_case ON storage_record_captures(json_extract(scope,'$.test_case_id'),position);
CREATE INDEX IF NOT EXISTS storage_capture_template ON storage_record_captures(json_extract(scope,'$.template_id'),position);
CREATE INDEX IF NOT EXISTS storage_capture_suite ON storage_record_captures(json_extract(scope,'$.suite_id'),position);
CREATE INDEX IF NOT EXISTS storage_record_case_sequence ON storage_record_visibility(json_extract(scope,'$.test_case_id'),epoch,sequence,position);
CREATE INDEX IF NOT EXISTS storage_record_template_sequence ON storage_record_visibility(json_extract(scope,'$.template_id'),epoch,sequence,position);
CREATE INDEX IF NOT EXISTS storage_record_suite_sequence ON storage_record_visibility(json_extract(scope,'$.suite_id'),epoch,sequence,position);
CREATE INDEX IF NOT EXISTS storage_capture_case_sequence ON storage_record_captures(json_extract(scope,'$.test_case_id'),epoch,sequence,position);
CREATE INDEX IF NOT EXISTS storage_capture_template_sequence ON storage_record_captures(json_extract(scope,'$.template_id'),epoch,sequence,position);
CREATE INDEX IF NOT EXISTS storage_capture_suite_sequence ON storage_record_captures(json_extract(scope,'$.suite_id'),epoch,sequence,position);
CREATE INDEX IF NOT EXISTS storage_record_execution_sequence ON storage_record_visibility(execution,epoch,sequence,position);
CREATE INDEX IF NOT EXISTS storage_record_cycle_sequence ON storage_record_visibility(cycle,epoch,sequence,position);
CREATE INDEX IF NOT EXISTS storage_capture_execution_sequence ON storage_record_captures(execution,epoch,sequence,position);
CREATE INDEX IF NOT EXISTS storage_document_record_attempt
    ON storage_record_visibility(execution,coalesce(json_extract(scope,'$.origin.attempt_id'),''),position);
CREATE INDEX IF NOT EXISTS storage_document_capture_attempt
    ON storage_record_captures(execution,coalesce(json_extract(scope,'$.origin.attempt_id'),''),position);
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
    #[cfg(feature = "native")]
    pub(crate) async fn document_records(
        &self, at: StorageCursor, execution: String, after: (String, u64), journal: Option<(String, u64)>,
    ) -> Result<(Vec<ScopedRecord>, Option<(String, u64)>), Error> {
        self.read(move |connection| {
            Ok((|| {
                saved(connection)?.history.check(&at.history)?;
                let selected = "SELECT position,scope,attempt FROM (
                SELECT position,scope,coalesce(json_extract(scope,'$.origin.attempt_id'),'') AS attempt
                FROM storage_record_visibility v WHERE execution=?1 AND position<=?2
                AND (?5 IS NULL OR epoch IS NULL OR epoch<>?5 OR sequence<=?6)
                AND (coalesce(json_extract(scope,'$.origin.attempt_id'),''),position)>(?3,?4)
                AND NOT EXISTS(SELECT 1 FROM storage_record_captures c WHERE c.execution=?1 AND c.position=v.position)
                ORDER BY attempt,position LIMIT 256)
                UNION ALL SELECT position,scope,attempt FROM (
                SELECT position,scope,coalesce(json_extract(scope,'$.origin.attempt_id'),'') AS attempt
                FROM storage_record_captures WHERE execution=?1 AND position<=?2
                AND (?5 IS NULL OR epoch IS NULL OR epoch<>?5 OR sequence<=?6)
                AND (coalesce(json_extract(scope,'$.origin.attempt_id'),''),position)>(?3,?4)
                ORDER BY attempt,position LIMIT 256)
                ORDER BY attempt,position LIMIT 256";
                let (epoch, revision) = journal.map_or((None, u64::MAX), |(epoch, revision)| (Some(epoch), revision));
                let args = vec![
                    execution.clone().into(),
                    position(at.sequence).into(),
                    after.0.into(),
                    position(after.1).into(),
                    epoch.map_or(rusqlite::types::Value::Null, Into::into),
                    position(revision).into(),
                ];
                let mut statement = connection.prepare(selected)?;
                let mut metadata = std::collections::BTreeMap::new();
                let mut rows = statement.query(rusqlite::params_from_iter(args.iter()))?;
                while let Some(row) = rows.next()? {
                    let position = super::sequence(&row.get::<_, String>(0)?)?;
                    let scope: dokime_domain::model::execution::RecordScope = serde_json::from_str(&row.get::<_, String>(1)?)?;
                    metadata.insert(position, (row.get::<_, String>(2)?, scope));
                }
                let sql = format!(
                    "WITH selected AS ({selected}) SELECT e.position,e.body,e.digest FROM selected s
                JOIN storage_data_entries e ON e.position=s.position ORDER BY s.attempt,s.position"
                );
                let records = record_page(connection, &at, &sql, args)?;
                let owner = dokime_domain::model::identity::TestExecutionId::from_uuid(&execution)?;
                for row in &records {
                    let (_, indexed) = metadata.get(&row.position).ok_or(Error::Corrupt("document record index is absent"))?;
                    let actual = row.record.captured_scope(&owner).or(row.record.scope.as_ref());
                    if actual != Some(indexed) {
                        return Err(harmos::Error::InvalidInput {
                            field: "document record scope",
                            reason: "record capture differs from its index",
                        }
                        .into());
                    }
                }
                let next = records.last().map(|row| (metadata[&row.position].0.clone(), row.position));
                Ok((records, next))
            })())
        })
        .await?
    }

    pub(crate) async fn record_frontier(&self) -> Result<u64, Error> {
        self.read(|connection| {
            Ok((|| {
                let mut query = connection.prepare(
                    "SELECT v.ordinal,e.body,e.digest FROM storage_record_visibility v
                     JOIN storage_data_entries e ON e.position=v.position ORDER BY v.ordinal DESC LIMIT 1",
                )?;
                let mut rows = query.query([])?;
                let Some(row) = rows.next()? else { return Ok(0) };
                let ordinal = super::sequence(&row.get::<_, String>(0)?)?;
                let body = blob(row, 1)?;
                check_digest(body, blob(row, 2)?)?;
                let Entry::Record(record) = Entry::decode("record", body)? else { unreachable!() };
                if RecordRow::decode_entry(&record.payload)?.ordinal != ordinal {
                    return Err(Error::Corrupt("record frontier differs from saved evidence"));
                }
                Ok(ordinal)
            })())
        })
        .await?
    }

    pub(crate) async fn scoped_records(
        &self, at: StorageCursor, scope: crate::evidence_runtime::ScopeSelection, epoch: Option<String>, after_position: u64,
        after_sequence: u64, through_sequence: u64,
    ) -> Result<Vec<ScopedRecord>, Error> {
        self.read(move |connection| {
            Ok((|| {
                saved(connection)?.history.check(&at.history)?;
                let (sql, args) =
                    scoped_query(&scope, epoch.as_deref(), (after_position, at.sequence), (after_sequence, through_sequence), None)?;
                record_page(connection, &at, &sql, args)
            })())
        })
        .await?
    }

    pub(crate) async fn context_records(
        &self, at: StorageCursor, execution: String, cycle: String, after: u64,
    ) -> Result<Vec<ScopedRecord>, Error> {
        self.read(move |connection| {
            Ok((|| {
                saved(connection)?.history.check(&at.history)?;
                let sql = "SELECT e.position,e.body,e.digest FROM (
                    SELECT position FROM (SELECT position FROM storage_record_visibility
                    WHERE position>?1 AND position<=?2 AND (execution=?3
                        OR (execution IS NULL AND (cycle=?4 OR (cycle IS NULL
                            AND json_extract(scope,'$.test_case_id') IS NULL
                            AND json_extract(scope,'$.template_id') IS NULL
                            AND json_extract(scope,'$.suite_id') IS NULL
                            AND json_extract(scope,'$.step_id') IS NULL
                            AND json_extract(scope,'$.criterion_id') IS NULL))))
                    ORDER BY position LIMIT 256)
                    UNION SELECT position FROM (SELECT position FROM storage_record_captures
                    WHERE position>?1 AND position<=?2 AND execution=?3
                    ORDER BY position LIMIT 256)
                ) selected JOIN storage_data_entries e ON e.position=selected.position
                ORDER BY e.position LIMIT 256";
                let args = vec![position(after).into(), position(at.sequence).into(), execution.into(), cycle.into()];
                record_page(connection, &at, sql, args)
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
        "INSERT INTO storage_record_visibility VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            position(row.position.sequence),
            record.record_id.as_str(),
            position(record.ordinal),
            scope.and_then(|scope| scope.test_cycle_id.as_ref()).map(|id| id.as_str()),
            scope.and_then(|scope| scope.test_execution_id.as_ref()).map(|id| id.as_str()),
            position(record.sequence),
            row.scope.as_ref().map(|epoch| epoch.as_str()),
            scope.map(serde_json::to_string).transpose()?
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
    for scope in record.scope.iter().chain(&record.captured_scopes) {
        super::distinct::insert(
            connection,
            super::distinct::Row {
                kind: "record",
                source: record.record_type.as_str(),
                position: row.position.sequence,
                file: None,
                record: true,
                scope,
            },
            record.details.iter().map(|(name, value)| (name.as_str(), value)),
        )?;
    }
    Ok(())
}

/* === Private Functions === */

fn record_page(
    connection: &Connection, at: &StorageCursor, sql: &str, args: Vec<rusqlite::types::Value>,
) -> Result<Vec<ScopedRecord>, Error> {
    let mut query = connection.prepare(sql)?;
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
        at.history.check(&record.position.history)?;
        if record.schema != crate::storage::records::schema() {
            return Err(Error::Corrupt("record capture schema differs from its index"));
        }
        let indexed = super::sequence(&row.get::<_, String>(0)?)?;
        if record.position.sequence != indexed {
            return Err(Error::Corrupt("record capture index position differs"));
        }
        let epoch = record.scope.as_ref().ok_or(Error::Corrupt("captured record has no process epoch"))?;
        if epoch.as_str().bytes().any(|byte| byte.is_ascii_uppercase()) || record.source.is_some() || record.source_time.is_some() {
            return Err(Error::Corrupt("captured record envelope differs from its declared schema"));
        }
        let entry = RecordRow::decode_entry(&record.payload)?;
        if record.item.as_str() != entry.record_id.as_str() {
            return Err(Error::Corrupt("captured record key differs from payload identity"));
        }
        output.push(ScopedRecord {
            position: indexed,
            epoch: dokime_domain::model::identity::SessionId::from_uuid(epoch.as_str())?,
            record: entry,
        });
        bytes += body.len();
    }
    Ok(output)
}

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
    for (field, value) in [
        ("test_case_id", scope.test_case_id.as_ref().map(ToString::to_string)),
        ("template_id", scope.template_id.as_ref().map(ToString::to_string)),
        ("suite_id", scope.suite_id.as_ref().map(ToString::to_string)),
    ] {
        if let Some(value) = value {
            args.push(Value::Text(value));
            conditions.push(format!("json_extract(scope,'$.{field}')=?{}", args.len()));
        }
    }
    if conditions.len() == 1 {
        return Err(Error::Invalid("indexed record read requires a test owner"));
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
        use dokime_domain::model::identity::{CaseId, SuiteId, TemplateId, TestCycleId, TestExecutionId};
        let execution = TestExecutionId::from_seed("indexed-owner");
        let shared = TestExecutionId::from_seed("shared-owner");
        let cycle = TestCycleId::from_seed("indexed-cycle");
        let case = CaseId::from_seed("indexed-case");
        let template = TemplateId::from_seed("indexed-template");
        let suite = SuiteId::from_seed("indexed-suite");
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
                    printf('%016x',row_number() OVER (PARTITION BY scope ORDER BY position)),scope,
                    '{{\"test_case_id\":\"{case}\",\"template_id\":\"{template}\",\"suite_id\":\"{suite}\"}}' FROM storage_data_entries;
                INSERT INTO storage_record_captures
                SELECT '{execution}',position,record_id,ordinal,scope,cycle,sequence,epoch FROM storage_record_visibility;
                INSERT INTO storage_record_captures
                SELECT '{shared}',position,record_id,ordinal,scope,cycle,sequence,epoch FROM storage_record_visibility;",
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
                ScopeSelection {
                    test_case_id: Some(case.clone()),
                    ..Default::default()
                },
                ScopeSelection {
                    template_id: Some(template.clone()),
                    ..Default::default()
                },
                ScopeSelection {
                    suite_id: Some(suite.clone()),
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
