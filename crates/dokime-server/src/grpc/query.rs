/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod codec;

use std::sync::Arc;

use dokime::harmos::{Context, RegistrationExt};
use tonic::Status;

use super::error::detailed;
use super::value::{self, Budget, canonical};
use dokime::RuntimeHandle;
use dokime::domain::model::diagnostics::admission::AdmissionFailure;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::query as wire;
use dokime::runtime::query::aggregate::{AggregateColumn, AggregateFilter, AggregateOperator, AggregateQuery, AggregateSource};

struct Resources {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

impl Serve<wire::Aggregate> for Resources {
    fn serve<'a>(
        &'a self, mut request: wire::Aggregate, _: Context<'a, wire::Aggregate>,
    ) -> Served<'a, dokime::domain::model::query::wire::AggregateReply> {
        Box::pin(async move {
            let query = decode(&mut request)?;
            self.runtime
                .aggregate(query)
                .await
                .map_err(status)
                .and_then(|outcome| codec::project(&outcome, self.max_message_bytes).map(Into::into).map_err(|error| status(error.into())))
        })
    }
}

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources {
        runtime: runtime.clone(),
        max_message_bytes: maximum,
    });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<wire::Aggregate>::new(served.clone()))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies).operation::<wire::Aggregate>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

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
