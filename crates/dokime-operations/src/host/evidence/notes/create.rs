/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Notes Create

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};
use sha2::Digest;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Notes/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::evidence::NotesCreateRequest, dokime_protocol::host::evidence::NotesCreateResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::evidence::NotesCreateRequest,
) -> Result<dokime_protocol::host::evidence::NotesCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let value = request.value.as_ref().ok_or_else(|| Error::Invalid("Create Note: Supply Its Writable Value".into()))?;
        if value.message.trim().is_empty() || value.message.len() > context.settings.message_bytes / 2 {
            return Err(Error::Invalid("Create Note: Supply a Bounded Nonempty Message".into()));
        }
        let owner = request.owner.as_ref().ok_or_else(|| Error::Invalid("Create Note: Select Its Owner".into()))?;
        let scope = context.host.store().note_owner(owner)?;
        let fingerprint = crate::host::views::hex(&crate::host::paging::bytes(&request));
        let note_id =
            crate::host::views::hex(&sha2::Sha256::digest(format!("{}/{}", context.host.identity().backend_id, request.request_id)));
        let reply = context.host.store().create_note(dokime_protocol::host::evidence::NotesCreateResponse {
            note: Some(dokime_protocol::host::evidence::OperatorNote {
                note_id,
                scope: Some(scope),
                author: Some("local-client".into()),
                message: value.message.clone(),
                created_at: Some(crate::host::projection::timestamp(context.host.now_ns())),
            }),
            revision: 0_u64.to_be_bytes().to_vec(),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })?;
        let _ = fingerprint;
        context.host.signal_change();
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
