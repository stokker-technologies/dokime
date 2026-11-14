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
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Notes/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::evidence::NotesGetRequest, dokime_protocol::host::evidence::NotesGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::evidence::NotesGetRequest,
) -> Result<dokime_protocol::host::evidence::NotesGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) = crate::host::views::read(context, &format!("note/{}", request.name), request.at.as_ref(), async {
            let note = context.host.store().note(&request.name)?;
            let (_, revision) = context.host.store().administration_value(&format!("note:{}", request.name), &[])?;
            Ok(dokime_protocol::host::evidence::NotesGetResponse {
                value: Some(dokime_protocol::host::evidence::NoteWritable { message: note.message.clone() }),
                revision,
                note: Some(note),
                resolved_at: None,
            })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
