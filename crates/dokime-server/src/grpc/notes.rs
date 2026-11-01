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

use crate::grpc::transcode::Transcode;

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::domain::Portable;
use dokime::harmos::{Context, Response};
use tonic::Status;

// Local Imports
use super::access::Principal;
use dokime::app::LiveRuntime;
use dokime::domain::model::identity::CommandId;
use dokime::domain::model::identity::NoteId;
use dokime::domain::model::identity::SessionId;
use dokime::domains::evidence::notes as evidence;
use dokime::domains::evidence::notes::NoteError;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::notes as contract;

/* === Constants === */

const SERVER_NOTE_RESPONSE_BYTES: u64 = 524_288;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<contract::ListNotes> for Resources {
    fn serve<'a>(
        &'a self, mut request: contract::ListNotes, _: Context<'a, contract::ListNotes>,
    ) -> Served<'a, dokime::domain::model::notes::wire::ListNotesResponse> {
        Box::pin(async move {
            let snapshot = dokime::app::live_runtime(&self.runtime).await?.watch_state().latest();
            let page = selection::Page::new(
                request.selection.take(),
                request.limit,
                std::mem::take(&mut request.after_id),
                request.cursor.take(),
                &snapshot,
                "notes",
            )?;
            let sets = snapshot.state().annotation_sets(page.owner.as_ref(), page.context).map_err(Status::resource_exhausted)?;
            let mut items: Vec<_> = sets
                .iter()
                .flat_map(|set| set.notes())
                .filter(|note| page.matches(&note.scope) && note.note_id.as_str() > page.after.as_str())
                .collect();
            items.sort_by(|a, b| a.note_id.cmp(&b.note_id));
            let more = items.len() > page.limit;
            items.truncate(page.limit);
            self.produce(|| {
                let next_after_id = if more { items.last().unwrap().note_id.to_string() } else { String::new() };
                Ok(dokime::domain::model::notes::wire::ListNotesResponse {
                    notes: items.into_iter().map(wire::note).collect::<Result<_, _>>()?,
                    cursor: Some(page.cursor),
                    next_after_id,
                })
            })
        })
    }
}

impl Serve<contract::ListHealth> for Resources {
    fn serve<'a>(
        &'a self, mut request: contract::ListHealth, _: Context<'a, contract::ListHealth>,
    ) -> Served<'a, dokime::domain::model::notes::wire::ListHealthResponse> {
        Box::pin(async move {
            let snapshot = dokime::app::live_runtime(&self.runtime).await?.watch_state().latest();
            let page = selection::Page::new(
                request.selection.take(),
                request.limit,
                std::mem::take(&mut request.after_id),
                request.cursor.take(),
                &snapshot,
                "health",
            )?;
            let sets = snapshot.state().annotation_sets(page.owner.as_ref(), page.context).map_err(Status::resource_exhausted)?;
            let mut items: Vec<_> = sets
                .iter()
                .flat_map(|set| set.health())
                .filter(|item| page.matches(&item.scope) && item.health_item_id.as_str() > page.after.as_str())
                .collect();
            items.sort_by(|a, b| a.health_item_id.cmp(&b.health_item_id));
            let more = items.len() > page.limit;
            items.truncate(page.limit);
            self.produce(|| {
                let next_after_id = if more { items.last().unwrap().health_item_id.to_string() } else { String::new() };
                Ok(dokime::domain::model::notes::wire::ListHealthResponse {
                    items: items.into_iter().map(wire::health).collect::<Result<_, _>>()?,
                    cursor: Some(page.cursor),
                    next_after_id,
                })
            })
        })
    }
}

impl Serve<contract::CreateNote> for Resources {
    fn serve<'a>(
        &'a self, mut request: contract::CreateNote, ctx: Context<'a, contract::CreateNote>,
    ) -> Served<'a, dokime::domain::model::notes::wire::NoteReceipt> {
        Box::pin(async move {
            let context = ctx.call();
            let actor = context.get::<Principal>().ok_or_else(|| Status::unauthenticated("authentication required"))?.subject().to_owned();
            let result = dokime::app::live_runtime(&self.runtime)
                .await?
                .create_current_note(evidence::CreateNote {
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
            self.produce(|| wire::receipt(&result))
        })
    }
}

impl Serve<contract::EditNote> for Resources {
    fn serve<'a>(
        &'a self, mut request: contract::EditNote, ctx: Context<'a, contract::EditNote>,
    ) -> Served<'a, dokime::domain::model::notes::wire::EditNoteResponse> {
        Box::pin(async move {
            let context = ctx.call();
            let actor = context.get::<Principal>().ok_or_else(|| Status::unauthenticated("authentication required"))?.subject().to_owned();
            let note = dokime::app::live_runtime(&self.runtime)
                .await?
                .edit_current_note(evidence::EditNote {
                    epoch: SessionId::from_uuid(std::mem::take(&mut request.epoch))
                        .map_err(|_| Status::invalid_argument("invalid epoch UUID"))?,
                    note_id: NoteId::from_uuid(std::mem::take(&mut request.note_id))
                        .map_err(|_| Status::invalid_argument("invalid note identity"))?,
                    actor,
                    message: std::mem::take(&mut request.message),
                })
                .await
                .map_err(status)?;
            self.produce(|| Ok(dokime::domain::model::notes::wire::EditNoteResponse { note: Some(wire::note(&note)?) }))
        })
    }
}

impl Resources {
    fn produce<T: Portable + Send + Sync + 'static>(&self, build: impl FnOnce() -> Result<T, Status>) -> Result<Response<T>, Status> {
        let output = build()?;
        if output.wire_len()? > self.maximum {
            return Err(Status::resource_exhausted("annotation response exceeds byte budget"));
        }
        Ok(output.into())
    }
}

/* === Public Functions === */

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources {
        runtime: runtime.clone(),
        maximum: maximum.min(usize::try_from(SERVER_NOTE_RESPONSE_BYTES).expect("validated note response size")),
    });

    setup.resource::<Host<contract::ListNotes>>(Host::<contract::ListNotes>::new(served.clone()));
    setup.resource::<Host<contract::ListHealth>>(Host::<contract::ListHealth>::new(served.clone()));
    setup.resource::<Host<contract::CreateNote>>(Host::<contract::CreateNote>::new(served.clone()));
    setup.resource::<Host<contract::EditNote>>(Host::<contract::EditNote>::new(served.clone()));

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
        NoteError::Unavailable(_) => Status::unavailable("note operation could not be confirmed; Get resolves commitment"),
    }
}

/* === Tests === */
