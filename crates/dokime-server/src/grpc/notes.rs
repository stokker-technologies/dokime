/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod selection;
mod wire;

/* === Re-Exports === */
pub(crate) use wire::{
    health as historical_health, health_memory as historical_health_memory,
    note as historical_note, note_memory as historical_note_memory,
};

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use prost::Message;
use tonic::{Request, Response, Status};
// Local Imports
use super::access::Principal;
use dokime_app::{
    RuntimeHandle,
    capabilities::evidence::notes::{CreateNoteRequest, NoteError},
    core::{CommandId, SessionId},
};
use dokime_protocol::v1::{self, health_server::Health, notes_server::Notes};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub runtime: RuntimeHandle,
    pub max_message_bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Service {
    fn reply<T: Message>(&self, value: T) -> Result<Response<T>, Status> {
        if value.encoded_len() > self.max_message_bytes.min(512 * 1024) {
            return Err(Status::resource_exhausted(
                "annotation response exceeds byte budget",
            ));
        }
        Ok(Response::new(value))
    }
    fn budget(&self, bytes: usize) -> Result<(), Status> {
        if bytes > self.max_message_bytes.min(512 * 1024) {
            Err(Status::resource_exhausted(
                "annotation page exceeds byte budget",
            ))
        } else {
            Ok(())
        }
    }
}

#[tonic::async_trait]
impl Notes for Service {
    async fn list_notes(
        &self,
        request: Request<v1::ListAnnotationsRequest>,
    ) -> Result<Response<v1::ListNotesResponse>, Status> {
        let snapshot = self.runtime.watch_state().latest();
        let page = selection::Page::new(request.into_inner(), &snapshot, "notes")?;
        let sets = snapshot
            .state()
            .annotation_sets(page.owner.as_ref(), page.context)
            .map_err(Status::resource_exhausted)?;
        let mut items: Vec<_> = sets
            .iter()
            .flat_map(|set| set.notes())
            .filter(|note| page.matches(&note.scope) && note.note_id.as_str() > page.after.as_str())
            .collect();
        items.sort_by(|a, b| a.note_id.cmp(&b.note_id));
        let more = items.len() > page.limit;
        items.truncate(page.limit);
        let bytes = items.iter().try_fold(1024usize, |bytes, item| {
            Ok::<_, Status>(bytes.saturating_add(wire::note_size(item)?))
        })?;
        self.budget(bytes)?;
        let next_after_id = if more {
            items.last().unwrap().note_id.to_string()
        } else {
            String::new()
        };
        self.reply(v1::ListNotesResponse {
            notes: items
                .into_iter()
                .map(wire::note)
                .collect::<Result<_, _>>()?,
            cursor: Some(page.cursor),
            next_after_id,
        })
    }
    async fn create_note(
        &self,
        request: Request<v1::CreateNoteRequest>,
    ) -> Result<Response<v1::NoteReceipt>, Status> {
        let actor = Principal::of(&request)?.subject().to_owned();
        let request = request.into_inner();
        let result = self
            .runtime
            .create_current_note(CreateNoteRequest {
                epoch: SessionId::from_uuid(request.epoch)
                    .map_err(|_| Status::invalid_argument("invalid epoch UUID"))?,
                mutation_id: CommandId::from_uuid(request.mutation_id)
                    .map_err(|_| Status::invalid_argument("invalid mutation UUID"))?,
                owner: wire::owner(request.owner)?,
                actor,
                message: request.message,
            })
            .await
            .map_err(status)?;
        self.reply(wire::receipt(result)?)
    }
}

#[tonic::async_trait]
impl Health for Service {
    async fn list_health(
        &self,
        request: Request<v1::ListAnnotationsRequest>,
    ) -> Result<Response<v1::ListHealthResponse>, Status> {
        let snapshot = self.runtime.watch_state().latest();
        let page = selection::Page::new(request.into_inner(), &snapshot, "health")?;
        let sets = snapshot
            .state()
            .annotation_sets(page.owner.as_ref(), page.context)
            .map_err(Status::resource_exhausted)?;
        let mut items: Vec<_> = sets
            .iter()
            .flat_map(|set| set.health())
            .filter(|item| {
                page.matches(&item.scope) && item.health_item_id.as_str() > page.after.as_str()
            })
            .collect();
        items.sort_by(|a, b| a.health_item_id.cmp(&b.health_item_id));
        let more = items.len() > page.limit;
        items.truncate(page.limit);
        let bytes = items.iter().try_fold(1024usize, |bytes, item| {
            Ok::<_, Status>(bytes.saturating_add(wire::health_size(item)?))
        })?;
        self.budget(bytes)?;
        let next_after_id = if more {
            items.last().unwrap().health_item_id.to_string()
        } else {
            String::new()
        };
        self.reply(v1::ListHealthResponse {
            items: items
                .into_iter()
                .map(wire::health)
                .collect::<Result<_, _>>()?,
            cursor: Some(page.cursor),
            next_after_id,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn status(error: NoteError) -> Status {
    match error {
        NoteError::Invalid(message) => Status::invalid_argument(message),
        NoteError::Owner(message) => Status::failed_precondition(message),
        NoteError::Epoch => Status::failed_precondition(error.to_string()),
        NoteError::Conflict => Status::already_exists(error.to_string()),
        NoteError::Overloaded => Status::resource_exhausted(error.to_string()),
        NoteError::Unavailable(_) => {
            Status::unavailable("note operation could not be confirmed; Get resolves commitment")
        }
    }
}

/* === Tests === */
