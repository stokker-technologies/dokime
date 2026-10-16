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
pub(in crate::grpc) use wire::{health, note};

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;
// Third Party Imports
use harmos::{
    MemorySize, Protobuf,
    operations::{Context, Operation, Response},
};
use tonic::Status;
// Local Imports
use super::access::Principal;
use dokime_app::{
    RuntimeHandle,
    core::{CommandId, NoteId, SessionId},
    domains::evidence::notes::{CreateNoteRequest, EditNoteRequest, NoteError},
};
use dokime_contract::operations::notes as contract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}
struct ListNotes {
    resources: Arc<Resources>,
}
struct ListHealth {
    resources: Arc<Resources>,
}
struct CreateNote {
    resources: Arc<Resources>,
}
struct EditNote {
    resources: Arc<Resources>,
}

/* === Enums === */
#[harmos::catalog]
enum Operations {
    ListNotes(ListNotes),
    ListHealth(ListHealth),
    CreateNote(CreateNote),
    EditNote(EditNote),
}

/* === Traits === */

/* === Implementations === */
impl Operation for ListNotes {
    type Declaration = contract::ListNotes;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut contract::ListAnnotationsRequest,
        _: &mut (),
    ) -> Result<Response<contract::ListNotesResponse>, Status> {
        let snapshot = self.resources.runtime.watch_state().latest();
        let page = selection::Page::new(request, &snapshot, "notes")?;
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
        self.resources.produce(|| {
            let next_after_id = if more {
                items.last().unwrap().note_id.to_string()
            } else {
                String::new()
            };
            Ok(contract::ListNotesResponse {
                notes: items
                    .into_iter()
                    .map(wire::note)
                    .collect::<Result<_, _>>()?,
                cursor: Some(page.cursor),
                next_after_id,
            })
        })
    }
}

impl Operation for ListHealth {
    type Declaration = contract::ListHealth;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut contract::ListAnnotationsRequest,
        _: &mut (),
    ) -> Result<Response<contract::ListHealthResponse>, Status> {
        let snapshot = self.resources.runtime.watch_state().latest();
        let page = selection::Page::new(request, &snapshot, "health")?;
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
        self.resources.produce(|| {
            let next_after_id = if more {
                items.last().unwrap().health_item_id.to_string()
            } else {
                String::new()
            };
            Ok(contract::ListHealthResponse {
                items: items
                    .into_iter()
                    .map(wire::health)
                    .collect::<Result<_, _>>()?,
                cursor: Some(page.cursor),
                next_after_id,
            })
        })
    }
}

impl Operation for CreateNote {
    type Declaration = contract::CreateNote;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut contract::CreateNoteRequest,
        _: &mut (),
    ) -> Result<Response<contract::NoteReceipt>, Status> {
        let actor = context
            .context()
            .get::<Principal>()
            .ok_or_else(|| Status::unauthenticated("authentication required"))?
            .subject()
            .to_owned();
        let result = self
            .resources
            .runtime
            .create_current_note(CreateNoteRequest {
                epoch: SessionId::from_uuid(std::mem::take(&mut request.epoch))
                    .map_err(|_| Status::invalid_argument("invalid epoch UUID"))?,
                mutation_id: CommandId::from_uuid(std::mem::take(&mut request.mutation_id))
                    .map_err(|_| Status::invalid_argument("invalid mutation UUID"))?,
                owner: wire::owner(request.owner.take())?,
                actor,
                message: std::mem::take(&mut request.message),
            })
            .await
            .map_err(status)?;
        self.resources.produce(|| wire::receipt(&result))
    }
}

impl Operation for EditNote {
    type Declaration = contract::EditNote;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut contract::EditNoteRequest,
        _: &mut (),
    ) -> Result<Response<contract::EditNoteResponse>, Status> {
        let actor = context
            .context()
            .get::<Principal>()
            .ok_or_else(|| Status::unauthenticated("authentication required"))?
            .subject()
            .to_owned();
        let note = self
            .resources
            .runtime
            .edit_current_note(EditNoteRequest {
                epoch: SessionId::from_uuid(std::mem::take(&mut request.epoch))
                    .map_err(|_| Status::invalid_argument("invalid epoch UUID"))?,
                note_id: NoteId::from_uuid(std::mem::take(&mut request.note_id))
                    .map_err(|_| Status::invalid_argument("invalid note identity"))?,
                actor,
                message: std::mem::take(&mut request.message),
            })
            .await
            .map_err(status)?;
        self.resources.produce(|| {
            Ok(contract::EditNoteResponse {
                note: Some(wire::note(&note)?),
            })
        })
    }
}

impl Resources {
    fn produce<T: MemorySize + Protobuf + Send + Sync + 'static>(
        &self,
        build: impl FnOnce() -> Result<T, Status>,
    ) -> Result<Response<T>, Status> {
        let output = build()?;
        if output.encoded_len() > self.maximum {
            return Err(Status::resource_exhausted(
                "annotation response exceeds byte budget",
            ));
        }
        Ok(output.into())
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let resources = Arc::new(Resources {
        runtime: runtime.clone(),
        maximum: maximum.min(512 * 1024),
    });
    runtime.operations().register_wire_operation::<ListNotes>(
        "dokime",
        dokime_app::app::Lanes::Open,
        resources.clone(),
    )?;
    runtime.operations().register_wire_operation::<ListHealth>(
        "dokime",
        dokime_app::app::Lanes::Open,
        resources.clone(),
    )?;
    runtime.operations().register_wire_operation::<CreateNote>(
        "dokime",
        dokime_app::app::Lanes::Open,
        resources.clone(),
    )?;
    runtime.operations().register_wire_operation::<EditNote>(
        "dokime",
        dokime_app::app::Lanes::Open,
        resources,
    )?;
    Ok(())
}

/* === Private Functions === */
fn status(error: NoteError) -> Status {
    match error {
        NoteError::Invalid(message) => Status::invalid_argument(message),
        NoteError::Owner(message) => Status::failed_precondition(message),
        NoteError::Epoch => Status::failed_precondition(error.to_string()),
        NoteError::Conflict => Status::already_exists(error.to_string()),
        NoteError::Overloaded => Status::resource_exhausted(error.to_string()),
        NoteError::Sealed => Status::failed_precondition(error.to_string()),
        NoteError::Absent => Status::not_found(error.to_string()),
        NoteError::NotAuthor => Status::permission_denied(error.to_string()),
        NoteError::Unavailable(_) => {
            Status::unavailable("note operation could not be confirmed; Get resolves commitment")
        }
    }
}

/* === Tests === */
