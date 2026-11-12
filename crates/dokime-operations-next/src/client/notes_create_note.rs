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
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.notes.create_note";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::NotesCreateNoteRequest, v1::NoteReceipt>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::NotesCreateNoteRequest) -> Result<v1::NoteReceipt> {
    super::complete(ID, context.settings.mutation_timeout, async {
        super::notes_selection::epoch(context, &request.epoch)?;
        super::notes_selection::message(context, &request.message)?;
        let mutation = uuid::Uuid::parse_str(&request.mutation_id).map_err(|_| Error::Invalid("Create Note: Supply a Mutation UUID".into()))?;
        if mutation.is_nil() { return Err(Error::Invalid("Create Note: Supply a Nonzero Mutation UUID".into())); }
        let owner = request.owner.ok_or_else(|| Error::Invalid("Create Note: Select Its Owner".into()))?;
        let store = context.host.store();
        let scope = store.note_owner(&owner)?;
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(format!("{}:local-client:{}", request.epoch, mutation));
        let note_id = uuid::Uuid::from_bytes(digest[..16].try_into().unwrap()).to_string();
        let now = context.host.now_ns();
        Ok(store.create_note(v1::NoteReceipt {
            epoch: request.epoch, mutation_id: request.mutation_id, owner: Some(owner),
            note: Some(v1::OperatorNote { note_id, scope: Some(scope), author: Some("local-client".into()),
                message: request.message, created_at: Some(prost_types::Timestamp { seconds: now.div_euclid(1_000_000_000), nanos: now.rem_euclid(1_000_000_000) as i32 }) }),
            publication: v1::NotePublication::Confirmed as i32, ..Default::default()
        })?)
    }).await
}

/* === Private Functions === */

/* === Tests === */
