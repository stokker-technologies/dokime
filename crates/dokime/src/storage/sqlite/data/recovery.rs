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
    ReadLimits, RecoveryPosition, StoreHistory,
    recovery::{RecoveryEntry, RecoveryPage, RecoverySnapshot},
};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
// Local Imports
use super::super::{Error, State};
use super::{blob, check_digest, position, sequence};

/* === Constants === */
const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_recovery_history (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1), history BLOB NOT NULL
);
CREATE TABLE IF NOT EXISTS storage_recovery_head (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    saved TEXT NOT NULL CHECK(length(saved)=16),
    base TEXT NOT NULL CHECK(length(base)=16),
    snapshot BLOB, digest BLOB,
    CHECK((snapshot IS NULL AND digest IS NULL) OR
          (snapshot IS NOT NULL AND length(digest)=32))
);
CREATE TABLE IF NOT EXISTS storage_recovery_entries (
    position TEXT PRIMARY KEY CHECK(length(position)=16),
    payload BLOB NOT NULL, digest BLOB NOT NULL CHECK(length(digest)=32)
) WITHOUT ROWID;
";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy)]
pub struct RecoveryLimits {
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
impl Default for RecoveryLimits {
    fn default() -> Self {
        Self {
            entry_bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
            batch_bytes: NonZeroUsize::new(16 * 1024 * 1024).unwrap(),
            snapshot_bytes: NonZeroUsize::new(16 * 1024 * 1024).unwrap(),
        }
    }
}

impl State {
    pub(crate) async fn recovery_open(
        &self,
        history: StoreHistory,
    ) -> Result<RecoveryPosition, Error> {
        self.call_recovery(move |connection| {
            Ok((|| {
                let transaction = connection.transaction()?;
                let encoded = serde_json::to_vec(&history)?;
                transaction.execute(
                    "INSERT OR IGNORE INTO storage_recovery_history VALUES (1,?1)",
                    [&encoded],
                )?;
                check_history(&transaction, &history)?;
                transaction.execute(
                    "INSERT OR IGNORE INTO storage_recovery_head VALUES (1,?1,?1,NULL,NULL)",
                    [position(0)],
                )?;
                let sequence = head(&transaction)?.saved;
                transaction.commit()?;
                Ok(RecoveryPosition { history, sequence })
            })())
        })
        .await?
    }

    pub(crate) async fn recovery_flush(
        &self,
        through: RecoveryPosition,
    ) -> Result<RecoveryPosition, Error> {
        self.call_recovery(move |connection| {
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

    pub(crate) async fn recovery_write(
        &self,
        history: StoreHistory,
        entries: Vec<RecoveryEntry>,
        limits: RecoveryLimits,
    ) -> Result<(), Error> {
        let mut bytes = 0usize;
        for entry in &entries {
            history.check(&entry.position.history)?;
            if entry.payload.len() > limits.entry_bytes.get() {
                return Err(Error::Data("recovery entry exceeds byte bound"));
            }
            // Include each row's fixed indexing and checksum overhead before
            // cloning even empty payloads, so row metadata is also bounded.
            bytes = bytes
                .checked_add(entry.payload.len())
                .and_then(|bytes| bytes.checked_add(64))
                .filter(|bytes| *bytes <= limits.batch_bytes.get())
                .ok_or(Error::Data("recovery batch exceeds byte bound"))?;
        }
        self.call_recovery(move |connection| {
            Ok((|| {
                let transaction = connection.transaction()?;
                check_history(&transaction, &history)?;
                let mut head = head(&transaction)?;
                for entry in entries {
                    let at = entry.position.sequence;
                    if at <= head.base {
                        return Err(gap(at, head.base));
                    }
                    if at <= head.saved {
                        let mut statement = transaction.prepare(
                            "SELECT payload,digest FROM storage_recovery_entries WHERE position=?1",
                        )?;
                        let mut rows = statement.query([position(at)])?;
                        let row = rows
                            .next()?
                            .ok_or(Error::Corrupt("missing durable recovery entry"))?;
                        let payload = blob(row, 0)?;
                        check_digest(payload, blob(row, 1)?)?;
                        if payload != entry.payload {
                            return Err(Error::Conflict("recovery retry has different bytes"));
                        }
                        continue;
                    }
                    if head.saved.checked_add(1) != Some(at) {
                        return Err(Error::Data("recovery write has a gap"));
                    }
                    let digest = Sha256::digest(&entry.payload);
                    transaction.execute(
                        "INSERT INTO storage_recovery_entries VALUES (?1,?2,?3)",
                        params![position(at), entry.payload, digest.as_slice()],
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

    pub(crate) async fn recovery_snapshot(
        &self,
        history: StoreHistory,
        maximum: NonZeroUsize,
    ) -> Result<Option<RecoverySnapshot>, Error> {
        let memory = self.clone();
        self.call_recovery(move |connection| {
            Ok((|| {
                check_history(connection, &history)?;
                let head = head(connection)?;
                let mut statement = connection.prepare(
                    "SELECT snapshot,digest FROM storage_recovery_head WHERE singleton=1",
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
                    .checked_add(std::mem::size_of::<RecoverySnapshot>() + super::HISTORY_BYTES)
                    .ok_or(Error::Data("recovery snapshot allocation size overflow"))?;
                let _reservation = memory.reserve_memory(allocation)?;
                check_digest(payload, blob(row, 1)?)?;
                Ok(Some(RecoverySnapshot {
                    through: RecoveryPosition {
                        history,
                        sequence: head.base,
                    },
                    payload: payload.to_vec(),
                }))
            })())
        })
        .await?
    }

    pub(crate) async fn recovery_store_snapshot(
        &self,
        snapshot: RecoverySnapshot,
        maximum: NonZeroUsize,
    ) -> Result<(), Error> {
        if snapshot.payload.len() > maximum.get() {
            return Err(Error::Data("recovery snapshot exceeds write byte bound"));
        }
        self.call_recovery(move |connection| Ok((|| {
            let transaction = connection.transaction()?;
            check_history(&transaction, &snapshot.through.history)?;
            let head = head(&transaction)?;
            let through = snapshot.through.sequence;
            if through < head.base { return Err(gap(through,head.base)); }
            if through > head.saved { return Err(Error::Data("snapshot exceeds durable recovery frontier")); }
            if through == head.base {
                let mut statement = transaction.prepare("SELECT snapshot,digest FROM storage_recovery_head WHERE singleton=1")?;
                let mut rows = statement.query([])?;
                let row = rows.next()?.ok_or(Error::Corrupt("missing recovery head"))?;
                if row.get_ref(0)?.data_type() != rusqlite::types::Type::Null {
                    let payload = blob(row,0)?;
                    check_digest(payload,blob(row,1)?)?;
                    return if payload == snapshot.payload { Ok(()) }
                        else { Err(Error::Conflict("snapshot retry has different bytes")) };
                }
            }
            let digest = Sha256::digest(&snapshot.payload);
            transaction.execute("UPDATE storage_recovery_head SET base=?1,snapshot=?2,digest=?3 WHERE singleton=1", params![position(through),snapshot.payload,digest.as_slice()])?;
            transaction.execute("DELETE FROM storage_recovery_entries WHERE position<=?1", [position(through)])?;
            transaction.commit()?;
            Ok(())
        })())).await?
    }

    pub(crate) async fn recovery_page(
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
        self.call_recovery(move |connection| Ok((|| {
            check_history(connection, &through.history)?;
            let head = head(connection)?;
            if after.sequence < head.base { return Err(gap(after.sequence,head.base)); }
            if through.sequence > head.saved { return Err(Error::Data("recovery read exceeds durable frontier")); }
            let mut statement = connection.prepare("SELECT position,payload,digest FROM storage_recovery_entries WHERE position>?1 AND position<=?2 ORDER BY position LIMIT ?3")?;
            let count = limits.rows.get().min(64);
            let mut rows = statement.query(params![position(after.sequence),position(through.sequence),count as i64])?;
            let allocation = count.checked_mul(std::mem::size_of::<RecoveryEntry>() + super::HISTORY_BYTES)
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
                let next_bytes = bytes.checked_add(payload.len()).filter(|bytes| *bytes <= limits.bytes.get());
                let Some(next_bytes) = next_bytes else {
                    if entries.is_empty() { return Err(Error::Data("recovery entry exceeds page byte bound")); }
                    break;
                };
                check_digest(payload,blob(row,2)?)?;
                entries.push(RecoveryEntry { position: RecoveryPosition { history: through.history.clone(),sequence: at },payload: payload.to_vec() });
                bytes = next_bytes;
                previous = at;
            }
            if entries.is_empty() && previous < through.sequence { return Err(Error::Corrupt("missing durable recovery tail")); }
            let next = (previous < through.sequence).then(|| RecoveryPosition { history: through.history.clone(), sequence: previous });
            let page = RecoveryPage { entries,next };
            page.check(&after,&through,limits,|entry| entry.payload.len())?;
            Ok(page)
        })())).await?
    }
}

/* === Public Functions === */

/* === Private Functions === */
pub(in crate::storage::sqlite) fn initialize(recovery: &mut Connection) -> Result<(), Error> {
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

fn gap(requested: u64, base: u64) -> Error {
    Error::Runtime(harmos::Error::Gap {
        requested,
        available: base.saturating_add(1),
    })
}

/* === Tests === */
