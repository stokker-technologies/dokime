/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod aggregate;

/* === Re-Exports === */
pub use aggregate::Aggregate;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::value::Value;
use super::{Access, Endpoint, ORDINARY};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// A zero limit means the server ceiling of 10,000 rows. Larger values are
// clamped to it rather than rejected. `group_by` may be empty, which
// aggregates the whole filtered selection into a single row. `aggregates` must
// name at least one output.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregateRequest {
    pub source: QuerySource,
    pub filters: Vec<QueryFilter>,
    pub group_by: Vec<String>,
    pub aggregates: Vec<QueryAggregate>,
    pub limit: u32,
}

// `values` carries exactly one value for EQ, NEQ, GT, GTE, LT and LTE, one or
// more for IN, and none for IS_NULL. Values are compared against the column's
// own type; a value that cannot be coerced to it is rejected.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryFilter {
    pub column: String,
    pub op: QueryFilterOperator,
    pub values: Vec<Value>,
}

// COUNT with an empty column counts rows; COUNT with a column counts its
// non-null values. SUM, MEAN, MIN and MAX require a column. Null inputs are
// skipped by every function, so MEAN over a nullable column is the mean of the
// rows that have a value. `alias` names the output column and must be unique
// across the request and distinct from every `group_by` name.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryAggregate {
    pub function: AggregateFunction,
    pub column: String,
    pub alias: String,
}

// `columns` lists the group-by names in request order followed by the aggregate
// aliases in request order; every row has exactly that many values in that
// order. `complete` is false when the selection had more groups than the
// effective limit returned.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregateReply {
    pub columns: Vec<String>,
    pub rows: Vec<QueryRow>,
    pub complete: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryRow {
    pub values: Vec<Value>,
}

/* === Enums === */
// Native saved sources a caller may aggregate. Telemetry samples retain their
// declared-schema reader through Telemetry.ListSamples.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuerySource {
    Unspecified,
    Cycles,
    Cases,
    Executions,
    // Step evidence joined to its execution and that execution's frozen case, so
    // template and execution-level questions are expressible without a
    // client-side join. Beside the stored step columns this source carries
    // `template_identifier`, `execution_status`, `cycle_identifier` and the
    // derived `duration_ms` (float milliseconds, null unless the step has both
    // `started_at` and `finished_at`).
    Steps,
    Records,
}

// Named for its service: `FilterOperator` already belongs to the plot
// contract, and enum value names are siblings of their type in one package.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryFilterOperator {
    Unspecified,
    Eq,
    Neq,
    In,
    Gt,
    Gte,
    Lt,
    Lte,
    IsNull,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregateFunction {
    Unspecified,
    Count,
    Sum,
    Mean,
    Min,
    Max,
}

#[harmos::catalog]
pub enum Operations {
    Aggregate(Aggregate),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for Aggregate {
    const PATH: &'static str = "/dokime.v1.Query/Aggregate";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
