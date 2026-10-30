/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Indexed Eligible Telemetry Anchors At A Durable Frontier.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::{Entry, State, blob, check_digest, entry, position, saved};
use crate::storage::{Error, telemetry::TypedCodec};
use harmos::storage::{SchemaKey, StorageCursor, StoredRow};
use rusqlite::{Connection, OptionalExtension, params, types::Value};

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

pub(super) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_sample_receipts (
    execution TEXT NOT NULL, stream TEXT NOT NULL, sample_id TEXT NOT NULL, position TEXT NOT NULL,
    PRIMARY KEY(execution,stream)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS storage_sample_identity ON storage_data_entries(scope,stream,item,position) WHERE kind='stream';
CREATE TABLE IF NOT EXISTS storage_telemetry_index (
    position TEXT PRIMARY KEY REFERENCES storage_data_entries(position) ON DELETE CASCADE,
    schema_key TEXT NOT NULL, source_time BLOB NOT NULL,
    scope TEXT NOT NULL, source TEXT NOT NULL
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_telemetry_signals (
    schema_key TEXT NOT NULL, signal TEXT NOT NULL, position TEXT NOT NULL REFERENCES storage_data_entries(position) ON DELETE CASCADE,
    source_time BLOB NOT NULL, scope TEXT NOT NULL, source TEXT NOT NULL,
    PRIMARY KEY(schema_key,signal,position)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS storage_telemetry_position ON storage_telemetry_signals(position);
CREATE INDEX IF NOT EXISTS storage_telemetry_signal_time ON storage_telemetry_signals(schema_key,signal,source_time DESC,position DESC);
CREATE INDEX IF NOT EXISTS storage_telemetry_signal_execution_time ON storage_telemetry_signals(
    schema_key,signal,json_extract(scope,'$.test_execution_id'),source_time DESC,position DESC);
CREATE INDEX IF NOT EXISTS storage_telemetry_scope_time ON storage_telemetry_index(schema_key,scope,source_time DESC,position DESC);
CREATE INDEX IF NOT EXISTS storage_telemetry_time ON storage_telemetry_index(schema_key, source_time DESC, position DESC);
CREATE INDEX IF NOT EXISTS storage_telemetry_execution_time ON storage_telemetry_index(
    schema_key, json_extract(scope, '$.test_execution_id'), source_time DESC, position DESC);
CREATE INDEX IF NOT EXISTS storage_telemetry_cycle_time ON storage_telemetry_index(
    schema_key, json_extract(scope, '$.test_cycle_id'), source_time DESC, position DESC);
CREATE INDEX IF NOT EXISTS storage_telemetry_case_time ON storage_telemetry_index(
    schema_key, json_extract(scope, '$.test_case_id'), source_time DESC, position DESC);
";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl State {
    pub(crate) async fn acknowledge_sample(
        &self, execution: String, stream: String, sample_id: String, through: u64,
    ) -> Result<u64, Error> {
        self.call(move |connection| {
            Ok((|| -> Result<u64, Error> {
                let transaction = connection.transaction()?;
                let selected = acknowledged_position(&transaction, &execution, &stream, &sample_id, through)?;
                transaction.execute(
                    "INSERT INTO storage_sample_receipts VALUES (?1,?2,?3,?4)
                 ON CONFLICT(execution,stream) DO UPDATE SET sample_id=excluded.sample_id,position=excluded.position",
                    params![execution, stream, sample_id, position(selected)],
                )?;
                super::super::commit(transaction)?;
                Ok(selected)
            })())
        })
        .await?
    }

    pub(crate) async fn telemetry_page(
        &self, at: StorageCursor, after: u64, execution: Option<String>, stream: Option<String>,
    ) -> Result<Vec<StoredRow>, Error> {
        let head = self.data_saved().await?;
        head.history.check(&at.history)?;
        if at.sequence > head.sequence {
            return Err(Error::Data("telemetry page exceeds saved frontier"));
        }
        let scan = super::Scan::new(
            "stream",
            at.sequence,
            head.retention_generation,
            harmos::storage::ScanPage {
                after,
                rows: std::num::NonZeroUsize::new(512).unwrap(),
                bytes: std::num::NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
            },
        )
        .filter("scope", execution)
        .filter("stream", stream);
        self.data_scan(scan)
            .await?
            .into_iter()
            .map(|entry| match entry {
                Entry::Stream(row) => Ok(row),
                _ => Err(Error::Corrupt("telemetry page contains another collection")),
            })
            .collect()
    }

    pub(crate) async fn latest_telemetry_time(
        &self, at: StorageCursor, schema: SchemaKey, terms: Vec<(&'static str, Value)>,
        selection: crate::evidence_runtime::TelemetrySelection, codec: TypedCodec,
    ) -> Result<Option<std::time::SystemTime>, Error> {
        let metrics = self.0.read_metrics.clone();
        self.read(move |connection| {
            Ok((|| {
                let head = saved(connection)?;
                head.history.check(&at.history)?;
                if at.sequence > head.sequence {
                    return Err(Error::Data("telemetry anchor exceeds saved frontier"));
                }
                let (sql, arguments) = anchor_query(&schema, at.sequence, &terms, &selection)?;
                let mut statement = connection.prepare(&sql)?;
                let mut rows = statement.query(rusqlite::params_from_iter(arguments))?;
                while let Some(row) = rows.next()? {
                    super::super::TELEMETRY_ANCHOR_ROWS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    metrics.telemetry_anchor_rows.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let encoded = blob(row, 0)?;
                    check_digest(encoded, blob(row, 1)?)?;
                    let indexed_time = blob(row, 2)?.to_vec();
                    let indexed_scope: String = row.get(3)?;
                    let indexed_source: String = row.get(4)?;
                    let indexed_position: String = row.get(5)?;
                    let Entry::Stream(row) = Entry::decode("stream", encoded)? else {
                        return Err(Error::Corrupt("telemetry anchor collection differs"));
                    };
                    let sample = codec.decode_stored(&row)?.sample;
                    let (scope, source, signals) = crate::storage::telemetry::index_metadata(&row.record.payload)?;
                    let timestamp = row.record.source_time.ok_or(Error::Corrupt("telemetry anchor has no timestamp"))?;
                    at.history.check(&row.record.position.history)?;
                    if indexed_position != position(row.record.position.sequence)
                        || row.record.position.sequence > at.sequence
                        || indexed_time != entry::timestamp_key(timestamp)
                        || indexed_scope != serde_json::to_string(&scope)?
                        || indexed_source != source
                        || signals.windows(2).any(|pair| pair[0] >= pair[1])
                    {
                        return Err(Error::Corrupt("telemetry anchor index differs from immutable evidence"));
                    }
                    if !selection.signals.iter().all(|signal| sample.values.contains_key(signal)) {
                        continue;
                    }
                    return Ok(row.record.source_time);
                }
                Ok(None)
            })())
        })
        .await?
    }
    pub(crate) async fn telemetry_predecessor_page(
        &self, at: StorageCursor, schema: SchemaKey, mut selection: crate::evidence_runtime::TelemetrySelection,
        terms: Vec<(&'static str, Value)>, start: std::time::SystemTime, after: String,
    ) -> Result<(String, Vec<StoredRow>), Error> {
        self.read(move |connection| {
            Ok((|| {
                let head = saved(connection)?;
                head.history.check(&at.history)?;
                if at.sequence > head.sequence {
                    return Err(Error::Data("telemetry predecessor exceeds saved frontier"));
                }
                let mut statement = connection.prepare(
                    "SELECT scope FROM storage_telemetry_index INDEXED BY storage_telemetry_scope_time
                WHERE schema_key=?1 AND scope>?2 ORDER BY scope LIMIT 1",
                )?;
                let schema_key = serde_json::to_string(&schema)?;
                let mut cursor = after;
                let mut output = Vec::new();
                let boundary: time::OffsetDateTime = start.into();
                let boundary = dokime_domain::model::time::Timestamp::new(
                    boundary
                        .format(&time::format_description::well_known::Rfc3339)
                        .map_err(|_| Error::Data("invalid predecessor boundary"))?,
                );
                if selection.end.as_ref().is_none_or(|end| {
                    end.to_datetime().is_ok_and(|end| end > boundary.to_datetime().expect("constructed boundary is valid"))
                }) {
                    selection.end = Some(boundary);
                }
                for _ in 0..256 {
                    let scope = statement.query_row(params![schema_key, cursor], |row| row.get::<_, String>(0));
                    let scope = match scope {
                        Ok(scope) => scope,
                        Err(rusqlite::Error::QueryReturnedNoRows) => break,
                        Err(error) => return Err(error.into()),
                    };
                    cursor = scope.clone();
                    let mut scoped = terms.clone();
                    scoped.push(("$scope", Value::Text(scope)));
                    let (sql, args) = anchor_query(&schema, at.sequence, &scoped, &selection)?;
                    let mut anchor = connection.prepare(&sql)?;
                    let mut rows = anchor.query(rusqlite::params_from_iter(args))?;
                    if let Some(row) = rows.next()? {
                        let body = blob(row, 0)?;
                        check_digest(body, blob(row, 1)?)?;
                        let Entry::Stream(row) = Entry::decode("stream", body)? else { unreachable!() };
                        output.push(row);
                    }
                }
                Ok((cursor, output))
            })())
        })
        .await?
    }
}

/* === Public Functions === */

pub(super) fn index(connection: &Connection, row: &StoredRow) -> Result<(), Error> {
    if !row.record.schema.id.as_str().starts_with("dokime.telemetry.") {
        return Ok(());
    }
    let (scope, source, signals) = crate::storage::telemetry::index_metadata(&row.record.payload)?;
    let scope = serde_json::to_string(&scope)?;
    let time = row.record.source_time.ok_or(Error::Data("telemetry index has no timestamp"))?;
    let position = position(row.record.position.sequence);
    let schema = serde_json::to_string(&row.record.schema)?;
    let time = entry::timestamp_key(time);

    // Reuse The Batch's Index Programs And Metadata For Every Signal.
    let mut index = connection.prepare_cached("INSERT INTO storage_telemetry_index VALUES (?1,?2,?3,?4,?5)")?;
    index.execute(params![position, schema, time.as_slice(), scope, source])?;
    let mut signal_index = connection.prepare_cached("INSERT INTO storage_telemetry_signals VALUES (?1,?2,?3,?4,?5,?6)")?;
    for signal in signals {
        signal_index.execute(params![schema, signal, position, time.as_slice(), scope, source])?;
    }
    Ok(())
}

/* === Private Functions === */

fn acknowledged_position(connection: &Connection, execution: &str, stream: &str, sample_id: &str, through: u64) -> Result<u64, Error> {
    let checkpoint: Option<String> = connection
        .query_row(
            "SELECT position FROM storage_sample_receipts WHERE execution=?1 AND stream=?2 AND sample_id=?3 AND position<=?4",
            params![execution, stream, sample_id, position(through)],
            |row| row.get(0),
        )
        .optional()?;
    let selected = match checkpoint {
        Some(position) => position,
        None => connection
            .query_row(
                "SELECT position FROM storage_data_entries WHERE kind='stream' AND scope=?1 AND stream=?2 AND item=?3 AND position<=?4
             ORDER BY position DESC LIMIT 1",
                params![execution, stream, sample_id, position(through)],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or(Error::Corrupt("acknowledged sample is absent from saved execution evidence"))?,
    };
    u64::from_str_radix(&selected, 16).map_err(|_| Error::Corrupt("invalid sample receipt position"))
}

fn anchor_query(
    schema: &SchemaKey, at: u64, terms: &[(&'static str, Value)], selection: &crate::evidence_runtime::TelemetrySelection,
) -> Result<(String, Vec<Value>), Error> {
    let signals = !selection.signals.is_empty();
    let execution = terms.iter().any(|(field, _)| *field == "payload.metadata.scope.test_execution_id");
    let index = if signals {
        if execution { "storage_telemetry_signal_execution_time" } else { "storage_telemetry_signal_time" }
    } else if terms.iter().any(|(field, _)| *field == "$scope") {
        "storage_telemetry_scope_time"
    } else if execution {
        "storage_telemetry_execution_time"
    } else if terms.iter().any(|(field, _)| *field == "payload.metadata.scope.test_cycle_id") {
        "storage_telemetry_cycle_time"
    } else if terms.iter().any(|(field, _)| *field == "payload.metadata.scope.test_case_id") {
        "storage_telemetry_case_time"
    } else {
        "storage_telemetry_time"
    };
    let table = if signals { "storage_telemetry_signals" } else { "storage_telemetry_index" };
    let mut sql = format!(
        "SELECT e.body,e.digest,i.source_time,i.scope,i.source,i.position FROM {table} i INDEXED BY {index}
        CROSS JOIN storage_data_entries e ON e.position=i.position WHERE i.schema_key=? AND i.position<=?"
    );
    let mut arguments = vec![Value::Text(serde_json::to_string(schema)?), Value::Text(position(at))];
    if let Some(signal) = selection.signals.first() {
        sql.push_str(" AND i.signal=?");
        arguments.push(Value::Text(signal.clone()));
        for signal in selection.signals.iter().skip(1) {
            sql.push_str(" AND EXISTS(SELECT 1 FROM storage_telemetry_signals s WHERE s.schema_key=i.schema_key AND s.position=i.position AND s.signal=?)");
            arguments.push(Value::Text(signal.clone()));
        }
    }
    for (field, value) in terms {
        if *field == "$scope" {
            sql.push_str(" AND i.scope=?");
            arguments.push(value.clone());
            continue;
        }
        let field = field.strip_prefix("payload.metadata.scope.").ok_or(Error::Data("invalid indexed telemetry scope field"))?;
        if !field.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.')) {
            return Err(Error::Data("invalid indexed telemetry scope path"));
        }
        sql.push_str(&format!(" AND json_extract(i.scope, '$.{field}')=?"));
        arguments.push(value.clone());
    }
    for (bound, operator) in [(selection.start.as_ref(), ">="), (selection.end.as_ref(), "<")] {
        if let Some(bound) = bound {
            let instant: std::time::SystemTime = bound.to_datetime()?.into();
            sql.push_str(&format!(" AND i.source_time{operator}?"));
            arguments.push(Value::Blob(entry::timestamp_key(instant).to_vec()));
        }
    }
    if !selection.sources.is_empty() {
        sql.push_str(" AND i.source IN (");
        for (index, source) in selection.sources.iter().enumerate() {
            if index != 0 {
                sql.push(',');
            }
            sql.push('?');
            arguments.push(Value::Text(source.clone()));
        }
        sql.push(')');
    }
    sql.push_str(" ORDER BY i.source_time DESC,i.position DESC");
    sql.push_str(" LIMIT 1");
    Ok((sql, arguments))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn acknowledged_sample_resumes_after_reopen_and_rejects_unsaved_receipts() {
        let root = tempfile::tempdir().unwrap();
        let store = State::open(root.path()).await.unwrap();
        store
            .call(|connection| {
                connection.execute_batch(super::super::SCHEMA)?;
                connection.execute_batch(SCHEMA)?;
                connection.execute_batch(
                    "WITH RECURSIVE rows(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM rows WHERE n<10080)
                 INSERT INTO storage_data_entries(position,kind,identity,item,scope,stream,body,digest)
                 SELECT printf('%016x',n),'stream',printf('row-%d',n),printf('sample-%d',n),'execution','stream',x'00',zeroblob(32)
                 FROM rows;",
                )?;
                Ok(())
            })
            .await
            .unwrap();
        let acknowledged = store.acknowledge_sample("execution".into(), "stream".into(), "sample-10079".into(), 10080).await.unwrap();
        assert_eq!(acknowledged, 10079);
        store.close().await.unwrap();
        drop(store);

        let reopened = State::open(root.path()).await.unwrap();
        let resumed = reopened.acknowledge_sample("execution".into(), "stream".into(), "sample-10079".into(), 10080).await.unwrap();
        assert_eq!(resumed, acknowledged);
        assert!(reopened.acknowledge_sample("execution".into(), "stream".into(), "sample-10080".into(), 10079).await.is_err());
        let checkpoint = reopened
            .call(|connection| connection.query_row("SELECT position FROM storage_sample_receipts", [], |row| row.get::<_, String>(0)))
            .await
            .unwrap();
        assert_eq!(checkpoint, position(acknowledged));
        let next = reopened.acknowledge_sample("execution".into(), "stream".into(), "sample-10080".into(), 10080).await.unwrap();
        assert_eq!(next, 10080);
        reopened.close().await.unwrap();
    }

    #[test]
    fn latest_anchor_skips_historical_rows() {
        let schema = SchemaKey {
            id: harmos::Identifier::new("dokime.telemetry.probe").unwrap(),
            version: std::num::NonZeroU32::new(1).unwrap(),
            payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
        };
        for history in [128_u64, 4096, 65536] {
            let connection = Connection::open_in_memory().unwrap();
            connection.execute_batch(super::super::SCHEMA).unwrap();
            connection.execute_batch(SCHEMA).unwrap();
            connection
                .execute_batch(&format!(
                    "WITH RECURSIVE rows(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM rows WHERE n<{history})
                INSERT INTO storage_data_entries(position,kind,identity,schema_key,item,body,digest)
                SELECT printf('%016x',n),'stream',printf('row-%d',n),'schema','item',x'00',zeroblob(32) FROM rows;"
                ))
                .unwrap();
            connection
                .execute(
                    "INSERT INTO storage_telemetry_index SELECT position,?1,CAST(position AS BLOB),
                '{\"test_execution_id\":\"execution\"}','source' FROM storage_data_entries",
                    [serde_json::to_string(&schema).unwrap()],
                )
                .unwrap();
            let terms = vec![("payload.metadata.scope.test_execution_id", Value::Text("execution".into()))];
            let (sql, arguments) = anchor_query(&schema, history, &terms, &Default::default()).unwrap();
            let mut statement = connection.prepare(&sql).unwrap();
            let selected: String = statement.query_row(rusqlite::params_from_iter(arguments), |row| row.get(5)).unwrap();
            assert_eq!(selected, position(history));
            assert!(statement.get_status(rusqlite::StatementStatus::VmStep) < 100);
        }
    }
}
