/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Revisioned Execution Values and Retained Read Positions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1::{History, Position, Receipt};
use prost::Message;
use rusqlite::{OptionalExtension, params};
use sha2::{Digest, Sha256};

// Local Imports
use crate::{Error, Evidence, Result, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct ResourceValue {
    pub payload: Vec<u8>,
    pub revision: Vec<u8>,
    pub resolved_at: Position,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn resource_value(&self, execution: &str, resource: &str, at: Option<&Position>) -> Result<ResourceValue> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let state: i32 = transaction
            .query_row("SELECT state FROM executions WHERE id=?1", [execution], |row| row.get(0))
            .optional()?
            .ok_or_else(|| Error::NotFound(format!("Get {resource}: Select an Existing Execution")))?;
        let frontier = crate::store::next_position(&transaction, execution)?.saturating_sub(1);
        let history = History {
            store: self.identity.to_string(),
            epoch: execution.into(),
        };
        let sequence = if let Some(at) = at {
            if at.history.as_ref() != Some(&history) || at.sequence > frontier {
                return Err(Error::Invalid(format!("Get {resource}: Use a Returned Position or the Execution Seal")));
            }
            let issued: bool = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM read_positions WHERE execution=?1 AND position=?2)",
                params![execution, at.sequence as i64],
                |row| row.get(0),
            )?;
            if !issued && !(state == 2 && at.sequence == frontier) {
                return Err(Error::Invalid(format!("Get {resource}: Use a Returned Position or the Execution Seal")));
            }
            at.sequence
        } else {
            frontier
        };
        let saved: Option<(i64, Vec<u8>)> = transaction.query_row(
            "SELECT position,payload FROM resource_values WHERE execution=?1 AND resource=?2 AND position<=?3 ORDER BY position DESC LIMIT 1",
            params![execution, resource, sequence as i64], |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional()?;
        transaction.execute("INSERT OR IGNORE INTO read_positions VALUES (?1,?2)", params![execution, sequence as i64])?;
        transaction.commit()?;
        let (revision, payload) = saved.unwrap_or_default();
        Ok(ResourceValue {
            payload,
            revision: (revision as u64).to_be_bytes().to_vec(),
            resolved_at: Position { history: Some(history), sequence },
        })
    }

    pub fn set_resource_value(
        &self, execution: &str, resource: &str, expected_revision: &[u8], request_id: &str, mut evidence: Evidence,
    ) -> Result<(ResourceValue, Receipt)> {
        if request_id.is_empty() || expected_revision.len() != 8 {
            return Err(Error::Invalid(format!("Set {resource}: Supply a Request ID and the Revision from Get")));
        }
        let fingerprint = Sha256::digest([expected_revision, evidence.payload.as_slice()].concat()).to_vec();
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let replay: Option<(Vec<u8>, i64)> = transaction
            .query_row(
                "SELECT fingerprint,position FROM resource_requests WHERE execution=?1 AND resource=?2 AND request=?3",
                params![execution, resource, request_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let position = if let Some((previous, position)) = replay {
            if previous != fingerprint {
                return Err(Error::Conflict(format!("Set {resource}: Use a New Request ID for a Changed Value")));
            }
            position as u64
        } else {
            crate::store::require_open(&transaction, execution)?;
            let current: i64 = transaction.query_row(
                "SELECT COALESCE(MAX(position),0) FROM resource_values WHERE execution=?1 AND resource=?2",
                params![execution, resource],
                |row| row.get(0),
            )?;
            if (current as u64).to_be_bytes() != expected_revision {
                return Err(Error::Conflict(format!("Set {resource}: Get Again and Retry the Revision")));
            }
            let position = crate::store::next_position(&transaction, execution)?;
            evidence.position = position;
            transaction.execute("INSERT INTO evidence VALUES (?1,?2,?3)", params![execution, position as i64, evidence.encode_to_vec()])?;
            transaction.execute(
                "INSERT INTO resource_values VALUES (?1,?2,?3,?4)",
                params![execution, resource, position as i64, evidence.payload],
            )?;
            transaction.execute(
                "INSERT INTO resource_requests VALUES (?1,?2,?3,?4,?5)",
                params![execution, resource, request_id, fingerprint, position as i64],
            )?;
            position
        };
        transaction.execute("INSERT OR IGNORE INTO read_positions VALUES (?1,?2)", params![execution, position as i64])?;
        transaction.commit()?;
        drop(connection);
        self.changed.send_modify(|revision| *revision += 1);
        let resolved_at = Position {
            history: Some(History {
                store: self.identity.to_string(),
                epoch: execution.into(),
            }),
            sequence: position,
        };
        let value = self.resource_value(execution, resource, Some(&resolved_at))?;
        let receipt = Receipt {
            request_id: request_id.into(),
            accepted_at: Some(resolved_at.clone()),
            durable_at: Some(resolved_at),
            completed: true,
        };
        Ok((value, receipt))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
