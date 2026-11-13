/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use sha2::Digest;

/* === Constants === */

pub const ID: &str = "dokime.v1.evidence.Notes/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::evidence::NotesCreateRequest, v1::evidence::NotesCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::evidence::NotesCreateRequest) -> Result<v1::evidence::NotesCreateResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
let value=request.value.as_ref().ok_or_else(||Error::Invalid("Create Note: Supply Its Writable Value".into()))?;
if value.message.trim().is_empty() || value.message.len()>context.settings.message_bytes/2 {return Err(Error::Invalid("Create Note: Supply a Bounded Nonempty Message".into()));}
let owner=request.owner.as_ref().ok_or_else(||Error::Invalid("Create Note: Select Its Owner".into()))?;
let scope=context.host.store().note_owner(owner)?;
let fingerprint=super::views::hex(&super::paging::bytes(&request));
let note_id=super::views::hex(&sha2::Sha256::digest(format!("{}/{}",context.host.identity().backend_id,request.request_id)));
let reply=context.host.store().create_note(v1::evidence::NotesCreateResponse {
 note:Some(v1::OperatorNote {note_id,scope:Some(scope),author:Some("local-client".into()),message:value.message.clone(),created_at:Some(super::projection::timestamp(context.host.now_ns()))}),
 revision:0_u64.to_be_bytes().to_vec(),receipt:Some(super::views::receipt(&request.request_id)?)
})?;
let _=fingerprint;context.host.signal_change();Ok(reply)
    }).await
}

/* === Private Functions === */

/* === Tests === */
