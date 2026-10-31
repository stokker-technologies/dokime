/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finite Query Reduction Contracts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::{
    error::{Failure, Reason},
    schema::Schema,
    snapshot::Snapshot,
};
use crate::model::identity::wire::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub alias: String,
    pub field: String,
    pub aggregate: harmos::Aggregate,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BucketRequest {
    pub every_nanos: u64,
    pub aggregate: harmos::Aggregate,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Bucket {
    pub start: Option<Timestamp>,
    pub end: Option<Timestamp>,
    pub execution_id: String,
    pub stream: String,
    pub source: String,
    pub versions: Vec<u32>,
    pub sample_count: u64,
    pub fields: Vec<BucketValue>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BucketValue {
    pub field: String,
    pub count: u64,
    pub value: Option<StatisticValue>,
    pub unit: Option<String>,
    pub reducer: harmos::Aggregate,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityCounts {
    pub field: String,
    pub finite: u64,
    pub missing: u64,
    pub null: u64,
    pub non_finite: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Statistics {
    pub schema: Option<Schema>,
    pub values: Vec<Statistic>,
    pub selected_rows: u64,
    pub quality_counts: Vec<QualityCounts>,
    pub snapshot: Option<Snapshot>,
    pub complete: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Statistic {
    pub alias: String,
    pub field: String,
    pub aggregate: harmos::Aggregate,
    pub value: Option<StatisticValue>,
}

/* === Enums === */

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum StatisticValue {
    Count(u64),
    Number(f64),
}

/* === Traits === */

/* === Implementations === */

impl BucketRequest {
    pub fn validate(&self, operation: &str) -> Result<(), Box<Failure>> {
        if self.every_nanos == 0
            || !matches!(
                self.aggregate,
                harmos::Aggregate::Count
                    | harmos::Aggregate::Sum
                    | harmos::Aggregate::Mean
                    | harmos::Aggregate::Min
                    | harmos::Aggregate::Max
            )
        {
            return Err(Failure::new(
                Reason::InvalidSelection,
                operation,
                "Buckets require a positive fixed duration and count/sum/mean/min/max",
            ));
        }
        Ok(())
    }
}

impl Output {
    pub fn validate(&self, operation: &str) -> Result<(), Box<Failure>> {
        if self.alias.is_empty()
            || (self.field.is_empty() && !matches!(self.aggregate, harmos::Aggregate::Count))
            || matches!(self.aggregate, harmos::Aggregate::Percentile(rank) if rank > 100)
        {
            return Err(Failure::new(
                Reason::InvalidSelection,
                operation,
                "Use a unique nonempty alias, a numeric field (except row count), and percentile rank 0..100",
            ));
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduction_shapes() {
        assert!(
            BucketRequest {
                every_nanos: 0,
                aggregate: harmos::Aggregate::Mean
            }
            .validate("test.query")
            .is_err()
        );
        assert!(
            BucketRequest {
                every_nanos: 1,
                aggregate: harmos::Aggregate::Median
            }
            .validate("test.query")
            .is_err()
        );
        assert!(
            Output {
                alias: "p".into(),
                field: "x".into(),
                aggregate: harmos::Aggregate::Percentile(101)
            }
            .validate("test.query")
            .is_err()
        );
        assert!(
            Output {
                alias: "rows".into(),
                field: String::new(),
                aggregate: harmos::Aggregate::Count
            }
            .validate("test.query")
            .is_ok()
        );
    }
}
