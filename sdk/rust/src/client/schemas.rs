/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Frozen Schema Cache

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

// Third Party Imports
use harmos_protocol::v1::PageRequest;
use prost::Message;
use prost_types::FileDescriptorSet;
use sha2::{Digest, Sha256};
use tonic::{Request, Status};

// Local Imports
use super::Client;
use dokime_protocol::client::{ExecutionDetail, executions::SchemasGetRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Caller-Owned Descriptor Cache; Drop It to Release Retained Sets.
#[derive(Default)]
pub struct FrozenSchemaCache {
    sets: HashMap<Vec<u8>, Arc<FileDescriptorSet>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl FrozenSchemaCache {
    /// Fetch Every Descriptor Page on a Miss; Reuse Equal Digests Across Executions.
    pub async fn get(&mut self, client: &Client, detail: &ExecutionDetail) -> Result<Arc<FileDescriptorSet>, Status> {
        let digest =
            &detail.schemas.as_ref().ok_or_else(|| Status::invalid_argument("Read Schemas: Supply Frozen Schema Identities"))?.digest;
        if digest.len() != 32 {
            return Err(Status::invalid_argument("Read Schemas: Supply a SHA-256 Digest"));
        }
        if let Some(types) = self.sets.get(digest) {
            return Ok(types.clone());
        }
        let mut request = SchemasGetRequest {
            name: detail
                .summary
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("Read Schemas: Supply an Execution Identity"))?
                .execution_id
                .clone(),
            at: detail.resolved_at.clone(),
            page: Some(PageRequest { page_size: 1, ..Default::default() }),
        };
        let mut types = FileDescriptorSet::default();
        let mut tokens = HashSet::new();
        loop {
            let page = client.executions.schemas.get(client.request(Request::new(request.clone()))).await?.into_inner();
            if &page.digest != digest || page.resolved_at.is_none() || (request.at.is_some() && page.resolved_at != request.at) {
                return Err(Status::data_loss("Read Schemas: Descriptor Digest or Read Cut Changed"));
            }
            types.file.extend(page.types.ok_or_else(|| Status::data_loss("Read Schemas: Missing Descriptor Page"))?.file);
            if page.next_page_token.is_empty() {
                break;
            }
            if !tokens.insert(page.next_page_token.clone()) {
                return Err(Status::data_loss("Read Schemas: Repeated Descriptor Page"));
            }
            request.at = page.resolved_at;
            request.page = Some(PageRequest {
                page_size: 1,
                page_token: page.next_page_token,
                ..Default::default()
            });
        }
        if Sha256::digest(types.encode_to_vec()).as_slice() != digest {
            return Err(Status::data_loss("Read Schemas: Frozen Descriptor Digest Does Not Match"));
        }
        let types = Arc::new(types);
        self.sets.insert(digest.clone(), types.clone());
        Ok(types)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
