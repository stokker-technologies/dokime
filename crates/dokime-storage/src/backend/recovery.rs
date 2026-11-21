/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Native Recovery State

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::Read;
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_domain::state::Position;
use harmos_storage::BackendFailure;
use harmos_storage::Error;
use harmos_storage::state::{RecoveryEntry, RecoveryPage, RecoverySnapshot, SnapshotPayload, StateBackend};
use harmos_storage::{ReadLimits, RecoveryPosition, Result, StorageBatch, StorageCursor, StorageFuture, StoreHistory};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// Local Imports
use super::{Backend, State, db, decode, encode, integer, invalid, meta, set_meta, unsigned};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct RecoveryBackend(pub Backend);

#[derive(Serialize, Deserialize)]
struct SnapshotHeader {
    required_data: StorageCursor,
    through: RecoveryPosition,
    position: Position,
    sha256: Vec<u8>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl StateBackend for RecoveryBackend {
    fn get_history(&self) -> &StoreHistory {
        &self.0.history
    }

    fn create_recovery_session(&self) -> StorageFuture<'_, RecoveryPosition> {
        Box::pin(self.0.call(|state| {
            state.check_lease()?;
            state.recovery_closed = false;
            let tx = state.connection.transaction().map_err(db)?;
            let saved = meta(&tx, "recovery_saved")?;
            tx.execute("DELETE FROM recovery_entries WHERE sequence>?1", [integer(saved)?]).map_err(db)?;
            tx.execute("DELETE FROM recovery_snapshots WHERE published=0 OR sequence>?1", [integer(saved)?]).map_err(db)?;
            set_meta(&tx, "recovery_written", saved)?;
            tx.commit().map_err(db)?;
            position(state, "recovery_saved")
        }))
    }

    fn get_recovery_snapshot(&self, maximum: NonZeroUsize) -> StorageFuture<'_, Option<RecoverySnapshot>> {
        Box::pin(self.0.call(move |state| {
            check(state)?;
            let value: Option<(Vec<u8>, Vec<u8>)> = state
                .connection
                .query_row("SELECT header,payload FROM recovery_snapshots WHERE published=1 ORDER BY sequence DESC LIMIT 1", [], |row| {
                    Ok((row.get(0)?, row.get(1)?))
                })
                .optional()
                .map_err(db)?;
            let Some((header, bytes)) = value else { return Ok(None) };
            let header: SnapshotHeader = decode(&header)?;
            if Sha256::digest(&bytes).as_slice() != header.sha256 {
                return Err(invalid("Read Recovery Snapshot: Restore a Valid Backup"));
            }
            let payload = SnapshotPayload::from_bytes(&bytes, maximum, header.position)
                .map_err(|source| Error::backend("Read Recovery Snapshot", BackendFailure::Corrupt, source))?;
            Ok(Some(RecoverySnapshot {
                required_data: header.required_data,
                through: header.through,
                payload,
            }))
        }))
    }

    fn set_recovery_snapshot(&self, snapshots: StorageBatch<RecoverySnapshot>) -> StorageFuture<'_, ()> {
        Box::pin(self.0.call(move |state| {
            check(state)?;
            let [snapshot] = snapshots.as_slice() else { return Err(invalid("Set Recovery Snapshot: Supply Exactly One Snapshot")) };
            state.history.check(&snapshot.through.history)?;
            state.history.check(&snapshot.required_data.history)?;
            snapshot
                .payload
                .verify(snapshot.payload.segment_bytes())
                .map_err(|source| Error::backend("Set Recovery Snapshot", BackendFailure::Corrupt, source))?;
            let mut bytes = Vec::new();
            snapshot
                .payload
                .reader()
                .and_then(|mut reader| reader.read_to_end(&mut bytes))
                .map_err(|source| Error::backend("Read Recovery Snapshot Payload", BackendFailure::Permanent, source))?;
            let header = encode(&SnapshotHeader {
                required_data: snapshot.required_data.clone(),
                through: snapshot.through.clone(),
                position: snapshot.payload.manifest().position,
                sha256: Sha256::digest(&bytes).to_vec(),
            })?;
            let tx = state.connection.transaction().map_err(db)?;
            let prior: Option<(Vec<u8>, Vec<u8>)> = tx
                .query_row(
                    "SELECT header,payload FROM recovery_snapshots WHERE sequence=?1",
                    [integer(snapshot.through.sequence)?],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .map_err(db)?;
            if prior.as_ref().is_some_and(|value| value != &(header.clone(), bytes.clone())) {
                return Err(invalid("Set Recovery Snapshot: Retain the Original Snapshot at This Position"));
            }
            tx.execute(
                "INSERT OR IGNORE INTO recovery_snapshots VALUES(?1,?2,?3,0)",
                params![integer(snapshot.through.sequence)?, header, bytes],
            )
            .map_err(db)?;
            set_meta(&tx, "recovery_written", meta(&tx, "recovery_written")?.max(snapshot.through.sequence))?;
            tx.commit().map_err(db)
        }))
    }

    fn list_recovery_entries<'a>(
        &'a self, after: &'a RecoveryPosition, through: &'a RecoveryPosition, limits: ReadLimits,
    ) -> StorageFuture<'a, RecoveryPage> {
        let (after, through) = (after.clone(), through.clone());
        Box::pin(self.0.call(move |state| {
            check(state)?;
            state.history.check(&after.history)?;
            state.history.check(&through.history)?;
            if after.sequence > through.sequence || through.sequence > meta(&state.connection, "recovery_saved")? {
                return Err(invalid("List Recovery Entries: Select an Ordered Saved Range"));
            }
            let mut statement = state
                .connection
                .prepare("SELECT data FROM recovery_entries WHERE sequence>?1 AND sequence<=?2 ORDER BY sequence")
                .map_err(db)?;
            let mut rows = statement.query(params![integer(after.sequence)?, integer(through.sequence)?]).map_err(db)?;
            let mut entries = Vec::new();
            let mut bytes = 0;
            while let Some(row) = rows.next().map_err(db)? {
                let data: Vec<u8> = row.get(0).map_err(db)?;
                if !entries.is_empty() && (entries.len() == limits.rows.get() || bytes + data.len() > limits.bytes.get()) {
                    break;
                }
                limits.check(entries.len() + 1, bytes + data.len())?;
                bytes += data.len();
                entries.push(decode::<RecoveryEntry>(&data)?);
            }
            let last = entries.last().map_or(after.sequence, |entry| entry.position.sequence);
            let next = (last < through.sequence).then_some(RecoveryPosition {
                history: state.history.clone(),
                sequence: last,
            });
            let page = RecoveryPage { entries, next };
            page.check(&after, &through, limits, |entry| encode(entry).map_or(usize::MAX, |bytes| bytes.len()))?;
            Ok(page)
        }))
    }

    fn append_recovery_entries(&self, entries: StorageBatch<RecoveryEntry>) -> StorageFuture<'_, ()> {
        Box::pin(self.0.call(move |state| {
            check(state)?;
            let tx = state.connection.transaction().map_err(db)?;
            let mut written = meta(&tx, "recovery_written")?;
            for entry in entries.as_slice() {
                state.history.check(&entry.position.history)?;
                state.history.check(&entry.required_data.history)?;
                let bytes = encode(entry)?;
                let prior: Option<Vec<u8>> = tx
                    .query_row("SELECT data FROM recovery_entries WHERE sequence=?1", [integer(entry.position.sequence)?], |row| row.get(0))
                    .optional()
                    .map_err(db)?;
                if let Some(prior) = prior {
                    if prior != bytes {
                        return Err(invalid("Append Recovery Entries: Retain the Original Payload"));
                    }
                    continue;
                }
                if entry.position.sequence
                    != written.checked_add(1).ok_or_else(|| invalid("Append Recovery Entries: Position Exhausted"))?
                {
                    return Err(invalid("Append Recovery Entries: Supply Consecutive Positions"));
                }
                tx.execute("INSERT INTO recovery_entries VALUES(?1,?2)", params![integer(entry.position.sequence)?, bytes]).map_err(db)?;
                written = entry.position.sequence;
            }
            set_meta(&tx, "recovery_written", written)?;
            tx.commit().map_err(db)
        }))
    }

    fn flush<'a>(&'a self, through: &'a RecoveryPosition) -> StorageFuture<'a, RecoveryPosition> {
        let through = through.clone();
        Box::pin(self.0.call(move |state| {
            check(state)?;
            state.history.check(&through.history)?;
            let tx = state.connection.transaction().map_err(db)?;
            let snapshot: Option<Vec<u8>> = tx
                .query_row(
                    "SELECT header FROM recovery_snapshots WHERE sequence<=?1 ORDER BY sequence DESC LIMIT 1",
                    [integer(through.sequence)?],
                    |row| row.get(0),
                )
                .optional()
                .map_err(db)?;
            if let Some(header) = snapshot {
                let header: SnapshotHeader = decode(&header)?;
                if header.required_data.sequence > meta(&tx, "saved")? {
                    return Err(invalid("Flush Recovery: Persist Required Primary Data First"));
                }
                tx.execute("UPDATE recovery_snapshots SET published=1 WHERE sequence=?1", [integer(header.through.sequence)?])
                    .map_err(db)?;
            }
            let written = meta(&tx, "recovery_written")?;
            let snapshot_written: u64 =
                tx.query_row("SELECT coalesce(max(sequence),0) FROM recovery_snapshots", [], |row| unsigned(row, 0)).map_err(db)?;
            if through.sequence > written.max(snapshot_written) {
                return Err(invalid("Flush Recovery: Append Through the Requested Position First"));
            }
            let required: u64 = tx.query_row(
                "SELECT coalesce(max(json_extract(CAST(data AS TEXT),'$.required_data.sequence')),0) FROM recovery_entries WHERE sequence<=?1",
                [integer(through.sequence)?], |row| unsigned(row,0)).map_err(db)?;
            if required > meta(&tx, "saved")? {
                return Err(invalid("Flush Recovery: Persist Required Primary Data First"));
            }
            let saved = meta(&tx, "recovery_saved")?.max(through.sequence);
            set_meta(&tx, "recovery_saved", saved)?;
            tx.commit().map_err(db)?;
            position(state, "recovery_saved")
        }))
    }

    fn compact_recovery(&self) -> StorageFuture<'_, ()> {
        Box::pin(self.0.call(|state| {
            check(state)?;
            let tx = state.connection.transaction().map_err(db)?;
            let newest: Option<i64> =
                tx.query_row("SELECT max(sequence) FROM recovery_snapshots WHERE published=1", [], |row| row.get(0)).map_err(db)?;
            let Some(newest) = newest else { return Ok(()) };
            tx.execute("DELETE FROM recovery_entries WHERE sequence<=?1", [newest]).map_err(db)?;
            tx.execute("DELETE FROM recovery_snapshots WHERE sequence<?1", [newest]).map_err(db)?;
            tx.commit().map_err(db)
        }))
    }

    fn delete_recovery_session(&self) -> StorageFuture<'_, ()> {
        Box::pin(self.0.call(|state| {
            state.recovery_closed = true;
            Ok(())
        }))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn check(state: &State) -> Result<()> {
    state.check_lease()?;
    if state.recovery_closed {
        return Err(Error::Closing { domain: "Recovery Storage" });
    }
    Ok(())
}
fn position(state: &State, key: &str) -> Result<RecoveryPosition> {
    Ok(RecoveryPosition {
        history: state.history.clone(),
        sequence: meta(&state.connection, key)?,
    })
}

/* === Tests === */
