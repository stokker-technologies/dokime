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

pub use crate::storage::sqlite::history::Key;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;

use crate::storage::Error;

use dokime_domain::model::query::{page::ExecutionSummary, selection::ExecutionSelection};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct State(crate::storage::sqlite::State);

pub struct Read {
    pub revision: i64,
    pub cut: Option<Cut>,
    pub after: Option<Key>,
    pub rows: u32,
    pub bytes: u64,
}

pub struct Page {
    pub rows: Vec<ExecutionSummary>,
    pub next: Option<Key>,
}

pub struct Tokens {
    pub key: [u8; 32],
    pub maximum: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Tokens {
    pub fn encode(&self, cut: &Cut, operation: &str) -> Result<String, Box<Failure>> {
        super::Tokens { key: self.key, maximum: self.maximum }.encode(&Token::Snapshot(cut.clone()), operation)
    }
    pub fn decode(&self, token: &str, operation: &str) -> Result<Cut, Box<Failure>> {
        match (super::Tokens { key: self.key, maximum: self.maximum }).decode(token, operation)? {
            Token::Snapshot(cut) => Ok(cut),
            _ => Err(unavailable(operation)),
        }
    }
}

impl std::ops::Deref for State {
    type Target = crate::storage::sqlite::State;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl State {
    pub async fn open(root: &std::path::Path) -> Result<Self, Error> {
        let state = crate::storage::sqlite::State::open(root).await?;
        state.data_open(None).await?;
        Ok(Self(state))
    }
    pub async fn snapshot_capture(
        &self, scope: Scope, at: Option<StorageCursor>, metadata: Option<u64>, expires: Timestamp, contracts: Vec<u8>, journal: Option<u64>,
    ) -> Result<Option<Cut>, Error> {
        self.0.snapshot_capture(scope, at, metadata, expires, contracts, journal).await
    }
    pub async fn snapshot_key(&self) -> Result<[u8; 32], Error> {
        self.0.snapshot_key().await
    }
    pub async fn snapshot_available(&self, cut: Cut) -> Result<bool, Error> {
        self.0.snapshot_available(cut).await
    }
    pub async fn snapshot_contracts(&self, cut: Cut, maximum: usize) -> Result<Vec<u8>, Error> {
        self.0.snapshot_contracts(cut, maximum).await
    }
    pub async fn snapshot_members(&self, cut: Cut, after: String, limit: u32) -> Result<Option<Vec<String>>, Error> {
        self.0.snapshot_members(cut, after, limit).await
    }
    pub async fn history_executions(&self, selection: ExecutionSelection, read: Read) -> Result<Page, Error> {
        let read = crate::storage::sqlite::history::Read {
            revision: read.revision,
            cut: read.cut,
            after: read.after,
            rows: read.rows,
            bytes: read.bytes,
        };
        let page = self.0.history_executions(selection, read).await?;
        Ok(Page { rows: page.rows, next: page.next })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
