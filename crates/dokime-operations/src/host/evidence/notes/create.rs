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
use prost::Message;
use sha2::{Digest, Sha256};

// Local Imports
use crate::host::Context;
use crate::host::storage::administration;
use crate::{Error, Result};
use dokime_protocol::host::evidence::{NotesCreateRequest, NotesCreateResponse, OperatorNote};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Notes/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

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
        let note_id = crate::host::views::hex(&Sha256::digest(format!("{}/{}", context.host.identity().backend_id, request.request_id)));
        let key = format!("note:{note_id}");
        let existing = administration::read(context, &key, &[], None).await?;
        let saved = (!existing.payload.is_empty()).then(|| OperatorNote::decode(existing.payload.as_slice())).transpose()?;
        let scope = super::owner(context, owner)?;
        if saved.is_none() {
            crate::host::evidence::require_open(context, scope.cycle_id.as_deref(), scope.execution_id.as_deref(), "Write Note")?;
        }
        let storage_scope = super::storage_scope(&scope)?;
        let note = OperatorNote {
            note_id,
            scope: Some(scope),
            author: Some("local-client".into()),
            message: value.message.clone(),
            created_at: saved.and_then(|note| note.created_at).or_else(|| Some(crate::host::projection::timestamp(context.host.now_ns()))),
        };
        let (saved, receipt) =
            administration::replace(context, &key, storage_scope, Vec::new(), &request.request_id, note.encode_to_vec()).await?;
        Ok(NotesCreateResponse {
            note: Some(OperatorNote::decode(saved.payload.as_slice())?),
            revision: saved.revision,
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(NotesCreateRequest, NotesCreateResponse, context, "dokime.host.v1.evidence", "Notes", "Create");
