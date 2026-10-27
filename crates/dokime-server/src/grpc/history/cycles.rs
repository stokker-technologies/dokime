/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::operations::{Context, Operation, Response};
use tonic::Status;

// Local Imports
use super::wire;
use crate::grpc::transcode::Transcode;
use crate::grpc::value::{canonical, projection};
use dokime::RuntimeHandle;
use dokime::domain::model::history::wire::{CycleBrowsePage, SavedCycleSummary};
use dokime::operations::history::BrowseCycles;
use dokime::runtime::query::cycle_history::{CycleBrowsePage as NativePage, CycleBrowseQuery};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

struct Browse {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for Browse {
    type Declaration = BrowseCycles;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }

    async fn run<'a>(&mut self, _: Self::Context<'a>, request: &mut BrowseCycles, _: &mut ()) -> Result<Response<CycleBrowsePage>, Status> {
        let at = request.at.take().map(|at| wire::cursor(Some(at))).transpose()?;
        let filter = crate::grpc::execution::cycle_filter(request.filter.as_ref())?;
        let limit = (request.limit != 0).then_some(request.limit as usize);
        let query = CycleBrowseQuery { at, after: request.after_sequence, limit, max_bytes: self.resources.maximum, filter };

        // Project Only The Selected Headers
        let page = self.resources.runtime.browse_cycles(query).await.map_err(wire::query_failure)?;
        project(page, self.resources.maximum).map_err(|source| wire::query_failure(source.into()))
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Browse>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum }),
    )?;
    Ok(())
}

/* === Private Functions === */

fn project(page: dokime::harmos::storage::ReadBatch<NativePage>, maximum: usize) -> dokime::harmos::Result<Response<CycleBrowsePage>> {
    let projected = page.try_map(project_wire)?;
    if projected.wire_len()? > maximum {
        return Err(exhausted());
    }
    Ok(Response::from_owner(projected.owner()))
}

fn project_wire(page: NativePage) -> dokime::harmos::Result<CycleBrowsePage> {
    let mut entries = projection::slots(page.entries.len())?;
    for cycle in page.entries.iter() {
        entries.push(SavedCycleSummary {
            cycle_id: cycle.cycle_id.as_str().to_owned(),
            metadata: Some(canonical::metadata(&cycle.metadata).map_err(original_error)?),
            phase: cycle.phase,
            created_at: Some(canonical::timestamp(&cycle.created_at).map_err(original_error)?),
            updated_at: Some(canonical::timestamp(&cycle.updated_at).map_err(original_error)?),
            execution_count: cycle.execution_count as u64,
            archived: cycle.archived,
        });
    }
    Ok(CycleBrowsePage { entries, at: Some(wire::cursor_wire_ref(&page.at)), next_after_sequence: page.next })
}

fn original_error(source: Status) -> dokime::harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return exhausted();
    }
    dokime::harmos::Error::backend("project historical cycle page", dokime::harmos::BackendFailure::Corrupt, source)
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted { resource: "canonical cycle browse page" }
}

/* === Tests === */
