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
use dokime::harmos::{Context, RegistrationExt, Response};
use tonic::Status;

// Local Imports
use super::wire;
use crate::grpc::transcode::Transcode;
use crate::grpc::value::{canonical, projection};
use dokime::RuntimeHandle;
use dokime::domain::model::history::wire::{CycleBrowsePage, SavedCycleSummary};
use dokime::operations::history::BrowseCycles;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::runtime::query::cycle_history::{CycleBrowsePage as NativePage, CycleBrowseQuery};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<BrowseCycles> for Resources {
    fn serve<'a>(&'a self, mut request: BrowseCycles, _: Context<'a, BrowseCycles>) -> Served<'a, CycleBrowsePage> {
        Box::pin(async move {
            let at = request.at.take().map(|at| wire::cursor(Some(at))).transpose()?;
            let filter = crate::grpc::execution::cycle_filter(request.filter.as_ref())?;
            let limit = (request.limit != 0).then_some(request.limit as usize);
            let query = CycleBrowseQuery {
                at,
                after: request.after_sequence,
                limit,
                max_bytes: self.maximum,
                filter,
            };

            // Project Only The Selected Headers
            let page = self.runtime.browse_cycles(query).await.map_err(wire::query_failure)?;
            project(page, self.maximum).map_err(|source| wire::query_failure(source.into()))
        })
    }
}

/* === Public Functions === */

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<BrowseCycles>::new(served.clone()))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies).operation::<BrowseCycles>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
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
    Ok(CycleBrowsePage {
        entries,
        at: Some(wire::cursor_wire_ref(&page.at)),
        next_after_sequence: page.next,
    })
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
