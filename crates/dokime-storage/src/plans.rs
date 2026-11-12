/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Registered Execution Plans

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use prost::Message;
use rusqlite::{OptionalExtension, params};
use crate::{Error, Result, Store};
use dokime_protocol::v1;

// Standard Library Imports

// Third Party Imports

// Local Imports


/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn registration(&self, key: &str, plan: &[u8]) -> Result<Option<v1::PreviewSelectionReply>> {
        let existing: Option<(Vec<u8>, Vec<u8>)> = self.lock()?.query_row("SELECT plan,payload FROM registrations WHERE key=?1", [key], |row| Ok((row.get(0)?, row.get(1)?))).optional()?;
        let Some((previous, payload)) = existing else { return Ok(None); };
        if previous != plan { return Err(Error::Invalid("Register Selection: Reuse the Key with Its Original Plan or Choose a New Key".into())); }
        Ok(Some(v1::PreviewSelectionReply::decode(payload.as_slice())?))
    }

    pub fn register_cases(&self, key: Option<&str>, plan: &[u8], cycle: &str, mut reply: v1::PreviewSelectionReply) -> Result<v1::PreviewSelectionReply> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        if let Some(key) = key {
            let existing: Option<(Vec<u8>, Vec<u8>)> = transaction.query_row("SELECT plan,payload FROM registrations WHERE key=?1", [key], |row| Ok((row.get(0)?, row.get(1)?))).optional()?;
            if let Some((previous, payload)) = existing {
                if previous != plan { return Err(Error::Invalid("Register Selection: Choose a New Key for the Changed Plan".into())); }
                return Ok(v1::PreviewSelectionReply::decode(payload.as_slice())?);
            }
        }
        transaction.execute("INSERT INTO registrations(key,plan,payload) VALUES (?1,?2,X'')", params![key,plan])?;
        let registration = transaction.last_insert_rowid();
        for (index, case) in reply.cases.iter_mut().enumerate() {
            case.case_id = format!("case-{registration}-{index}");
            transaction.execute("INSERT INTO cases VALUES (?1,?2,?3)", params![case.case_id,cycle,case.encode_to_vec()])?;
        }
        transaction.execute("UPDATE registrations SET payload=?1 WHERE id=?2", params![reply.encode_to_vec(),registration])?;
        transaction.commit()?;
        Ok(reply)
    }

    pub fn planned_case(&self, identifier: &str) -> Result<v1::PlannedCase> {
        let payload: Vec<u8> = self.lock()?.query_row("SELECT payload FROM cases WHERE id=?1", [identifier], |row| row.get(0))?;
        Ok(v1::PlannedCase::decode(payload.as_slice())?)
    }

    pub fn planned_cases(&self, cycle: &str) -> Result<Vec<v1::PlannedCase>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT payload FROM cases WHERE cycle=?1 ORDER BY rowid")?;
        let payloads = statement.query_map([cycle], |row| row.get::<_, Vec<u8>>(0))?.collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(payloads.iter().map(|payload| v1::PlannedCase::decode(payload.as_slice())).collect::<std::result::Result<_, _>>()?)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
