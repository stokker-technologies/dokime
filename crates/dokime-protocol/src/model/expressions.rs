/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Expressions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::model::{
    Aggregate, AggregateExpression, DataExpression, FieldReference, TransformExpression, data_expression, transform_expression,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl FieldReference {
    pub fn new(identifier: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            field: String::new(),
        }
    }

    pub fn field(&self, field: impl Into<String>) -> DataExpression {
        DataExpression {
            label: String::new(),
            kind: Some(data_expression::Kind::Stream(Self {
                identifier: self.identifier.clone(),
                field: field.into(),
            })),
        }
    }
}

impl DataExpression {
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn scale(self, factor: f64) -> Self {
        self.transform(transform_expression::Kind::Scale(factor))
    }

    pub fn offset(self, value: f64) -> Self {
        self.transform(transform_expression::Kind::Offset(value))
    }

    pub fn detail(self, field: impl Into<String>) -> Self {
        self.transform(transform_expression::Kind::Detail(field.into()))
    }

    pub fn time_between(self, record: impl Into<String>) -> Self {
        self.transform(transform_expression::Kind::TimeBetween(record.into()))
    }

    pub fn per_case(self) -> Self {
        self.transform(transform_expression::Kind::PerCase(true))
    }

    pub fn aggregate(self, kind: Aggregate) -> Self {
        self.aggregate_with_percentile(kind, 0.0)
    }

    pub fn percentile(self, rank: f64) -> Self {
        self.aggregate_with_percentile(Aggregate::Percentile, rank)
    }

    fn aggregate_with_percentile(self, kind: Aggregate, percentile: f64) -> Self {
        let label = self.label.clone();
        let aggregate = AggregateExpression {
            input: Some(Box::new(self)),
            kind: kind as i32,
            percentile,
        };
        Self {
            label,
            kind: Some(data_expression::Kind::Aggregate(Box::new(aggregate))),
        }
    }

    fn transform(self, kind: transform_expression::Kind) -> Self {
        let label = self.label.clone();
        let transform = TransformExpression {
            input: Some(Box::new(self)),
            kind: Some(kind),
        };
        Self {
            label,
            kind: Some(data_expression::Kind::Transform(Box::new(transform))),
        }
    }

    pub fn count(self) -> Self {
        self.aggregate(Aggregate::Count)
    }

    pub fn sum(self) -> Self {
        self.aggregate(Aggregate::Sum)
    }

    pub fn min(self) -> Self {
        self.aggregate(Aggregate::Min)
    }

    pub fn max(self) -> Self {
        self.aggregate(Aggregate::Max)
    }

    pub fn mean(self) -> Self {
        self.aggregate(Aggregate::Mean)
    }

    pub fn std(self) -> Self {
        self.aggregate(Aggregate::Stddev)
    }

    pub fn median(self) -> Self {
        self.aggregate(Aggregate::Median)
    }

    pub fn rms(self) -> Self {
        self.aggregate(Aggregate::Rms)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
