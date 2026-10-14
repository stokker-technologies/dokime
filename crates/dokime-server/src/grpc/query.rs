/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod codec;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use harmos::operations::{Context, Operation, Response};
use tonic::Status;
// Local Imports
use super::error::detailed;
use super::value::{self, Budget, canonical};
use dokime_app::RuntimeHandle;
use dokime_app::runtime::query::aggregate::{
    AggregateColumn, AggregateFilter, AggregateOperator, AggregateQuery, AggregateSource,
};
use dokime_contract::operations::query as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

struct Aggregate {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Operation for Aggregate {
    type Declaration = wire::Aggregate;
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
        request: &mut wire::AggregateRequest,
        _: &mut (),
    ) -> Result<Response<wire::AggregateReply>, Status> {
        let query = decode(request)?;
        self.resources
            .runtime
            .aggregate_projected(query, |outcome, allowance| {
                codec::project(outcome, allowance, self.resources.max_message_bytes)
            })
            .await
            .map_err(status)?
            .try_into_response()
            .map_err(|error| status(error.into()))
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    runtime.operations().register_wire_operation::<Aggregate>(
        "dokime",
        dokime_app::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            max_message_bytes: maximum,
        }),
    )?;
    Ok(())
}

/* === Private Functions === */
fn decode(request: &mut wire::AggregateRequest) -> Result<AggregateQuery, Status> {
    let mut budget = Budget::new();
    value::collection(request.filters.len())?;
    value::collection(request.group_by.len())?;
    value::collection(request.aggregates.len())?;
    let source = match request.source {
        wire::QuerySource::Cycles => AggregateSource::Cycles,
        wire::QuerySource::Cases => AggregateSource::Cases,
        wire::QuerySource::Executions => AggregateSource::Executions,
        wire::QuerySource::Steps => AggregateSource::Steps,
        wire::QuerySource::Records => AggregateSource::Records,
        wire::QuerySource::Unspecified => {
            return Err(Status::invalid_argument("aggregate source is unspecified"));
        }
    };
    let mut filters = Vec::with_capacity(request.filters.len());
    for filter in std::mem::take(&mut request.filters) {
        value::text(&filter.column)?;
        value::collection(filter.values.len())?;
        let op = match filter.op {
            wire::QueryFilterOperator::Eq => AggregateOperator::Eq,
            wire::QueryFilterOperator::Neq => AggregateOperator::Neq,
            wire::QueryFilterOperator::In => AggregateOperator::In,
            wire::QueryFilterOperator::Gt => AggregateOperator::Gt,
            wire::QueryFilterOperator::Gte => AggregateOperator::Gte,
            wire::QueryFilterOperator::Lt => AggregateOperator::Lt,
            wire::QueryFilterOperator::Lte => AggregateOperator::Lte,
            wire::QueryFilterOperator::IsNull => AggregateOperator::IsNull,
            wire::QueryFilterOperator::Unspecified => {
                return Err(Status::invalid_argument(
                    "aggregate filter operator is unspecified",
                ));
            }
        };
        let values = filter
            .values
            .into_iter()
            .map(|value| canonical::decode(value, &mut budget, 0))
            .collect::<Result<_, Status>>()?;
        filters.push(AggregateFilter {
            column: filter.column,
            op,
            values,
        });
    }
    for column in &request.group_by {
        value::text(column)?;
    }
    let mut aggregates = Vec::with_capacity(request.aggregates.len());
    for aggregate in std::mem::take(&mut request.aggregates) {
        if !aggregate.column.is_empty() {
            value::text(&aggregate.column)?;
        }
        value::text(&aggregate.alias)?;
        if !aggregate.aggregate.answerable() {
            return Err(Status::invalid_argument(
                "aggregate percentile rank exceeds 100",
            ));
        }
        aggregates.push(AggregateColumn {
            aggregate: aggregate.aggregate,
            column: aggregate.column,
            alias: aggregate.alias,
        });
    }

    Ok(AggregateQuery {
        source,
        filters,
        group_by: std::mem::take(&mut request.group_by),
        aggregates,
        limit: request.limit as usize,
    })
}

fn status(error: dokime_app::Error) -> Status {
    let status = match &error {
        dokime_app::Error::Validation { message, .. } => Status::invalid_argument(message.clone()),
        dokime_app::Error::NotFound { .. } => Status::not_found("aggregate source was not found"),
        dokime_app::Error::Harmos(harmos::Error::ResourceExhausted { .. }) => {
            Status::resource_exhausted("aggregate response exceeds its original budget")
        }
        dokime_app::Error::Overload { .. } => {
            Status::resource_exhausted("aggregate reads are busy")
        }
        dokime_app::Error::Unavailable { .. }
        | dokime_app::Error::ShuttingDown
        | dokime_app::Error::Interrupted => {
            Status::unavailable("aggregate query could not be served")
        }
        _ => Status::internal("aggregate query failed"),
    };
    detailed(status, &error)
}

/* === Tests === */
