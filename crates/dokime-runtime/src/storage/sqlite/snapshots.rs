/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Fixed Local Catalog And Evidence Cuts Without Retained Reader Transactions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use harmos::storage::StorageCursor;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// Local Imports
use super::{
    Error, State,
    data::{position, saved, sequence},
};
use dokime_domain::model::identity::wire::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cut {
    pub source: String,
    pub backend: String,
    pub scope: Scope,
    pub at: StorageCursor,
    pub metadata: u64,
    pub journal: Option<u64>,
    pub catalog: i64,
    pub retention: u64,
    pub expires: Timestamp,
    pub contracts: String,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    History,
    Execution(String),
    Cycle(String),
}

/* === Traits === */

/* === Implementations === */

impl State {
    pub(crate) async fn snapshot_key(&self) -> Result<[u8; 32], Error> {
        self.read(|connection| {
            Ok((|| {
                let bytes: Vec<u8> = connection.query_row("SELECT secret FROM query_token_key WHERE singleton=1", [], |row| row.get(0))?;
                bytes.try_into().map_err(|_| Error::Corrupt("invalid query signing key"))
            })())
        })
        .await?
    }

    pub(crate) async fn snapshot_capture(
        &self, scope: Scope, at: Option<StorageCursor>, metadata: Option<u64>, expires: Timestamp, contracts: Vec<u8>, journal: Option<u64>,
    ) -> Result<Option<Cut>, Error> {
        let identity = URL_SAFE_NO_PAD.encode(Sha256::digest(&contracts));
        self.call(move |connection| {
            Ok((|| {
                let transaction = connection.transaction()?;
                let head = saved(&transaction)?;
                let durable = at.is_none();
                let at = at.unwrap_or(StorageCursor {
                    history: head.history.clone(),
                    sequence: head.sequence.min(metadata.unwrap_or(head.sequence)),
                });
                let metadata = if durable { at.sequence } else { metadata.unwrap_or(at.sequence) };
                let catalog = transaction.query_row("SELECT revision FROM query_history_head", [], |row| row.get(0))?;
                transaction.execute("INSERT OR IGNORE INTO query_contract_cuts VALUES(?1,?2)", params![identity, contracts])?;
                let cut = Cut {
                    backend: head.history.store.to_string(),
                    source: "local".into(),
                    scope,
                    at,
                    metadata,
                    journal,
                    catalog,
                    retention: generation(&transaction)?,
                    expires,
                    contracts: identity,
                };
                if !available(&transaction, &cut)? {
                    return Ok(None);
                }
                super::commit(transaction)?;
                Ok(Some(cut))
            })())
        })
        .await?
    }

    pub(crate) async fn snapshot_contracts(&self, cut: Cut, maximum: usize) -> Result<Vec<u8>, Error> {
        self.read(move |connection| {
            Ok((|| {
                if !available(connection, &cut)? {
                    return Err(Error::QuerySnapshotUnavailable);
                }
                let mut statement = connection.prepare("SELECT body FROM query_contract_cuts WHERE identity=?1")?;
                let mut rows = statement.query([&cut.contracts])?;
                let row = rows.next()?.ok_or(Error::QuerySnapshotUnavailable)?;
                let body = row.get_ref(0)?.as_blob().map_err(|_| Error::Corrupt("invalid snapshot contracts"))?;
                if body.len() > maximum {
                    return Err(Error::Data("snapshot contracts exceed schema budget"));
                }
                if URL_SAFE_NO_PAD.encode(Sha256::digest(body)) != cut.contracts {
                    return Err(Error::Corrupt("snapshot contract digest differs"));
                }
                Ok(body.to_vec())
            })())
        })
        .await?
    }

    pub(crate) async fn snapshot_available(&self, cut: Cut) -> Result<bool, Error> {
        self.read(move |connection| Ok(available(connection, &cut))).await?
    }

    pub(crate) async fn snapshot_members(&self, cut: Cut, after: String, limit: u32) -> Result<Option<Vec<String>>, Error> {
        self.read(move |connection| {
            Ok((|| {
                if !available(connection, &cut)? {
                    return Ok(None);
                }
                let Scope::Cycle(cycle) = &cut.scope else {
                    return Err(Error::Data("member paging requires a cycle cut"));
                };
                let mut statement = connection.prepare(
                    "SELECT m.execution FROM query_members m
                WHERE m.cycle=?1 AND m.position=(SELECT h.position FROM query_history h
                    WHERE h.kind='cycle' AND h.id=?1 AND h.revision<=?2 AND h.position<=?3 ORDER BY h.revision DESC LIMIT 1)
                AND m.execution>?4 ORDER BY m.execution LIMIT ?5",
                )?;
                let members: Vec<String> = statement
                    .query_map(params![cycle, cut.catalog, position(cut.metadata), after, limit], |row| row.get(0))?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                for execution in &members {
                    if object_frontier(connection, &cut, execution)?.is_none() {
                        return Err(Error::QuerySnapshotUnavailable);
                    }
                }
                Ok(Some(members))
            })())
        })
        .await?
    }
}

impl Cut {
    pub fn compatible(&self, scope: &Scope) -> bool {
        self.scope == *scope || self.scope == Scope::History
    }
}

/* === Public Functions === */

pub(super) fn initialize(connection: &Connection) -> Result<(), Error> {
    connection.execute_batch("CREATE TABLE IF NOT EXISTS query_contract_cuts(identity TEXT PRIMARY KEY,body BLOB NOT NULL) WITHOUT ROWID; CREATE TABLE IF NOT EXISTS query_token_key(singleton INTEGER PRIMARY KEY CHECK(singleton=1), secret BLOB NOT NULL CHECK(length(secret)=32), generation TEXT NOT NULL CHECK(length(generation)=16))")?;
    let key = rand::random::<[u8; 32]>();
    connection.execute("INSERT OR IGNORE INTO query_token_key VALUES(1,?1,'0000000000000000')", [key.as_slice()])?;
    Ok(())
}

pub(super) fn check_format(connection: &Connection) -> Result<(), Error> {
    connection
        .prepare("SELECT secret,generation FROM query_token_key LIMIT 0")
        .map_err(|_| Error::Incompatible("local catalog lacks the query token identity".into()))?;
    connection
        .prepare("SELECT identity,body FROM query_contract_cuts LIMIT 0")
        .map_err(|_| Error::Incompatible("local catalog lacks captured query contracts".into()))?;
    Ok(())
}

pub(crate) fn available(connection: &Connection, cut: &Cut) -> Result<bool, Error> {
    let head = saved(connection)?;
    if cut.source != "local"
        || cut.at.history != head.history
        || cut.retention != generation(connection)?
        || cut.at.sequence > head.sequence
        || cut.metadata > head.sequence
        || cut.catalog < 0
    {
        return Ok(false);
    }
    let contracts: bool =
        connection.query_row("SELECT EXISTS(SELECT 1 FROM query_contract_cuts WHERE identity=?1)", [&cut.contracts], |row| row.get(0))?;
    if !contracts {
        return Ok(false);
    }
    let current: i64 = connection.query_row("SELECT revision FROM query_history_head", [], |row| row.get(0))?;
    if cut.catalog > current {
        return Ok(false);
    }
    let (kind, id) = match &cut.scope {
        Scope::History => return Ok(true),
        Scope::Execution(id) => ("execution", id),
        Scope::Cycle(id) => ("cycle", id),
    };
    let entry: Option<String> = connection
        .query_row(
            "SELECT position FROM query_history WHERE kind=?1 AND id=?2 AND revision<=?3 AND position<=?4 ORDER BY revision DESC LIMIT 1",
            params![kind, id, cut.catalog, position(cut.metadata)],
            |row| row.get(0),
        )
        .optional()?;
    let Some(_entry) = entry else {
        return Ok(false);
    };
    // The Index Is A Projection, Never Proof That Deleted Authoritative Data Exists.
    let present = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM storage_data_entries WHERE kind='object' AND schema_id=?1 AND item=?2 AND retired_at IS NULL)",
        params![format!("dokime.object.{kind}"), id],
        |row| row.get(0),
    )?;
    Ok(present)
}

pub(crate) fn object_frontier(connection: &Connection, cut: &Cut, id: &str) -> Result<Option<u64>, Error> {
    // Work Projections Share The Catalog Cut But Are Not Native Object Anchors.
    let value:Option<String>=connection.query_row("SELECT position FROM storage_data_entries WHERE kind='object' AND schema_id='dokime.object.execution' AND item=?1 AND position<=?2 ORDER BY position DESC LIMIT 1",params![id,position(cut.metadata)],|row|row.get(0)).optional()?;
    value.as_deref().map(sequence).transpose()
}

pub(super) fn invalidate(connection: &Connection) -> Result<(), Error> {
    let next = generation(connection)?.checked_add(1).ok_or(Error::Data("query deletion generation exhausted"))?;
    connection.execute("UPDATE query_token_key SET generation=?1", [position(next)])?;
    Ok(())
}

/* === Private Functions === */

fn generation(connection: &Connection) -> Result<u64, Error> {
    let generation: String = connection.query_row("SELECT generation FROM query_token_key WHERE singleton=1", [], |row| row.get(0))?;
    sequence(&generation)
}

/* === Tests === */
