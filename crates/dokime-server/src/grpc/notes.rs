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
pub(in crate::grpc) use wire::{health, health_memory, note, note_memory};

/* === Imports === */
// Standard Library Imports
use std::{num::NonZeroUsize, sync::Arc};
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
    capabilities::evidence::notes::{CreateNoteRequest, NoteError},
    core::{CommandId, SessionId},
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

/* === Enums === */
#[harmos::catalog]
enum Operations {
    ListNotes(ListNotes),
    ListHealth(ListHealth),
    CreateNote(CreateNote),
}

/* === Traits === */

/* === Implementations === */
impl Operation for ListNotes {
    type Declaration = contract::ListNotes;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
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
        let bytes = items.iter().try_fold(
            page_memory::<contract::ListNotesResponse>(&page),
            |bytes, item| {
                Ok::<_, Status>(
                    bytes.saturating_add(
                        wire::note_memory(item)
                            .map_err(memory_failure)?
                            .saturating_add(3 * std::mem::size_of::<contract::OperatorNote>()),
                    ),
                )
            },
        )?;
        self.resources.produce(bytes, || {
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

    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
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
        let bytes = items.iter().try_fold(
            page_memory::<contract::ListHealthResponse>(&page),
            |bytes, item| {
                Ok::<_, Status>(
                    bytes.saturating_add(
                        wire::health_memory(item)
                            .map_err(memory_failure)?
                            .saturating_add(3 * std::mem::size_of::<contract::HealthItem>()),
                    ),
                )
            },
        )?;
        self.resources.produce(bytes, || {
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

    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
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
        self.resources
            .produce(wire::receipt_memory(&result)?, || wire::receipt(&result))
    }
}

impl Resources {
    fn produce<T: MemorySize + Protobuf + Send + Sync + 'static>(
        &self,
        required: usize,
        build: impl FnOnce() -> Result<T, Status>,
    ) -> Result<Response<T>, Status> {
        if required > self.maximum {
            return Err(Status::resource_exhausted(
                "annotation page exceeds byte budget",
            ));
        }
        let bytes = NonZeroUsize::new(required)
            .ok_or_else(|| Status::resource_exhausted("invalid annotation allowance"))?;
        let preparation = self
            .runtime
            .storage()
            .prepare_production(bytes)
            .map_err(memory_failure)?;
        let output = build()?;
        if output.encoded_len() > self.maximum {
            return Err(Status::resource_exhausted(
                "annotation response exceeds byte budget",
            ));
        }
        preparation
            .finish(output)
            .map_err(memory_failure)?
            .try_into_value()
            .map_err(memory_failure)?
            .try_into_response()
            .map_err(memory_failure)
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
        resources,
    )?;
    Ok(())
}

/* === Private Functions === */
fn page_memory<T>(page: &selection::Page) -> usize {
    std::mem::size_of::<T>()
        .saturating_add(page.cursor.epoch.capacity())
        .saturating_add(page.cursor.stream.capacity())
        .saturating_add(36)
}
fn memory_failure(source: harmos::Error) -> Status {
    let status = if matches!(source, harmos::Error::ResourceExhausted { .. }) {
        Status::resource_exhausted("annotation admission failed")
    } else {
        Status::internal("annotation admission failed")
    };
    super::error::detailed(status, &source.into())
}
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
