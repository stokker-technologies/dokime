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
mod entry;
mod files;
mod mutations;
mod production;
mod records;
mod retention;
pub(crate) mod telemetry;

/* === Re-Exports === */

pub use backend::Backend;
pub(crate) use entry::Entry;
pub(super) use files::VerifiedPrefix;

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
CREATE INDEX IF NOT EXISTS storage_data_object ON storage_data_entries(kind, schema_id, item, position);
CREATE INDEX IF NOT EXISTS storage_data_live_object_position_covering
    ON storage_data_entries(schema_id, position, item, kind, retired_at)
    WHERE kind='object' AND retired_at IS NULL;
CREATE INDEX IF NOT EXISTS storage_data_live_object_key_covering
    ON storage_data_entries(schema_id, item, position, kind, retired_at)
    WHERE kind='object' AND retired_at IS NULL;
CREATE INDEX IF NOT EXISTS storage_data_retired_object_position
    ON storage_data_entries(position, retired_at)
    WHERE kind='object' AND retired_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS storage_data_scope ON storage_data_entries(kind, scope, position);
CREATE INDEX IF NOT EXISTS storage_data_schema_scope ON storage_data_entries(kind, schema_key, scope, position);
CREATE INDEX IF NOT EXISTS storage_data_schema_position ON storage_data_entries(kind, schema_key, position);
CREATE INDEX IF NOT EXISTS storage_data_stream_owner ON storage_data_entries(scope,stream,position) WHERE kind='stream';
CREATE INDEX IF NOT EXISTS storage_data_stream_time ON storage_data_entries(schema_key, source_time, position) WHERE kind='stream';
CREATE INDEX IF NOT EXISTS storage_data_stream_scope_time ON storage_data_entries(schema_key, scope, source_time, position)
    WHERE kind='stream';
";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct SavedData {
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
        self.call(move |connection| Ok(saved(connection))).await?
    }

    async fn data_write<T>(&self, root: std::path::PathBuf, batch: StorageBatch<T>) -> Result<SavedData, Error>
    where
        T: Send + Sync + 'static,
        for<'a> Borrowed<'a>: From<&'a T>,
    {
        let owner = self.0.clone();
        self.call(move |connection| {
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
            "SELECT count(*)=7 FROM sqlite_master WHERE type='table' AND name IN
                ('files','streams','storage_record_visibility','storage_record_captures',
                 'storage_numeric_summaries','storage_numeric_ranks','storage_report_previews')",
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
            for query in [
                "SELECT id,kind,path,size_bytes,sha256,media_type FROM files LIMIT 0",
                "SELECT file_id,execution,relative_path FROM artifacts LIMIT 0",
                "SELECT file_id,contract,durable_end,hash_state,finalized,sealed FROM streams LIMIT 0",
                "SELECT file_id,offset,metadata_bytes,body_bytes,sha256 FROM stream_batches LIMIT 0",
                "SELECT file_id,batch,scope,first_at,last_at FROM stream_batch_scopes LIMIT 0",
                "SELECT execution,cycle,file_id,batch,first_row,rows,scope FROM stream_captures LIMIT 0",
                "SELECT cycle,sequence,epoch FROM storage_record_captures LIMIT 0",
                "SELECT sequence,epoch FROM storage_record_visibility LIMIT 0",
            ] {
                connection
                    .prepare(query)
                    .map_err(|_| Error::Incompatible("local.sqlite requires a fresh storage root for Arrow telemetry".into()))?;
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

pub(super) fn saved(connection: &Connection) -> Result<SavedData, Error> {
    let mut statement = connection.prepare("SELECT format,history,saved,retention_generation FROM storage_data_head WHERE singleton=1")?;
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
    let transaction = connection.transaction()?;
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
    super::commit(transaction)?;
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
    let transaction = connection.transaction()?;
    transaction.execute_batch(HEAD_SCHEMA)?;
    transaction.execute_batch(SCHEMA)?;
    transaction.execute_batch(production::SCHEMA)?;
    transaction.execute_batch(telemetry::SCHEMA)?;
    transaction.execute_batch(telemetry::aggregates::SCHEMA)?;
    transaction.execute_batch(telemetry::previews::SCHEMA)?;
    transaction.execute_batch(crate::plot::preview::index::SCHEMA)?;
    transaction.execute_batch(records::SCHEMA)?;
    transaction.execute_batch(mutations::SCHEMA)?;
    initialize_head(&transaction, expected.as_ref())?;
    let head = saved(&transaction)?;
    if let Some(expected) = expected {
        expected.check(&head.history)?;
    }
    super::commit(transaction)?;
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
    // Encode And Validate The Bounded Batch Once Before Any File Confirmation.
    let mut encoded = Vec::new();
    encoded.try_reserve_exact(values.len()).map_err(std::io::Error::other)?;
    for value in values {
        let value = Borrowed::from(value);
        history.check(&value.position().history)?;
        encoded.push(Encoded::new(value)?);
    }

    let transaction = connection.transaction()?;
    let mut head = saved(&transaction)?;
    head.history.check(history)?;
    let mut plots = crate::plot::preview::index::Batch::default();
    for (value, entry) in values.iter().zip(encoded) {
        let value = Borrowed::from(value);
        // Exact Retries Of Saved Positions Must Repeat The Same Bytes.
        let retry = retried(&transaction, &entry, head.sequence)?;
        if !retry {
            if head.sequence.checked_add(1) != Some(entry.position) {
                return Err(Error::Data("data write has a gap or pruned retry"));
            }
            let mut identity = transaction.prepare_cached("SELECT EXISTS(SELECT 1 FROM storage_data_entries WHERE identity=?1)")?;
            if identity.query_row([&entry.identity], |row| row.get::<_, bool>(0))? {
                return Err(Error::Conflict("data identity already belongs to another position"));
            }
        }
        if let Borrowed::File(reference) = value {
            files::confirm(&transaction, root, reference, retained.get_mut(&reference.metadata.id), prefix)?;
        }
        if retry {
            continue;
        }
        let mut insert =
            transaction.prepare_cached("INSERT INTO storage_data_entries VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,NULL)")?;
        insert.execute(params![
            position(entry.position),
            entry.kind,
            entry.identity,
            entry.schema_id,
            entry.schema_key,
            entry.item,
            entry.scope,
            entry.stream,
            entry.source_time.as_ref().map(|time| time.as_slice()),
            entry.object_version,
            entry.body,
            entry.digest.as_slice()
        ])?;
        // Release The Bound Row Before Decoding Its Secondary Index Metadata.
        drop(insert);
        if let Borrowed::File(reference) = value {
            telemetry::catalog_file(&transaction, reference)?;
        }
        if let Borrowed::Record(row) = value {
            records::index(&transaction, row)?;
            plots.record(&transaction, row)?;
        }
        head.sequence = entry.position;
    }
    plots.save(&transaction)?;
    transaction.execute("UPDATE storage_data_head SET saved=?1 WHERE singleton=1", [position(head.sequence)])?;
    super::commit(transaction)?;
    super::DATA_BATCHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    for value in values {
        if let Borrowed::File(reference) = Borrowed::from(value) {
            retained.remove(&reference.metadata.id);
        }
    }
    Ok(head)
}

fn retried(connection: &Connection, entry: &Encoded<'_>, saved: u64) -> Result<bool, Error> {
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
    if row.get::<_, String>(0)? != entry.kind || body != entry.body.as_slice() {
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
    if scan.kind == "stream" {
        // SQLite Must Prove The Partial Index Predicate During Preparation.
        clauses.push("entries.kind='stream'".to_owned());
    }
    let index = match (scan.kind, scan.source_window.is_some(), schema, scoped) {
        ("stream", false, false, true) if stream => "storage_data_stream_owner",
        ("stream", true, true, true) => "storage_data_stream_scope_time",
        ("stream", true, true, false) => "storage_data_stream_time",
        (_, _, true, true) => "storage_data_schema_scope",
        (_, _, true, false) => "storage_data_schema_position",
        (_, _, false, true) => "storage_data_scope",
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

    #[test]
    fn retiring_metadata_does_not_scan_evidence_children() {
        for count in [4096, 16384] {
            let connection = Connection::open_in_memory().unwrap();
            connection.execute_batch("PRAGMA foreign_keys=ON").unwrap();
            for schema in [SCHEMA, telemetry::SCHEMA, records::SCHEMA] {
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
                 VALUES ('ffffffffffffffff','object','metadata','object',x'00',zeroblob(32));",
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
            for table in ["storage_record_captures"] {
                let remaining: i64 = connection.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row.get(0)).unwrap();
                assert_eq!(remaining, count - 1, "only the deleted evidence row cascades");
            }
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
