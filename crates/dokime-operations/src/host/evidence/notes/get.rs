/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Notes Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::host::storage::administration;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::evidence::NoteWritable;
use dokime_protocol::host::evidence::NotesGetRequest;
use dokime_protocol::host::evidence::NotesGetResponse;
use dokime_protocol::host::evidence::OperatorNote;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Notes/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: NotesGetRequest) -> Result<NotesGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let saved = administration::read(context, &format!("note:{}", request.name), &[], request.at).await?;
        if saved.payload.is_empty() {
            return Err(Error::NotFound("Get Note: Select an Existing Note".into()));
        }
        let note = OperatorNote::decode(saved.payload.as_slice())?;
        Ok(NotesGetResponse {
            value: Some(NoteWritable { message: note.message.clone() }),
            revision: saved.revision,
            note: Some(note),
            resolved_at: saved.resolved_at,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(NotesGetRequest, NotesGetResponse, context, "dokime.host.v1.evidence", "Notes", "Get");
