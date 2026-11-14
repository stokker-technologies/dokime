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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.evidence.Notes/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::evidence::NotesGetRequest, dokime_protocol::client::evidence::NotesGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::evidence::NotesGetRequest,
) -> Result<dokime_protocol::client::evidence::NotesGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) = super::views::read(context, &format!("note/{}", request.name), request.at.as_ref(), async {
            let note = context.host.store().note(&request.name)?;
            let (_, revision) = context.host.store().administration_value(&format!("note:{}", request.name), &[])?;
            Ok(dokime_protocol::client::evidence::NotesGetResponse {
                value: Some(dokime_protocol::client::evidence::NoteWritable { message: note.message.clone() }),
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
