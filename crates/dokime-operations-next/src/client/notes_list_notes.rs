/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.notes.list_notes";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::NotesListNotesRequest, v1::ListNotesResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::NotesListNotesRequest) -> Result<v1::ListNotesResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        use prost::Message;
        use sha2::{Digest, Sha256};
        let mut selection = request.selection.unwrap_or_default();
        if selection.owner.is_none() { selection.include_context = false; }
        let store = context.host.store();
        let scope = selection.owner.as_ref().map(|owner| store.note_owner(owner)).transpose()?;
        let key = format!("notes:{:?}:{:?}", Sha256::digest(selection.encode_to_vec()).as_slice(), store.notes_fingerprint()?);
        let cursor = v1::Cursor { epoch: context.host.identity().epoch, stream: key, position: 0 };
        if request.cursor.as_ref().is_some_and(|given| given != &cursor) || !request.after_id.is_empty() && request.cursor.is_none() {
            return Err(Error::Invalid("List Notes: Restart Pagination After the Selection, Epoch or Notes Changed".into()));
        }
        let limit = if request.limit == 0 { context.settings.page_items } else { (request.limit as usize).min(context.settings.page_items) };
        let cycle = scope.as_ref().and_then(|scope| scope.cycle_id.as_deref());
        let execution = scope.as_ref().and_then(|scope| scope.execution_id.as_deref());
        let mut notes = store.notes_page(cycle, execution, selection.include_context, &request.after_id, limit + 1)?;
        let more = notes.len() > limit;
        notes.truncate(limit);
        let next_after_id = if more { notes.last().unwrap().note_id.clone() } else { String::new() };
        let reply = v1::ListNotesResponse { notes, cursor: Some(cursor), next_after_id };
        if reply.encoded_len() > context.settings.message_bytes { return Err(Error::Invalid("List Notes: Reduce the Page Size".into())); }
        Ok(reply)
    }).await
}

/* === Private Functions === */

/* === Tests === */
