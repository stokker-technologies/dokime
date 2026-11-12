/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Published Execution Reports

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

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
    pub fn administration<T: Message + Default>(&self, key: &str) -> Result<Option<T>> {
        let bytes: Option<Vec<u8>> = self.lock()?.query_row("SELECT payload FROM administration WHERE id=?1", [key], |row| row.get(0)).optional()?;
        bytes.map(|bytes| T::decode(bytes.as_slice()).map_err(Into::into)).transpose()
    }
    pub fn save_administration<T: Message>(&self, key: &str, value: &T) -> Result<()> {
        self.lock()?.execute("INSERT INTO administration VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload", params![key, value.encode_to_vec()])?;
        Ok(())
    }
    pub fn update_preferences(&self, update: dokime_protocol::v1::UpdateSettings, initial: &dokime_protocol::v1::SettingsSnapshot) -> Result<dokime_protocol::v1::SettingsSnapshot> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let bytes: Option<Vec<u8>> = transaction.query_row("SELECT payload FROM administration WHERE id='preferences'", [], |row| row.get(0)).optional()?;
        let current = bytes.map(|bytes| dokime_protocol::v1::SettingsSnapshot::decode(bytes.as_slice())).transpose()?.unwrap_or_else(|| initial.clone());
        if current.revision != update.expected_revision { return Err(Error::Invalid("Save Settings: Refresh Settings After Another Client's Edit".into())); }
        let snapshot = dokime_protocol::v1::SettingsSnapshot { revision: current.revision.checked_add(1).ok_or_else(|| Error::Invalid("Save Settings: Revision Exhausted; Back Up and Choose a Fresh Root".into()))?, settings: update.settings };
        transaction.execute("INSERT INTO administration VALUES ('preferences',?1) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload", [snapshot.encode_to_vec()])?;
        transaction.commit()?;
        Ok(snapshot)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
