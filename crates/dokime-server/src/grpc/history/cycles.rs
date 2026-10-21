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
use crate::grpc::value::canonical;
use dokime::RuntimeHandle;
use dokime::domain::model::history::wire::{CycleBrowsePage, SavedCycleSummary};
use dokime::domain::operations::history::BrowseCycles;
use dokime::runtime::query::cycle_history::CycleBrowseQuery;

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
        // Validate The Page Before Reading Storage
        let limit = if request.limit == 0 { 100 } else { request.limit as usize };
        if limit > 1000 || (request.after_sequence != 0 && request.at.is_none()) {
            return Err(Status::invalid_argument("cycle continuation requires its snapshot; limit must be 1..=1000"));
        }
        let at = request.at.take().map(|at| wire::cursor(Some(at))).transpose()?;
        let saved = self.resources.runtime.storage().cursor_saved();
        let position = at.as_ref().unwrap_or(&saved);
        if position.history != saved.history || position.sequence > saved.sequence || request.after_sequence > position.sequence {
            return Err(Status::invalid_argument("invalid historical cycle snapshot or continuation"));
        }
        let filter = crate::grpc::execution::cycle_filter(request.filter.as_ref())?;
        let query = CycleBrowseQuery { at, after: request.after_sequence, limit, max_bytes: self.resources.maximum, filter };

        // Project Only The Selected Headers
        let page = self.resources.runtime.browse_cycles(query).await.map_err(wire::query_failure)?;
        let mut entries = Vec::with_capacity(page.entries.len());
        let mut bytes = std::mem::size_of::<CycleBrowsePage>();
        for cycle in page.entries {
            bytes = bytes.saturating_add(canonical::metadata_memory(&cycle.metadata)?);
            if bytes > self.resources.maximum {
                return Err(Status::resource_exhausted("historical cycle page exceeds its response budget"));
            }
            entries.push(SavedCycleSummary {
                cycle_id: cycle.cycle_id.to_string(),
                metadata: Some(canonical::metadata(&cycle.metadata)?),
                phase: cycle.phase,
                created_at: Some(canonical::timestamp(&cycle.created_at)?),
                updated_at: Some(canonical::timestamp(&cycle.updated_at)?),
                execution_count: cycle.execution_count as u64,
                archived: cycle.archived,
            });
        }
        let reply = CycleBrowsePage { entries, at: Some(wire::cursor_wire_ref(&page.at)), next_after_sequence: page.next };
        let length = reply.wire_len().map_err(|error| Status::internal(format!("historical cycle projection failed: {error}")))?;
        if length > self.resources.maximum {
            return Err(Status::resource_exhausted("historical cycle page exceeds its response budget"));
        }
        Ok(reply.into())
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Browse>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum: maximum.min(4 * 1024 * 1024) }),
    )?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
