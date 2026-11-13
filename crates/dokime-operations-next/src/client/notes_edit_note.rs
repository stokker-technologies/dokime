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
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.notes.edit_note";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::NotesEditNoteRequest, v1::EditNoteResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::NotesEditNoteRequest) -> Result<v1::EditNoteResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        super::notes_selection::epoch(context, &request.epoch)?;
        super::notes_selection::message(context, &request.message)?;
        Ok(v1::EditNoteResponse {
            note: Some(context.host.store().edit_note(&request.note_id, "local-client", request.message)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
