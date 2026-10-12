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
mod bootstrap;
mod bootstrap_files;
mod browse;
mod entry;
mod files;
mod index;
mod inventory;
mod mutations;
mod production;
pub(super) mod recovery;
mod retained;
mod retention;
mod scan;

/* === Re-Exports === */
pub use backend::{Backend, Limits, Recovery};
pub(crate) use entry::Entry;
pub(super) use index::IndexSchema;
pub use recovery::RecoveryLimits;

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;
// Third Party Imports
use harmos::storage::*;
use rusqlite::{Connection, params, types::Value};
use sha2::{Digest, Sha256};
// Local Imports
use super::{Error, State};
use entry::{Encoded, key, timestamp_key};

/* === Constants === */
const FORMAT: &str = "dokime.sqlite.storage.v1";
pub(super) const HISTORY_BYTES: usize = 4096;
const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_data_head (
    singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
    format TEXT NOT NULL, history BLOB NOT NULL,
    saved TEXT NOT NULL CHECK(length(saved) = 16),
    lost_objects_through TEXT NOT NULL CHECK(length(lost_objects_through) = 16),
    retention_generation TEXT NOT NULL CHECK(length(retention_generation) = 16)
);
CREATE TABLE IF NOT EXISTS storage_data_entries (
    position TEXT PRIMARY KEY CHECK(length(position) = 16),
    kind TEXT NOT NULL CHECK(kind IN ('schema','object','record','stream','file')),
    identity TEXT NOT NULL UNIQUE,
    schema_id TEXT, schema_key TEXT, item TEXT NOT NULL,
    scope TEXT, stream TEXT, source_time BLOB,
    object_version TEXT, body BLOB NOT NULL,
    digest BLOB NOT NULL CHECK(length(digest) = 32), retired_at TEXT, object_fields BLOB
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_object_scalars (
    position TEXT NOT NULL CHECK(length(position)=16),
    tag INTEGER NOT NULL CHECK(tag>0), kind TEXT NOT NULL,
    value BLOB,
    PRIMARY KEY(position, tag)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS storage_object_scalar_value
    ON storage_object_scalars(tag, kind, value, position);
CREATE TABLE IF NOT EXISTS storage_bootstrap (
    singleton INTEGER PRIMARY KEY CHECK(singleton = 1), metadata BLOB NOT NULL
);
CREATE TABLE IF NOT EXISTS storage_bootstrap_entries (
    position TEXT PRIMARY KEY CHECK(length(position)=16),
    kind TEXT NOT NULL CHECK(kind IN ('schema','object','record','stream','file')),
    body BLOB NOT NULL, digest BLOB NOT NULL CHECK(length(digest)=32)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS storage_bootstrap_kind_position
    ON storage_bootstrap_entries(kind, position);
CREATE TABLE IF NOT EXISTS storage_bootstrap_schemas (
    schema_key TEXT NOT NULL,
    position TEXT NOT NULL UNIQUE CHECK(length(position)=16),
    PRIMARY KEY(schema_key, position)
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_bootstrap_abort (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1), identity BLOB NOT NULL
);
CREATE TABLE IF NOT EXISTS storage_bootstrap_files (
    id TEXT PRIMARY KEY, owner BLOB NOT NULL, metadata BLOB NOT NULL,
    finish_intent BLOB, finished BLOB
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_bootstrap_installs (
    id TEXT PRIMARY KEY, reference BLOB NOT NULL,
    created INTEGER NOT NULL CHECK(created IN (0,1))
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_bootstrap_completed (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1), receipt BLOB NOT NULL,
    digest BLOB NOT NULL CHECK(length(digest)=32)
);
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
    pub lost_objects_through: u64,
    pub retention_generation: u64,
}

pub(super) struct PageRead {
    pub cut: SavedData,
    pub selection: ReadSelection,
    pub after: Option<u64>,
    pub limits: ReadLimits,
    pub entry_bytes: NonZeroUsize,
}

pub(crate) struct Page {
    pub entries: Vec<Entry>,
    pub has_more: bool,
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
        entry_bytes: NonZeroUsize,
        batch_bytes: NonZeroUsize,
    ) -> Result<SavedData, Error> {
        if batch.encoded_bytes() > batch_bytes.get() {
            return Err(Error::Data("SQLite batch exceeds its encoded bound"));
        }
        let planning = batch.entries().try_fold(0usize, |peak, value| {
            entry::encoding_memory_bytes(value).map(|bytes| peak.max(bytes))
        })?;
        let _planning = self.reserve_memory(planning)?;
        let mut allocation = batch
            .len()
            .checked_mul(std::mem::size_of::<Encoded>())
            .and_then(|bytes| bytes.checked_add(2 * HISTORY_BYTES))
            .and_then(|bytes| {
                root.as_os_str()
                    .len()
                    .checked_mul(4)
                    .and_then(|root_bytes| bytes.checked_add(root_bytes))
            })
            .ok_or(Error::Data("batch allocation size overflow"))?;
        let length_bytes = batch
            .len()
            .checked_mul(std::mem::size_of::<usize>())
            .ok_or(Error::Data("batch length metadata overflows"))?;
        let _lengths = self.reserve_memory(length_bytes)?;
        let mut lengths = Vec::with_capacity(batch.len());
        for value in batch.entries() {
            batch.first().history.check(&value.position().history)?;
            let (bytes, length) = Encoded::allocation_bytes(value, entry_bytes.get())?;
            allocation = allocation
                .checked_add(bytes)
                .ok_or(Error::Data("batch allocation size overflow"))?;
            lengths.push(length);
        }
        if batch
            .entries()
            .any(|entry| matches!(entry, StorageEntry::File(_)))
        {
            allocation = allocation
                .checked_add(8 * 16 * 1024 + 4 * HISTORY_BYTES + 64 * 1024)
                .ok_or(Error::Data("file publication working allocation overflow"))?;
        }
        let reservation = self.reserve_memory(allocation)?;
        let root = root.to_path_buf();
        let mut prepared = Vec::with_capacity(batch.len());
        let mut bytes = 0usize;
        for (value, length) in batch.entries().zip(lengths) {
            batch.first().history.check(&value.position().history)?;
            let encoded = Encoded::new(value, length)?;
            bytes = bytes
                .checked_add(encoded.body.len())
                .filter(|bytes| *bytes <= batch_bytes.get())
                .ok_or(Error::Data("SQLite batch framing exceeds its byte bound"))?;
            prepared.push(encoded);
        }
        let history = batch.first().history.clone();
        let memory = self.0.memory.clone();
        let owner = self.0.clone();
        self.call(move |connection| {
            let _reservation = reservation;
            let mut schema = owner.index_schema.lock().unwrap();
            let result = write(
                connection,
                &root,
                &history,
                &prepared,
                &memory,
                entry_bytes,
                &mut schema,
            );
            if result.is_err() {
                *schema = None;
            }
            Ok(result)
        })
        .await?
    }

    pub(crate) async fn data_object(
        &self,
        through: StorageCursor,
        generation: u64,
        query: ObjectPoint,
        maximum: NonZeroUsize,
        execution: Option<std::sync::Arc<super::query::Budget>>,
    ) -> Result<Option<ObjectEntry>, Error> {
        let memory = self.clone();
        self.call_read(execution,move |connection| Ok((|| {
        let schema = key(query.schema.as_str())?;
        let item = key(query.item.as_str())?;
        let exact = match &query.version {
            ObjectVersion::Latest => None,
            ObjectVersion::Exact(version) => Some(serde_json::to_string(version)?),
        };
            let head = saved(connection)?;
            head.history.check(&through.history)?;
            if through.sequence > head.sequence { return Err(Error::Data("object read exceeds saved frontier")); }
            let mut statement = connection.prepare(
                "SELECT position, body, digest FROM storage_data_entries WHERE kind='object'
                 AND schema_id=?1 AND item=?2 AND position<=?3
                 AND (?4 IS NULL OR object_version=?4) AND (retired_at IS NULL OR retired_at>?5) ORDER BY position DESC LIMIT 1")?;
            let mut rows = statement.query(params![schema, item, position(through.sequence), exact, position(generation)])?;
            let Some(row) = rows.next()? else { return Ok(None); };
            let encoded = blob(row, 1)?;
            if encoded.len() > maximum.get() { return Err(Error::Data("object exceeds read byte bound")); }
            check_digest(encoded, blob(row, 2)?)?;
            let _decoded = memory.0.memory.reserve_owned(
                Entry::decoding_bytes("object", encoded.len())?, usize::MAX
            )?;
            let entry = Entry::decode("object", encoded)?;
            let Entry::Object(object) = entry else { return Err(Error::Corrupt("object index collection differs")); };
            if object.value.schema.id != query.schema || object.value.item != query.item
                || position(object.position.sequence) != row.get::<_, String>(0)?
                || matches!(&query.version, ObjectVersion::Exact(version) if *version != object.version)
            { return Err(Error::Corrupt("object index identity differs")); }
            object.position.history.check(&through.history)?;
            Ok(Some(object))
        })())).await?
    }

    pub(super) async fn data_page<T: Send + 'static>(
        &self,
        read: PageRead,
        execution: Option<std::sync::Arc<super::query::Budget>>,
        finish: impl FnOnce(Page) -> Result<T, Error> + Send + 'static,
    ) -> Result<T, Error> {
        let PageRead {
            cut,
            selection,
            after,
            limits,
            entry_bytes,
        } = read;
        // Planning Executes Under The Same Active Budget
        // The physical owner excludes queue waiting while charging parameter
        // construction, statement preparation, stepping and row construction.
        let memory = self.clone();
        self.call_read(execution, move |connection| {
            Ok((|| {
        let through = StorageCursor {
            history: cut.history.clone(),
            sequence: cut.sequence,
        };
        let mut arguments = vec![
            Value::Text(kind(selection.kind).into()),
            Value::Text(position(selection.after)),
            Value::Text(position(
                selection
                    .through
                    .unwrap_or(through.sequence)
                    .min(through.sequence),
            )),
            Value::Text(position(cut.retention_generation)),
        ];
        let mut clauses = vec![
            "NOT EXISTS(SELECT 1 FROM storage_deleted_files WHERE entries.kind='file' AND storage_deleted_files.id=entries.item)".to_owned(),
            "entries.kind=?1".to_owned(),
            "entries.position>?2".into(),
            "entries.position<=?3".into(),
            "(entries.retired_at IS NULL OR entries.retired_at>?4)".into(),
        ];
        for (column, value) in [
            (
                "schema_key",
                selection
                    .schema
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()?,
            ),
            (
                "scope",
                selection
                    .scope
                    .as_ref()
                    .map(|value| key(value.as_str()))
                    .transpose()?,
            ),
            (
                "stream",
                selection
                    .stream
                    .as_ref()
                    .map(|value| key(value.as_str()))
                    .transpose()?,
            ),
            (
                "item",
                selection
                    .item
                    .as_ref()
                    .map(|value| key(value.as_str()))
                    .transpose()?,
            ),
            (
                "item",
                selection
                    .object_key
                    .as_ref()
                    .map(|value| key(value.as_str()))
                    .transpose()?,
            ),
        ] {
            if let Some(value) = value {
                arguments.push(Value::Text(value));
                clauses.push(format!("entries.{column}=?{}", arguments.len()));
            }
        }
        if let Some(after) = after {
            arguments.push(Value::Text(position(after)));
            clauses.push(format!(
                "entries.position {} ?{}",
                if selection.order == ReadOrder::Ascending {
                    ">"
                } else {
                    "<"
                },
                arguments.len()
            ));
        }
        if let Some(range) = selection.source_time {
            arguments.push(Value::Blob(timestamp_key(range.start).to_vec()));
            clauses.push(format!("entries.source_time>=?{}", arguments.len()));
            arguments.push(Value::Blob(timestamp_key(range.end).to_vec()));
            clauses.push(format!("entries.source_time<?{}", arguments.len()));
        }
        if selection.kind == ReadKind::Objects
            && selection.object_versions == ObjectVersions::Latest
        {
            arguments.push(Value::Text(position(
                selection.through.unwrap_or(through.sequence).min(through.sequence),
            )));
            clauses.push(format!("NOT EXISTS (SELECT 1 FROM storage_data_entries AS newer
                WHERE newer.kind='object' AND newer.schema_id=entries.schema_id AND newer.item=entries.item
                AND newer.position>entries.position AND newer.position<=?{}
                AND (newer.retired_at IS NULL OR newer.retired_at>?4))", arguments.len()));
        }
        let count = if selection.kind == ReadKind::Schemas {
            1
        } else {
            limits.rows.get().min(64)
        };
        let sql = format!("SELECT entries.position,length(entries.body) FROM storage_data_entries AS entries
            INDEXED BY storage_data_kind_position WHERE {} ORDER BY entries.position {} LIMIT {}",
            clauses.join(" AND "), if selection.order == ReadOrder::Ascending { "ASC" } else { "DESC" }, count+1);

                let head = saved(connection)?;
                head.history.check(&through.history)?;
                if through.sequence > head.sequence {
                    return Err(Error::Data("page exceeds saved frontier"));
                }
                if selection.kind == ReadKind::Objects
                    && selection.history == ReadHistory::Complete
                    && selection.after < cut.lost_objects_through
                {
                    return Err(harmos::Error::Gap {
                        requested: selection.after.saturating_add(1),
                        available: cut.lost_objects_through.saturating_add(1),
                    }
                    .into());
                }
                let mut statement = connection.prepare(&sql)?;
                let mut rows = statement.query(rusqlite::params_from_iter(arguments))?;
                let metadata = count.checked_mul(std::mem::size_of::<Entry>() + std::mem::size_of::<super::memory::Reservation>())
                    .ok_or(Error::Data("page metadata allocation size overflow"))?;
                let _metadata = memory.0.memory.reserve_owned(metadata, usize::MAX)?;
                let mut decoded = Vec::with_capacity(count);
                let mut entries = Vec::with_capacity(count);
                let mut bytes = 0usize;
                while let Some(row) = rows.next()? {
                    let length = usize::try_from(row.get::<_, i64>(1)?)
                        .map_err(|_| Error::Corrupt("stored entry length exceeds address space"))?;
                    if length > entry_bytes.get() {
                        return Err(Error::Corrupt("stored entry exceeds backend bound"));
                    }
                    if entries.len() == count
                    {
                        if entries.is_empty() {
                            return Err(Error::Data("first entry exceeds page byte bound"));
                        }
                        return finish(Page {
                            entries,
                            has_more: true,
                        });
                    }
                    // Inspect Lookahead Metadata Before Loading Another Owned Blob.
                    let selected: String = row.get(0)?;
                    let mut content = connection.prepare("SELECT body,digest FROM storage_data_entries WHERE position=?1")?;
                    let mut content_rows = content.query([&selected])?;
                    let content_row = content_rows.next()?.ok_or(Error::Corrupt("selected page entry disappeared"))?;
                    let body = blob(content_row, 0)?;
                    if body.len() != length {
                        return Err(Error::Corrupt("selected page length differs"));
                    }
                    check_digest(body, blob(content_row, 1)?)?;
                    let portable = Entry::portable_bytes(kind(selection.kind), body)?;
                    if portable > limits.bytes.get().saturating_sub(bytes) {
                        if entries.is_empty() {
                            return Err(Error::Data("first entry exceeds page byte bound"));
                        }
                        return finish(Page { entries, has_more: true });
                    }
                    decoded.push(memory.0.memory.reserve_owned(
                        Entry::decoding_bytes(kind(selection.kind), body.len())?, usize::MAX
                    )?);
                    let entry = Entry::decode(kind(selection.kind), body)?;
                    if position(entry.position().sequence) != selected {
                        return Err(Error::Corrupt("entry index position differs"));
                    }
                    entry.position().history.check(&through.history)?;
                    bytes += portable;
                    entries.push(entry);
                }
                finish(Page {
                    entries,
                    has_more: false,
                })
            })())
        })
        .await?
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
        saved(connection)?;
    }
    Ok(())
}

fn open(connection: &mut Connection, expected: Option<StoreHistory>) -> Result<SavedData, Error> {
    let transaction = connection.transaction()?;
    let initialized: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND
            (name GLOB 'storage_data_*' OR name GLOB 'storage_bootstrap*'
                OR name IN ('storage_object_scalars','storage_file_stages')))",
        [],
        |row| row.get(0),
    )?;
    transaction.execute_batch(SCHEMA)?;
    transaction.execute_batch(production::SCHEMA)?;
    transaction.execute_batch(mutations::SCHEMA)?;
    transaction.execute_batch(inventory::SCHEMA)?;
    let exists = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM storage_data_head)",
        [],
        |row| row.get::<_, bool>(0),
    )?;
    if !exists {
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
            "INSERT INTO storage_data_head VALUES (1,?1,?2,?3,?3,?3)",
            params![FORMAT, encoded, position(0)],
        )?;
        transaction.execute(
            "INSERT INTO storage_data_origin VALUES (1,?1)",
            [position(0)],
        )?;
    }
    let head = saved(&transaction)?;
    if let Some(expected) = expected {
        expected.check(&head.history)?;
    }
    inventory::verify(&transaction, &head)?;
    transaction.commit()?;
    Ok(head)
}

fn saved(connection: &Connection) -> Result<SavedData, Error> {
    let mut statement = connection.prepare("SELECT format,history,saved,lost_objects_through,retention_generation FROM storage_data_head WHERE singleton=1")?;
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
    let saved_sequence = sequence(&row.get::<_, String>(2)?)?;
    let lost_objects_through = sequence(&row.get::<_, String>(3)?)?;
    if lost_objects_through > saved_sequence {
        return Err(Error::Corrupt("retention coverage exceeds saved frontier"));
    }
    Ok(SavedData {
        history: serde_json::from_slice(history)?,
        sequence: saved_sequence,
        lost_objects_through,
        retention_generation: sequence(&row.get::<_, String>(4)?)?,
    })
}

fn write(
    connection: &mut Connection,
    root: &std::path::Path,
    history: &StoreHistory,
    entries: &[Encoded],
    memory: &std::sync::Arc<super::memory::Budget>,
    maximum: NonZeroUsize,
    schema: &mut Option<index::IndexSchema>,
) -> Result<SavedData, Error> {
    let transaction = connection.transaction()?;
    bootstrap::check_writes(&transaction)?;
    let mut head = saved(&transaction)?;
    head.history.check(history)?;
    for entry in entries {
        {
            let mut statement = transaction.prepare(
                "SELECT kind,body,digest,retired_at FROM storage_data_entries WHERE position=?1",
            )?;
            let mut rows = statement.query([position(entry.position)])?;
            if let Some(row) = rows.next()? {
                if row.get_ref(3)?.data_type() != rusqlite::types::Type::Null {
                    return Err(Error::Data("position retry was logically pruned"));
                }
                let body = blob(row, 1)?;
                if body.len() > maximum.get() {
                    return Err(Error::Corrupt("position retry exceeds entry bound"));
                }
                let digest = blob(row, 2)?;
                check_digest(body, digest)?;
                if digest != entry.digest.as_slice() {
                    return Err(Error::Conflict("position retry has different bytes"));
                }
                if row.get_ref(0)?.as_str().map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })? != entry.kind
                    || body != entry.body.as_slice()
                {
                    return Err(Error::Corrupt("position retry content differs"));
                }
                inventory::confirm(&transaction, entry)?;
                if entry.position > head.sequence {
                    return Err(Error::Corrupt("position retry exceeds saved frontier"));
                }
                if entry.kind == "file" {
                    let reference: FileReference = serde_json::from_slice(&entry.body)?;
                    files::confirm(&transaction, root, &reference)?;
                }
                continue;
            }
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
        store_entry(&transaction, entry, memory, maximum, schema)?;
        head.sequence = entry.position;
    }
    transaction.execute(
        "UPDATE storage_data_head SET saved=?1 WHERE singleton=1",
        [position(head.sequence)],
    )?;
    transaction.commit()?;
    Ok(head)
}

fn store_entry(
    connection: &Connection,
    entry: &Encoded,
    memory: &std::sync::Arc<super::memory::Budget>,
    maximum: NonZeroUsize,
    schema: &mut Option<index::IndexSchema>,
) -> Result<(), Error> {
    connection.execute(
            "INSERT INTO storage_data_entries VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,NULL,NULL)",
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
    if entry.kind == "object" {
        index::write(connection, entry, memory, maximum, schema)?;
    }
    connection.execute(
        "INSERT INTO storage_data_inventory VALUES (?1,?2,?3,0)",
        params![
            position(entry.position),
            entry.kind,
            entry.digest.as_slice()
        ],
    )?;
    Ok(())
}

fn blob<'a>(row: &'a rusqlite::Row<'_>, column: usize) -> Result<&'a [u8], Error> {
    row.get_ref(column)?.as_blob().map_err(|source| {
        Error::Sqlite(rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Blob,
            Box::new(source),
        ))
    })
}

fn check_digest(body: &[u8], digest: &[u8]) -> Result<(), Error> {
    if Sha256::digest(body).as_slice() != digest {
        return Err(Error::Corrupt("stored entry digest differs"));
    }
    Ok(())
}

fn kind(value: ReadKind) -> &'static str {
    match value {
        ReadKind::Schemas => "schema",
        ReadKind::Objects => "object",
        ReadKind::Records => "record",
        ReadKind::Stream => "stream",
        ReadKind::Files => "file",
    }
}
fn position(sequence: u64) -> String {
    format!("{sequence:016x}")
}
fn sequence(value: &str) -> Result<u64, Error> {
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
