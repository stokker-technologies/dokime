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
use crate::grpc::transcode::Transcode;
use crate::grpc::value::{self, canonical};
use dokime::RuntimeHandle;
use dokime::domain::model::execution::result::ExecutionStatus;
use dokime::domain::model::identity::TestExecutionId;
use dokime::domain::operations::history as wire;
use dokime::runtime::query::execution_history::{ExecutionBrowsePage, ExecutionBrowseQuery};
use dokime::runtime::query::{ExecutionFilter, ExecutionOrder, ExecutionOrderKey};

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
    type Declaration = wire::BrowseExecutions;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::BrowseExecutions, _: &mut (),
    ) -> Result<Response<dokime::domain::model::history::wire::ExecutionBrowsePage>, Status> {
        value::text(&request.after_execution_id)?;
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
        let page = self.resources.runtime.browse_executions(query).await.map_err(super::wire::query_failure)?;
        project(&page, self.resources.maximum).map(Into::into).map_err(|source| super::wire::query_failure(source.into()))
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

fn project(
    page: &ExecutionBrowsePage, maximum: usize,
) -> dokime::harmos::Result<dokime::domain::model::history::wire::ExecutionBrowsePage> {
    let projected = super::wire::browse_page(page).map_err(|source| {
        if source.code() == tonic::Code::ResourceExhausted {
            return exhausted();
        }
        dokime::harmos::Error::backend("project historical execution page", dokime::harmos::BackendFailure::Corrupt, source)
    })?;
    if projected.wire_len()? > maximum {
        return Err(exhausted());
    }
    Ok(projected)
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted { resource: "historical response" }
}

fn execution_filter(filter: Option<dokime::domain::model::history::wire::ExecutionFilter>) -> Result<ExecutionFilter, Status> {
    let Some(filter) = filter else {
        return Ok(ExecutionFilter::default());
    };
    value::text(&filter.text)?;
    value::collection(filter.statuses.len())?;
    value::collection(filter.template_ids.len())?;
    value::collection(filter.cycle_ids.len())?;
    for identifier in filter.template_ids.iter().chain(filter.cycle_ids.iter()) {
        value::text(identifier)?;
    }
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
