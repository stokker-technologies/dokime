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
mod retention;

/* === Re-Exports === */
pub use backend::Backend;
pub(crate) use entry::Entry;

/* === Imports === */
// Standard Library Imports
// Third Party Imports
use harmos::storage::*;
use rusqlite::{Connection, params, types::Value};
use sha2::{Digest, Sha256};
// Local Imports
use super::{Error, State};
use entry::{Encoded, key, timestamp_key};

/* === Constants === */
pub(super) const FORMAT: &str = "dokime.sqlite.storage.v1";
pub(super) const HISTORY_BYTES: usize = 4096;
const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_data_head (
    singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
    format TEXT NOT NULL, history BLOB NOT NULL,
    saved TEXT NOT NULL CHECK(length(saved) = 16),
    retention_generation TEXT NOT NULL CHECK(length(retention_generation) = 16)
);
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
CREATE INDEX IF NOT EXISTS storage_data_live_object_position
    ON storage_data_entries(schema_id, position, item)
    WHERE kind='object' AND retired_at IS NULL;
CREATE INDEX IF NOT EXISTS storage_data_live_object_key
    ON storage_data_entries(schema_id, item, position)
    WHERE kind='object' AND retired_at IS NULL;
CREATE INDEX IF NOT EXISTS storage_data_retired_object_position
    ON storage_data_entries(position, retired_at)
    WHERE kind='object' AND retired_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS storage_data_scope ON storage_data_entries(kind, scope, position);
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

/// A scan of one entry kind within a pinned cut, ascending by position.
pub(super) struct Scan {
    pub kind: &'static str,
    pub at: u64,
    pub generation: u64,
    pub filters: Vec<(&'static str, Value)>,
    pub source_window: Option<(std::time::SystemTime, std::time::SystemTime)>,
    pub latest_objects: bool,
    pub page: ScanPage,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl State {
    pub(crate) async fn data_open(
        &self,
        expected: Option<StoreHistory>,
    ) -> Result<SavedData, Error> {
        self.call(move |connection| Ok(open(connection, expected)))
            .await?
    }

    pub(crate) async fn data_saved(&self) -> Result<SavedData, Error> {
        self.call(|connection| Ok(saved(connection))).await?
    }

    pub(crate) async fn data_write(
        &self,
        root: &std::path::Path,
        batch: StorageBatch<'_>,
    ) -> Result<SavedData, Error> {
        let root = root.to_path_buf();
        let mut prepared = Vec::with_capacity(batch.len());
        for value in batch.entries() {
            batch.first().history.check(&value.position().history)?;
            let length = Encoded::encoded_bytes(value, usize::MAX)?;
            prepared.push(Encoded::new(value, length)?);
        }
        let history = batch.first().history.clone();
        self.call(move |connection| Ok(write(connection, &root, &history, &prepared)))
            .await?
    }

    pub(crate) async fn data_object(
        &self,
        at: u64,
        generation: u64,
        point: ObjectPoint,
    ) -> Result<Option<ObjectEntry>, Error> {
        self.read(move |connection| Ok(read_object(connection, at, generation, &point)))
            .await?
    }

    pub(super) async fn data_scan(&self, scan: Scan) -> Result<Vec<Entry>, Error> {
        self.read(move |connection| Ok(read_scan(connection, &scan)))
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
            latest_objects: false,
            page,
        }
    }

    pub(super) fn filter(
        mut self,
        column: &'static str,
        value: Option<String>,
    ) -> Self {
        if let Some(value) = value {
            self.filters.push((column, Value::Text(value)));
        }
        self
    }
}

/* === Public Functions === */

/* === Private Functions === */
pub(super) fn check_format(connection: &Connection) -> Result<(), Error> {
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='storage_data_head')",
        [],
        |row| row.get(0),
    )?;
    if exists {
        let format: Option<String> = connection
            .query_row(
                "SELECT format FROM storage_data_head WHERE singleton=1",
                [],
                |row| row.get(0),
            )
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

fn open(connection: &mut Connection, expected: Option<StoreHistory>) -> Result<SavedData, Error> {
    let transaction = connection.transaction()?;
    transaction.execute_batch(SCHEMA)?;
    transaction.execute_batch(production::SCHEMA)?;
    transaction.execute_batch(mutations::SCHEMA)?;
    let exists = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM storage_data_head)",
        [],
        |row| row.get::<_, bool>(0),
    )?;
    if !exists {
        let initialized: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM storage_data_entries)",
            [],
            |row| row.get(0),
        )?;
        if initialized {
            return Err(Error::Corrupt("initialized data owner has no saved head"));
        }
        let history = expected.clone().unwrap_or_else(|| StoreHistory {
            store: harmos::Identifier::new(uuid::Uuid::now_v7().to_string())
                .expect("UUID identifier"),
            epoch: harmos::Identifier::new(uuid::Uuid::now_v7().to_string())
                .expect("UUID identifier"),
        });
        let encoded = serde_json::to_vec(&history)?;
        if encoded.len() > HISTORY_BYTES {
            return Err(Error::Data("history identity exceeds its bound"));
        }
        transaction.execute(
            "INSERT INTO storage_data_head VALUES (1,?1,?2,?3,?3)",
            params![FORMAT, encoded, position(0)],
        )?;
    }
    let head = saved(&transaction)?;
    if let Some(expected) = expected {
        expected.check(&head.history)?;
    }
    transaction.commit()?;
    Ok(head)
}

pub(super) fn saved(connection: &Connection) -> Result<SavedData, Error> {
    let mut statement = connection.prepare(
        "SELECT format,history,saved,retention_generation FROM storage_data_head WHERE singleton=1",
    )?;
    let mut rows = statement.query([])?;
    let row = rows
        .next()?
        .ok_or(Error::Corrupt("missing saved data head"))?;
    if row.get::<_, String>(0)? != FORMAT {
        return Err(Error::Data("unsupported SQLite data format"));
    }
    let history = blob(row, 1)?;
    if history.len() > HISTORY_BYTES {
        return Err(Error::Corrupt("stored history exceeds its bound"));
    }
    Ok(SavedData {
        history: serde_json::from_slice(history)?,
        sequence: sequence(&row.get::<_, String>(2)?)?,
        retention_generation: sequence(&row.get::<_, String>(3)?)?,
    })
}

fn write(
    connection: &mut Connection,
    root: &std::path::Path,
    history: &StoreHistory,
    entries: &[Encoded],
) -> Result<SavedData, Error> {
    let transaction = connection.transaction()?;
    let mut head = saved(&transaction)?;
    head.history.check(history)?;
    for entry in entries {
        // Exact Retries Of Saved Positions Must Repeat The Same Bytes.
        if retried(&transaction, root, entry, head.sequence)? {
            continue;
        }
        if head.sequence.checked_add(1) != Some(entry.position) {
            return Err(Error::Data("data write has a gap or pruned retry"));
        }
        if transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM storage_data_entries WHERE identity=?1)",
            [&entry.identity],
            |row| row.get::<_, bool>(0),
        )? {
            return Err(Error::Conflict(
                "data identity already belongs to another position",
            ));
        }
        if entry.kind == "file" {
            let reference: FileReference = serde_json::from_slice(&entry.body)?;
            files::confirm(&transaction, root, &reference)?;
        }
        transaction.execute(
            "INSERT INTO storage_data_entries VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,NULL)",
            params![
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
            ],
        )?;
        head.sequence = entry.position;
    }
    transaction.execute(
        "UPDATE storage_data_head SET saved=?1 WHERE singleton=1",
        [position(head.sequence)],
    )?;
    transaction.commit()?;
    Ok(head)
}

fn retried(
    connection: &Connection,
    root: &std::path::Path,
    entry: &Encoded,
    saved: u64,
) -> Result<bool, Error> {
    let mut statement = connection
        .prepare("SELECT kind,body,digest,retired_at FROM storage_data_entries WHERE position=?1")?;
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
    if entry.kind == "file" {
        let reference: FileReference = serde_json::from_slice(&entry.body)?;
        files::confirm(connection, root, &reference)?;
    }
    Ok(true)
}

fn read_object(
    connection: &Connection,
    at: u64,
    generation: u64,
    point: &ObjectPoint,
) -> Result<Option<ObjectEntry>, Error> {
    let schema = key(point.schema.as_str())?;
    let item = key(point.item.as_str())?;
    let exact = match &point.version {
        ObjectVersion::Latest => None,
        ObjectVersion::Exact(version) => Some(serde_json::to_string(version)?),
    };
    let mut statement = connection.prepare(
        "SELECT position, body, digest FROM storage_data_entries WHERE kind='object'
         AND schema_id=?1 AND item=?2 AND position<=?3
         AND (?4 IS NULL OR object_version=?4) AND (retired_at IS NULL OR retired_at>?5)
         ORDER BY position DESC LIMIT 1",
    )?;
    let mut rows = statement.query(params![
        schema,
        item,
        position(at),
        exact,
        position(generation)
    ])?;
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
        || position(object.position.sequence) != row.get::<_, String>(0)?
        || matches!(&point.version, ObjectVersion::Exact(version) if *version != object.version)
    {
        return Err(Error::Corrupt("object index identity differs"));
    }
    Ok(Some(object))
}

fn read_scan(connection: &Connection, scan: &Scan) -> Result<Vec<Entry>, Error> {
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
        clauses.push(
            "NOT EXISTS(SELECT 1 FROM storage_deleted_files WHERE storage_deleted_files.id=entries.item)"
                .to_owned(),
        );
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
    let rows = i64::try_from(scan.page.rows.get())
        .map_err(|_| Error::Data("page row limit exceeds SQLite range"))?;
    // A Scoped Selection Walks Only Its Owner's Entries In Position Order.
    let index = if scan.filters.iter().any(|(column, _)| *column == "scope") {
        "storage_data_scope"
    } else {
        "storage_data_kind_position"
    };
    let sql = format!(
        "SELECT entries.position,entries.body,entries.digest FROM storage_data_entries AS entries
         INDEXED BY {index} WHERE {} ORDER BY entries.position ASC LIMIT {rows}",
        clauses.join(" AND ")
    );
    let mut statement = connection.prepare(&sql)?;
    let mut selected = statement.query(rusqlite::params_from_iter(arguments))?;
    let mut entries = Vec::new();
    let mut bytes = 0usize;
    while let Some(row) = selected.next()? {
        let body = blob(row, 1)?;
        check_digest(body, blob(row, 2)?)?;
        // Charge The Portable Encoding Harmos Measures; The First Entry Always Fits.
        let portable = Entry::portable_bytes(scan.kind, body)?;
        if !entries.is_empty() && portable > scan.page.bytes.get().saturating_sub(bytes) {
            break;
        }
        let entry = Entry::decode(scan.kind, body)?;
        if position(entry.position().sequence) != row.get::<_, String>(0)? {
            return Err(Error::Corrupt("entry index position differs"));
        }
        bytes = bytes.saturating_add(portable);
        entries.push(entry);
    }
    Ok(entries)
}

pub(super) fn blob<'a>(row: &'a rusqlite::Row<'_>, column: usize) -> Result<&'a [u8], Error> {
    row.get_ref(column)?.as_blob().map_err(|source| {
        Error::Sqlite(rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Blob,
            Box::new(source),
        ))
    })
}

pub(super) fn check_digest(body: &[u8], digest: &[u8]) -> Result<(), Error> {
    if Sha256::digest(body).as_slice() != digest {
        return Err(Error::Corrupt("stored entry digest differs"));
    }
    Ok(())
}

pub(super) fn position(sequence: u64) -> String {
    format!("{sequence:016x}")
}

pub(super) fn sequence(value: &str) -> Result<u64, Error> {
    if value.len() != 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(Error::Corrupt("invalid saved sequence"));
    }
    u64::from_str_radix(value, 16).map_err(|_| Error::Corrupt("invalid saved sequence"))
}

/* === Tests === */
