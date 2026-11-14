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
        self.document_value_filtered(cut, expression, &[])
    }
    pub fn document_value_filtered(
        &self, cut: &ExecutionCut, expression: &v1::DataExpression, filters: &[v1::PlotFilter],
    ) -> Result<v1::Value> {
        if let Some(v1::data_expression::Kind::Aggregate(aggregate)) = &expression.kind {
            let value = self.reduce_expression(
                cut,
                aggregate.input.as_deref().ok_or(Error::Data("supply the aggregate input"))?,
                filters,
                v1::Aggregate::try_from(aggregate.kind).map_err(|_| Error::Data("select a reducer"))?,
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
        &self, cut: &ExecutionCut, expression: &v1::DataExpression, filters: &[v1::PlotFilter], kind: v1::Aggregate, percentile: f64,
    ) -> Result<ScalarValue> {
        let mut edge = None;
        if matches!(kind, v1::Aggregate::First | v1::Aggregate::Last) {
            self.expression_values(cut, expression, None, filters, |_, _, value, _| {
                if !matches!(value, ScalarValue::Missing | ScalarValue::Null) && (edge.is_none() || kind == v1::Aggregate::Last) {
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
        Ok(accumulator.finish(&mut || Ok(()))?)
    }
    pub fn document_distinct(
        &self, cut: &ExecutionCut, expression: &v1::DataExpression, order: Option<&v1::DataExpression>, filters: &[v1::PlotFilter],
        offset: usize, maximum: usize,
    ) -> Result<(Vec<v1::Value>, u64)> {
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
        &self, cut: &ExecutionCut, expression: &v1::DataExpression, order: Option<&v1::DataExpression>, filters: &[v1::PlotFilter],
        mut read: impl FnMut(u64, i64, ScalarValue, Option<ScalarValue>) -> Result<()>,
    ) -> Result<()> {
        use arrow_array::{Int64Array, StructArray, UInt64Array};
        let source = crate::plot_selection::reference(expression).map(|(r, record)| (record, r.identifier.clone()));
        let expressions = std::iter::once(expression.clone())
            .chain(order.cloned())
            .chain(filters.iter().filter_map(|f| f.expression.clone()))
            .collect::<Vec<_>>();
        if source.is_none() {
            let source = self.stream_names(&cut.execution.identifier)?.first().cloned();
            if let Some(source) = source {
                return self.evaluate(cut, (false, source), 0, cut.frontier, i64::MIN, i64::MAX, &expressions, |evaluation, batch| {
                    if batch.num_rows() > 0 {
                        read(0, 0, evaluation.value(expression, &batch, 0, 0)?, None)?;
                    }
                    Ok(false)
                });
            }
            if let Some(v1::data_expression::Kind::Literal(value)) = &expression.kind {
                return read(0, 0, scalar::from_wire(value)?, None);
            }
            return Err(Error::Data("capture evidence before evaluating this expression"));
        }
        self.evaluate(cut, source.unwrap(), 0, cut.frontier, i64::MIN, i64::MAX, &expressions, |evaluation, batch| {
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
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
