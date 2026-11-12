/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Fixed Document Values

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Error, ExecutionCut, NumericWindow, Result, Store};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn document_value(&self, cut: &ExecutionCut, expression: &v1::DataExpression) -> Result<v1::Value> {
        use v1::data_expression::Kind;
        let value = match expression.kind.as_ref() {
            Some(Kind::Literal(value)) => return Ok(value.clone()),
            Some(Kind::Parameter(key) | Kind::Dimension(key)) => {
                let Some(v1::origin::Kind::PluginRun(run)) = cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) else {
                    return Err(Error::Invalid("Read Document Value: Select a Parameter from a PluginRun Origin".into()));
                };
                return run
                    .test_case
                    .as_ref()
                    .and_then(|case| case.parameters.get(key))
                    .cloned()
                    .ok_or_else(|| Error::Invalid(format!("Read Document Value: Declare Parameter {key}")));
            }
            Some(Kind::Stream(reference)) => self.document_statistic(cut, reference, v1::Aggregate::Last)?,
            Some(Kind::Aggregate(aggregate)) => {
                let Some(Kind::Stream(reference)) = aggregate.input.as_ref().and_then(|input| input.kind.as_ref()) else {
                    return Err(Error::Invalid("Read Document Aggregate: Select a Numeric Stream Field".into()));
                };
                self.document_statistic(
                    cut,
                    reference,
                    v1::Aggregate::try_from(aggregate.kind)
                        .map_err(|_| Error::Invalid("Read Document Aggregate: Select a Declared Reducer".into()))?,
                )?
            }
            Some(Kind::Transform(transform)) => {
                let input = transform.input.as_ref().ok_or_else(|| Error::Invalid("Read Document Transform: Select Its Input".into()))?;
                let value = self.document_value(cut, input)?;
                let number = match value.kind {
                    Some(v1::value::Kind::Number(value)) => value,
                    Some(v1::value::Kind::Integer(value)) => value as f64,
                    Some(v1::value::Kind::UnsignedInteger(value)) => value as f64,
                    _ => return Err(Error::Invalid("Read Document Transform: Select a Numeric Input".into())),
                };
                match transform.kind {
                    Some(v1::transform_expression::Kind::Scale(scale)) => number * scale,
                    Some(v1::transform_expression::Kind::Offset(offset)) => number + offset,
                    _ => return Err(Error::Invalid("Read Document Transform: Select Scale or Offset".into())),
                }
            }
            _ => return Err(Error::Invalid("Read Document Value: Bind a Stored Numeric Stream, Parameter or Literal".into())),
        };
        if !value.is_finite() {
            return Err(Error::Invalid("Read Document Value: Select Finite Input Values".into()));
        }
        Ok(v1::Value {
            kind: Some(v1::value::Kind::Number(value)),
        })
    }
    fn document_statistic(&self, cut: &ExecutionCut, reference: &v1::FieldReference, aggregate: v1::Aggregate) -> Result<f64> {
        let catalog = harmos_protocol::v1::Catalog::decode(cut.execution.catalog.as_slice())?;
        if !catalog.streams.iter().any(|stream| stream.id == reference.identifier) {
            return Err(Error::Invalid(format!("Read Document Value: Declare Stream {}", reference.identifier)));
        }
        let bounds =
            self.numeric_bounds(&cut.execution.identifier, &reference.identifier, &reference.field, cut.frontier)?.unwrap_or((0, 0));
        let window = NumericWindow {
            execution: &cut.execution.identifier,
            stream: &reference.identifier,
            field: &reference.field,
            after: 0,
            through: cut.frontier,
            start_ns: bounds.0,
            end_ns: bounds.1,
            bins: std::num::NonZeroUsize::new(1).unwrap(),
        };
        let stats = self.numeric_statistics(&window)?;
        if stats.count == 0 && aggregate != v1::Aggregate::Count {
            return Err(Error::Invalid("Read Document Value: Capture Samples Before Evaluating This Summary".into()));
        }
        Ok(match aggregate {
            v1::Aggregate::Count => stats.count as f64,
            v1::Aggregate::Sum => stats.mean * stats.count as f64,
            v1::Aggregate::Min => stats.min,
            v1::Aggregate::Max => stats.max,
            v1::Aggregate::Mean => stats.mean,
            v1::Aggregate::Stddev => stats.stddev,
            v1::Aggregate::Median => stats.median,
            v1::Aggregate::First => stats.first,
            v1::Aggregate::Last => stats.last,
            v1::Aggregate::Rms => {
                let bins = self.numeric_bins(window)?;
                (bins.iter().map(|bin| bin.sum_squares).sum::<f64>() / stats.count as f64).sqrt()
            }
            _ => return Err(Error::Invalid("Read Document Value: Select a Declared Summary Reducer".into())),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
