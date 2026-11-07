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
use dokime::harmos::{Context, Response};
use tonic::Status;

// Local Imports
use crate::grpc::transcode::Transcode;
use crate::grpc::value::{self, canonical};
use dokime::app::LiveRuntime;
use dokime::domain::model::execution::result::ExecutionStatus;
use dokime::domain::model::identity::TestExecutionId;
use dokime::operations::history as wire;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::runtime::query::execution_history::{ExecutionBrowsePage, ExecutionBrowseQuery};
use dokime::runtime::query::{ExecutionFilter, ExecutionOrder, ExecutionOrderKey};

/* === Constants === */

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

impl Serve<wire::BrowseExecutions> for Resources {
    fn serve<'a>(
        &'a self, mut request: wire::BrowseExecutions, _: Context<'a, wire::BrowseExecutions>,
    ) -> Served<'a, dokime::domain::model::history::wire::ExecutionBrowsePage> {
        Box::pin(async move {
            let after = if request.after_execution_id.is_empty() {
                None
            } else {
                Some(
                    TestExecutionId::from_uuid(request.after_execution_id.clone())
                        .map_err(|_| Status::invalid_argument("invalid execution cursor ID"))?,
                )
            };
            let query = ExecutionBrowseQuery {
                after,
                limit: (request.limit != 0).then_some(request.limit as usize),
                filter: execution_filter(request.filter.take())?,
                order: execution_order(request.order.take())?,
            };
            let page =
                dokime::app::live_runtime(&self.runtime).await?.browse_executions(query).await.map_err(super::wire::query_failure)?;
            project(page, self.maximum).map_err(|source| super::wire::query_failure(source.into()))
        })
    }
}

/* === Public Functions === */

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });

    setup.resource::<Host<wire::BrowseExecutions>>(Host::<wire::BrowseExecutions>::new(served));

    Ok(())
}

/* === Private Functions === */

fn project(
    page: dokime::harmos::storage::ReadBatch<ExecutionBrowsePage>, maximum: usize,
) -> dokime::harmos::Result<Response<dokime::domain::model::history::wire::ExecutionBrowsePage>> {
    let projected = page.try_map(|page| super::wire::browse_page(page, maximum).map_err(projection_failure))?;
    if projected.wire_len()? > maximum {
        return Err(exhausted());
    }
    Ok(Response::from_owner(projected.owner()))
}

fn projection_failure(source: Status) -> dokime::harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return exhausted();
    }
    dokime::harmos::Error::backend("project historical execution page", dokime::harmos::BackendFailure::Corrupt, source)
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted { resource: "historical response" }
}

fn execution_filter(filter: Option<dokime::domain::model::history::wire::ExecutionFilter>) -> Result<ExecutionFilter, Status> {
    let Some(filter) = filter else {
        return Ok(ExecutionFilter::default());
    };
    value::collection(filter.statuses.len())?;
    value::collection(filter.template_ids.len())?;
    value::collection(filter.cycle_ids.len())?;
    Ok(ExecutionFilter {
        text: filter.text,
        statuses: filter.statuses.into_iter().map(execution_status).collect::<Result<_, Status>>()?,
        template_ids: filter.template_ids,
        cycle_ids: filter.cycle_ids,
        started_after: filter.started_after.map(canonical::decoded_timestamp).transpose()?,
        started_before: filter.started_before.map(canonical::decoded_timestamp).transpose()?,
        archive: crate::grpc::execution::archive_scope(filter.archive),
    })
}

fn execution_order(order: Option<dokime::domain::model::history::wire::ExecutionOrder>) -> Result<ExecutionOrder, Status> {
    let Some(order) = order else {
        return Ok(ExecutionOrder::default());
    };
    let key = match order.key {
        dokime::domain::model::history::wire::ExecutionOrderKey::Unspecified
        | dokime::domain::model::history::wire::ExecutionOrderKey::StartedAt => ExecutionOrderKey::StartedAt,
        dokime::domain::model::history::wire::ExecutionOrderKey::FinishedAt => ExecutionOrderKey::FinishedAt,
        dokime::domain::model::history::wire::ExecutionOrderKey::Title => ExecutionOrderKey::Title,
    };
    Ok(ExecutionOrder { key, descending: order.descending })
}

fn execution_status(status: dokime::domain::model::execution::wire::ExecutionStatus) -> Result<ExecutionStatus, Status> {
    match status {
        dokime::domain::model::execution::wire::ExecutionStatus::Unspecified => {
            Err(Status::invalid_argument("unspecified execution status"))
        }
        dokime::domain::model::execution::wire::ExecutionStatus::Invalidated => Ok(ExecutionStatus::Invalidated),
        dokime::domain::model::execution::wire::ExecutionStatus::Skipped => Ok(ExecutionStatus::Skipped),
        dokime::domain::model::execution::wire::ExecutionStatus::Pending => Ok(ExecutionStatus::Pending),
        dokime::domain::model::execution::wire::ExecutionStatus::Running => Ok(ExecutionStatus::Running),
        dokime::domain::model::execution::wire::ExecutionStatus::Passed => Ok(ExecutionStatus::Passed),
        dokime::domain::model::execution::wire::ExecutionStatus::Failed => Ok(ExecutionStatus::Failed),
        dokime::domain::model::execution::wire::ExecutionStatus::Blocked => Ok(ExecutionStatus::Blocked),
        dokime::domain::model::execution::wire::ExecutionStatus::Cancelled => Ok(ExecutionStatus::Cancelled),
        dokime::domain::model::execution::wire::ExecutionStatus::Interrupted => Ok(ExecutionStatus::Interrupted),
    }
}

/* === Tests === */
