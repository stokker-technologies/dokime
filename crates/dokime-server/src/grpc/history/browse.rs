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
use std::num::NonZeroUsize;
use std::sync::Arc;

// Third Party Imports
use harmos::Protobuf;
use harmos::operations::{Context, Operation, Response};
use tonic::Status;

// Local Imports
use crate::grpc::value::{self, canonical};
use dokime_app::RuntimeHandle;
use dokime_app::core::TestExecutionId;
use dokime_app::execution::ExecutionStatus;
use dokime_app::runtime::query::execution_history::{ExecutionBrowsePage, ExecutionBrowseQuery};
use dokime_app::runtime::query::{ExecutionFilter, ExecutionOrder, ExecutionOrderKey};
use dokime_contract::operations::{execution as execution_wire, history as wire};

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
    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::BrowseExecutionsRequest,
        _: &mut (),
    ) -> Result<Response<wire::ExecutionBrowsePage>, Status> {
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
        let page = self
            .resources
            .runtime
            .browse_executions(query, |page, allowance| {
                project(page, allowance, self.resources.maximum)
            })
            .await
            .map_err(super::wire::query_failure)?;
        page.try_into_response()
            .map_err(|source| super::wire::query_failure(source.into()))
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Browse>(
        "dokime",
        dokime_app::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            maximum: maximum.min(4 * 1024 * 1024),
        }),
    )?;
    Ok(())
}

/* === Private Functions === */
fn project(
    page: &ExecutionBrowsePage,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<wire::ExecutionBrowsePage> {
    if construction_memory(page) > allowance.get() {
        return Err(exhausted());
    }
    let projected = super::wire::browse_page(page).map_err(|source| {
        if source.code() == tonic::Code::ResourceExhausted {
            return exhausted();
        }
        harmos::Error::backend(
            "project historical execution page",
            harmos::BackendFailure::Corrupt,
            source,
        )
    })?;
    if Protobuf::encoded_len(&projected) > maximum {
        return Err(exhausted());
    }
    Ok(projected)
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "historical response",
    }
}

fn construction_memory(page: &ExecutionBrowsePage) -> usize {
    let slots = std::mem::size_of::<wire::ExecutionBrowsePage>()
        .saturating_add(
            page.entries
                .len()
                .saturating_mul(std::mem::size_of::<wire::ExecutionBrowseEntry>()),
        )
        .saturating_add(
            page.conflicts
                .len()
                .saturating_mul(std::mem::size_of::<String>()),
        )
        .saturating_add(
            page.issues
                .len()
                .saturating_mul(std::mem::size_of::<wire::HistoryIssue>()),
        );
    let mut text = page.next.as_ref().map_or(0, |id| id.as_str().len());
    for id in &page.conflicts {
        text = text.saturating_add(id.as_str().len());
    }
    for issue in &page.issues {
        text = text
            .saturating_add(issue.message.len())
            .saturating_add(issue.source.as_ref().map_or(0, String::len))
            .saturating_add(issue.dataset.as_ref().map_or(0, String::len));
    }
    for entry in &page.entries {
        let metadata = &entry.metadata;
        for length in [
            metadata.execution_id.as_str().len(),
            metadata.cycle_id.as_str().len(),
            metadata.case_id.as_str().len(),
            metadata.title.len(),
            metadata.template_id.as_ref().map_or(0, String::len),
            metadata.suite_label.as_ref().map_or(0, String::len),
        ] {
            text = text.saturating_add(length);
        }
        if let Some(checkpoint) = &entry.checkpoint {
            text = text
                .saturating_add(checkpoint.execution_id.as_str().len())
                .saturating_add(harmos::Message::encoded_len(&checkpoint.version));
            for cursor in [&checkpoint.position, &checkpoint.evidence_through] {
                text = text
                    .saturating_add(cursor.history.store.as_str().len())
                    .saturating_add(cursor.history.epoch.as_str().len());
            }
        }
    }
    // Optional Canonical Messages Are Inline In Each Entry Slot. Timestamp And
    // Result Conversion Own No Heap Fields; All Copied Text Uses Exact Capacity.
    slots.saturating_add(text)
}

fn execution_filter(filter: Option<wire::ExecutionFilter>) -> Result<ExecutionFilter, Status> {
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
        statuses: filter
            .statuses
            .into_iter()
            .map(execution_status)
            .collect::<Result<_, Status>>()?,
        template_ids: filter.template_ids,
        cycle_ids: filter.cycle_ids,
        started_after: filter
            .started_after
            .map(canonical::decoded_timestamp)
            .transpose()?,
        started_before: filter
            .started_before
            .map(canonical::decoded_timestamp)
            .transpose()?,
    })
}

fn execution_order(order: Option<wire::ExecutionOrder>) -> Result<ExecutionOrder, Status> {
    let Some(order) = order else {
        return Ok(ExecutionOrder::default());
    };
    let key = match order.key {
        wire::ExecutionOrderKey::Unspecified | wire::ExecutionOrderKey::StartedAt => {
            ExecutionOrderKey::StartedAt
        }
        wire::ExecutionOrderKey::FinishedAt => ExecutionOrderKey::FinishedAt,
        wire::ExecutionOrderKey::Title => ExecutionOrderKey::Title,
    };
    Ok(ExecutionOrder {
        key,
        descending: order.descending,
    })
}

fn execution_status(status: execution_wire::ExecutionStatus) -> Result<ExecutionStatus, Status> {
    match status {
        execution_wire::ExecutionStatus::Unspecified => {
            Err(Status::invalid_argument("unspecified execution status"))
        }
        execution_wire::ExecutionStatus::Invalidated => Ok(ExecutionStatus::Invalidated),
        execution_wire::ExecutionStatus::Skipped => Ok(ExecutionStatus::Skipped),
        execution_wire::ExecutionStatus::Pending => Ok(ExecutionStatus::Pending),
        execution_wire::ExecutionStatus::Running => Ok(ExecutionStatus::Running),
        execution_wire::ExecutionStatus::Passed => Ok(ExecutionStatus::Passed),
        execution_wire::ExecutionStatus::Failed => Ok(ExecutionStatus::Failed),
        execution_wire::ExecutionStatus::Blocked => Ok(ExecutionStatus::Blocked),
        execution_wire::ExecutionStatus::Cancelled => Ok(ExecutionStatus::Cancelled),
        execution_wire::ExecutionStatus::Interrupted => Ok(ExecutionStatus::Interrupted),
    }
}

/* === Tests === */
