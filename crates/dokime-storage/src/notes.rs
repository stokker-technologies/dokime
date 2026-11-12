/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Operator Notes

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;
use rusqlite::{OptionalExtension, params};

// Local Imports
use crate::{Error, Execution, Result, Store};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn create_note(&self, receipt: v1::NoteReceipt) -> Result<v1::NoteReceipt> {
        let note = receipt.note.as_ref().ok_or_else(|| Error::Invalid("Create Note: Supply Its Body".into()))?;
        let scope = note.scope.as_ref().ok_or_else(|| Error::Invalid("Create Note: Select Its Owner".into()))?;
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let existing: Option<Vec<u8>> =
            transaction.query_row("SELECT receipt FROM notes WHERE id=?1", [&note.note_id], |row| row.get(0)).optional()?;
        if let Some(bytes) = existing {
            let existing = v1::NoteReceipt::decode(bytes.as_slice())?;
            if existing.owner != receipt.owner
                || existing.note.as_ref().is_none_or(|existing| {
                    existing.scope != note.scope || existing.author != note.author || existing.message != note.message
                })
            {
                return Err(Error::AlreadyExists("Create Note: Use a New Mutation ID for Different Content or Ownership".into()));
            }
            return Ok(existing);
        }
        require_owner(&transaction, scope)?;
        transaction.execute(
            "INSERT INTO notes(id,cycle,execution,receipt,payload) VALUES(?1,?2,?3,?4,?5)",
            params![note.note_id, scope.cycle_id, scope.execution_id, receipt.encode_to_vec(), note.encode_to_vec()],
        )?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(receipt)
    }
    pub fn edit_note(&self, id: &str, author: &str, message: String) -> Result<v1::OperatorNote> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let bytes: Vec<u8> = transaction
            .query_row("SELECT payload FROM notes WHERE id=?1", [id], |row| row.get(0))
            .optional()?
            .ok_or_else(|| Error::NotFound("Edit Note: Select an Existing Note from List Notes".into()))?;
        let mut note = v1::OperatorNote::decode(bytes.as_slice())?;
        if note.author.as_deref() != Some(author) {
            return Err(Error::Permission("Edit Note: Sign In as Its Author".into()));
        }
        require_owner(&transaction, note.scope.as_ref().ok_or_else(|| Error::Incompatible("Missing Note Scope".into()))?)?;
        note.message = message;
        transaction.execute("UPDATE notes SET payload=?2 WHERE id=?1", params![id, note.encode_to_vec()])?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(note)
    }
    pub fn note_owner(&self, owner: &v1::AnnotationOwner) -> Result<v1::EvidenceScope> {
        let kind = owner.owner.as_ref().and_then(|owner| owner.kind.as_ref());
        let (cycle, execution) = match kind {
            Some(v1::annotation_owner_owner::Kind::CycleId(cycle)) => (cycle.as_str(), None),
            Some(v1::annotation_owner_owner::Kind::Execution(owner)) => (owner.cycle_id.as_str(), Some(owner.execution_id.as_str())),
            Some(v1::annotation_owner_owner::Kind::PlannedCase(_)) => {
                return Err(Error::Invalid("Notes: Select an Execution or Cycle; Planned Cases Were Replaced by Frozen Origins".into()));
            }
            None => return Err(Error::Invalid("Notes: Select an Execution or Cycle Owner".into())),
        };
        let connection = self.lock()?;
        let payload: Option<Vec<u8>> = connection
            .query_row(
                "SELECT payload FROM executions WHERE cycle=?1 AND (?2 IS NULL OR id=?2) LIMIT 1",
                params![cycle, execution],
                |row| row.get(0),
            )
            .optional()?;
        let execution = Execution::decode(
            payload.ok_or_else(|| Error::Invalid("Notes: Select an Existing Execution in Its Cycle".into()))?.as_slice(),
        )?;
        Ok(v1::EvidenceScope {
            cycle_id: Some(execution.cycle),
            execution_id: owner.owner.as_ref().and_then(|owner| match owner.kind.as_ref() {
                Some(v1::annotation_owner_owner::Kind::Execution(owner)) => Some(owner.execution_id.clone()),
                _ => None,
            }),
            ..Default::default()
        })
    }
    pub fn notes_page(
        &self, cycle: Option<&str>, execution: Option<&str>, context: bool, after: &str, limit: usize,
    ) -> Result<Vec<v1::OperatorNote>> {
        let connection = self.lock()?;
        let mut query = connection.prepare("SELECT payload FROM notes WHERE id>?1 AND (?2 IS NULL OR cycle=?2) AND (?3 IS NULL OR execution=?3 OR (?4 AND execution IS NULL)) ORDER BY id LIMIT ?5")?;
        let rows = query.query_map(params![after, cycle, execution, context, limit as i64], |row| row.get::<_, Vec<u8>>(0))?;
        rows.map(|row| Ok(v1::OperatorNote::decode(row?.as_slice())?)).collect()
    }
    pub fn notes_fingerprint(&self) -> Result<Vec<u8>> {
        use sha2::{Digest, Sha256};
        let connection = self.lock()?;
        let mut query = connection.prepare("SELECT payload FROM notes ORDER BY id")?;
        let mut digest = Sha256::new();
        for row in query.query_map([], |row| row.get::<_, Vec<u8>>(0))? {
            let bytes = row?;
            digest.update((bytes.len() as u64).to_le_bytes());
            digest.update(bytes);
        }
        Ok(digest.finalize().to_vec())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn require_owner(connection: &rusqlite::Connection, scope: &v1::EvidenceScope) -> Result<()> {
    let (count, open): (i64, i64) = connection.query_row(
        "SELECT COUNT(*),COALESCE(SUM(state=0),0) FROM executions WHERE cycle=?1 AND (?2 IS NULL OR id=?2)",
        params![scope.cycle_id, scope.execution_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    if count == 0 || open == 0 || scope.execution_id.is_some() && count != open {
        return Err(Error::Invalid("Write Note: Select an Open Owner; Closed Evidence and Published Reports Are Immutable".into()));
    }
    Ok(())
}

/* === Tests === */
