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
use harmos::MemorySize;
use harmos::builtins::host::{Catalog, CatalogKind, DiscoveredOperation};
use harmos::domain::Portable;
use harmos::operations::{OperationDeclaration, StreamItem};

// Local Imports
use super::{Client, Error as ClientError, Result as ClientResult};
use dokime_domain::model::runtime::wire::Health;
use dokime_domain::model::runtime::wire::Identity;
use dokime_operations::runtime::GetHealth;
use dokime_operations::runtime::GetIdentity;

/* === Constants === */

const CATALOG_BYTES: u64 = 16_777_216;
const CATALOG_PAGE_BYTES: u64 = 4_194_304;
const CATALOG_PAGE_ROWS: u64 = 64;
const CATALOG_ROWS: u64 = 4_096;

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = std::result::Result<T, Error>;
pub type Description = DiscoveredOperation;

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown operation: {0}")]
    Unknown(String),

    #[error("operation {0} has multiple providers; select an owner")]
    Ambiguous(String),

    #[error("operation {0} has no input stream")]
    InputStream(String),

    #[error("operation input exceeds its byte or frame limit")]
    InputTooLarge,

    #[error("operation catalog exceeds its client limit")]
    CatalogTooLarge,

    #[error("operation catalog is invalid")]
    Catalog,

    #[error(transparent)]
    Client(#[from] ClientError),
}

/* === Traits === */

/* === Implementations === */

impl Client {
    pub async fn operations(&self) -> Result<Vec<Description>> {
        let page_rows = CATALOG_PAGE_ROWS as usize;
        let page_bytes = CATALOG_PAGE_BYTES as usize;
        let deadline = self.timeout.map(|timeout| tokio::time::Instant::now() + timeout);
        let mut entries = Vec::new();
        let mut keys = BTreeSet::new();
        let mut cursors = BTreeSet::new();
        let mut after = None;
        let mut bytes = 0usize;
        loop {
            let request = self.unary::<Catalog>(Catalog {
                kind: CatalogKind::Operations,
                schema: None,
                after,
                rows: Some(page_rows),
                bytes: Some(page_bytes),
            });
            let page = match deadline {
                Some(deadline) => tokio::time::timeout_at(deadline, request)
                    .await
                    .map_err(|_| ClientError::Rpc(tonic::Status::deadline_exceeded("catalog deadline exceeded")))??,
                None => request.await?,
            };
            bytes = bytes.checked_add(page.entries.len()).ok_or(Error::CatalogTooLarge)?;
            if bytes > CATALOG_BYTES as usize {
                return Err(Error::CatalogTooLarge);
            }
            let batch: Vec<DiscoveredOperation> = serde_json::from_slice(&page.entries).map_err(|_| Error::Catalog)?;
            if batch.len() > page_rows || entries.len() + batch.len() > CATALOG_ROWS as usize {
                return Err(Error::CatalogTooLarge);
            }
            if batch.is_empty() && page.next.is_some() {
                return Err(Error::Catalog);
            }
            for entry in batch {
                let key = (entry.contract.operation().id.to_string(), entry.owner.to_string());
                if !keys.insert(key) {
                    return Err(Error::Catalog);
                }
                entries.push(entry);
            }
            match &page.next {
                None => return Ok(entries),
                Some(cursor) if cursor.is_empty() || cursor.len() > page_bytes => {
                    return Err(Error::Catalog);
                }
                Some(cursor) => {
                    if entries.is_empty() || !cursors.insert(cursor.clone()) {
                        return Err(Error::Catalog);
                    }
                    // Cursor Memory Counts Against The Same Catalog Budget.
                    bytes = bytes.checked_add(cursor.len()).ok_or(Error::CatalogTooLarge)?;
                    if bytes > CATALOG_BYTES as usize {
                        return Err(Error::CatalogTooLarge);
                    }
                    after = Some(cursor.clone());
                }
            }
        }
    }

    pub async fn health(&self) -> ClientResult<StreamItem<Health>> {
        self.unary::<GetHealth>(GetHealth {}).await
    }

    pub async fn identity(&self) -> ClientResult<StreamItem<Identity>> {
        self.unary::<GetIdentity>(GetIdentity {}).await
    }

    pub(super) async fn unary<O>(&self, message: O::Request) -> ClientResult<StreamItem<O::Response>>
    where
        O: OperationDeclaration,
        O::Request: Sync,
        O::Response: Portable + MemorySize,
    {
        self.require(&dokime_operations::path::<O>())?;
        self.execute(async {
            super::transport::unary::<O>(
                self.channel().map_err(|error| tonic::Status::cancelled(error.to_string()))?,
                self.inner.authorization.as_ref(),
                message,
                self.response_limit(),
                self.request_limit(),
            )
            .await
            .map_err(|error| match error {
                ClientError::Rpc(status) => status,
                error => tonic::Status::internal(error.to_string()),
            })
        })
        .await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
