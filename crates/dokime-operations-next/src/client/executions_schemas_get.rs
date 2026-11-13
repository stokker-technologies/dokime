/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Schemas Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;
use prost_types::FileDescriptorSet;
use sha2::{Digest, Sha256};

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::client::executions::{SchemasGetRequest, SchemasGetResponse};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.executions.Schemas/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<SchemasGetRequest, SchemasGetResponse>(ID, settings.read_timeout)
}

pub async fn handle(context: &Context, request: SchemasGetRequest) -> Result<SchemasGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let page = request.page.clone().unwrap_or_default();
        if !page.page_token.is_empty() && request.at.is_none() {
            return Err(Error::Invalid("Get Schemas: Continue with the Returned Cut".into()));
        }
        let saved = context.host.store().execution_schemas(&request.name, request.at.as_ref())?;
        let types = FileDescriptorSet::decode(saved.payload.as_slice())?;
        let mut binding = Sha256::new();
        binding.update(request.name.as_bytes());
        binding.update(saved.resolved_at.encode_to_vec());
        binding.update(&saved.revision);
        let binding = super::views::hex(&binding.finalize());
        let start = if page.page_token.is_empty() {
            0
        } else {
            let (identity, offset) =
                page.page_token.split_once(':').ok_or_else(|| Error::Invalid("Get Schemas: Use the Returned Page Token".into()))?;
            if identity != binding {
                return Err(Error::Invalid("Get Schemas: Continue the Same Execution, Cut and Digest".into()));
            }
            offset.parse::<usize>().map_err(|_| Error::Invalid("Get Schemas: Use the Returned Page Token".into()))?
        };
        if start > types.file.len() {
            return Err(Error::Invalid("Get Schemas: Continue Within the Frozen Descriptor Set".into()));
        }
        let limit = super::page_size(page.page_size, context.settings.page_items)?;
        let maximum = if page.max_bytes == 0 {
            context.settings.message_bytes
        } else {
            usize::try_from(page.max_bytes).unwrap_or(usize::MAX).min(context.settings.message_bytes)
        };
        let total = types.file.len();
        let mut reply = SchemasGetResponse {
            digest: saved.revision,
            types: Some(FileDescriptorSet::default()),
            resolved_at: Some(saved.resolved_at),
            next_page_token: String::new(),
        };
        for file in types.file.into_iter().skip(start).take(limit) {
            reply.types.as_mut().unwrap().file.push(file);
            let next = start + reply.types.as_ref().unwrap().file.len();
            reply.next_page_token = if next < total { format!("{binding}:{next}") } else { String::new() };
            if reply.encoded_len() > maximum {
                reply.types.as_mut().unwrap().file.pop();
                let next = start + reply.types.as_ref().unwrap().file.len();
                if next == start {
                    return Err(Error::Bound("Get Schemas: Increase the Page Byte Bound to Fit One Descriptor File".into()));
                }
                reply.next_page_token = format!("{binding}:{next}");
                break;
            }
        }
        if reply.encoded_len() > maximum {
            return Err(Error::Bound("Get Schemas: Increase the Page Byte Bound to Fit the Reply Envelope".into()));
        }
        super::bounded_reply(context, reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
