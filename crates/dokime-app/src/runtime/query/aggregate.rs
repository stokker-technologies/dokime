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

// Third Party Imports
use harmos::{MemorySize, storage::QueryValue};
use serde::{Deserialize, Serialize};
// Local Imports
use crate::{Result, RuntimeHandle};
use dokime::core::Value;

/* === Constants === */
pub const AGGREGATE_ROW_CEILING: usize = 10_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregateQuery {
    pub source: AggregateSource,
    pub filters: Vec<AggregateFilter>,
    pub group_by: Vec<String>,
    pub aggregates: Vec<AggregateColumn>,
    pub limit: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregateFilter {
    pub column: String,
    pub op: AggregateOperator,
    pub values: Vec<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregateColumn {
    pub function: AggregateFunction,
    pub column: String,
    pub alias: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AggregateOutcome {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub complete: bool,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregateSource {
    Cycles,
    Cases,
    Executions,
    Steps,
    Records,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregateOperator {
    Eq,
    Neq,
    In,
    Gt,
    Gte,
    Lt,
    Lte,
    IsNull,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregateFunction {
    Count,
    Sum,
    Mean,
    Min,
    Max,
}

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn aggregate(&self, query: AggregateQuery) -> Result<QueryValue<AggregateOutcome>> {
        crate::runtime::storage::aggregate::evaluate(&self.execution_storage, &query).await
    }

    pub async fn aggregate_projected<U: MemorySize>(
        &self,
        query: AggregateQuery,
        map: impl FnOnce(&AggregateOutcome, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        crate::runtime::storage::aggregate::evaluate_projected(&self.execution_storage, &query, map)
            .await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
