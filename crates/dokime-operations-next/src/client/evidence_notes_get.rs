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
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.evidence.Notes/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::evidence::NotesGetRequest, v1::evidence::NotesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::evidence::NotesGetRequest) -> Result<v1::evidence::NotesGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) = super::views::read(context, &format!("note/{}", request.name), request.at.as_ref(), async {
            let note = context.host.store().note(&request.name)?;
            let (_, revision) = context.host.store().administration_value(&format!("note:{}", request.name), &[])?;
            Ok(v1::evidence::NotesGetResponse {
                value: Some(v1::evidence::NoteWritable { message: note.message.clone() }),
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
