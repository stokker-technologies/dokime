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
use std::collections::BTreeSet;
// Third Party Imports
use rusqlite::{Connection, params};
// Local Imports
use super::{Error, State};

/* === Constants === */
pub(super) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_sources (
    identifier TEXT PRIMARY KEY, adapter TEXT NOT NULL,
    priority INTEGER NOT NULL UNIQUE, enabled INTEGER NOT NULL CHECK(enabled IN (0,1))
);
INSERT OR IGNORE INTO storage_sources VALUES ('local', 'local', 0, 1);
";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub identifier: String,
    pub adapter: String,
    pub priority: i64,
    pub enabled: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl State {
    pub async fn register_source(&self, identifier: &str, adapter: &str) -> Result<(), Error> {
        if identifier.is_empty()
            || adapter.is_empty()
            || (identifier == "local") != (adapter == "local")
        {
            return Err(invalid("invalid storage source identity"));
        }
        let identifier = identifier.to_owned();
        let adapter = adapter.to_owned();
        let matched = self
            .call(move |connection| {
                let transaction = connection.transaction()?;
                transaction.execute(
                    "INSERT OR IGNORE INTO storage_sources
                SELECT ?1, ?2, COALESCE(MAX(priority), -1) + 1, 1 FROM storage_sources",
                    params![identifier, adapter],
                )?;
                let matched = transaction.query_row(
                    "SELECT adapter = ?2 FROM storage_sources WHERE identifier = ?1",
                    params![identifier, adapter],
                    |row| row.get::<_, bool>(0),
                )?;
                if !matched {
                    return Ok(false);
                }
                transaction.commit()?;
                Ok(true)
            })
            .await?;
        if !matched {
            return Err(invalid("a source cannot change adapter identity"));
        }
        Ok(())
    }

    pub async fn sources(&self) -> Result<Vec<Source>, Error> {
        self.call(|connection| sources(connection)).await
    }

    pub async fn reorder_sources(&self, identifiers: Vec<String>) -> Result<(), Error> {
        let applied = self
            .call(move |connection| {
                let transaction = connection.transaction()?;
                let current = sources(&transaction)?;
                let expected: BTreeSet<_> =
                    current.iter().map(|source| &source.identifier).collect();
                let supplied: BTreeSet<_> = identifiers.iter().collect();
                if supplied != expected || supplied.len() != identifiers.len() {
                    return Ok(false);
                }
                let offset = current
                    .iter()
                    .map(|source| source.priority)
                    .max()
                    .unwrap_or(0)
                    + 1;
                transaction.execute(
                    "UPDATE storage_sources SET priority = priority + ?1",
                    [offset],
                )?;
                for (priority, identifier) in identifiers.iter().enumerate() {
                    transaction.execute(
                        "UPDATE storage_sources SET priority = ?1 WHERE identifier = ?2",
                        params![priority as i64, identifier],
                    )?;
                }
                transaction.commit()?;
                Ok(true)
            })
            .await?;
        if !applied {
            return Err(invalid(
                "source order must contain every source exactly once",
            ));
        }
        Ok(())
    }

    pub async fn set_source_enabled(&self, identifier: &str, enabled: bool) -> Result<(), Error> {
        if identifier == "local" && !enabled {
            return Err(invalid("local storage cannot be disabled"));
        }
        let identifier = identifier.to_owned();
        let changed = self
            .call(move |connection| {
                connection.execute(
                    "UPDATE storage_sources SET enabled = ?2 WHERE identifier = ?1",
                    params![identifier, enabled],
                )
            })
            .await?;
        if changed == 0 {
            return Err(invalid("unknown storage source"));
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn invalid(message: &str) -> Error {
    Error::Catalog(message.into())
}

fn sources(connection: &Connection) -> rusqlite::Result<Vec<Source>> {
    connection
        .prepare(
            "SELECT identifier, adapter, priority, enabled FROM storage_sources ORDER BY priority",
        )?
        .query_map([], |row| {
            Ok(Source {
                identifier: row.get(0)?,
                adapter: row.get(1)?,
                priority: row.get(2)?,
                enabled: row.get(3)?,
            })
        })?
        .collect()
}

/* === Tests === */
