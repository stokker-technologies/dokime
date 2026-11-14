/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Issued Durable Positions for Shared Evidence Reads

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1::{History, Position};
use prost::Message;

// Local Imports
use crate::{Error, Result, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn evidence_position(&self, at: Option<&Position>) -> Result<Position> {
        let mut connection = self.lock()?;
        if self.cancelled.is_some() {
            return resolve(&connection, at);
        }
        let transaction = connection.transaction()?;
        let position = resolve(&transaction, at)?;
        transaction.execute("INSERT OR IGNORE INTO evidence_reads VALUES(?1)", [position.sequence as i64])?;
        transaction.commit()?;
        Ok(position)
    }

    pub fn retain_evidence_position(&self, position: &Position) -> Result<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let current = resolve(&transaction, None)?;
        if position.history != current.history || position.sequence > current.sequence {
            return Err(Error::Invalid("Read Records: A History Was Deleted; Resynchronize".into()));
        }
        transaction.execute("INSERT OR IGNORE INTO evidence_reads VALUES(?1)", [position.sequence as i64])?;
        transaction.commit()?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn resolve(connection: &rusqlite::Connection, at: Option<&Position>) -> Result<Position> {
    let bytes: Vec<u8> =
        connection.query_row("SELECT payload FROM administration WHERE id='evidence-read-history'", [], |row| row.get(0))?;
    let history = History::decode(bytes.as_slice())?;
    let frontier: i64 =
        connection.query_row("SELECT COALESCE((SELECT seq FROM sqlite_sequence WHERE name='evidence'),0)", [], |row| row.get(0))?;
    let sequence = u64::try_from(frontier).map_err(|_| Error::Invalid("Read Records: Restore Valid Evidence Sequencing".into()))?;
    if let Some(at) = at {
        let issued: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM evidence_reads WHERE position=?1)",
            [i64::try_from(at.sequence).unwrap_or(-1)],
            |row| row.get(0),
        )?;
        if at.history.as_ref() != Some(&history) || at.sequence > sequence || !issued {
            return Err(Error::Invalid("Read Records: Use a Returned Position; Resynchronize After History Deletion".into()));
        }
        return Ok(at.clone());
    }
    Ok(Position { history: Some(history), sequence })
}

/* === Tests === */
