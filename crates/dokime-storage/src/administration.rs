/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Published Execution Reports

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;
use rusqlite::{OptionalExtension, params};

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
    pub fn administration_value(&self, key: &str, initial: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let connection = self.lock()?;
        let payload: Option<Vec<u8>> =
            connection.query_row("SELECT payload FROM administration WHERE id=?1", [key], |row| row.get(0)).optional()?;
        let revision: Option<Vec<u8>> = connection
            .query_row("SELECT payload FROM administration WHERE id=?1", [format!("revision:{key}")], |row| row.get(0))
            .optional()?;
        Ok((payload.unwrap_or_else(|| initial.to_vec()), revision.unwrap_or_else(|| 0_u64.to_be_bytes().to_vec())))
    }

    pub fn replace_administration_value(
        &self, key: &str, expected: &[u8], request_id: &str, fingerprint: &[u8], payload: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>, harmos_protocol::v1::Receipt)> {
        use harmos_protocol::v1::{History, Position, Receipt, Section};
        if request_id.is_empty() {
            return Err(Error::Invalid(format!("Set {key}: Supply a Request ID")));
        }
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let request_key = format!("request:{key}:{request_id}");
        let saved: Option<Vec<u8>> =
            transaction.query_row("SELECT payload FROM administration WHERE id=?1", [&request_key], |row| row.get(0)).optional()?;
        let result = if let Some(saved) = saved {
            let saved = Section::decode(saved.as_slice())?;
            if saved.r#type.as_bytes() != fingerprint {
                return Err(Error::Conflict(format!("Set {key}: Use a New Request ID for a Changed Value")));
            }
            Section::decode(saved.payload.as_slice())?
        } else {
            let revision_key = format!("revision:{key}");
            let revision: Option<Vec<u8>> =
                transaction.query_row("SELECT payload FROM administration WHERE id=?1", [&revision_key], |row| row.get(0)).optional()?;
            let revision = revision.unwrap_or_else(|| 0_u64.to_be_bytes().to_vec());
            if expected != revision {
                return Err(Error::Conflict(format!("Set {key}: Get Its Current Revision")));
            }
            let sequence = u64::from_be_bytes(revision.as_slice().try_into().map_err(|_| Error::Incompatible(format!("{key} Revision")))?)
                .checked_add(1)
                .ok_or_else(|| Error::Invalid(format!("Set {key}: Revision Exhausted")))?;
            let result = Section {
                r#type: sequence.to_string(),
                payload: payload.to_vec(),
            };
            let saved = Section {
                r#type: String::from_utf8(fingerprint.to_vec())
                    .map_err(|_| Error::Invalid(format!("Set {key}: Supply an ASCII Fingerprint")))?,
                payload: result.encode_to_vec(),
            };
            for (id, bytes) in
                [(key.to_string(), payload.to_vec()), (revision_key, sequence.to_be_bytes().to_vec()), (request_key, saved.encode_to_vec())]
            {
                transaction.execute(
                    "INSERT INTO administration VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",
                    params![id, bytes],
                )?;
            }
            result
        };
        let sequence = result.r#type.parse::<u64>().map_err(|_| Error::Incompatible(format!("{key} Request Receipt")))?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        let position = Position {
            history: Some(History {
                store: self.identity.to_string(),
                epoch: format!("value/{key}"),
            }),
            sequence,
        };
        Ok((
            result.payload,
            sequence.to_be_bytes().to_vec(),
            Receipt {
                request_id: request_id.into(),
                accepted_at: Some(position.clone()),
                durable_at: Some(position),
                completed: true,
            },
        ))
    }
    pub fn delete_administration(&self, key: &str) -> Result<()> {
        self.lock()?.execute("DELETE FROM administration WHERE id=?1", [key])?;
        Ok(())
    }
    pub fn administration<T: Message + Default>(&self, key: &str) -> Result<Option<T>> {
        let bytes: Option<Vec<u8>> =
            self.lock()?.query_row("SELECT payload FROM administration WHERE id=?1", [key], |row| row.get(0)).optional()?;
        bytes.map(|bytes| T::decode(bytes.as_slice()).map_err(Into::into)).transpose()
    }
    pub fn save_administration<T: Message>(&self, key: &str, value: &T) -> Result<()> {
        self.lock()?.execute(
            "INSERT INTO administration VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",
            params![key, value.encode_to_vec()],
        )?;
        Ok(())
    }
    pub fn save_plugin_administration(
        &self, entries: &dokime_protocol::client::PluginEntries, installations: &dokime_protocol::client::InstallationPage,
        executables: &dokime_protocol::v1::MapValue,
    ) -> Result<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        for (key, payload) in [
            ("plugin-entries", entries.encode_to_vec()),
            ("plugin-installations", installations.encode_to_vec()),
            ("managed-executables", executables.encode_to_vec()),
        ] {
            transaction.execute(
                "INSERT INTO administration VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",
                params![key, payload],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }
    pub fn compare_administration<T: Message>(&self, key: &str, expected: Option<&T>, value: &T) -> Result<bool> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let current: Option<Vec<u8>> =
            transaction.query_row("SELECT payload FROM administration WHERE id=?1", [key], |row| row.get(0)).optional()?;
        if current != expected.map(Message::encode_to_vec) {
            return Ok(false);
        }
        transaction.execute(
            "INSERT INTO administration VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",
            params![key, value.encode_to_vec()],
        )?;
        transaction.commit()?;
        Ok(true)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
