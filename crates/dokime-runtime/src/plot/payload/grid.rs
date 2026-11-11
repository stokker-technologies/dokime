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
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Third Party Imports

// Local Imports
use super::{PlotPayloadRequest, XInput, expression_x_input};
use crate::document::{GridDefinition, Reduction};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::plots::wire::{GridCell, GridFrame, GridFrames};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Frames {
    definition: GridDefinition,
    request: PlotPayloadRequest,
    maximum: usize,
    stride: u64,
    first: Option<i128>,
    last: Option<i128>,
    rows: BTreeMap<String, usize>,
    columns: BTreeMap<String, usize>,
    frames: BTreeMap<i128, (GridFrame, Vec<u64>)>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Frames {
    pub(crate) fn new(request: &PlotPayloadRequest, maximum: usize) -> Result<Self, String> {
        let definition = request.grid.as_ref().ok_or("GRID_CHART requires a grid definition")?;
        definition.validate()?;
        if request.reduction.is_none() {
            return Err("GRID_CHART requires an explicit mean, min or max reducer".into());
        }
        let mut rows = definition.rows.iter().collect::<Vec<_>>();
        let mut columns = definition.columns.iter().collect::<Vec<_>>();
        rows.sort_by_key(|entry| entry.order);
        columns.sort_by_key(|entry| entry.order);
        let cells = rows.len().checked_mul(columns.len()).ok_or("GRID_CHART dimensions overflow")?;
        if maximum > 0 && cells > maximum {
            return Err("GRID_CHART declared cells exceed the display budget".into());
        }
        Ok(Self {
            definition: definition.clone(),
            request: request.clone(),
            maximum: if maximum == 0 { 0 } else { (maximum / cells.max(1)).max(1) },
            stride: 1,
            first: None,
            last: None,
            rows: rows.into_iter().enumerate().map(|(index, entry)| (entry.key.clone(), index)).collect(),
            columns: columns.into_iter().enumerate().map(|(index, entry)| (entry.key.clone(), index)).collect(),
            frames: BTreeMap::new(),
        })
    }

    pub(crate) fn push(&mut self, sample: &TelemetrySample) -> Result<(), String> {
        let instant = key(sample, self.definition.frame.as_ref().expect("validated frame"))?;
        let time = super::parse_timestamp(&instant).ok_or("GRID_CHART frame must be a timestamp")?.unix_timestamp_nanos();
        let row = key(sample, self.definition.row.as_ref().expect("validated row"))
            .ok()
            .map(|key| self.rows.get(&key).copied().ok_or_else(|| format!("GRID_CHART unknown row key {key}")))
            .transpose()?;
        let column = key(sample, self.definition.column.as_ref().expect("validated column"))
            .ok()
            .map(|key| self.columns.get(&key).copied().ok_or_else(|| format!("GRID_CHART unknown column key {key}")))
            .transpose()?;
        let mut highlights = Vec::new();
        for highlight in &self.definition.highlights {
            let Ok(row) = key(sample, &highlight.row) else { continue };
            let Ok(column) = key(sample, &highlight.column) else { continue };
            let row = *self.rows.get(&row).ok_or_else(|| format!("GRID_CHART unknown highlight row {row}"))? as u32;
            let column = *self.columns.get(&column).ok_or_else(|| format!("GRID_CHART unknown highlight column {column}"))? as u32;
            highlights.push(GridCell {
                row,
                column,
                label: highlight.label.clone(),
            });
        }
        self.first = Some(self.first.map_or(time, |first| first.min(time)));
        self.last = Some(self.last.map_or(time, |last| last.max(time)));
        let endpoint = Some(time) == self.last || (self.maximum != 1 && Some(time) == self.first);
        let selected = self.maximum == 0
            || endpoint
            || self.frames.contains_key(&time)
            || (self.maximum > 1 && self.stride > 0 && frame_hash(time).is_multiple_of(self.stride));
        if !selected {
            return Ok(());
        }
        let cells = self.rows.len() * self.columns.len();
        let (frame, counts) = self.frames.entry(time).or_insert_with(|| {
            (
                GridFrame {
                    instant,
                    values: vec![0.0; cells],
                    valid: vec![false; cells],
                    highlights: Vec::new(),
                },
                vec![0; cells],
            )
        });
        for highlight in highlights {
            if !frame
                .highlights
                .iter()
                .any(|cell| cell.row == highlight.row && cell.column == highlight.column && cell.label == highlight.label)
            {
                frame.highlights.push(highlight);
            }
        }
        let value = match expression_x_input(
            sample,
            self.request.y_expressions.first(),
            self.request.y_keys.first().map(String::as_str).unwrap_or(""),
        ) {
            XInput::Number(value) => Some(value),
            XInput::Value(value) => super::numeric_value(value),
            _ => None,
        };
        if let (Some(row), Some(column), Some(value)) = (row, column, value.filter(|value| value.is_finite())) {
            let cell = row * self.columns.len() + column;
            frame.values[cell] = match (self.request.reduction.expect("validated reducer"), counts[cell]) {
                (_, 0) => value,
                (Reduction::Mean, n) => frame.values[cell] * (n as f64 / (n + 1) as f64) + value / (n + 1) as f64,
                (Reduction::Min, _) => frame.values[cell].min(value),
                (Reduction::Max, _) => frame.values[cell].max(value),
            };
            counts[cell] += 1;
            frame.valid[cell] = true;
        }
        self.bound();
        Ok(())
    }

    fn bound(&mut self) {
        while self.maximum > 0 && self.frames.len() > self.maximum {
            self.stride = self.stride.checked_mul(2).unwrap_or(0);
            self.frames.retain(|time, _| {
                Some(*time) == self.last
                    || (self.maximum > 1
                        && (Some(*time) == self.first || (self.stride > 0 && frame_hash(*time).is_multiple_of(self.stride))))
            });
        }
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.first = self.first.into_iter().chain(other.first).min();
        self.last = self.last.into_iter().chain(other.last).max();
        self.stride = if self.stride == 0 || other.stride == 0 { 0 } else { self.stride.max(other.stride) };
        self.frames.retain(|time, _| {
            self.maximum == 0
                || Some(*time) == self.last
                || (self.maximum > 1 && (Some(*time) == self.first || (self.stride > 0 && frame_hash(*time).is_multiple_of(self.stride))))
        });
        let reduction = self.request.reduction.expect("validated reducer");
        for (time, (other, other_counts)) in other.frames {
            let selected = self.maximum == 0
                || Some(time) == self.last
                || (self.maximum > 1 && (Some(time) == self.first || (self.stride > 0 && frame_hash(time).is_multiple_of(self.stride))));
            if !selected {
                continue;
            }
            if let Some((frame, counts)) = self.frames.get_mut(&time) {
                for (index, count) in other_counts.into_iter().enumerate() {
                    if count == 0 {
                        continue;
                    }
                    let total = counts[index].saturating_add(count);
                    frame.values[index] = if counts[index] == 0 {
                        other.values[index]
                    } else {
                        match reduction {
                            Reduction::Mean => {
                                frame.values[index] * (counts[index] as f64 / total as f64)
                                    + other.values[index] * (count as f64 / total as f64)
                            }
                            Reduction::Min => frame.values[index].min(other.values[index]),
                            Reduction::Max => frame.values[index].max(other.values[index]),
                        }
                    };
                    counts[index] = total;
                    frame.valid[index] = true;
                }
                for cell in other.highlights {
                    if !frame
                        .highlights
                        .iter()
                        .any(|current| current.row == cell.row && current.column == cell.column && current.label == cell.label)
                    {
                        frame.highlights.push(cell);
                    }
                }
            } else {
                self.frames.insert(time, (other, other_counts));
            }
        }
        self.bound();
    }

    pub(crate) fn finish(&self) -> GridFrames {
        let mut rows = self.definition.rows.clone();
        let mut columns = self.definition.columns.clone();
        rows.sort_by_key(|entry| entry.order);
        columns.sort_by_key(|entry| entry.order);
        GridFrames {
            rows,
            columns,
            frames: self.frames.values().map(|(frame, _)| frame.clone()).collect(),
            summary_label: self.definition.summary_label.clone(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

// A Fixed Hash Makes Thinning Monotone: A Dropped Frame Never Reappears With Partial Cells.
fn frame_hash(time: i128) -> u64 {
    let mut value = (time as u64) ^ ((time >> 64) as u64);
    value = value.wrapping_add(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    (value ^ (value >> 31)).max(1)
}

fn key(sample: &TelemetrySample, expression: &crate::expressions::DataExpression) -> Result<String, String> {
    match expression_x_input(sample, Some(expression), "") {
        XInput::Number(value) if value.is_finite() => Ok(value.to_string()),
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
    use crate::model::identity::new_id;
    use dokime_domain::model::{identity::Identifier, time::Timestamp};

    #[test]
    fn generic_frames_reduce_duplicates_and_refuse_unknown_keys() {
        let entry = |key: &str, order| GridAxisEntry {
            key: key.into(),
            label: key.into(),
            order,
            group: Some("Sensors".into()),
        };
        let mut request = PlotPayloadRequest {
            reduction: Some(Reduction::Max),
            y_expressions: vec![telemetry("readings", "value")],
            grid: Some(GridDefinition {
                row: Some(telemetry("readings", "row")),
                column: Some(telemetry("readings", "column")),
                frame: Some(telemetry("readings", "timestamp")),
                rows: vec![entry("b", 1), entry("a", 0)],
                columns: vec![entry("left", 0), entry("right", 1)],
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut frames = Frames::new(&request, 100).unwrap();
        let mut sample =
            TelemetrySample::new(new_id(), Identifier::new("readings").unwrap(), 0, Timestamp::new("2026-10-01T00:00:01Z"), "fixture");
        sample.values.insert("row".into(), crate::Value::Text("a".into()));
        sample.values.insert("column".into(), crate::Value::Text("left".into()));
        for value in [2.0, 6.0, f64::NAN] {
            sample.values.insert("value".into(), crate::Value::Number(value));
            frames.push(&sample).unwrap();
        }
        sample.values.insert("value".into(), crate::Value::Bool(true));
        frames.push(&sample).unwrap();
        sample.values.insert("column".into(), crate::Value::Text("right".into()));
        sample.values.insert("value".into(), crate::Value::Number(0.0));
        frames.push(&sample).unwrap();
        sample.values.insert("row".into(), crate::Value::Text("unknown".into()));
        assert!(frames.push(&sample).is_err());
        let result = frames.finish();
        assert_eq!(result.rows[0].key, "a");
        assert_eq!(result.frames[0].values, vec![6.0, 0.0, 0.0, 0.0]);
        assert_eq!(result.frames[0].valid, vec![true, true, false, false]);
        sample.values.insert("row".into(), crate::Value::Text("a".into()));
        sample.values.insert("column".into(), crate::Value::Text("left".into()));
        for (reduction, expected) in [(Reduction::Mean, 4.0), (Reduction::Min, 2.0), (Reduction::Max, 6.0)] {
            let mut reduced_request = request.clone();
            reduced_request.reduction = Some(reduction);
            let mut reduced = Frames::new(&reduced_request, 100).unwrap();
            for value in [2.0, 6.0] {
                sample.values.insert("value".into(), crate::Value::Number(value));
                reduced.push(&sample).unwrap();
            }
            sample.values.insert("value".into(), crate::Value::Bool(true));
            reduced.push(&sample).unwrap();
            assert_eq!(reduced.finish().frames[0].values[0], expected);
            let mut other = Frames::new(&reduced_request, 100).unwrap();
            sample.values.insert("value".into(), crate::Value::Number(10.0));
            other.push(&sample).unwrap();
            reduced.merge(other);
            let expected = match reduction {
                Reduction::Mean => 6.0,
                Reduction::Min => 2.0,
                Reduction::Max => 10.0,
            };
            assert!((reduced.finish().frames[0].values[0] - expected).abs() < 1e-12);
        }
        let mut bounded = Frames::new(&request, 12).unwrap();
        for second in 0..20 {
            sample.timestamp = Timestamp::new(format!("2026-10-01T00:00:{second:02}Z"));
            for value in [2.0, 6.0] {
                sample.values.insert("value".into(), crate::Value::Number(value));
                bounded.push(&sample).unwrap();
                assert!(bounded.frames.len() <= 3);
            }
            bounded = serde_json::from_slice(&serde_json::to_vec(&bounded).unwrap()).unwrap();
        }
        for second in (0..20).rev() {
            sample.timestamp = Timestamp::new(format!("2026-10-01T00:00:{second:02}Z"));
            sample.values.insert("value".into(), crate::Value::Number(8.0));
            bounded.push(&sample).unwrap();
        }
        let selected = bounded.finish();
        assert!(selected.frames.first().unwrap().instant.ends_with("00Z"));
        assert!(selected.frames.last().unwrap().instant.ends_with("19Z"));
        assert!(selected.frames.iter().all(|frame| frame.valid[0] && frame.values[0] == 8.0));
        request.reduction = None;
        assert!(Frames::new(&request, 100).is_err());
    }
}
