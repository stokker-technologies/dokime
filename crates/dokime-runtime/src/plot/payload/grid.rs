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
use std::collections::BTreeMap;

// Third Party Imports

// Local Imports
use super::{PlotPayloadRequest, XInput, expression_numeric_value, expression_x_input};
use crate::document::{GridDefinition, Reduction};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::plots::wire::{GridCell, GridFrame, GridFrames};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Frames<'a> {
    definition: &'a GridDefinition,
    request: &'a PlotPayloadRequest,
    rows: BTreeMap<String, usize>,
    columns: BTreeMap<String, usize>,
    frames: BTreeMap<time::OffsetDateTime, (GridFrame, Vec<u64>)>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<'a> Frames<'a> {
    pub(super) fn new(request: &'a PlotPayloadRequest) -> Result<Self, String> {
        let definition = request.grid.as_ref().ok_or("GRID_CHART requires a grid definition")?;
        definition.validate()?;
        if request.reduction.is_none() {
            return Err("GRID_CHART requires an explicit mean, min or max reducer".into());
        }
        let mut rows = definition.rows.iter().collect::<Vec<_>>();
        let mut columns = definition.columns.iter().collect::<Vec<_>>();
        rows.sort_by_key(|entry| entry.order);
        columns.sort_by_key(|entry| entry.order);
        Ok(Self {
            definition, request,
            rows: rows.into_iter().enumerate().map(|(index, entry)| (entry.key.clone(), index)).collect(),
            columns: columns.into_iter().enumerate().map(|(index, entry)| (entry.key.clone(), index)).collect(),
            frames: BTreeMap::new(),
        })
    }

    pub(super) fn push(&mut self, sample: &TelemetrySample) -> Result<(), String> {
        let row_key = key(sample, self.definition.row.as_ref().expect("validated row"))?;
        let column_key = key(sample, self.definition.column.as_ref().expect("validated column"))?;
        let row = *self.rows.get(&row_key).ok_or_else(|| format!("GRID_CHART unknown row key {row_key}"))?;
        let column = *self.columns.get(&column_key).ok_or_else(|| format!("GRID_CHART unknown column key {column_key}"))?;
        let instant = key(sample, self.definition.frame.as_ref().expect("validated frame"))?;
        let time = super::parse_timestamp(&instant).ok_or("GRID_CHART frame must be a timestamp")?;
        let value = self.request.y_expressions.first().and_then(|expression| expression_numeric_value(sample, expression))
            .or_else(|| self.request.y_keys.first().and_then(|field| sample.values.get(field)).and_then(super::numeric_value));
        let cells = self.rows.len().checked_mul(self.columns.len()).ok_or("GRID_CHART dimensions overflow")?;
        let (frame, counts) = self.frames.entry(time).or_insert_with(|| (GridFrame {
            instant, values: vec![0.0; cells], valid: vec![false; cells], highlights: Vec::new(),
        }, vec![0; cells]));
        for highlight in &self.definition.highlights {
            let Ok(row) = key(sample, &highlight.row) else { continue };
            let Ok(column) = key(sample, &highlight.column) else { continue };
            let row = *self.rows.get(&row).ok_or_else(|| format!("GRID_CHART unknown highlight row {row}"))? as u32;
            let column = *self.columns.get(&column).ok_or_else(|| format!("GRID_CHART unknown highlight column {column}"))? as u32;
            if !frame.highlights.iter().any(|cell| cell.row == row && cell.column == column && cell.label == highlight.label) {
                frame.highlights.push(GridCell { row, column, label: highlight.label.clone() });
            }
        }
        let Some(value) = value.filter(|value| value.is_finite()) else { return Ok(()) };
        let cell = row * self.columns.len() + column;
        frame.values[cell] = match (self.request.reduction.expect("validated reducer"), counts[cell]) {
            (_, 0) => value,
            (Reduction::Mean, n) => frame.values[cell] * (n as f64 / (n + 1) as f64) + value / (n + 1) as f64,
            (Reduction::Min, _) => frame.values[cell].min(value),
            (Reduction::Max, _) => frame.values[cell].max(value),
        };
        counts[cell] += 1;
        frame.valid[cell] = true;
        Ok(())
    }

    pub(super) fn finish(self, maximum: usize) -> GridFrames {
        let mut rows = self.definition.rows.clone();
        let mut columns = self.definition.columns.clone();
        rows.sort_by_key(|entry| entry.order);
        columns.sort_by_key(|entry| entry.order);
        let cells = rows.len().saturating_mul(columns.len()).max(1);
        let maximum = if maximum == 0 { 0 } else { (maximum / cells).max(1) };
        let selected = super::marker_indices(self.frames.len(), maximum);
        let frames = self.frames.into_values().enumerate()
            .filter_map(|(index, (frame, _))| selected.binary_search(&index).is_ok().then_some(frame)).collect();
        GridFrames { rows, columns, frames, summary_label: self.definition.summary_label.clone() }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn key(sample: &TelemetrySample, expression: &crate::expressions::DataExpression) -> Result<String, String> {
    match expression_x_input(sample, Some(expression), "") {
        XInput::Timestamp(value) => Ok(value.to_owned()),
        XInput::Value(crate::Value::Text(value)) => Ok(value.clone()),
        XInput::Value(crate::Value::Integer(value)) => Ok(value.to_string()),
        XInput::Value(crate::Value::Unsigned(value)) => Ok(value.to_string()),
        XInput::Value(crate::Value::Number(value)) if value.is_finite() => Ok(value.to_string()),
        XInput::Value(crate::Value::Datetime(value)) => crate::storage::encode_datetime(*value).map_err(|error| error.to_string()),
        _ => Err("GRID_CHART binding requires a declared key or timestamp".into()),
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::GridAxisEntry;
    use crate::expressions::telemetry;
    use dokime_domain::model::{identity::Identifier, time::Timestamp};
    use crate::model::identity::new_id;

    #[test]
    fn generic_frames_reduce_duplicates_and_refuse_unknown_keys() {
        let entry = |key: &str, order| GridAxisEntry { key: key.into(), label: key.into(), order, group: Some("Sensors".into()) };
        let mut request = PlotPayloadRequest {
            reduction: Some(Reduction::Max),
            y_expressions: vec![telemetry("readings", "value")],
            grid: Some(GridDefinition {
                row: Some(telemetry("readings", "row")), column: Some(telemetry("readings", "column")),
                frame: Some(telemetry("readings", "timestamp")),
                rows: vec![entry("b", 1), entry("a", 0)], columns: vec![entry("left", 0), entry("right", 1)],
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut frames = Frames::new(&request).unwrap();
        let mut sample = TelemetrySample::new(new_id(), Identifier::new("readings").unwrap(), 0, Timestamp::new("2026-10-01T00:00:01Z"), "fixture");
        sample.values.insert("row".into(), crate::Value::Text("a".into()));
        sample.values.insert("column".into(), crate::Value::Text("left".into()));
        for value in [2.0, 6.0, f64::NAN] {
            sample.values.insert("value".into(), crate::Value::Number(value));
            frames.push(&sample).unwrap();
        }
        sample.values.insert("column".into(), crate::Value::Text("right".into()));
        sample.values.insert("value".into(), crate::Value::Number(0.0));
        frames.push(&sample).unwrap();
        sample.values.insert("row".into(), crate::Value::Text("unknown".into()));
        assert!(frames.push(&sample).is_err());
        let result = frames.finish(100);
        assert_eq!(result.rows[0].key, "a");
        assert_eq!(result.frames[0].values, vec![6.0, 0.0, 0.0, 0.0]);
        assert_eq!(result.frames[0].valid, vec![true, true, false, false]);
        request.reduction = None;
        assert!(Frames::new(&request).is_err());
    }
}
