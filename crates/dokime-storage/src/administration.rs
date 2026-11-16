/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Product Administration

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;
use rusqlite::{OptionalExtension, params};

// Local Imports
use crate::{Result, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
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
