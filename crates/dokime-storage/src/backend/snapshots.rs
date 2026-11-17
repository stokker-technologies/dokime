/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Scope-Qualified Snapshot Ownership

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

// Local Imports
use super::{State, compound, db, integer, invalid, meta, unsigned};
use harmos_storage::{BackendSnapshot, Error, Identifier, Result, StorageCursor};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Lease {
    pub position: u64,
    pub generation: u64,
    pub scope: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn read(connection: &Connection, snapshot: &BackendSnapshot) -> Result<Lease> {
    let lease = lookup(connection, "storage_snapshot_leases", snapshot)?;
    lease.ok_or_else(|| invalid("Read Snapshot: List a Fresh Image or Reopen Its Retained Cut"))
}

pub(super) fn create(state: &State, at: &StorageCursor) -> Result<BackendSnapshot> {
    state.check()?;
    state.history.check(&at.history)?;
    if at.sequence > state.cursor()?.sequence {
        return Err(Error::NotReady { domain: "requested storage cut" });
    }
    let floor = meta(&state.connection, "floor")?;
    let generation = if at.sequence < floor {
        state
            .connection
            .query_row(
                "SELECT generation FROM storage_snapshot_leases WHERE position=?1 AND scope IS NULL LIMIT 1",
                [integer(at.sequence)?],
                |row| unsigned(row, 0),
            )
            .optional()
            .map_err(db)?
            .ok_or(Error::Gap { requested: at.sequence, available: floor })?
    } else {
        meta(&state.connection, "deletion_generation")?
    };
    let partial: bool = state
        .connection
        .query_row("SELECT EXISTS(SELECT 1 FROM storage_compound_ranges WHERE first<=?1 AND last>?1)", [integer(at.sequence)?], |row| {
            row.get(0)
        })
        .map_err(db)?;
    if partial {
        return Err(invalid("Create Snapshot: Select a Complete Compound Receipt Frontier"));
    }
    lease(&state.connection, at, generation, None)
}

pub(super) fn retain(state: &State, snapshot: &BackendSnapshot, owner: Option<&Identifier>) -> Result<BackendSnapshot> {
    state.snapshot(snapshot)?;
    let image = read(&state.connection, snapshot)?;
    if owner.is_some_and(|owner| image.scope.as_ref().is_some_and(|scope| scope != owner.as_str())) {
        return Err(invalid("Retain Snapshot: Keep Its Original Scope or Select a Global Image"));
    }
    if let Some(owner) = owner {
        let deleted: bool = state
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM storage_scope_deletions WHERE scope=?1 AND position>?2)",
                params![owner.as_str(), integer(snapshot.at.sequence)?],
                |row| row.get(0),
            )
            .map_err(db)?;
        if deleted {
            return Err(invalid("Retain Snapshot: Select a Fresh Scope and Cut After Deletion"));
        }
    }
    let token = Uuid::now_v7().as_bytes().to_vec();
    let scope = owner.map(Identifier::as_str).or(image.scope.as_deref());
    state
        .connection
        .execute(
            "INSERT INTO storage_retained_snapshots VALUES(?1,?2,?3,?4,?5)",
            params![token, integer(image.position)?, integer(image.generation)?, scope, owner.map(Identifier::as_str)],
        )
        .map_err(db)?;
    Ok(BackendSnapshot { at: snapshot.at.clone(), token })
}

pub(super) fn open(state: &State, retained: &BackendSnapshot) -> Result<BackendSnapshot> {
    state.check()?;
    state.history.check(&retained.at.history)?;
    let image = lookup(&state.connection, "storage_retained_snapshots", retained)?
        .ok_or_else(|| invalid("Open Retained Snapshot: Ownership Was Released; List a Fresh Image"))?;
    lease(&state.connection, &retained.at, image.generation, image.scope.as_deref())
}

pub(super) fn release(state: &mut State, retained: &BackendSnapshot) -> Result<()> {
    state.check()?;
    state.history.check(&retained.at.history)?;
    lookup(&state.connection, "storage_retained_snapshots", retained)?;
    state.connection.execute("DELETE FROM storage_retained_snapshots WHERE token=?1", [&retained.token]).map_err(db)?;
    compound::reclaim(state)
}

pub(super) fn close(state: &mut State, snapshot: &BackendSnapshot) -> Result<()> {
    state.history.check(&snapshot.at.history)?;
    lookup(&state.connection, "storage_snapshot_leases", snapshot)?;
    state.connection.execute("DELETE FROM storage_snapshot_leases WHERE token=?1", [&snapshot.token]).map_err(db)?;
    compound::reclaim(state)
}

pub(super) fn frontier(state: &State, after: &StorageCursor, through: &StorageCursor) -> Result<StorageCursor> {
    state.check()?;
    state.history.check(&after.history)?;
    state.history.check(&through.history)?;
    if after.sequence > through.sequence || through.sequence > state.cursor()?.sequence {
        return Err(invalid("Watch Rows: Select an Ordered Durable Range"));
    }
    let floor = meta(&state.connection, "floor")?;
    if through.sequence < floor {
        return Err(Error::Gap {
            requested: after.sequence,
            available: floor,
        });
    }
    let lost: Option<i64> = state.connection.query_row(
        "SELECT min(position) FROM (SELECT position FROM storage_object_deletions UNION ALL SELECT position FROM storage_scope_deletions UNION ALL SELECT position FROM storage_object_gaps) WHERE position>?1 AND position<=?2",
        params![integer(after.sequence)?, integer(through.sequence)?], |row| row.get(0),
    ).map_err(db)?;
    if let Some(available) = lost {
        return Err(Error::Gap {
            requested: after.sequence,
            available: available as u64,
        });
    }
    let first: Option<i64> = state
        .connection
        .query_row("SELECT first FROM storage_compound_ranges WHERE first<=?1 AND last>?1", [integer(through.sequence)?], |row| row.get(0))
        .optional()
        .map_err(db)?;
    let sequence = first.map_or(through.sequence, |first| first as u64 - 1);
    Ok(StorageCursor { history: state.history.clone(), sequence })
}

/* === Private Functions === */

fn lookup(connection: &Connection, table: &str, snapshot: &BackendSnapshot) -> Result<Option<Lease>> {
    let image = connection
        .query_row(&format!("SELECT position,generation,scope FROM {table} WHERE token=?1"), [&snapshot.token], |row| {
            Ok(Lease {
                position: unsigned(row, 0)?,
                generation: unsigned(row, 1)?,
                scope: row.get(2)?,
            })
        })
        .optional()
        .map_err(db)?;
    if image.as_ref().is_some_and(|image| image.position != snapshot.at.sequence) {
        return Err(invalid("Read Snapshot: Keep Its Original Position and Handle"));
    }
    Ok(image)
}

fn lease(connection: &Connection, at: &StorageCursor, generation: u64, scope: Option<&str>) -> Result<BackendSnapshot> {
    let token = Uuid::now_v7().as_bytes().to_vec();
    connection
        .execute(
            "INSERT INTO storage_snapshot_leases VALUES(?1,?2,?3,?4)",
            params![token, integer(at.sequence)?, integer(generation)?, scope],
        )
        .map_err(db)?;
    Ok(BackendSnapshot { at: at.clone(), token })
}

/* === Tests === */
