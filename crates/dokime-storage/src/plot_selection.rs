/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Compiled Plot Selections

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports

// Local Imports
use crate::{
    Error, ExecutionCut, NumericWindow, Result, Store,
    expressions::{self, Projection},
    scalar::{self, ScalarValue},
};
use dokime_protocol::v1::{self, DataExpression, FieldReference, PlotRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct PlotGroup {
    pub label: String,
    value: ScalarValue,
}
pub struct PlotSelection {
    pub request: PlotRequest,
    pub projected: BTreeSet<String>,
    pub labels: BTreeMap<String, String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn plot_group_values(&self, window: NumericWindow<'_>, field: &str, maximum: usize) -> Result<Vec<PlotGroup>> {
        let mut groups = Vec::<ScalarValue>::new();
        self.evaluated_batches(window, &[field.into()], |batch| {
            for row in 0..batch.num_rows() {
                let value = scalar::field(&batch, field, row)?;
                if matches!(value, ScalarValue::Missing | ScalarValue::Null) {
                    continue;
                }
                if !groups.iter().any(|v| v.compare(&value).is_ok_and(|o| o.is_eq())) {
                    groups.push(value);
                    if groups.len() > maximum {
                        return Err(Error::Data("narrow the grouped window or reduce its groups"));
                    }
                }
            }
            Ok(true)
        })?;
        groups.sort_by(|a, b| compare_groups(&scalar::text(a), &scalar::text(b)));
        Ok(groups.into_iter().map(|value| PlotGroup { label: scalar::text(&value), value }).collect())
    }
    pub fn select_plot_group(&self, group: Option<&PlotGroup>) -> Result<()> {
        if let Some(plan) = self.projection.lock().map_err(|_| Error::Data("restart after the plot projection lock failed"))?.as_mut() {
            plan.group = group.map(|g| g.value.clone());
        }
        Ok(())
    }
    pub fn plot_selection(&self, cut: &ExecutionCut, request: &PlotRequest) -> Result<PlotSelection> {
        let mut selected = request.clone();
        let grouping = request
            .series_by_expression
            .clone()
            .or_else(|| request.series_by.as_ref().map(|field| expressions::reference(&request.stream, field, false)));
        let mut fields = BTreeMap::new();
        let mut labels = BTreeMap::new();
        let mut projected = BTreeSet::new();
        let grid_bindings = selected.grid.iter_mut().flat_map(|grid| {
            grid.row
                .iter_mut()
                .chain(grid.column.iter_mut())
                .chain(grid.frame.iter_mut())
                .chain(grid.highlights.iter_mut().flat_map(|h| h.row.iter_mut().chain(h.column.iter_mut())))
        });
        for expression in selected
            .y_expressions
            .iter_mut()
            .chain(selected.x_expression.iter_mut())
            .chain(selected.z_expression.iter_mut())
            .chain(selected.series_by_expression.iter_mut())
            .chain(grid_bindings)
        {
            let label = expression.label.clone();
            let input = match expression.kind.as_mut() {
                Some(v1::data_expression::Kind::Aggregate(a)) => {
                    a.input.as_deref_mut().ok_or(Error::Data("supply the plot aggregate input"))?
                }
                _ => expression,
            };
            if matches!(input.kind.as_ref(),Some(v1::data_expression::Kind::Stream(r)) if r.identifier==request.stream) {
                continue;
            }
            let key = format!("__dokime_expression_{}", fields.len());
            fields.insert(key.clone(), input.clone());
            projected.insert(key.clone());
            labels.insert(key.clone(), if label.is_empty() { format!("Value {}", fields.len()) } else { label });
            *input = expressions::reference(&request.stream, &key, false);
        }
        if let Some(stream) = &selected.z_stream
            && stream != &request.stream
        {
            let key = format!("__dokime_expression_{}", fields.len());
            fields.insert(key.clone(), expressions::reference(stream, selected.z_field.as_deref().unwrap_or_default(), false));
            projected.insert(key.clone());
            selected.z_stream = Some(request.stream.clone());
            selected.z_field = Some(key);
        }
        *self.projection.lock().map_err(|_| Error::Data("restart after the plot projection lock failed"))? = Some(Projection {
            cut: cut.clone(),
            stream: request.stream.clone(),
            fields,
            filters: request.filters.clone(),
            grouping,
            group: None,
        });
        selected.filters.clear();
        Ok(PlotSelection { request: selected, projected, labels })
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(crate) fn compare_groups(left: &str, right: &str) -> Ordering {
    let number = |value: &str| value.parse::<f64>().ok().filter(|value| value.is_finite());
    match (number(left), number(right)) {
        (Some(a), Some(b)) => a.total_cmp(&b).then_with(|| left.cmp(right)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        _ => left.cmp(right),
    }
}
pub(crate) fn reference(expression: &DataExpression) -> Option<(&FieldReference, bool)> {
    match expression.kind.as_ref()? {
        v1::data_expression::Kind::Stream(reference) => Some((reference, false)),
        v1::data_expression::Kind::Record(reference) => Some((reference, true)),
        v1::data_expression::Kind::Aggregate(aggregate) => reference(aggregate.input.as_deref()?),
        v1::data_expression::Kind::Transform(transform) => reference(transform.input.as_deref()?),
        _ => None,
    }
}

/* === Tests === */
