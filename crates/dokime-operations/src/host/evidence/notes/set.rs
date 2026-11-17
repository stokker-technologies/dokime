/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Notes Set

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::OperationContract;
use prost::Message;

// Local Imports
use crate::host::storage::administration;
use crate::host::{ClientSettings, Context};
use crate::{Error, Result};
use dokime_protocol::host::evidence::{NoteWritable, NotesSetRequest, NotesSetResponse, OperatorNote};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Notes/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<NotesSetRequest, NotesSetResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: NotesSetRequest) -> Result<NotesSetResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let value = request.value.as_ref().ok_or_else(|| Error::Invalid("Set Note: Supply Its Whole Writable Value".into()))?;
        if value.message.trim().is_empty() {
            return Err(Error::Invalid("Set Note: Supply a Nonempty Message".into()));
        }
        if request.request_id.is_empty() {
            return Err(Error::Invalid("Set Note: Supply a Request ID".into()));
        }
        let _guard = context.administration.lock().await;
        let key = format!("note:{}", request.name);
        let existing = administration::read(context, &key, &[], None).await?;
        if existing.payload.is_empty() {
            return Err(Error::NotFound("Set Note: Select an Existing Note".into()));
        }
        let mut note = OperatorNote::decode(existing.payload.as_slice())?;
        if note.author.as_deref() != Some("local-client") {
            return Err(Error::Permission("Set Note: Sign In as Its Author".into()));
        }
        if existing.revision == request.expected_revision {
            let scope = note.scope.as_ref().ok_or_else(|| Error::Precondition("Set Note: Restore Its Owner Scope".into()))?;
            super::require_open(context, scope)?;
        }
        note.message = value.message.clone();
        let scope = note.scope.as_ref().ok_or_else(|| Error::Precondition("Set Note: Restore Its Owner Scope".into()))?;
        let storage_scope = super::storage_scope(scope)?;
        let (saved, receipt) =
            administration::replace(context, &key, storage_scope, request.expected_revision, &request.request_id, note.encode_to_vec())
                .await?;
        let note = OperatorNote::decode(saved.payload.as_slice())?;
        Ok(NotesSetResponse {
            value: Some(NoteWritable { message: note.message }),
            revision: saved.revision,
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(NotesSetRequest, NotesSetResponse, context, "dokime.host.v1.evidence", "Notes", "Set");
