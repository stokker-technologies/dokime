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

// Local Imports
use crate::host::storage::administration;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::evidence::OperatorNote;
use prost::Message;

use dokime_protocol::host::evidence::NoteWritable;
use dokime_protocol::host::evidence::NotesGetRequest;
use dokime_protocol::host::evidence::NotesGetResponse;

use crate::host::ClientSettings;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Notes/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<NotesGetRequest, NotesGetResponse>(ID, settings.read_timeout)
}
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
