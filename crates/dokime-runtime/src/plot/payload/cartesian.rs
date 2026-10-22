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
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports
use serde_json::Value as JsonValue;

// Local Imports
use super::{PlotCartesianFrame, PlotCartesianMode, PlotCartesianSeriesFrame, PlotDomain, PlotSeries, parse_timestamp};
use crate::document::{AxisScale, PlotType};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug, PartialEq)]
struct OrderedFloat(f64);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

/* === Public Functions === */

pub(super) fn cartesian_frame(
    series: &[PlotSeries], plot_type: Option<&PlotType>, x_scale: &AxisScale, y_scale: &AxisScale,
) -> Option<PlotCartesianFrame> {
    let mode = plot_type.and_then(cartesian_mode)?;
    match mode {
        PlotCartesianMode::Aligned => Some(aligned_cartesian_frame(series, x_scale, y_scale)),
        PlotCartesianMode::Faceted => Some(faceted_cartesian_frame(series, x_scale, y_scale)),
    }
}

pub(super) fn draws_cartesian(plot_type: &PlotType) -> bool {
    cartesian_mode(plot_type).is_some()
}

/* === Private Functions === */

fn aligned_cartesian_frame(series: &[PlotSeries], x_scale: &AxisScale, y_scale: &AxisScale) -> PlotCartesianFrame {
    let mut x_values = Vec::<f64>::new();
    let mut x_seen = BTreeSet::<OrderedFloat>::new();
    let series_maps = series
        .iter()
        .map(|series| {
            let mut points = BTreeMap::<OrderedFloat, Option<f64>>::new();
            for (index, point) in series.points.iter().enumerate() {
                let Some(x) = plot_point_x_value(&point.x, index) else {
                    continue;
                };
                if matches!(x_scale, AxisScale::Log) && x <= 0.0 {
                    continue;
                }
                let y = plot_y_value(point.y, y_scale);
                if y.is_none() {
                    continue;
                }
                let ordered = OrderedFloat(x);
                if x_seen.insert(ordered) {
                    x_values.push(x);
                }
                points.insert(ordered, y);
            }
            points
        })
        .collect::<Vec<_>>();

    x_values.sort_by(f64::total_cmp);
    let frames = series_maps
        .iter()
        .map(|points| PlotCartesianSeriesFrame {
            x: x_values.clone(),
            y: x_values.iter().map(|x| points.get(&OrderedFloat(*x)).copied().flatten()).collect(),
        })
        .collect::<Vec<_>>();

    cartesian_frame_from_series(PlotCartesianMode::Aligned, frames)
}

fn faceted_cartesian_frame(series: &[PlotSeries], x_scale: &AxisScale, y_scale: &AxisScale) -> PlotCartesianFrame {
    let frames = series
        .iter()
        .map(|series| {
            let mut x_values = Vec::<f64>::new();
            let mut y_values = Vec::<Option<f64>>::new();
            for (index, point) in series.points.iter().enumerate() {
                let Some(x) = plot_point_x_value(&point.x, index) else {
                    continue;
                };
                if matches!(x_scale, AxisScale::Log) && x <= 0.0 {
                    continue;
                }
                let Some(y) = plot_y_value(point.y, y_scale) else {
                    continue;
                };
                x_values.push(x);
                y_values.push(Some(y));
            }
            PlotCartesianSeriesFrame { x: x_values, y: y_values }
        })
        .collect::<Vec<_>>();

    cartesian_frame_from_series(PlotCartesianMode::Faceted, frames)
}

fn cartesian_frame_from_series(mode: PlotCartesianMode, series: Vec<PlotCartesianSeriesFrame>) -> PlotCartesianFrame {
    let point_count = series.iter().map(|series| series.y.iter().filter(|value| value.is_some()).count()).sum();
    let signature = cartesian_signature(&series);
    PlotCartesianFrame {
        mode,
        x_domain: plot_domain(series.iter().flat_map(|series| series.x.iter().copied())),
        y_domain: plot_domain(series.iter().flat_map(|series| series.y.iter().flatten().copied())),
        series,
        point_count,
        signature,
    }
}

fn cartesian_mode(plot_type: &PlotType) -> Option<PlotCartesianMode> {
    match plot_type {
        PlotType::TimeSeries | PlotType::FrequencySeries => Some(PlotCartesianMode::Aligned),
        PlotType::XyLine | PlotType::XyScatter | PlotType::XyInterval | PlotType::Trajectory => Some(PlotCartesianMode::Faceted),
        _ => None,
    }
}

fn plot_point_x_value(value: &JsonValue, fallback: usize) -> Option<f64> {
    match value {
        JsonValue::Number(number) => number.as_f64().filter(|value| value.is_finite()),
        JsonValue::String(value) => parse_timestamp(value)
            .map(|time| time.unix_timestamp_nanos() as f64 / 1_000_000_000.0)
            .or_else(|| value.parse::<f64>().ok().filter(|value| value.is_finite())),
        JsonValue::Null => Some(fallback as f64),
        _ => Some(fallback as f64),
    }
}

fn plot_y_value(value: Option<f64>, scale: &AxisScale) -> Option<f64> {
    value.filter(|value| value.is_finite()).filter(|value| !matches!(scale, AxisScale::Log) || *value > 0.0)
}

fn plot_domain(values: impl Iterator<Item = f64>) -> Option<PlotDomain> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for value in values {
        if !value.is_finite() {
            continue;
        }
        min = min.min(value);
        max = max.max(value);
    }
    min.is_finite().then_some(PlotDomain { min, max })
}

fn cartesian_signature(series: &[PlotCartesianSeriesFrame]) -> String {
    use std::hash::{Hash, Hasher};

    let mut hash = std::collections::hash_map::DefaultHasher::new();
    for entry in series {
        entry.x.len().hash(&mut hash);
        for value in &entry.x {
            value.to_bits().hash(&mut hash);
        }
        for value in &entry.y {
            value.map(f64::to_bits).hash(&mut hash);
        }
    }
    format!("{:016x}", hash.finish())
}

/* === Tests === */
