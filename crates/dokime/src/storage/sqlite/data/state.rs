/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;
// Third Party Imports
use harmos::storage::{
    ReadLimits, RecoveryPosition, StorageCursor, StoreHistory,
    state::{RecoveryEntry, RecoveryPage, RecoverySnapshot},
};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
// Local Imports
use super::super::{Error, State};
use super::{blob, position, sequence};

/* === Constants === */
const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_recovery_history (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1), history BLOB NOT NULL
);
CREATE TABLE IF NOT EXISTS storage_recovery_head (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    saved TEXT NOT NULL CHECK(length(saved)=16),
    base TEXT NOT NULL CHECK(length(base)=16),
    snapshot BLOB, digest BLOB, required_data BLOB,
    CHECK((snapshot IS NULL AND digest IS NULL AND required_data IS NULL) OR
          (snapshot IS NOT NULL AND length(digest)=32 AND required_data IS NOT NULL))
);
CREATE TABLE IF NOT EXISTS storage_recovery_entries (
    position TEXT PRIMARY KEY CHECK(length(position)=16),
    payload BLOB NOT NULL, digest BLOB NOT NULL CHECK(length(digest)=32), required_data BLOB NOT NULL
) WITHOUT ROWID;
";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy)]
pub struct StateLimits {
    pub entry_bytes: NonZeroUsize,
    pub batch_bytes: NonZeroUsize,
    pub snapshot_bytes: NonZeroUsize,
}

struct Head {
    saved: u64,
    base: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Default for StateLimits {
    fn default() -> Self {
        Self {
            entry_bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
            batch_bytes: NonZeroUsize::new(16 * 1024 * 1024).unwrap(),
            snapshot_bytes: NonZeroUsize::new(16 * 1024 * 1024).unwrap(),
        }
    }
}

impl State {
    pub(crate) async fn state_open(
        &self,
        history: StoreHistory,
    ) -> Result<RecoveryPosition, Error> {
        self.call(move |connection| {
            Ok((|| {
                let transaction = connection.transaction()?;
                let encoded = serde_json::to_vec(&history)?;
                transaction.execute(
                    "INSERT OR IGNORE INTO storage_recovery_history VALUES (1,?1)",
                    [&encoded],
                )?;
                check_history(&transaction, &history)?;
                transaction.execute(
                    "INSERT OR IGNORE INTO storage_recovery_head VALUES (1,?1,?1,NULL,NULL,NULL)",
                    [position(0)],
                )?;
                let sequence = head(&transaction)?.saved;
                transaction.commit()?;
                Ok(RecoveryPosition { history, sequence })
            })())
        })
        .await?
    }

    pub(crate) async fn state_flush(
        &self,
        through: RecoveryPosition,
    ) -> Result<RecoveryPosition, Error> {
        self.call(move |connection| {
            Ok((|| {
                check_history(connection, &through.history)?;
                let head = head(connection)?;
                if through.sequence > head.saved {
                    return Err(Error::Data("recovery flush exceeds admitted frontier"));
                }
                Ok(RecoveryPosition {
                    history: through.history,
                    sequence: head.saved,
                })
            })())
        })
        .await?
    }

    pub(crate) async fn state_write(
        &self,
        history: StoreHistory,
        entries: Vec<RecoveryEntry>,
        limits: StateLimits,
    ) -> Result<(), Error> {
        let mut bytes = 0usize;
        for entry in &entries {
            history.check(&entry.position.history)?;
            if entry.payload.len() > limits.entry_bytes.get() {
                return Err(Error::Data("recovery entry exceeds byte bound"));
            }
            let dependency_bytes =
                super::entry::check_size(&entry.required_data, super::HISTORY_BYTES)?;
            // Include The Durable Dependency And Fixed Journal Indexing Before Cloning.
            bytes = bytes
                .checked_add(entry.payload.len())
                .and_then(|bytes| bytes.checked_add(dependency_bytes))
                .and_then(|bytes| bytes.checked_add(64))
                .filter(|bytes| *bytes <= limits.batch_bytes.get())
                .ok_or(Error::Data("recovery batch exceeds byte bound"))?;
        }
        self.call(move |connection| {
            Ok((|| {
                let transaction = connection.transaction()?;
                check_history(&transaction, &history)?;
                let mut head = head(&transaction)?;
                for entry in entries {
                    let required = serde_json::to_vec(&entry.required_data)?;
                    let at = entry.position.sequence;
                    if at <= head.base {
                        return Err(gap(at, head.base));
                    }
                    if at <= head.saved {
                        let mut statement = transaction.prepare(
                            "SELECT payload,digest,required_data FROM storage_recovery_entries WHERE position=?1",
                        )?;
                        let mut rows = statement.query([position(at)])?;
                        let row = rows
                            .next()?
                            .ok_or(Error::Corrupt("missing durable recovery entry"))?;
                        let payload = blob(row, 0)?;
                        let stored_required = blob(row, 2)?;
                        check_state_digest(payload, stored_required, blob(row, 1)?)?;
                        if payload != entry.payload || stored_required != required {
                            return Err(Error::Conflict("recovery retry has different bytes"));
                        }
                        continue;
                    }
                    if head.saved.checked_add(1) != Some(at) {
                        return Err(Error::Data("recovery write has a gap"));
                    }
                    let digest = state_digest(&entry.payload, &required);
                    transaction.execute(
                        "INSERT INTO storage_recovery_entries VALUES (?1,?2,?3,?4)",
                        params![position(at), entry.payload, digest.as_slice(), required],
                    )?;
                    head.saved = at;
                }
                transaction.execute(
                    "UPDATE storage_recovery_head SET saved=?1 WHERE singleton=1",
                    [position(head.saved)],
                )?;
                transaction.commit()?;
                Ok(())
            })())
        })
        .await?
    }

    pub(crate) async fn state_snapshot(
        &self,
        history: StoreHistory,
        maximum: NonZeroUsize,
    ) -> Result<Option<RecoverySnapshot>, Error> {
        let memory = self.clone();
        self.call(move |connection| {
            Ok((|| {
                check_history(connection, &history)?;
                let head = head(connection)?;
                let mut statement = connection.prepare(
                    "SELECT snapshot,digest,required_data FROM storage_recovery_head WHERE singleton=1",
                )?;
                let mut rows = statement.query([])?;
                let row = rows
                    .next()?
                    .ok_or(Error::Corrupt("missing recovery head"))?;
                if row.get_ref(0)?.data_type() == rusqlite::types::Type::Null {
                    return Ok(None);
                }
                let payload = blob(row, 0)?;
                if payload.len() > maximum.get() {
                    return Err(Error::Data("recovery snapshot exceeds read byte bound"));
                }
                let allocation = payload
                    .len()
                    .checked_add(std::mem::size_of::<RecoverySnapshot>() + 4 * super::HISTORY_BYTES)
                    .ok_or(Error::Data("recovery snapshot allocation size overflow"))?;
                let _reservation = memory.reserve_memory(allocation)?;
                let required_data = decode_dependency(blob(row, 2)?)?;
                check_state_digest(payload, blob(row, 2)?, blob(row, 1)?)?;
                Ok(Some(RecoverySnapshot {
                    through: RecoveryPosition {
                        history,
                        sequence: head.base,
                    },
                    payload: payload.to_vec(),
                    required_data,
                }))
            })())
        })
        .await?
    }

    pub(crate) async fn state_store_snapshot(
        &self,
        snapshot: RecoverySnapshot,
        maximum: NonZeroUsize,
    ) -> Result<(), Error> {
        if snapshot.payload.len() > maximum.get() {
            return Err(Error::Data("recovery snapshot exceeds write byte bound"));
        }
        self.call(move |connection| Ok((|| {
            let transaction = connection.transaction()?;
            check_history(&transaction, &snapshot.through.history)?;
            let required = serde_json::to_vec(&snapshot.required_data)?;
            let head = head(&transaction)?;
            let through = snapshot.through.sequence;
            if through < head.base { return Err(gap(through,head.base)); }
            if through > head.saved { return Err(Error::Data("snapshot exceeds durable recovery frontier")); }
            if through == head.base {
                let mut statement = transaction.prepare("SELECT snapshot,digest,required_data FROM storage_recovery_head WHERE singleton=1")?;
                let mut rows = statement.query([])?;
                let row = rows.next()?.ok_or(Error::Corrupt("missing recovery head"))?;
                if row.get_ref(0)?.data_type() != rusqlite::types::Type::Null {
                    let payload = blob(row,0)?;
                    check_state_digest(payload,blob(row,2)?,blob(row,1)?)?;
                    return if payload == snapshot.payload && blob(row,2)? == required { Ok(()) }
                        else { Err(Error::Conflict("snapshot retry has different bytes")) };
                }
            }
            let digest = state_digest(&snapshot.payload, &required);
            transaction.execute("UPDATE storage_recovery_head SET base=?1,snapshot=?2,digest=?3,required_data=?4 WHERE singleton=1", params![position(through),snapshot.payload,digest.as_slice(),required])?;
            transaction.execute("DELETE FROM storage_recovery_entries WHERE position<=?1", [position(through)])?;
            transaction.commit()?;
            Ok(())
        })())).await?
    }

    pub(crate) async fn state_page(
        &self,
        after: RecoveryPosition,
        through: RecoveryPosition,
        limits: ReadLimits,
        entry_bytes: NonZeroUsize,
    ) -> Result<RecoveryPage, Error> {
        after.history.check(&through.history)?;
        if after.sequence > through.sequence {
            return Err(Error::Data("recovery range is reversed"));
        }
        let memory = self.clone();
        self.call(move |connection| Ok((|| {
            check_history(connection, &through.history)?;
            let head = head(connection)?;
            if after.sequence < head.base { return Err(gap(after.sequence,head.base)); }
            if through.sequence > head.saved { return Err(Error::Data("recovery read exceeds durable frontier")); }
            let mut statement = connection.prepare("SELECT position,payload,digest,required_data FROM storage_recovery_entries WHERE position>?1 AND position<=?2 ORDER BY position LIMIT ?3")?;
            let count = limits.rows.get().min(64);
            let mut rows = statement.query(params![position(after.sequence),position(through.sequence),count as i64])?;
            let allocation = count.checked_mul(std::mem::size_of::<RecoveryEntry>() + 4 * super::HISTORY_BYTES)
                .and_then(|bytes| bytes.checked_add(limits.bytes.get()))
                .ok_or(Error::Data("recovery page allocation size overflow"))?;
            let _reservation = memory.reserve_memory(allocation)?;
            let mut entries = Vec::with_capacity(count);
            let mut bytes = 0usize;
            let mut previous = after.sequence;
            while let Some(row) = rows.next()? {
                let at = sequence(&row.get::<_, String>(0)?)?;
                if previous.checked_add(1) != Some(at) { return Err(Error::Corrupt("durable recovery range has a gap")); }
                let payload = blob(row,1)?;
                if payload.len() > entry_bytes.get() { return Err(Error::Corrupt("stored recovery entry exceeds its byte bound")); }
                let required_data = decode_dependency(blob(row,3)?)?;
                check_state_digest(payload,blob(row,3)?,blob(row,2)?)?;
                let entry = RecoveryEntry { position: RecoveryPosition { history: through.history.clone(),sequence: at },payload: payload.to_vec(), required_data };
                let next_bytes = bytes.checked_add(encoded_bytes(&entry)?).filter(|bytes| *bytes <= limits.bytes.get());
                let Some(next_bytes) = next_bytes else {
                    if entries.is_empty() { return Err(Error::Data("recovery entry exceeds page byte bound")); }
                    break;
                };
                entries.push(entry);
                bytes = next_bytes;
                previous = at;
            }
            if entries.is_empty() && previous < through.sequence { return Err(Error::Corrupt("missing durable recovery tail")); }
            let next = (previous < through.sequence).then(|| RecoveryPosition { history: through.history.clone(), sequence: previous });
            let page = RecoveryPage { entries,next };
            page.check(&after,&through,limits,|entry| encoded_bytes(entry).unwrap_or(usize::MAX))?;
            Ok(page)
        })())).await?
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn state_digest(payload: &[u8], required: &[u8]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update((payload.len() as u64).to_le_bytes());
    digest.update(payload);
    digest.update(required);
    digest.finalize().into()
}
fn check_state_digest(payload: &[u8], required: &[u8], digest: &[u8]) -> Result<(), Error> {
    if state_digest(payload, required).as_slice() != digest {
        return Err(Error::Corrupt(
            "state payload or evidence dependency checksum differs",
        ));
    }
    Ok(())
}
// Harmos validates this cursor against durable evidence before writing or replaying state.
fn decode_dependency(bytes: &[u8]) -> Result<StorageCursor, Error> {
    if bytes.len() > super::HISTORY_BYTES {
        return Err(Error::Corrupt("state dependency exceeds history bound"));
    }
    let required: StorageCursor = serde_json::from_slice(bytes)?;
    Ok(required)
}

pub(in crate::storage::sqlite) fn initialize(recovery: &mut Connection) -> Result<(), Error> {
    let incompatible: bool = recovery.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='storage_recovery_head')
         AND (NOT EXISTS(SELECT 1 FROM pragma_table_info('storage_recovery_head') WHERE name='required_data')
              OR NOT EXISTS(SELECT 1 FROM pragma_table_info('storage_recovery_entries') WHERE name='required_data'))",
        [], |row| row.get(0),
    )?;
    if incompatible {
        return Err(Error::Conflict(
            "state journal without evidence dependencies requires a fresh storage root",
        ));
    }
    let transaction = recovery.transaction()?;
    transaction.execute_batch(SCHEMA)?;
    transaction.pragma_update(None, "user_version", super::super::SCHEMA_VERSION)?;
    transaction.commit()?;
    Ok(())
}

fn check_history(connection: &Connection, expected: &StoreHistory) -> Result<(), Error> {
    let encoded: Vec<u8> = connection.query_row(
        "SELECT history FROM storage_recovery_history WHERE singleton=1",
        [],
        |row| row.get(0),
    )?;
    if encoded.len() > super::HISTORY_BYTES {
        return Err(Error::Corrupt("recovery history exceeds its bound"));
    }
    let history: StoreHistory = serde_json::from_slice(&encoded)?;
    history.check(expected)?;
    Ok(())
}

fn head(connection: &Connection) -> Result<Head, Error> {
    let mut statement = connection.prepare(
        "SELECT saved,base,snapshot IS NOT NULL FROM storage_recovery_head WHERE singleton=1",
    )?;
    let mut rows = statement.query([])?;
    let row = rows
        .next()?
        .ok_or(Error::Corrupt("missing recovery head"))?;
    let head = Head {
        saved: sequence(&row.get::<_, String>(0)?)?,
        base: sequence(&row.get::<_, String>(1)?)?,
    };
    if head.base > head.saved || (head.base > 0 && !row.get::<_, bool>(2)?) {
        return Err(Error::Corrupt("invalid recovery snapshot frontier"));
    }
    Ok(head)
}

// Harmos bounds recovery pages by each entry's serde_json size, where the byte payload encodes as
// a decimal array of up to four bytes per payload byte. Packing by raw blob length let a page of
// JSON transaction groups pass here at 1.2 MB and fail Harmos's replay check at 4.4 MB.
fn encoded_bytes(entry: &RecoveryEntry) -> Result<usize, Error> {
    struct Count(usize);
    impl std::io::Write for Count {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .ok_or_else(|| std::io::Error::other("recovery entry size overflow"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut count = Count(0);
    serde_json::to_writer(&mut count, entry)?;
    Ok(count.0)
}

fn gap(requested: u64, base: u64) -> Error {
    Error::Runtime(harmos::Error::Gap {
        requested,
        available: base.saturating_add(1),
    })
}

/* === Tests === */
