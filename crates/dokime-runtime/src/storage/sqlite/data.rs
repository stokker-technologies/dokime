/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod backend;
pub(crate) mod distinct;
mod entry;
pub(super) mod files;
mod mutations;
mod production;
mod records;
mod retention;
pub(crate) mod telemetry;

/* === Re-Exports === */

pub use backend::Backend;
pub(crate) use entry::Entry;
pub(super) use files::VerifiedPrefix;
pub(crate) use records::query::{Read as RecordQueryRead, Scan as RecordQueryScan};

/* === Imports === */

// Standard Library Imports
use std::result::Result;

// Third Party Imports
use harmos::storage::*;
use rusqlite::{Connection, params, types::Value};
use sha2::{Digest, Sha256};

// Local Imports
use super::{Error, State};
use entry::{Borrowed, Encoded, key, timestamp_key};

/* === Constants === */

pub(super) const FORMAT: &str = "dokime.sqlite.storage.v1";
pub(super) const HISTORY_BYTES: usize = 4096;
// Rows Per Multi-Row Insert; Twelve Bound Columns Each Stay Well Below SQLite's Variable Limit.
const INSERT_ROWS: usize = 128;
const HEAD_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_data_head (
    singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
    format TEXT NOT NULL, history BLOB NOT NULL,
    saved TEXT NOT NULL CHECK(length(saved) = 16),
    retention_generation TEXT NOT NULL CHECK(length(retention_generation) = 16)
);
";
const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_data_entries (
    position TEXT PRIMARY KEY CHECK(length(position) = 16),
    kind TEXT NOT NULL CHECK(kind IN ('schema','object','record','stream','file')),
    identity TEXT NOT NULL UNIQUE,
    schema_id TEXT, schema_key TEXT, item TEXT NOT NULL,
    scope TEXT, stream TEXT, source_time BLOB,
    object_version TEXT, body BLOB NOT NULL,
    digest BLOB NOT NULL CHECK(length(digest) = 32), retired_at TEXT
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_object_releases (
    schema_id TEXT NOT NULL, schema_key TEXT NOT NULL, schema_version INTEGER NOT NULL,
    encoding INTEGER NOT NULL, item TEXT NOT NULL,
    through TEXT NOT NULL CHECK(length(through)=16),
    generation TEXT NOT NULL CHECK(length(generation)=16),
    PRIMARY KEY(schema_id,item,generation)
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_staging (
    owner TEXT NOT NULL, id TEXT NOT NULL, phase TEXT NOT NULL,
    uuid TEXT NOT NULL UNIQUE, PRIMARY KEY(owner,id,phase)
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_file_stages (
    id TEXT PRIMARY KEY, metadata BLOB NOT NULL, finish_intent BLOB, finished BLOB
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS storage_data_kind_position ON storage_data_entries(kind, position);
CREATE INDEX IF NOT EXISTS storage_data_object ON storage_data_entries(kind, schema_id, item, position) WHERE kind='object';
CREATE INDEX IF NOT EXISTS storage_data_live_object_position_covering
    ON storage_data_entries(schema_id, position, item, kind, retired_at)
    WHERE kind='object' AND retired_at IS NULL;
CREATE INDEX IF NOT EXISTS storage_data_live_object_key_covering
    ON storage_data_entries(schema_id, item, position, kind, retired_at)
    WHERE kind='object' AND retired_at IS NULL;
CREATE INDEX IF NOT EXISTS storage_data_retired_object_position
    ON storage_data_entries(position, retired_at)
    WHERE kind='object' AND retired_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS storage_object_releases_generation
    ON storage_object_releases(schema_id, generation, item);
DROP INDEX IF EXISTS storage_data_scope;
DROP INDEX IF EXISTS storage_data_schema_scope;
DROP INDEX IF EXISTS storage_data_stream_owner;
DROP INDEX IF EXISTS storage_data_stream_scope_time;
CREATE INDEX IF NOT EXISTS storage_data_scoped ON storage_data_entries(kind, scope, position) WHERE scope IS NOT NULL;
CREATE INDEX IF NOT EXISTS storage_data_schema_scoped ON storage_data_entries(kind, schema_key, scope, position) WHERE scope IS NOT NULL;
CREATE INDEX IF NOT EXISTS storage_data_schema_position ON storage_data_entries(kind, schema_key, position);
CREATE INDEX IF NOT EXISTS storage_data_stream_scoped_owner ON storage_data_entries(scope,stream,position)
    WHERE kind='stream' AND scope IS NOT NULL;
CREATE INDEX IF NOT EXISTS storage_data_stream_time ON storage_data_entries(schema_key, source_time, position) WHERE kind='stream';
CREATE INDEX IF NOT EXISTS storage_data_stream_scoped_time ON storage_data_entries(schema_key, scope, source_time, position)
    WHERE kind='stream' AND scope IS NOT NULL;
";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct SavedData {
    pub history: StoreHistory,
    pub sequence: u64,
    pub retention_generation: u64,
}

pub(super) struct Scan {
    pub kind: &'static str,
    pub at: u64,
    pub generation: u64,
    pub filters: Vec<(&'static str, Value)>,
    pub source_window: Option<(std::time::SystemTime, std::time::SystemTime)>,
    pub source_bounds: (Option<std::time::SystemTime>, Option<std::time::SystemTime>),
    pub latest_objects: bool,
    pub page: ScanPage,
    metrics: Option<std::sync::Arc<super::ReadMetrics>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl State {
    // Enumerate Index Keys Without Opening Any Stored Value.
    pub(crate) async fn object_keys(&self, at: StorageCursor, schema: String, prefix: String, after: String) -> Result<Vec<String>, Error> {
        self.read(move |connection| {
            Ok((|| {
                saved(connection)?.history.check(&at.history)?;
                let mut statement = connection.prepare(
                    "SELECT DISTINCT item FROM storage_data_entries
                     WHERE kind='object' AND schema_id=?1 AND position<=?2 AND retired_at IS NULL
                     AND item>=?3 AND item<?4 AND item>?5 ORDER BY item LIMIT 256",
                )?;
                let end = format!("{prefix}\u{10ffff}");
                let rows = statement.query_map(params![schema, position(at.sequence), prefix, end, after], |row| row.get(0))?;
                rows.collect::<Result<Vec<_>, _>>().map_err(Error::from)
            })())
        })
        .await?
    }

    pub(crate) async fn artifact_owner(&self, id: String) -> Result<Option<String>, Error> {
        self.read(move |connection| {
            use rusqlite::OptionalExtension;
            Ok((|| {
                let row = connection
                    .query_row(
                        "SELECT scope,body,digest FROM storage_data_entries WHERE kind='file' AND item=?1 ORDER BY position DESC LIMIT 1",
                        [&id],
                        |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Vec<u8>>(1)?, row.get::<_, Vec<u8>>(2)?)),
                    )
                    .optional()?;
                let Some((owner, body, digest)) = row else { return Ok(None) };
                check_digest(&body, &digest)?;
                let Entry::File(file) = Entry::decode("file", &body)? else { unreachable!() };
                if file.metadata.id.as_str() != id || file.metadata.scope.as_ref().map(ToString::to_string) != owner {
                    return Err(Error::Corrupt("artifact file owner differs from its index"));
                }
                Ok(owner)
            })())
        })
        .await?
    }

    pub(crate) async fn selected_execution_ids(
        &self, at: StorageCursor, scope: dokime_domain::model::execution::RecordScope,
    ) -> Result<Vec<dokime_domain::model::identity::TestExecutionId>, Error> {
        self.read(move |connection| {
            Ok((|| {
                saved(connection)?.history.check(&at.history)?;
                let mut sql = String::from(
                    "SELECT h.id FROM query_history h WHERE h.kind='execution' AND h.position<=?1
                    AND NOT EXISTS(SELECT 1 FROM query_history n WHERE n.kind=h.kind AND n.id=h.id
                    AND n.position<=?1 AND n.revision>h.revision)",
                );
                let mut args = vec![Value::Text(position(at.sequence))];
                for (column, value) in [
                    ("h.id", scope.test_execution_id.as_ref().map(ToString::to_string)),
                    ("h.cycle", scope.test_cycle_id.as_ref().map(ToString::to_string)),
                    ("json_extract(h.body,'$.case_id')", scope.test_case_id.as_ref().map(ToString::to_string)),
                    ("h.template", scope.template_id.as_ref().map(ToString::to_string)),
                    ("json_extract(h.body,'$.suite_id')", scope.suite_id.as_ref().map(ToString::to_string)),
                ] {
                    if let Some(value) = value {
                        args.push(value.into());
                        sql.push_str(&format!(" AND {column}=?{}", args.len()));
                    }
                }
                sql.push_str(" ORDER BY h.id");
                let mut statement = connection.prepare(&sql)?;
                let keys = statement.query_map(rusqlite::params_from_iter(args), |row| row.get::<_, String>(0))?;
                keys.map(|key| dokime_domain::model::identity::TestExecutionId::from_uuid(key?).map_err(Error::from)).collect()
            })())
        })
        .await?
    }

    pub(crate) async fn cycle_execution_cases(
        &self, at: StorageCursor, cycle: String,
    ) -> Result<std::collections::BTreeMap<dokime_domain::model::identity::TestExecutionId, dokime_domain::model::identity::CaseId>, Error>
    {
        self.read(move |connection| {
            Ok((|| {
                saved(connection)?.history.check(&at.history)?;
                let mut statement = connection.prepare(
                    "SELECT h.id,json_extract(h.body,'$.case_id') FROM query_history h
                    WHERE h.kind='execution' AND h.cycle=?1 AND h.position<=?2
                    AND NOT EXISTS(SELECT 1 FROM query_history n WHERE n.kind=h.kind AND n.id=h.id
                        AND n.position<=?2 AND n.revision>h.revision)",
                )?;
                let rows = statement
                    .query_map(params![cycle, position(at.sequence)], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;
                rows.map(|row| {
                    let (execution, case) = row?;
                    Ok((
                        dokime_domain::model::identity::TestExecutionId::from_uuid(execution)?,
                        dokime_domain::model::identity::CaseId::from_uuid(case)?,
                    ))
                })
                .collect()
            })())
        })
        .await?
    }

    pub(crate) async fn data_open(&self, expected: Option<StoreHistory>) -> Result<SavedData, Error> {
        let owner = self.0.clone();
        self.call(move |connection| {
            Ok((|| {
                let head = open(connection, expected)?;
                owner.telemetry.lock().unwrap().recover(connection)?;
                Ok(head)
            })())
        })
        .await?
    }

    pub(crate) async fn data_history(&self) -> Result<StoreHistory, Error> {
        self.call(move |connection| Ok(bootstrap(connection))).await?
    }

    pub(crate) async fn data_saved(&self) -> Result<SavedData, Error> {
        self.read(move |connection| Ok(saved(connection))).await?
    }

    async fn data_write<T>(&self, root: std::path::PathBuf, batch: StorageBatch<T>) -> Result<SavedData, Error>
    where
        T: Send + Sync + 'static,
        for<'a> Borrowed<'a>: From<&'a T>,
    {
        let bytes = batch.as_slice().iter().try_fold(0usize, |total, value| {
            entry::check_size(&Borrowed::from(value), usize::MAX).map(|bytes| total.saturating_add(bytes))
        })?;
        let owner = self.0.clone();
        self.grouped(batch.len(), bytes, move |connection| {
            let mut files = owner.files.lock().unwrap();
            let mut prefix = owner.verification_prefix.lock().unwrap();
            Ok(write(connection, &root, batch, &mut files, &mut prefix))
        })
        .await?
    }

    async fn data_object(&self, at: u64, generation: u64, point: ObjectPoint) -> Result<Option<ObjectEntry>, Error> {
        self.read(move |connection| {
            let result = read_object(connection, at, generation, &point);
            Ok(result)
        })
        .await?
    }

    async fn data_scan(&self, mut scan: Scan) -> Result<Vec<Entry>, Error> {
        scan.metrics = Some(self.0.read_metrics.clone());
        if scan.kind == "stream"
            && scan.filters.iter().any(|(field, value)| {
                *field == "schema_key"
                    && matches!(value, Value::Text(key) if
                    serde_json::from_str::<SchemaKey>(key).is_ok_and(|key|key.id.as_str().starts_with("dokime.telemetry.")))
            })
        {
            return self.arrow_scan(scan).await;
        }
        let metrics = self.0.read_metrics.clone();
        self.read(move |connection| {
            let result = read_scan(connection, &scan, &metrics);
            Ok(result)
        })
        .await?
    }
}

impl Scan {
    pub(super) fn new(kind: &'static str, at: u64, generation: u64, page: ScanPage) -> Self {
        Self {
            kind,
            at,
            generation,
            filters: Vec::new(),
            source_window: None,
            source_bounds: (None, None),
            latest_objects: false,
            page,
            metrics: None,
        }
    }

    pub(super) fn filter(mut self, column: &'static str, value: Option<String>) -> Self {
        if let Some(value) = value {
            self.filters.push((column, Value::Text(value)));
        }
        self
    }
}

/* === Public Functions === */

pub(super) fn check_format(connection: &Connection) -> Result<(), Error> {
    let exists: bool =
        connection.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='storage_data_head')", [], |row| row.get(0))?;
    if exists {
        let indexed: bool = connection.query_row(
            "SELECT count(*)=11 FROM sqlite_master WHERE type='table' AND name IN
                ('files','streams','storage_record_visibility','storage_record_captures',
                 'storage_numeric_summaries','storage_numeric_checkpoints','storage_report_previews','stream_samples',
                 'storage_distinct_values','storage_value_postings','storage_distinct_fields')",
            [],
            |row| row.get(0),
        )?;
        let initialized: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='storage_data_entries')",
            [],
            |row| row.get(0),
        )?;
        if initialized && !indexed {
            return Err(Error::Incompatible("local.sqlite does not satisfy the current indexed evidence schema".into()));
        }
        if initialized {
            super::history::check_format(connection)?;
            super::snapshots::check_format(connection)?;
            for query in [
                "SELECT id,kind,path,size_bytes,sha256,media_type FROM files LIMIT 0",
                "SELECT file_id,execution,relative_path FROM artifacts LIMIT 0",
                "SELECT file_id,contract,durable_end,hash_state,finalized,sealed FROM streams LIMIT 0",
                "SELECT execution,plot,definition,shared FROM report_plot_previews LIMIT 0",
                "SELECT execution,plot,projection FROM report_plot_bindings LIMIT 0",
                "SELECT execution,stream,through,state FROM storage_report_context LIMIT 0",
                "SELECT file_id,offset,metadata_bytes,body_bytes,sha256 FROM stream_batches LIMIT 0",
                "SELECT sample_id,position,file_id,batch,batch_row FROM stream_samples LIMIT 0",
                "SELECT file_id,batch,scope,first_at,last_at FROM stream_batch_scopes LIMIT 0",
                "SELECT execution,cycle,file_id,batch,first_row,rows,scope FROM stream_captures LIMIT 0",
                "SELECT cycle,sequence,epoch FROM storage_record_captures LIMIT 0",
                "SELECT sequence,epoch,scope FROM storage_record_visibility LIMIT 0",
                "SELECT value_type,position,scope FROM storage_value_postings LIMIT 0",
                "SELECT execution,source,field FROM storage_distinct_fields LIMIT 0",
            ] {
                connection
                    .prepare(query)
                    .map_err(|_| Error::Incompatible("local.sqlite requires a fresh storage root for indexed evidence".into()))?;
            }
        }
        let format: Option<String> = connection
            .query_row("SELECT format FROM storage_data_head WHERE singleton=1", [], |row| row.get(0))
            .map(Some)
            .or_else(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                error => Err(error),
            })?;
        if format.as_deref().is_some_and(|format| format != FORMAT) {
            return Err(Error::Incompatible(format!(
                "local.sqlite declares data format {}, and this build reads {FORMAT}",
                format.unwrap_or_default()
            )));
        }
    }
    Ok(())
}

pub(crate) fn saved(connection: &Connection) -> Result<SavedData, Error> {
    let mut statement =
        connection.prepare_cached("SELECT format,history,saved,retention_generation FROM storage_data_head WHERE singleton=1")?;
    let mut rows = statement.query([])?;
    let row = rows.next()?.ok_or(Error::Corrupt("missing saved data head"))?;
    if text(row, 0)? != FORMAT {
        return Err(Error::Data("unsupported SQLite data format"));
    }
    Ok(SavedData {
        history: decode_history(blob(row, 1)?)?,
        sequence: sequence(text(row, 2)?)?,
        retention_generation: sequence(text(row, 3)?)?,
    })
}

pub(super) fn blob<'a>(row: &'a rusqlite::Row<'_>, column: usize) -> Result<&'a [u8], Error> {
    row.get_ref(column)?
        .as_blob()
        .map_err(|source| Error::Sqlite(rusqlite::Error::FromSqlConversionFailure(column, rusqlite::types::Type::Blob, Box::new(source))))
}

pub(super) fn check_digest(body: &[u8], digest: &[u8]) -> Result<(), Error> {
    if Sha256::digest(body).as_slice() != digest {
        return Err(Error::Corrupt("stored entry digest differs"));
    }
    Ok(())
}

pub(super) fn text<'a>(row: &'a rusqlite::Row<'_>, column: usize) -> Result<&'a str, Error> {
    row.get_ref(column)?
        .as_str()
        .map_err(|source| Error::Sqlite(rusqlite::Error::FromSqlConversionFailure(column, rusqlite::types::Type::Text, Box::new(source))))
}

pub(super) fn position(sequence: u64) -> String {
    format!("{sequence:016x}")
}

pub(super) fn sequence(value: &str) -> Result<u64, Error> {
    if value.len() != 16 || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return Err(Error::Corrupt("invalid saved sequence"));
    }
    u64::from_str_radix(value, 16).map_err(|_| Error::Corrupt("invalid saved sequence"))
}

/* === Private Functions === */

// The Backend Must Own Its Durable Identity Before The Native Manager Exists.
// Only This Bounded Identity Decode Runs Outside Manager-Admitted Scratch.
fn bootstrap(connection: &mut Connection) -> Result<StoreHistory, Error> {
    let transaction = connection.savepoint()?;
    transaction.execute_batch(HEAD_SCHEMA)?;
    initialize_head(&transaction, None)?;

    let history = {
        let mut statement = transaction.prepare("SELECT format,history FROM storage_data_head WHERE singleton=1")?;
        let mut rows = statement.query([])?;
        let row = rows.next()?.ok_or(Error::Corrupt("missing saved data head"))?;
        if text(row, 0)? != FORMAT {
            return Err(Error::Data("unsupported SQLite data format"));
        }
        decode_history(blob(row, 1)?)?
    };
    super::release(transaction)?;
    Ok(history)
}

fn decode_history(bytes: &[u8]) -> Result<StoreHistory, Error> {
    if bytes.len() > HISTORY_BYTES {
        return Err(Error::Corrupt("stored history exceeds its bound"));
    }
    Ok(serde_json::from_slice(bytes)?)
}

fn initialize_head(connection: &Connection, expected: Option<&StoreHistory>) -> Result<(), Error> {
    let exists = connection.query_row("SELECT EXISTS(SELECT 1 FROM storage_data_head)", [], |row| row.get::<_, bool>(0))?;
    if exists {
        return Ok(());
    }
    let entries: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='storage_data_entries')",
        [],
        |row| row.get(0),
    )?;
    if entries && connection.query_row("SELECT EXISTS(SELECT 1 FROM storage_data_entries)", [], |row| row.get::<_, bool>(0))? {
        return Err(Error::Corrupt("initialized data owner has no saved head"));
    }
    let history = expected.cloned().unwrap_or_else(|| StoreHistory {
        store: harmos::Identifier::new(uuid::Uuid::now_v7().to_string()).expect("UUID identifier"),
        epoch: harmos::Identifier::new(uuid::Uuid::now_v7().to_string()).expect("UUID identifier"),
    });
    let encoded = serde_json::to_vec(&history)?;
    if encoded.len() > HISTORY_BYTES {
        return Err(Error::Data("history identity exceeds its bound"));
    }
    connection.execute("INSERT INTO storage_data_head VALUES (1,?1,?2,?3,?3)", params![FORMAT, encoded, position(0)])?;
    Ok(())
}

fn open(connection: &mut Connection, expected: Option<StoreHistory>) -> Result<SavedData, Error> {
    let transaction = connection.savepoint()?;
    transaction.execute_batch(HEAD_SCHEMA)?;
    transaction.execute_batch(SCHEMA)?;
    transaction.execute_batch(production::SCHEMA)?;
    transaction.execute_batch(telemetry::SCHEMA)?;
    transaction.execute_batch(telemetry::aggregates::SCHEMA)?;
    transaction.execute_batch(telemetry::previews::SCHEMA)?;
    transaction.execute_batch(crate::plot::preview::index::SCHEMA)?;
    transaction.execute_batch(records::SCHEMA)?;
    transaction.execute_batch(distinct::SCHEMA)?;
    transaction.execute_batch(mutations::SCHEMA)?;
    transaction.execute_batch(super::history::SCHEMA)?;
    super::snapshots::initialize(&transaction)?;
    initialize_head(&transaction, expected.as_ref())?;
    let head = saved(&transaction)?;
    if let Some(expected) = expected {
        expected.check(&head.history)?;
    }
    super::release(transaction)?;
    Ok(head)
}

fn write<T>(
    connection: &mut Connection, root: &std::path::Path, batch: StorageBatch<T>,
    retained: &mut std::collections::BTreeMap<harmos::Identifier, std::fs::File>, prefix: &mut VerifiedPrefix,
) -> Result<SavedData, Error>
where
    for<'a> Borrowed<'a>: From<&'a T>,
{
    let values = batch.as_slice();
    let first = Borrowed::from(values.first().ok_or(Error::Data("data batch must not be empty"))?);
    let history = &first.position().history;
    let encoding = super::probe::Timing::start("encoding");
    // Encode And Validate The Bounded Batch Once Before Any File Confirmation.
    let mut encoded = Vec::new();
    encoded.try_reserve_exact(values.len()).map_err(std::io::Error::other)?;
    let mut encoder = entry::Encoder::default();
    for value in values {
        let value = Borrowed::from(value);
        history.check(&value.position().history)?;
        encoded.push(encoder.encode(value)?);
    }

    drop(encoding);
    let _sqlite = super::probe::Timing::start("sqlite");
    let transaction = connection.savepoint()?;
    let mut head = saved(&transaction)?;
    head.history.check(history)?;
    let mut plots = crate::plot::preview::index::Batch::default();
    // Rows Without Secondary Indexes Accumulate For Multi-Row Inserts; Indexed Rows Insert In Order After Them.
    let mut pending = Vec::new();
    for (value, entry) in values.iter().zip(encoded) {
        let value = Borrowed::from(value);
        // Exact Retries Of Saved Positions Must Repeat The Same Bytes. A Row Above The Saved Frontier
        // Is Refused By The Position Key On Insert, Exactly As The Former Lookup Refused It.
        let retry = entry.position <= head.sequence && retried(&transaction, &entry, &encoder.bytes, head.sequence)?;
        if !retry && head.sequence.checked_add(1) != Some(entry.position) {
            return Err(Error::Data("data write has a gap or pruned retry"));
        }
        if let Borrowed::File(reference) = value {
            // Only A File's Identity Is Independent Of Its Position.
            if !retry {
                let mut identity = transaction.prepare_cached("SELECT EXISTS(SELECT 1 FROM storage_data_entries WHERE identity=?1)")?;
                if identity.query_row([entry.identity.as_str()], |row| row.get::<_, bool>(0))? {
                    return Err(Error::Conflict("data identity already belongs to another position"));
                }
            }
            files::confirm(&transaction, root, reference, retained.get_mut(&reference.metadata.id), prefix)?;
        }
        if retry {
            continue;
        }
        head.sequence = entry.position;
        if matches!(value, Borrowed::Schema(_) | Borrowed::Stream(_)) {
            pending.push(entry);
            continue;
        }
        insert_entries(&transaction, std::mem::take(&mut pending), &encoder.bytes)?;
        insert_entries(&transaction, vec![entry], &encoder.bytes)?;
        if let Borrowed::Object(object) = value {
            super::history::index(&transaction, object)?;
        }
        if let Borrowed::File(reference) = value {
            telemetry::catalog_file(&transaction, reference)?;
        }
        if let Borrowed::Record(row) = value {
            super::history::work::index(&transaction, row)?;
            records::index(&transaction, row)?;
            plots.record(&transaction, row)?;
        }
    }
    insert_entries(&transaction, pending, &encoder.bytes)?;
    plots.save(&transaction)?;
    transaction.prepare_cached("UPDATE storage_data_head SET saved=?1 WHERE singleton=1")?.execute([position(head.sequence)])?;
    super::release(transaction)?;
    super::DATA_BATCHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    for value in values {
        if let Borrowed::File(reference) = Borrowed::from(value) {
            retained.remove(&reference.metadata.id);
        }
    }
    Ok(head)
}

// Inserts Entries In Position Order Through Cached Multi-Row Statements, Then Single Rows For The Remainder.
fn insert_entries(transaction: &Connection, entries: Vec<Encoded<'_>>, bytes: &[u8]) -> Result<(), Error> {
    const COLUMNS: &str = "(?,?,?,?,?,?,?,?,?,?,?,?,NULL)";
    const SINGLE: &str = "INSERT INTO storage_data_entries VALUES (?,?,?,?,?,?,?,?,?,?,?,?,NULL)";
    static CHUNK: std::sync::LazyLock<String> =
        std::sync::LazyLock::new(|| format!("INSERT INTO storage_data_entries VALUES {}", vec![COLUMNS; INSERT_ROWS].join(",")));
    static SHARED: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        let rows = (0..INSERT_ROWS)
            .map(|index| {
                let first = 8 + index * 5;
                format!("(?{first},?1,?{},?2,?3,?4,?5,?6,?{},?7,?{},?{},NULL)", first + 1, first + 2, first + 3, first + 4)
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("INSERT INTO storage_data_entries VALUES {rows}")
    });
    let (chunks, remainder) = entries.as_chunks::<INSERT_ROWS>();
    for chunk in chunks {
        let first = &chunk[0];
        let shared = chunk.iter().all(|entry| {
            entry.kind == first.kind
                && entry.schema_id == first.schema_id
                && entry.schema_key == first.schema_key
                && entry.item == first.item
                && entry.scope == first.scope
                && entry.stream == first.stream
                && entry.object_version == first.object_version
        });
        if shared {
            let [_, kind, _, schema_id, schema_key, item, scope, stream, _, version, _, _] = entry_parameters(first, bytes);
            let common = [kind, schema_id, schema_key, item, scope, stream, version];
            let rows = chunk.iter().flat_map(|entry| {
                let [position, _, identity, _, _, _, _, _, time, _, body, digest] = entry_parameters(entry, bytes);
                [position, identity, time, body, digest]
            });
            let mut insert = transaction.prepare_cached(&SHARED)?;
            insert.execute(rusqlite::params_from_iter(common.into_iter().chain(rows))).map_err(inserted)?;
            continue;
        }
        let mut insert = transaction.prepare_cached(&CHUNK)?;
        let parameters = chunk.iter().flat_map(|entry| entry_parameters(entry, bytes));
        insert.execute(rusqlite::params_from_iter(parameters)).map_err(inserted)?;
    }
    if remainder.is_empty() {
        return Ok(());
    }
    let mut insert = transaction.prepare_cached(SINGLE)?;
    for entry in remainder {
        insert.execute(rusqlite::params_from_iter(entry_parameters(entry, bytes))).map_err(inserted)?;
    }
    Ok(())
}

fn entry_parameters<'a>(entry: &'a Encoded<'_>, bytes: &'a [u8]) -> [rusqlite::types::ToSqlOutput<'a>; 12] {
    use rusqlite::types::{ToSqlOutput, ValueRef};
    let text = |value: Option<&'a str>| ToSqlOutput::Borrowed(value.map_or(ValueRef::Null, |value| ValueRef::Text(value.as_bytes())));
    [
        ToSqlOutput::Borrowed(ValueRef::Text(entry.identity.position())),
        text(Some(entry.kind)),
        text(Some(entry.identity.as_str())),
        text(entry.schema_id),
        text(entry.schema_key.as_deref()),
        text(Some(entry.item)),
        text(entry.scope),
        text(entry.stream),
        ToSqlOutput::Borrowed(entry.source_time.as_ref().map_or(ValueRef::Null, |time| ValueRef::Blob(time.as_slice()))),
        text(entry.object_version.as_deref()),
        ToSqlOutput::Borrowed(ValueRef::Blob(&bytes[entry.body.clone()])),
        ToSqlOutput::Borrowed(ValueRef::Blob(entry.digest.as_slice())),
    ]
}

// File Identities Are Checked Before Insert, So Every Other Identity Derives From Its Position:
// Either Key Being Taken Means A Row Already Exists Above The Saved Frontier.
fn inserted(error: rusqlite::Error) -> Error {
    match error.sqlite_error().map(|error| error.extended_code) {
        Some(rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY | rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE) => {
            Error::Corrupt("position retry exceeds saved frontier")
        }
        _ => error.into(),
    }
}

fn retried(connection: &Connection, entry: &Encoded<'_>, bytes: &[u8], saved: u64) -> Result<bool, Error> {
    let mut statement = connection.prepare_cached("SELECT kind,body,digest,retired_at FROM storage_data_entries WHERE position=?1")?;
    let mut rows = statement.query([position(entry.position)])?;
    let Some(row) = rows.next()? else {
        return Ok(false);
    };
    if row.get_ref(3)?.data_type() != rusqlite::types::Type::Null {
        return Err(Error::Data("position retry was logically pruned"));
    }
    let body = blob(row, 1)?;
    let digest = blob(row, 2)?;
    check_digest(body, digest)?;
    if digest != entry.digest.as_slice() {
        return Err(Error::Conflict("position retry has different bytes"));
    }
    if row.get::<_, String>(0)? != entry.kind || body != &bytes[entry.body.clone()] {
        return Err(Error::Corrupt("position retry content differs"));
    }
    if entry.position > saved {
        return Err(Error::Corrupt("position retry exceeds saved frontier"));
    }
    Ok(true)
}

fn read_object(connection: &Connection, at: u64, generation: u64, point: &ObjectPoint) -> Result<Option<ObjectEntry>, Error> {
    let schema = key(point.schema.as_str())?;
    let item = key(point.item.as_str())?;
    let exact = match &point.version {
        ObjectVersion::Latest => None,
        ObjectVersion::Exact(version) => Some(serde_json::to_string(version)?),
    };
    let mut statement = connection.prepare(
        "SELECT position,body,digest FROM storage_data_entries WHERE kind='object'
         AND schema_id=?1 AND item=?2 AND position<=?3
         AND (?4 IS NULL OR object_version=?4) AND (retired_at IS NULL OR retired_at>?5)
         ORDER BY position DESC LIMIT 1",
    )?;
    let mut rows = statement.query(params![schema, item, position(at), exact, position(generation)])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    let encoded = blob(row, 1)?;
    check_digest(encoded, blob(row, 2)?)?;
    let Entry::Object(object) = Entry::decode("object", encoded)? else {
        return Err(Error::Corrupt("object index collection differs"));
    };
    if object.value.schema.id != point.schema
        || object.value.item != point.item
        || position(object.position.sequence) != text(row, 0)?
        || matches!(&point.version, ObjectVersion::Exact(version) if *version != object.version)
    {
        return Err(Error::Corrupt("object index identity differs"));
    }
    Ok(Some(object))
}

fn scan_query(scan: &Scan) -> Result<(String, Vec<Value>), Error> {
    let mut arguments = vec![
        Value::Text(scan.kind.into()),
        Value::Text(position(scan.page.after)),
        Value::Text(position(scan.at)),
        Value::Text(position(scan.generation)),
    ];
    let mut clauses = vec![
        "entries.kind=?1".to_owned(),
        "entries.position>?2".to_owned(),
        "entries.position<=?3".to_owned(),
        "(entries.retired_at IS NULL OR entries.retired_at>?4)".to_owned(),
    ];
    if scan.kind == "file" {
        clauses.push("NOT EXISTS(SELECT 1 FROM storage_deleted_files WHERE storage_deleted_files.id=entries.item)".to_owned());
    }
    for (column, value) in &scan.filters {
        arguments.push(value.clone());
        clauses.push(format!("entries.{column}=?{}", arguments.len()));
    }
    if let Some((start, end)) = scan.source_window {
        arguments.push(Value::Blob(timestamp_key(start).to_vec()));
        clauses.push(format!("entries.source_time>=?{}", arguments.len()));
        arguments.push(Value::Blob(timestamp_key(end).to_vec()));
        clauses.push(format!("entries.source_time<?{}", arguments.len()));
    }
    if scan.latest_objects {
        // The Newest Revision Of Each Key At The Cut, Ordered By Its Position.
        clauses.push(
            "NOT EXISTS (SELECT 1 FROM storage_data_entries AS newer
                WHERE newer.kind='object' AND newer.schema_id=entries.schema_id
                AND newer.item=entries.item AND newer.position>entries.position
                AND newer.position<=?3 AND (newer.retired_at IS NULL OR newer.retired_at>?4))"
                .to_owned(),
        );
    }
    let rows = i64::try_from(scan.page.rows.get()).map_err(|_| Error::Data("page row limit exceeds SQLite range"))?;
    // Schema And Owner Must Both Narrow The Index Before Reading Row Bodies.
    // In particular, an absent stream must not scan another stream's payloads.
    let scoped = scan.filters.iter().any(|(column, _)| *column == "scope");
    let schema = scan.filters.iter().any(|(column, _)| *column == "schema_key");
    let stream = scan.filters.iter().any(|(column, _)| *column == "stream");
    let object_key =
        scan.filters.iter().any(|(column, _)| *column == "schema_id") && scan.filters.iter().any(|(column, _)| *column == "item");
    // SQLite Must Prove The Partial Index Predicate During Preparation.
    if scan.kind == "stream" {
        clauses.push("entries.kind='stream'".to_owned());
    }
    if scan.kind == "object" && object_key {
        clauses.push("entries.kind='object'".to_owned());
    }
    let index = match (scan.kind, scan.source_window.is_some(), schema, scoped) {
        ("object", _, _, _) if object_key => "storage_data_object",
        ("stream", false, false, true) if stream => "storage_data_stream_scoped_owner",
        ("stream", true, true, true) => "storage_data_stream_scoped_time",
        ("stream", true, true, false) => "storage_data_stream_time",
        (_, _, true, true) => "storage_data_schema_scoped",
        (_, _, true, false) => "storage_data_schema_position",
        (_, _, false, true) => "storage_data_scoped",
        (_, _, false, false) => "storage_data_kind_position",
    };
    let sql = format!(
        "SELECT entries.position,length(entries.body),entries.body,entries.digest FROM storage_data_entries AS entries
         INDEXED BY {index} WHERE {} ORDER BY entries.position ASC LIMIT {rows}",
        clauses.join(" AND ")
    );
    Ok((sql, arguments))
}

fn read_scan(connection: &Connection, scan: &Scan, metrics: &super::ReadMetrics) -> Result<Vec<Entry>, Error> {
    let (sql, arguments) = scan_query(scan)?;
    let mut statement = connection.prepare(&sql)?;
    let mut selected = statement.query(rusqlite::params_from_iter(arguments))?;
    let mut entries = Vec::new();
    let mut bytes = 0usize;
    while let Some(row) = selected.next()? {
        if entries.len() == entries.capacity() {
            let capacity = entries.capacity();
            let target = capacity.max(1).checked_mul(2).ok_or_else(entry::read_exhausted)?.min(scan.page.rows.get());
            entries.try_reserve_exact(target - entries.len()).map_err(std::io::Error::other)?;
        }
        let length = usize::try_from(row.get::<_, i64>(1)?).map_err(|_| Error::Corrupt("invalid entry body length"))?;
        if !matches!(scan.kind, "object" | "stream") && !entries.is_empty() && length > scan.page.bytes.get().saturating_sub(bytes) {
            break;
        }
        super::DATA_SCAN_ROWS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        metrics.data_scan_rows.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let body = blob(row, 2)?;
        if body.len() != length {
            return Err(Error::Corrupt("entry body length changed"));
        }
        check_digest(body, blob(row, 3)?)?;
        // Charge The Portable Encoding Harmos Measures; The First Entry Always Fits.
        let portable = Entry::portable_bytes(scan.kind, body)?;
        if !entries.is_empty() && portable > scan.page.bytes.get().saturating_sub(bytes) {
            break;
        }
        let entry = Entry::decode(scan.kind, body)?;
        if position(entry.position().sequence) != text(row, 0)? {
            return Err(Error::Corrupt("entry index position differs"));
        }
        bytes = bytes.saturating_add(portable);
        entries.push(entry);
    }
    Ok(entries)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroUsize;

    fn stream_rows(history: &StoreHistory, after: u64, count: u64) -> Vec<StoredRow> {
        let schema = SchemaKey {
            id: harmos::Identifier::new("fixture.samples").unwrap(),
            version: std::num::NonZeroU32::new(1).unwrap(),
            payload_encoding: PayloadEncoding::PostcardV1,
        };
        (after + 1..=after + count)
            .map(|sequence| StoredRow {
                stream: harmos::Identifier::new("samples").unwrap(),
                sequence,
                record: StoredRecord {
                    position: StorageCursor { history: history.clone(), sequence },
                    schema: schema.clone(),
                    scope: (sequence % 3 == 0).then(|| harmos::Identifier::new(format!("scope-{}", sequence % 2)).unwrap()),
                    item: harmos::Identifier::new("fixture").unwrap(),
                    source: None,
                    source_time: (sequence % 5 != 0).then(|| std::time::UNIX_EPOCH + std::time::Duration::from_micros(sequence * 7)),
                    payload: sequence.to_le_bytes().to_vec(),
                },
            })
            .collect()
    }

    fn entries(connection: &Connection) -> Vec<Vec<Value>> {
        let mut query = connection.prepare("SELECT * FROM storage_data_entries ORDER BY position").unwrap();
        let columns = query.column_count();
        query
            .query_map([], |row| (0..columns).map(|column| row.get::<_, Value>(column)).collect())
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    }

    fn consistent(connection: &Connection) -> bool {
        connection.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0)).unwrap() == "ok"
    }

    #[test]
    fn batched_inserts_store_the_rows_and_indexes_of_single_row_inserts() {
        let root = std::path::Path::new("");
        let mut batched = Connection::open_in_memory().unwrap();
        let head = open(&mut batched, None).unwrap();
        // Shared Metadata, Mixed Metadata And A Single-Row Remainder.
        let mut rows = stream_rows(&head.history, 0, 2 * INSERT_ROWS as u64 + 7);
        for row in &mut rows[..INSERT_ROWS] {
            row.record.scope = None;
        }
        let saved = write(&mut batched, root, StorageBatch::new(rows.clone()), &mut Default::default(), &mut Default::default()).unwrap();
        assert_eq!(saved.sequence, rows.len() as u64);

        // The Former Path Inserted Each Encoded Row With One Statement.
        let mut single = Connection::open_in_memory().unwrap();
        open(&mut single, None).unwrap();
        for row in &rows {
            let mut encoder = entry::Encoder::default();
            let entry = encoder.encode(Borrowed::from(row)).unwrap();
            single
                .execute(
                    "INSERT INTO storage_data_entries VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,NULL)",
                    params![
                        position(entry.position),
                        entry.kind,
                        entry.identity.as_str(),
                        entry.schema_id,
                        entry.schema_key.as_deref(),
                        entry.item,
                        entry.scope,
                        entry.stream,
                        entry.source_time.as_ref().map(|time| time.as_slice()),
                        entry.object_version,
                        &encoder.bytes[entry.body],
                        entry.digest.as_slice()
                    ],
                )
                .unwrap();
        }
        assert_eq!(entries(&batched), entries(&single));
        assert!(consistent(&batched) && consistent(&single));

        // A Batch Interrupted Before Its Commit Leaves Nothing Of Itself Visible.
        batched
            .execute_batch(&format!(
                "CREATE TRIGGER interrupt BEFORE INSERT ON storage_data_entries WHEN NEW.position='{}' BEGIN SELECT RAISE(ABORT,'interrupted'); END",
                position(rows.len() as u64 + INSERT_ROWS as u64 + 3)
            ))
            .unwrap();
        let later = stream_rows(&head.history, rows.len() as u64, 2 * INSERT_ROWS as u64);
        assert!(write(&mut batched, root, StorageBatch::new(later), &mut Default::default(), &mut Default::default()).is_err());
        assert!(batched.is_autocommit());
        assert_eq!(entries(&batched), entries(&single));
        assert_eq!(saved_position(&batched), rows.len() as u64);
        assert!(consistent(&batched));
    }

    #[test]
    fn rows_above_the_saved_frontier_are_refused_as_before() {
        let root = std::path::Path::new("");
        let mut connection = Connection::open_in_memory().unwrap();
        let head = open(&mut connection, None).unwrap();
        let rows = stream_rows(&head.history, 0, 2);
        let entry = entry::Encoder::default().encode(Borrowed::from(&rows[0])).unwrap();
        connection
            .execute(
                "INSERT INTO storage_data_entries(position,kind,identity,item,body,digest) VALUES (?1,'stream',?2,'fixture',x'00',?3)",
                params![position(entry.position), entry.identity.as_str(), entry.digest.as_slice()],
            )
            .unwrap();
        let refused = write(&mut connection, root, StorageBatch::new(rows), &mut Default::default(), &mut Default::default());
        assert!(matches!(refused, Err(Error::Corrupt("position retry exceeds saved frontier"))));
    }

    fn saved_position(connection: &Connection) -> u64 {
        saved(connection).unwrap().sequence
    }

    #[test]
    fn selected_object_seeks_past_unrelated_history() {
        for count in [4096, 65536] {
            let connection = Connection::open_in_memory().unwrap();
            connection.execute_batch(SCHEMA).unwrap();
            connection.execute(
                "WITH RECURSIVE rows(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM rows WHERE n<?1)
                 INSERT INTO storage_data_entries(position,kind,identity,schema_id,schema_key,item,body,digest)
                 SELECT printf('%016x',n),'object',printf('object-%d',n),'execution','v1',printf('owner-%d',n),x'00',zeroblob(32) FROM rows",
                [count],
            ).unwrap();
            let page = ScanPage {
                after: 0,
                rows: NonZeroUsize::new(256).unwrap(),
                bytes: NonZeroUsize::new(1048576).unwrap(),
            };
            let mut scan = Scan::new("object", count as u64, 0, page)
                .filter("schema_key", Some("v1".into()))
                .filter("schema_id", Some("execution".into()))
                .filter("item", Some(format!("owner-{count}")));
            scan.latest_objects = true;
            let (sql, arguments) = scan_query(&scan).unwrap();
            let mut query = connection.prepare(&sql).unwrap();
            let positions = query
                .query_map(rusqlite::params_from_iter(arguments), |row| row.get::<_, String>(0))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert_eq!(positions, [position(count as u64)]);
            assert_eq!(query.get_status(rusqlite::StatementStatus::FullscanStep), 0);
            let work = query.get_status(rusqlite::StatementStatus::VmStep);
            assert!(work < 512, "selected object query took {work} VM steps with {count} unrelated objects");
        }
    }

    #[test]
    fn retiring_metadata_does_not_scan_evidence_children() {
        for count in [4096, 16384] {
            let connection = Connection::open_in_memory().unwrap();
            connection.execute_batch("PRAGMA foreign_keys=ON").unwrap();
            for schema in [SCHEMA, telemetry::SCHEMA, records::SCHEMA, super::super::history::SCHEMA] {
                connection.execute_batch(schema).unwrap();
            }
            connection
                .execute(
                    "WITH RECURSIVE rows(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM rows WHERE n<?1)
                 INSERT INTO storage_data_entries(position,kind,identity,item,body,digest)
                 SELECT printf('%016x',n),'stream',printf('sample-%d',n),printf('sample-%d',n),x'00',zeroblob(32) FROM rows",
                    [count],
                )
                .unwrap();
            connection
                .execute_batch(
                    "INSERT INTO storage_record_captures
                 SELECT 'execution',position,identity,position,'{}',NULL,position,NULL FROM storage_data_entries;
                 INSERT INTO storage_data_entries(position,kind,identity,item,body,digest)
                 VALUES ('ffffffffffffffff','object','metadata','object',x'00',zeroblob(32));
                 INSERT INTO query_execution_outputs
                 SELECT identity,position,'logs/output.log',x'00' FROM storage_data_entries WHERE kind='stream';",
                )
                .unwrap();
            // Foreign-Key Checks Must Seek The Parent Position Even When It Has No Children.
            for position in ["ffffffffffffffff", "0000000000000001"] {
                let mut delete = connection.prepare("DELETE FROM storage_data_entries WHERE position=?1").unwrap();
                assert_eq!(delete.execute([position]).unwrap(), 1);
                assert_eq!(delete.get_status(rusqlite::StatementStatus::FullscanStep), 0);
                let steps = delete.get_status(rusqlite::StatementStatus::VmStep);
                assert!(steps < 512, "retirement took {steps} VM steps with {count} evidence children");
            }
            let remaining: i64 = connection.query_row("SELECT count(*) FROM storage_record_captures", [], |row| row.get(0)).unwrap();
            assert_eq!(remaining, count - 1, "only the deleted evidence row cascades");
            let remaining: i64 = connection.query_row("SELECT count(*) FROM query_execution_outputs", [], |row| row.get(0)).unwrap();
            assert_eq!(remaining, count - 1);
            assert!(!connection.prepare("PRAGMA foreign_key_check").unwrap().exists([]).unwrap());
        }
    }

    #[tokio::test]
    async fn absent_schema_does_not_scan_other_streams() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(SCHEMA).unwrap();
        connection
            .execute_batch(
                "WITH RECURSIVE rows(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM rows WHERE n<4096)
             INSERT INTO storage_data_entries(position,kind,identity,schema_key,item,scope,body,digest)
             SELECT printf('%016x',n),'stream',printf('row-%d',n),'populated','item','execution',x'00',zeroblob(32) FROM rows;",
            )
            .unwrap();

        for scope in [None, Some("execution".to_owned())] {
            let page = ScanPage {
                after: 0,
                rows: NonZeroUsize::new(1024).unwrap(),
                bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
            };
            let scan = Scan::new("stream", 4096, 0, page).filter("schema_key", Some("absent".to_owned())).filter("scope", scope);
            let (sql, arguments) = scan_query(&scan).unwrap();
            let mut statement = connection.prepare(&sql).unwrap();
            assert!(!statement.exists(rusqlite::params_from_iter(arguments)).unwrap());
            let steps = statement.get_status(rusqlite::StatementStatus::VmStep);
            assert!(steps < 100, "an absent schema performed {steps} VM steps over unrelated rows");
            assert!(read_scan(&connection, &scan, &super::super::ReadMetrics::default()).unwrap().is_empty());
        }
    }
}
