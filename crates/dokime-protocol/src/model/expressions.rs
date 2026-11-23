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
    Aggregate, AggregateExpression, AtExpression, DataExpression, FieldReference, TransformExpression, data_expression,
    transform_expression,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/// One Source an Expression Reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExpressionSource<'a> {
    Stream(&'a str),
    Record(&'a str),
}

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
        let reference = Self {
            identifier: self.identifier.clone(),
            field: field.into(),
        };
        DataExpression {
            label: String::new(),
            kind: Some(data_expression::Kind::Stream(reference)),
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

    pub fn at(self, record_type: impl Into<String>) -> Self {
        self.transform(transform_expression::Kind::At(AtExpression {
            record_type: record_type.into(),
            within_s: None,
        }))
    }

    pub fn within_s(mut self, tolerance_s: f64) -> Self {
        let Some(data_expression::Kind::Transform(transform)) = self.kind.as_mut() else {
            panic!("Set Within Seconds: Apply at Before within_s");
        };
        let Some(transform_expression::Kind::At(at)) = transform.kind.as_mut() else {
            panic!("Set Within Seconds: Apply at Before within_s");
        };
        at.within_s = Some(tolerance_s);
        self
    }

    pub fn at_record(&self) -> Option<&AtExpression> {
        match self.kind.as_ref()? {
            data_expression::Kind::Transform(transform) => match transform.kind.as_ref()? {
                transform_expression::Kind::At(at) => Some(at),
                _ => transform.input.as_deref()?.at_record(),
            },
            data_expression::Kind::Aggregate(aggregate) => aggregate.input.as_deref()?.at_record(),
            _ => None,
        }
    }

    /// Every Stream and Record Type the Expression Reads, Including Records That Sample or Time It.
    pub fn sources(&self) -> Vec<ExpressionSource<'_>> {
        let mut sources = Vec::new();
        self.collect_sources(&mut sources);
        sources
    }

    fn collect_sources<'a>(&'a self, sources: &mut Vec<ExpressionSource<'a>>) {
        let input = match self.kind.as_ref() {
            Some(data_expression::Kind::Stream(reference)) => {
                sources.push(ExpressionSource::Stream(&reference.identifier));
                None
            }
            Some(data_expression::Kind::Record(reference)) => {
                sources.push(ExpressionSource::Record(&reference.identifier));
                None
            }
            Some(data_expression::Kind::Aggregate(aggregate)) => aggregate.input.as_deref(),
            Some(data_expression::Kind::Transform(transform)) => {
                match transform.kind.as_ref() {
                    Some(transform_expression::Kind::At(at)) => sources.push(ExpressionSource::Record(&at.record_type)),
                    Some(transform_expression::Kind::TimeBetween(previous)) => sources.push(ExpressionSource::Record(previous)),
                    _ => (),
                }
                transform.input.as_deref()
            }
            _ => None,
        };
        if let Some(input) = input {
            input.collect_sources(sources);
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        self.validate_at(0)
    }

    fn validate_at(&self, depth: usize) -> Result<(), String> {
        let name = if self.label.is_empty() { "at" } else { &self.label };
        let invalid = |action: &str| format!("Validate Expression '{name}': {action}");
        if depth > 32 {
            return Err(invalid("Reduce Nesting to 32 Levels"));
        }
        match self.kind.as_ref() {
            Some(data_expression::Kind::Transform(transform)) => {
                let input = transform.input.as_deref().ok_or_else(|| invalid("Supply the Transform Input"))?;
                if let Some(transform_expression::Kind::At(at)) = &transform.kind {
                    if !input.is_telemetry() {
                        return Err(invalid("Apply at to a Telemetry Source"));
                    }
                    if at.record_type.is_empty() {
                        return Err(invalid("Name the Record Type for at"));
                    }
                    if at.within_s.is_some() {
                        return Err(invalid("Remove within_s Until the Pinned Library Supports Nearest-Within Merging"));
                    }
                }
                input.validate_at(depth + 1)
            }
            Some(data_expression::Kind::Aggregate(aggregate)) => {
                aggregate.input.as_deref().ok_or_else(|| invalid("Supply the Aggregate Input"))?.validate_at(depth + 1)
            }
            _ => Ok(()),
        }
    }

    fn is_telemetry(&self) -> bool {
        match self.kind.as_ref() {
            Some(data_expression::Kind::Stream(_)) => true,
            Some(data_expression::Kind::Transform(transform))
                if matches!(
                    transform.kind,
                    Some(
                        transform_expression::Kind::Scale(_)
                            | transform_expression::Kind::Offset(_)
                            | transform_expression::Kind::Detail(_)
                    )
                ) =>
            {
                transform.input.as_deref().is_some_and(Self::is_telemetry)
            }
            _ => false,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;

    #[test]
    fn at_wire_preserves_optional_tolerance_and_owned_operands() {
        let input = FieldReference::new("readings").field("value").label("Distance");
        let sampled = input.clone().at("status");
        assert_eq!(sampled.at_record().unwrap().within_s, None);
        let bounded = sampled.clone().within_s(0.5);
        let decoded = DataExpression::decode(bounded.encode_to_vec().as_slice()).unwrap();
        assert_eq!(decoded.at_record().unwrap().within_s, Some(0.5));
        assert_eq!(decoded.at_record().unwrap().record_type, "status");
        assert_eq!(decoded.label, "Distance");
        assert_eq!(sampled.at_record().unwrap().within_s, None);
        assert!(input.at_record().is_none());
        assert!(sampled.validate().is_ok());
        assert!(bounded.validate().unwrap_err().contains("Pinned Library"));
    }

    #[test]
    fn at_refuses_non_telemetry_sources_and_nested_sampling() {
        let sources = [
            data_expression::Kind::Record(FieldReference::new("status")),
            data_expression::Kind::Parameter("limit".into()),
            data_expression::Kind::Dimension("case".into()),
            data_expression::Kind::Artifact("capture".into()),
        ];
        for source in sources {
            let expression = DataExpression {
                label: "Invalid Sample".into(),
                kind: Some(source),
            }
            .at("status");
            let error = expression.validate().unwrap_err();
            assert!(error.contains("Invalid Sample"));
            assert!(error.contains("Telemetry Source"));
        }
        assert!(FieldReference::new("readings").field("value").at("status").at("status").validate().is_err());
    }
}
