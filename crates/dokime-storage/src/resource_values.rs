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
    pub fn requested_evidence(
        &self, execution: &str, operation: &str, request_id: &str, fingerprint: &[u8],
    ) -> Result<Option<(Evidence, Receipt)>> {
        let connection = self.lock()?;
        let saved: Option<(Vec<u8>, i64)> = connection
            .query_row(
                "SELECT fingerprint,position FROM resource_requests WHERE execution=?1 AND resource=?2 AND request=?3",
                params![execution, operation, request_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((saved, sequence)) = saved else {
            return Ok(None);
        };
        if saved != fingerprint {
            return Err(Error::Conflict(format!("{operation}: Use a New Request ID for Changed Content")));
        }
        let payload: Vec<u8> = connection.query_row(
            "SELECT payload FROM evidence WHERE execution=?1 AND position=?2",
            params![execution, sequence],
            |row| row.get(0),
        )?;
        let position = Position {
            history: Some(History {
                store: self.identity.to_string(),
                epoch: execution.into(),
            }),
            sequence: sequence as u64,
        };
        Ok(Some((
            Evidence::decode(payload.as_slice())?,
            Receipt {
                request_id: request_id.into(),
                accepted_at: Some(position.clone()),
                durable_at: Some(position),
                completed: true,
            },
        )))
    }
    pub fn resource_value(&self, execution: &str, resource: &str, at: Option<&Position>) -> Result<ResourceValue> {
        let mut connection = self.lock()?;
        if self.cancelled.is_some() {
            return resolve_value(&connection, &self.identity, execution, resource, at);
        }
        let transaction = connection.transaction()?;
        let value = resolve_value(&transaction, &self.identity, execution, resource, at)?;
        transaction
            .execute("INSERT OR IGNORE INTO read_positions VALUES (?1,?2)", params![execution, value.resolved_at.sequence as i64])?;
        transaction.commit()?;
        Ok(value)
    }

    /// Read Immutable Frozen Descriptors at an Issued Execution Cut.
    pub fn execution_schemas(&self, execution: &str, at: Option<&Position>) -> Result<ResourceValue> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let mut value = resolve_value(&transaction, &self.identity, execution, "schemas", at)?;
        let payload: Vec<u8> = transaction.query_row("SELECT payload FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
        let execution = crate::Execution::decode(payload.as_slice())?;
        let catalog = harmos_protocol::v1::Catalog::decode(execution.catalog.as_slice())?;
        value.payload = catalog.types.unwrap_or_default().encode_to_vec();
        value.revision = Sha256::digest(&value.payload).to_vec();
        transaction.execute(
            "INSERT OR IGNORE INTO read_positions VALUES (?1,?2)",
            params![execution.identifier, value.resolved_at.sequence as i64],
        )?;
        transaction.commit()?;
        Ok(value)
    }

    pub fn retain_read_positions(&self, positions: &[Position]) -> Result<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        for position in positions {
            let history = position.history.as_ref().ok_or_else(|| Error::Invalid("Retain Read: Supply a History".into()))?;
            if history.store != self.identity.as_ref()
                || position.sequence > crate::store::next_position(&transaction, &history.epoch)?.saturating_sub(1)
            {
                return Err(Error::Invalid("Retain Read: Use a Position in This Storage History".into()));
            }
            transaction.execute("INSERT OR IGNORE INTO read_positions VALUES (?1,?2)", params![history.epoch, position.sequence as i64])?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn resource_events(&self, execution: &str, after: u64, through: u64, limit: std::num::NonZeroUsize) -> Result<Vec<Evidence>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT e.payload FROM resource_values v JOIN evidence e ON e.execution=v.execution AND e.position=v.position WHERE v.execution=?1 AND v.position>?2 AND v.position<=?3 ORDER BY v.position LIMIT ?4")?;
        let bytes = statement
            .query_map(params![execution, after as i64, through as i64, limit.get() as i64], |row| row.get::<_, Vec<u8>>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        bytes.iter().map(|bytes| Evidence::decode(bytes.as_slice()).map_err(Into::into)).collect()
    }

    pub fn attempt_start(&self, execution: &str, evidence: &Evidence) -> Result<Option<Evidence>> {
        let connection = self.lock()?;
        let bytes: Option<Vec<u8>> = connection.query_row("SELECT e.payload FROM attempts a JOIN evidence e ON e.execution=a.execution AND e.position=a.position WHERE a.execution=?1 AND a.step=?2 AND a.operation=?3 AND a.attempt=?4 AND a.position<=?5 ORDER BY a.position DESC LIMIT 1", params![execution, evidence.step, evidence.operation, evidence.attempt, evidence.position as i64], |row| row.get(0)).optional()?;
        bytes.map(|bytes| Evidence::decode(bytes.as_slice()).map_err(Into::into)).transpose()
    }

    pub fn execution_identifier_page(&self, cycle: Option<&str>, after: &str, maximum: std::num::NonZeroUsize) -> Result<Vec<String>> {
        let connection = self.lock()?;
        let sql = if cycle.is_some() {
            "SELECT id FROM executions WHERE cycle=?1 AND id>?2 ORDER BY id LIMIT ?3"
        } else {
            "SELECT id FROM executions WHERE ?1 IS NULL AND id>?2 ORDER BY id LIMIT ?3"
        };
        let mut statement = connection.prepare(sql)?;
        Ok(statement
            .query_map(params![cycle, after, maximum.get() as i64], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn set_resource_value(
        &self, execution: &str, resource: &str, expected_revision: &[u8], request_id: &str, mut evidence: Evidence,
    ) -> Result<(ResourceValue, Receipt)> {
        if request_id.is_empty() || expected_revision.len() != 8 {
            return Err(Error::Invalid(format!("Set {resource}: Supply a Request ID and the Revision from Get")));
        }
        let mut digest = Sha256::new();
        digest.update(expected_revision);
        if evidence.kind == "traceability" {
            let mut value = dokime_protocol::v1::TraceabilityEntry::decode(evidence.payload.as_slice())?;
            let attributes = value.metadata.as_mut().map(|metadata| std::mem::take(&mut metadata.attributes)).unwrap_or_default();
            let payload = value.encode_to_vec();
            digest.update((payload.len() as u64).to_be_bytes());
            digest.update(payload);
            for (key, value) in attributes.into_iter().collect::<std::collections::BTreeMap<_, _>>() {
                digest.update((key.len() as u64).to_be_bytes());
                digest.update(key.as_bytes());
                let payload = value.encode_to_vec();
                digest.update((payload.len() as u64).to_be_bytes());
                digest.update(payload);
            }
        } else {
            digest.update(&evidence.payload);
        }
        let fingerprint = digest.finalize().to_vec();
        self.flush_samples()?;
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
            transaction.execute(
                "INSERT INTO evidence(execution,position,payload) VALUES (?1,?2,?3)",
                params![execution, position as i64, evidence.encode_to_vec()],
            )?;
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

fn resolve_value(
    connection: &rusqlite::Connection, identity: &str, execution: &str, resource: &str, at: Option<&Position>,
) -> Result<ResourceValue> {
    let state: i32 = connection
        .query_row("SELECT state FROM executions WHERE id=?1", [execution], |row| row.get(0))
        .optional()?
        .ok_or_else(|| Error::NotFound(format!("Get {resource}: Select an Existing Execution")))?;
    let frontier = crate::store::saved_position(connection, execution)?;
    let history = History {
        store: identity.to_owned(),
        epoch: execution.into(),
    };
    let sequence = if let Some(at) = at {
        if at.history.as_ref() != Some(&history) || at.sequence > frontier {
            return Err(Error::Invalid(format!("Get {resource}: Use a Returned Position or the Execution Seal")));
        }
        let issued: bool = connection.query_row(
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
    let saved: Option<(i64, Vec<u8>)> = connection.query_row(
            "SELECT position,payload FROM resource_values WHERE execution=?1 AND resource=?2 AND position<=?3 ORDER BY position DESC LIMIT 1",
            params![execution, resource, sequence as i64], |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional()?;
    let (revision, payload) = saved.unwrap_or_default();
    Ok(ResourceValue {
        payload,
        revision: (revision as u64).to_be_bytes().to_vec(),
        resolved_at: Position { history: Some(history), sequence },
    })
}

/* === Tests === */
