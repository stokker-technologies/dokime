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

// Local Imports
use crate::host::storage::administration;
use crate::{Error, Result, host::Context};
use prost::Message;
use sha2::Digest;

use dokime_protocol::host::evidence::NotesCreateRequest;
use dokime_protocol::host::evidence::NotesCreateResponse;
use dokime_protocol::host::evidence::OperatorNote;

use crate::host::ClientSettings;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Notes/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<NotesCreateRequest, NotesCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: NotesCreateRequest) -> Result<NotesCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let value = request.value.as_ref().ok_or_else(|| Error::Invalid("Create Note: Supply Its Writable Value".into()))?;
        if value.message.trim().is_empty() || value.message.len() > context.settings.message_bytes / 2 {
            return Err(Error::Invalid("Create Note: Supply a Bounded Nonempty Message".into()));
        }
        let owner = request.owner.as_ref().ok_or_else(|| Error::Invalid("Create Note: Select Its Owner".into()))?;
        if request.request_id.is_empty() {
            return Err(Error::Invalid("Create Note: Supply a Request ID".into()));
        }
        let _guard = context.administration.lock().await;
        let note_id =
            crate::host::views::hex(&sha2::Sha256::digest(format!("{}/{}", context.host.identity().backend_id, request.request_id)));
        let key = format!("note:{note_id}");
        let existing = administration::read(context, &key, &[], None).await?;
        let saved = (!existing.payload.is_empty()).then(|| OperatorNote::decode(existing.payload.as_slice())).transpose()?;
        let scope = context.host.store().note_owner(owner)?;
        if saved.is_none() {
            super::require_open(context, &scope)?;
        }
        let note = OperatorNote {
            note_id,
            scope: Some(scope),
            author: Some("local-client".into()),
            message: value.message.clone(),
            created_at: saved.and_then(|note| note.created_at).or_else(|| Some(crate::host::projection::timestamp(context.host.now_ns()))),
        };
        let (saved, receipt) = administration::replace(context, &key, Vec::new(), &request.request_id, note.encode_to_vec()).await?;
        let reply = NotesCreateResponse {
            note: Some(OperatorNote::decode(saved.payload.as_slice())?),
            revision: saved.revision,
            receipt: Some(receipt),
        };
        context.host.signal_change();
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
