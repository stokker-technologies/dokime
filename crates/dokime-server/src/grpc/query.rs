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
use dokime::harmos::operations::{Context, Operation, Response};
use tonic::Status;

// Local Imports
use super::error::detailed;
use super::value::{self, Budget, canonical};
use dokime::RuntimeHandle;
use dokime::domain::model::diagnostics::admission::AdmissionFailure;
use dokime::domain::operations::query as wire;
use dokime::runtime::query::aggregate::{AggregateColumn, AggregateFilter, AggregateOperator, AggregateQuery, AggregateSource};

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
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::Aggregate, _: &mut (),
    ) -> Result<Response<dokime::domain::model::query::wire::AggregateReply>, Status> {
        let query = decode(request)?;
        self.resources.runtime.aggregate(query).await.map_err(status).and_then(|outcome| {
            codec::project(&outcome, self.resources.max_message_bytes).map(Into::into).map_err(|error| status(error.into()))
        })
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    runtime.operations().register_wire_operation::<Aggregate>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            max_message_bytes: maximum,
        }),
    )?;
    Ok(())
}

/* === Private Functions === */

fn decode(request: &mut wire::Aggregate) -> Result<AggregateQuery, Status> {
    let mut budget = Budget::new()?;
    value::collection(request.filters.len())?;
    value::collection(request.group_by.len())?;
    value::collection(request.aggregates.len())?;
    let source = match request.source {
        dokime::domain::model::query::wire::QuerySource::Cycles => AggregateSource::Cycles,
        dokime::domain::model::query::wire::QuerySource::Cases => AggregateSource::Cases,
        dokime::domain::model::query::wire::QuerySource::Executions => AggregateSource::Executions,
        dokime::domain::model::query::wire::QuerySource::Steps => AggregateSource::Steps,
        dokime::domain::model::query::wire::QuerySource::Records => AggregateSource::Records,
        dokime::domain::model::query::wire::QuerySource::Unspecified => {
            return Err(Status::invalid_argument("aggregate source is unspecified"));
        }
    };
    let mut filters = Vec::with_capacity(request.filters.len());
    for filter in std::mem::take(&mut request.filters) {
        value::collection(filter.values.len())?;
        let op = match filter.op {
            dokime::domain::model::query::wire::QueryFilterOperator::Eq => AggregateOperator::Eq,
            dokime::domain::model::query::wire::QueryFilterOperator::Neq => AggregateOperator::Neq,
            dokime::domain::model::query::wire::QueryFilterOperator::In => AggregateOperator::In,
            dokime::domain::model::query::wire::QueryFilterOperator::Gt => AggregateOperator::Gt,
            dokime::domain::model::query::wire::QueryFilterOperator::Gte => AggregateOperator::Gte,
            dokime::domain::model::query::wire::QueryFilterOperator::Lt => AggregateOperator::Lt,
            dokime::domain::model::query::wire::QueryFilterOperator::Lte => AggregateOperator::Lte,
            dokime::domain::model::query::wire::QueryFilterOperator::IsNull => AggregateOperator::IsNull,
            dokime::domain::model::query::wire::QueryFilterOperator::Unspecified => {
                return Err(Status::invalid_argument("aggregate filter operator is unspecified"));
            }
        };
        let values = filter.values.into_iter().map(|value| canonical::decode(value, &mut budget, 0)).collect::<Result<_, Status>>()?;
        filters.push(AggregateFilter { column: filter.column, op, values });
    }
    let mut aggregates = Vec::with_capacity(request.aggregates.len());
    for aggregate in std::mem::take(&mut request.aggregates) {
        if !aggregate.aggregate.answerable() {
            return Err(Status::invalid_argument("aggregate percentile rank exceeds 100"));
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

fn status(error: dokime::runtime::Error) -> Status {
    let status = match &error {
        dokime::runtime::Error::Admission(AdmissionFailure::Validation { .. }) => {
            Status::invalid_argument(dokime::plugin::Diagnostic::capture(&error).redacted().message)
        }
        dokime::runtime::Error::Admission(AdmissionFailure::NotFound { .. }) => Status::not_found("aggregate source was not found"),
        dokime::runtime::Error::Harmos(dokime::harmos::Error::ResourceExhausted { .. }) => {
            Status::resource_exhausted("aggregate response exceeds its original budget")
        }
        dokime::runtime::Error::Admission(AdmissionFailure::Overload { .. }) => Status::resource_exhausted("aggregate reads are busy"),
        dokime::runtime::Error::Admission(AdmissionFailure::Unavailable { .. })
        | dokime::runtime::Error::ShuttingDown
        | dokime::runtime::Error::Interrupted => Status::unavailable("aggregate query could not be served"),
        _ => Status::internal("aggregate query failed"),
    };
    detailed(status, &error)
}

/* === Tests === */
