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
use std::cmp::Ordering;

// Third Party Imports

// Local Imports
use crate::{
    Error, ExecutionCut, Result, Store, expressions,
    scalar::{self, ScalarValue},
};
use dokime_protocol::v1::{self, Aggregate, DataExpression, PlotFilter, Value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn document_value(&self, cut: &ExecutionCut, expression: &DataExpression) -> Result<Value> {
        self.document_value_filtered(cut, expression, &[])
    }
    pub fn document_value_filtered(&self, cut: &ExecutionCut, expression: &DataExpression, filters: &[PlotFilter]) -> Result<Value> {
        if let Some(v1::data_expression::Kind::Aggregate(aggregate)) = &expression.kind {
            let value = self.reduce_expression(
                cut,
                aggregate.input.as_deref().ok_or(Error::Data("supply the aggregate input"))?,
                filters,
                Aggregate::try_from(aggregate.kind).map_err(|_| Error::Data("select a reducer"))?,
                aggregate.percentile,
            )?;
            return scalar::wire(&value);
        }
        let mut selected = None;
        self.expression_values(cut, expression, None, filters, |_, _, value, _| {
            if !matches!(value, ScalarValue::Missing | ScalarValue::Null) {
                selected = Some(value);
            }
            Ok(())
        })?;
        scalar::wire(&selected.ok_or(Error::Data("capture matching evidence before evaluating this expression"))?)
    }
    pub(crate) fn reduce_expression(
        &self, cut: &ExecutionCut, expression: &DataExpression, filters: &[PlotFilter], kind: Aggregate, percentile: f64,
    ) -> Result<ScalarValue> {
        let numeric = if let Some(v1::data_expression::Kind::Stream(reference)) = &expression.kind {
            let codec = self.source_codec(cut, &(false, reference.identifier.clone()))?;
            let empty = codec.batch(&[])?;
            matches!(
                scalar::column(&empty, &reference.field)?.data_type(),
                arrow_schema::DataType::Int64 | arrow_schema::DataType::UInt64 | arrow_schema::DataType::Float64
            )
        } else {
            false
        };
        if numeric && let Some(v1::data_expression::Kind::Stream(reference)) = &expression.kind {
            let (reader, _cancellation) = self.read_session()?;
            *reader.projection.lock().map_err(|_| Error::Data("restart after the document projection lock failed"))? =
                Some(expressions::Projection {
                    cut: cut.clone(),
                    stream: reference.identifier.clone(),
                    fields: Default::default(),
                    filters: filters.to_vec(),
                    grouping: None,
                    group: None,
                });
            let window = crate::NumericWindow {
                execution: &cut.execution.identifier,
                stream: &reference.identifier,
                field: &reference.field,
                after: 0,
                through: cut.frontier,
                start_ns: i64::MIN,
                end_ns: i64::MAX,
                bins: std::num::NonZeroUsize::MIN,
            };
            let moments = reader.moments(window)?;
            if moments.count == 0 {
                return Ok(if kind == Aggregate::Count { ScalarValue::Signed(0) } else { ScalarValue::Missing });
            }
            let exact_mean = || {
                moments
                    .total
                    .map(|total| total.divide(moments.count as u128).map(ScalarValue::Exact))
                    .transpose()
                    .map_err(|e| Error::Invalid(e.to_string()))
            };
            let answer = match kind {
                Aggregate::Count => Some(ScalarValue::Signed(moments.count as i64)),
                Aggregate::Min => Some(moments.min.clone()),
                Aggregate::Max => Some(moments.max.clone()),
                Aggregate::First => Some(moments.first.clone()),
                Aggregate::Last => Some(moments.last.clone()),
                Aggregate::Sum => {
                    Some(moments.total.map(ScalarValue::Exact).unwrap_or(ScalarValue::Float(moments.mean()? * moments.count as f64)))
                }
                Aggregate::Mean => Some(exact_mean()?.unwrap_or(ScalarValue::Float(moments.mean()?))),
                Aggregate::Stddev => Some(ScalarValue::Float(moments.stddev()?)),
                Aggregate::Rms => Some(ScalarValue::Float(moments.rms()?)),
                Aggregate::Median => Some(reader.ranks(window, moments.count, &[50])?[&50].clone()),
                Aggregate::Percentile if percentile.is_finite() && (0.0..=100.0).contains(&percentile) && percentile.fract() == 0.0 => {
                    Some(reader.ranks(window, moments.count, &[percentile as u8])?[&(percentile as u8)].clone())
                }
                _ => None,
            };
            if let Some(answer) = answer {
                return Ok(answer);
            }
        }
        let mut edge = None;
        if matches!(kind, Aggregate::First | Aggregate::Last) {
            self.expression_values(cut, expression, None, filters, |_, _, value, _| {
                if !matches!(value, ScalarValue::Missing | ScalarValue::Null) && (edge.is_none() || kind == Aggregate::Last) {
                    edge = Some(value);
                }
                Ok(())
            })?;
            return Ok(edge.unwrap_or(ScalarValue::Missing));
        }
        let mut accumulator = expressions::accumulator(kind, percentile)?;
        self.expression_values(cut, expression, None, filters, |position, _, value, _| {
            accumulator.push(position, &value)?;
            Ok(())
        })?;
        let value = accumulator.finish(&mut || Ok(()))?;
        Ok(if kind == Aggregate::Count {
            match value {
                ScalarValue::Unsigned(count) => ScalarValue::Signed(count as i64),
                value => value,
            }
        } else {
            value
        })
    }
    pub fn document_distinct(
        &self, cut: &ExecutionCut, expression: &DataExpression, order: Option<&DataExpression>, filters: &[PlotFilter], offset: usize,
        maximum: usize,
    ) -> Result<(Vec<Value>, u64)> {
        let mut values = Vec::<(ScalarValue, ScalarValue)>::new();
        self.expression_values(cut, expression, order, filters, |_, _, value, ordering| {
            if matches!(value, ScalarValue::Missing | ScalarValue::Null) {
                return Ok(());
            }
            if !values.iter().any(|(v, _)| v.compare(&value).is_ok_and(|o| o.is_eq())) {
                if values.len().saturating_mul(256) > self.storage_settings()?.query_bytes.get() {
                    return Err(Error::Data("narrow distinct values to fit the configured query budget"));
                }
                values.push((value.clone(), ordering.unwrap_or(value)));
            }
            Ok(())
        })?;
        values.sort_by(|a, b| a.1.compare(&b.1).unwrap_or(Ordering::Equal));
        let total = values.len() as u64;
        let page = values.into_iter().skip(offset).take(maximum).map(|(v, _)| scalar::wire(&v)).collect::<Result<Vec<_>>>()?;
        Ok((page, total))
    }
    fn expression_values(
        &self, cut: &ExecutionCut, expression: &DataExpression, order: Option<&DataExpression>, filters: &[PlotFilter],
        mut read: impl FnMut(u64, i64, ScalarValue, Option<ScalarValue>) -> Result<()>,
    ) -> Result<()> {
        use arrow_array::{Int64Array, StructArray, UInt64Array};
        let source = crate::plot_selection::reference(expression).map(|(r, record)| (record, r.identifier.clone()));
        let expressions = std::iter::once(expression.clone())
            .chain(order.cloned())
            .chain(filters.iter().filter_map(|f| f.expression.clone()))
            .collect::<Vec<_>>();
        if source.is_none() {
            return read(0, 0, self.constant_expression(cut, expression)?, None);
        }
        let source = source.unwrap();
        self.evaluate(
            cut,
            source.clone(),
            crate::BatchSelection {
                execution: &cut.execution.identifier,
                stream: &source.1,
                after: 0,
                through: cut.frontier,
                start_ns: i64::MIN,
                end_ns: i64::MAX,
            },
            &expressions,
            |evaluation, batch| {
                let metadata = batch.column(0).as_any().downcast_ref::<StructArray>().unwrap();
                let positions = metadata.column_by_name("position").unwrap().as_any().downcast_ref::<UInt64Array>().unwrap();
                let times = metadata.column_by_name("source_ns").unwrap().as_any().downcast_ref::<Int64Array>().unwrap();
                for row in 0..batch.num_rows() {
                    let mut keep = true;
                    for filter in filters {
                        keep &= evaluation.filter(filter, &batch, row)?;
                    }
                    if keep {
                        read(
                            positions.value(row),
                            times.value(row),
                            evaluation.value(expression, &batch, row, 0)?,
                            order.map(|order| evaluation.value(order, &batch, row, 0)).transpose()?,
                        )?;
                    }
                }
                Ok(true)
            },
        )
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
