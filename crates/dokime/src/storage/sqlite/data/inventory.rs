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

// Third Party Imports
use rusqlite::{Connection, params, types::Type};

// Local Imports
use super::{SavedData, blob, entry::Encoded, position, sequence};
use crate::storage::sqlite::{Error, query};

/* === Constants === */
pub(super) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_data_inventory (
    position TEXT PRIMARY KEY CHECK(length(position)=16),
    kind TEXT NOT NULL CHECK(kind IN ('schema','object','record','stream','file')),
    digest BLOB NOT NULL CHECK(length(digest)=32),
    reclaimed INTEGER NOT NULL CHECK(reclaimed IN (0,1)),
    CHECK(reclaimed=0 OR kind='object')
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_data_origin (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    complete_after TEXT NOT NULL CHECK(length(complete_after)=16)
);
";
const PAGE_ROWS: usize = 64;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn confirm(connection: &Connection, entry: &Encoded) -> Result<(), Error> {
    let mut statement = connection
        .prepare("SELECT kind,digest,reclaimed FROM storage_data_inventory WHERE position=?1")?;
    let mut rows = statement.query([position(entry.position)])?;
    let row = rows
        .next()?
        .ok_or(Error::Corrupt("position retry lacks admission inventory"))?;
    if text(row, 0)? != entry.kind
        || blob(row, 1)? != entry.digest.as_slice()
        || row.get::<_, i64>(2)? != 0
    {
        return Err(Error::Corrupt(
            "position retry differs from admission inventory",
        ));
    }
    Ok(())
}

pub(super) fn verify(connection: &Connection, cut: &SavedData) -> Result<(), Error> {
    let origin = {
        let mut statement = connection
            .prepare("SELECT complete_after FROM storage_data_origin WHERE singleton=1")?;
        let mut rows = statement.query([])?;
        let row = rows
            .next()?
            .ok_or(Error::Corrupt("missing inventory origin"))?;
        sequence(text(row, 0)?)?
    };
    if origin > cut.lost_objects_through || origin > cut.sequence {
        return Err(Error::Corrupt("inventory origin exceeds saved coverage"));
    }
    let zero: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM storage_data_inventory WHERE position=?1)
            OR EXISTS(SELECT 1 FROM storage_data_entries WHERE position=?1)",
        [position(0)],
        |row| row.get(0),
    )?;
    if zero {
        return Err(Error::Corrupt("position zero has saved data"));
    }
    let mut statement = connection.prepare(
        "SELECT inventory.position,inventory.kind,inventory.digest,inventory.reclaimed,
            entries.position,entries.kind,entries.digest
         FROM storage_data_inventory AS inventory
         LEFT JOIN storage_data_entries AS entries ON entries.position=inventory.position
         WHERE inventory.position>?1 ORDER BY inventory.position LIMIT ?2",
    )?;
    let mut after = 0;
    let mut complete = origin;
    loop {
        query::checkpoint()?;
        let mut rows = statement.query(params![position(after), PAGE_ROWS as i64])?;
        let mut count = 0;
        while let Some(row) = rows.next()? {
            query::checkpoint()?;
            let index = text(row, 0)?;
            let at = sequence(index)?;
            if at == 0 || at <= after || at > cut.sequence {
                return Err(Error::Corrupt("inventory position exceeds saved history"));
            }
            if at > origin {
                if complete.checked_add(1) != Some(at) {
                    return Err(Error::Corrupt("admission inventory has a gap"));
                }
                complete = at;
            }
            let kind = text(row, 1)?;
            if !matches!(kind, "schema" | "object" | "record" | "stream" | "file") {
                return Err(Error::Corrupt("inventory kind is unsupported"));
            }
            let digest = blob(row, 2)?;
            if digest.len() != 32 {
                return Err(Error::Corrupt("inventory digest has invalid width"));
            }
            let reclaimed: i64 = row.get(3)?;
            let present = row.get_ref(4)?.data_type() != Type::Null;
            match reclaimed {
                0 if present => {
                    if text(row, 5)? != kind || blob(row, 6)? != digest {
                        return Err(Error::Corrupt(
                            "saved entry differs from admission inventory",
                        ));
                    }
                }
                1 if !present && kind == "object" && at <= cut.lost_objects_through => {}
                _ => {
                    return Err(Error::Corrupt(
                        "saved admission lacks exact reclamation proof",
                    ));
                }
            }
            after = at;
            count += 1;
        }
        if count < PAGE_ROWS {
            break;
        }
    }
    if complete != cut.sequence {
        return Err(Error::Corrupt(
            "admission inventory ends before saved frontier",
        ));
    }
    // Check The Reverse Membership In Bounded Primary-Key Pages
    let mut statement = connection.prepare(
        "SELECT entries.position,inventory.position
         FROM storage_data_entries AS entries
         LEFT JOIN storage_data_inventory AS inventory ON inventory.position=entries.position
         WHERE entries.position>?1 ORDER BY entries.position LIMIT ?2",
    )?;
    let mut after = 0;
    loop {
        query::checkpoint()?;
        let mut rows = statement.query(params![position(after), PAGE_ROWS as i64])?;
        let mut count = 0;
        while let Some(row) = rows.next()? {
            query::checkpoint()?;
            let at = sequence(text(row, 0)?)?;
            if at == 0
                || at <= after
                || at > cut.sequence
                || row.get_ref(1)?.data_type() == Type::Null
            {
                return Err(Error::Corrupt("saved entry lacks admission inventory"));
            }
            after = at;
            count += 1;
        }
        if count < PAGE_ROWS {
            break;
        }
    }
    Ok(())
}

/* === Private Functions === */
fn text<'a>(row: &'a rusqlite::Row<'_>, column: usize) -> Result<&'a str, Error> {
    row.get_ref(column)?
        .as_str()
        .map_err(|_| Error::Corrupt("inventory index is not text"))
}

/* === Tests === */
