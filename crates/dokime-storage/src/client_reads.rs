/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Client Evidence Reads

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;
use rusqlite::params;

// Local Imports
use crate::{Evidence, Execution, ExecutionCut, ExecutionState, Result, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    // Host Startup Verifies Seals; Client Reads Capture Metadata Without Rehashing Streams.
    pub fn metadata(&self, execution: &str) -> Result<ExecutionCut> {
        let connection = self.lock()?;
        let (state, payload, seal): (i32, Vec<u8>, Vec<u8>) =
            connection.query_row("SELECT state,payload,seal FROM executions WHERE id=?1", [execution], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;
        let state = match state {
            0 => ExecutionState::Open,
            1 => ExecutionState::Closed,
            2 => ExecutionState::Sealed,
            _ => return Err(crate::Error::Incompatible("Invalid Execution State".into())),
        };
        Ok(ExecutionCut {
            execution: Execution::decode(payload.as_slice())?,
            state,
            seal,
            frontier: crate::store::saved_position(&connection, execution)?,
        })
    }
    pub fn subscribe(&self) -> tokio::sync::watch::Receiver<u64> {
        self.changed.subscribe()
    }
    pub fn execution_evidence_frontier(&self, execution: &str) -> Result<u64> {
        Ok(self.lock()?.query_row(
            "SELECT COALESCE((SELECT rowid FROM evidence WHERE execution=?1 ORDER BY position DESC LIMIT 1),0)",
            [execution],
            |row| row.get::<_, i64>(0),
        )? as u64)
    }
    pub fn cycle_evidence_frontier(&self, cycle: &str) -> Result<u64> {
        Ok(self.lock()?.query_row("SELECT COALESCE(MAX((SELECT rowid FROM evidence WHERE execution=x.id ORDER BY position DESC LIMIT 1)),0) FROM executions x WHERE cycle=?1",[cycle],|row|row.get::<_,i64>(0))? as u64)
    }
    pub fn evidence_frontier(&self) -> Result<u64> {
        Ok(self.lock()?.query_row("SELECT COALESCE(MAX(rowid),0) FROM evidence", [], |row| row.get::<_, i64>(0))? as u64)
    }
    pub fn evidence_entries(
        &self, after: u64, through: u64, limit: std::num::NonZeroUsize, latest: bool,
    ) -> Result<Vec<(u64, String, Evidence)>> {
        let connection = self.lock()?;
        let order = if latest { "DESC" } else { "ASC" };
        let mut query = connection.prepare(&format!(
            "SELECT rowid,execution,payload FROM evidence WHERE rowid>?1 AND rowid<=?2 ORDER BY rowid {order} LIMIT ?3"
        ))?;
        let rows = query.query_map(params![after as i64, through as i64, limit.get() as i64], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, Vec<u8>>(2)?))
        })?;
        rows.map(|row| {
            let (position, execution, bytes) = row?;
            Ok((position as u64, execution, Evidence::decode(bytes.as_slice())?))
        })
        .collect()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
