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
mod generated {
    include!(concat!(env!("OUT_DIR"), "/dokime.v1.Query.rs"));
}

/* === Re-Exports === */
pub(super) use generated::query_server::QueryServer;

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use tonic::{Request, Response, Status};
// Local Imports
use super::error::detailed;
use super::value::{self, Budget};
use dokime_app::RuntimeHandle;
use dokime_app::runtime::query::aggregate::{
    AggregateColumn, AggregateFilter, AggregateFunction, AggregateOperator, AggregateQuery,
    AggregateSource,
};
use dokime_protocol::v1;
use generated::query_server::Query;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[tonic::async_trait]
impl Query for Service {
    async fn aggregate(
        &self,
        request: Request<v1::AggregateRequest>,
    ) -> Result<Response<super::charged::Bytes>, Status> {
        let query = decode(request.into_inner())?;
        let reply = self
            .runtime
            .aggregate_projected(query, |outcome, allowance| {
                codec::encode(outcome, allowance, self.max_message_bytes)
            })
            .await
            .map_err(status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn decode(request: v1::AggregateRequest) -> Result<AggregateQuery, Status> {
    let mut budget = Budget::new();
    value::collection(request.filters.len())?;
    value::collection(request.group_by.len())?;
    value::collection(request.aggregates.len())?;
    let source = match v1::QuerySource::try_from(request.source) {
        Ok(v1::QuerySource::Cycles) => AggregateSource::Cycles,
        Ok(v1::QuerySource::Cases) => AggregateSource::Cases,
        Ok(v1::QuerySource::Executions) => AggregateSource::Executions,
        Ok(v1::QuerySource::Steps) => AggregateSource::Steps,
        Ok(v1::QuerySource::Records) => AggregateSource::Records,
        Ok(v1::QuerySource::Unspecified) | Err(_) => {
            return Err(Status::invalid_argument("aggregate source is unspecified"));
        }
    };
    let mut filters = Vec::with_capacity(request.filters.len());
    for filter in request.filters {
        value::text(&filter.column)?;
        value::collection(filter.values.len())?;
        let op = match v1::QueryFilterOperator::try_from(filter.op) {
            Ok(v1::QueryFilterOperator::Eq) => AggregateOperator::Eq,
            Ok(v1::QueryFilterOperator::Neq) => AggregateOperator::Neq,
            Ok(v1::QueryFilterOperator::In) => AggregateOperator::In,
            Ok(v1::QueryFilterOperator::Gt) => AggregateOperator::Gt,
            Ok(v1::QueryFilterOperator::Gte) => AggregateOperator::Gte,
            Ok(v1::QueryFilterOperator::Lt) => AggregateOperator::Lt,
            Ok(v1::QueryFilterOperator::Lte) => AggregateOperator::Lte,
            Ok(v1::QueryFilterOperator::IsNull) => AggregateOperator::IsNull,
            Ok(v1::QueryFilterOperator::Unspecified) | Err(_) => {
                return Err(Status::invalid_argument(
                    "aggregate filter operator is unspecified",
                ));
            }
        };
        let values = filter
            .values
            .into_iter()
            .map(|value| value::decode(value, &mut budget, 0))
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
    for aggregate in request.aggregates {
        if !aggregate.column.is_empty() {
            value::text(&aggregate.column)?;
        }
        value::text(&aggregate.alias)?;
        let function = match v1::AggregateFunction::try_from(aggregate.function) {
            Ok(v1::AggregateFunction::Count) => AggregateFunction::Count,
            Ok(v1::AggregateFunction::Sum) => AggregateFunction::Sum,
            Ok(v1::AggregateFunction::Mean) => AggregateFunction::Mean,
            Ok(v1::AggregateFunction::Min) => AggregateFunction::Min,
            Ok(v1::AggregateFunction::Max) => AggregateFunction::Max,
            Ok(v1::AggregateFunction::Unspecified) | Err(_) => {
                return Err(Status::invalid_argument(
                    "aggregate function is unspecified",
                ));
            }
        };
        aggregates.push(AggregateColumn {
            function,
            column: aggregate.column,
            alias: aggregate.alias,
        });
    }

    Ok(AggregateQuery {
        source,
        filters,
        group_by: request.group_by,
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
