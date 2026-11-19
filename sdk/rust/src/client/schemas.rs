/* ***
 *
 * Dokime
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
use prost::Message;
use prost_types::FileDescriptorSet;
use sha2::{Digest, Sha256};
use tonic::Status;

// Local Imports
use super::Client;
use super::{ExecutionDetail, ExecutionsSchemasGetOptions};
use crate::PageRequest;

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
        let name = detail
            .summary
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Read Schemas: Supply an Execution Identity"))?
            .execution_id
            .clone();
        let mut at = detail.resolved_at.clone();
        let mut next_page = PageRequest { page_size: 1, ..Default::default() };
        let mut types = FileDescriptorSet::default();
        let mut tokens = HashSet::new();
        loop {
            let page = client
                .resources
                .executions
                .schemas
                .get_with(
                    name.clone(),
                    ExecutionsSchemasGetOptions {
                        at: at.clone(),
                        page: Some(next_page.clone()),
                        context: client.context.clone(),
                    },
                )
                .await?;
            if &page.digest != digest || page.resolved_at.is_none() || (at.is_some() && page.resolved_at != at) {
                return Err(Status::data_loss("Read Schemas: Descriptor Digest or Read Cut Changed"));
            }
            if page.records != detail.schemas.as_ref().unwrap().records || page.streams != detail.schemas.as_ref().unwrap().streams {
                return Err(Status::data_loss("Read Schemas: Frozen Message Bindings Changed"));
            }
            types.file.extend(page.types.ok_or_else(|| Status::data_loss("Read Schemas: Missing Descriptor Page"))?.file);
            if page.next_page_token.is_empty() {
                break;
            }
            if !tokens.insert(page.next_page_token.clone()) {
                return Err(Status::data_loss("Read Schemas: Repeated Descriptor Page"));
            }
            at = page.resolved_at;
            next_page = PageRequest {
                page_size: 1,
                page_token: page.next_page_token,
                ..Default::default()
            };
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
