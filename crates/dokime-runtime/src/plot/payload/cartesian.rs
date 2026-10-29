/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use serde_json::Value as JsonValue;

use super::{PlotCartesianFrame, PlotCartesianMode, PlotCartesianSeriesFrame, PlotDomain, PlotSeries, parse_timestamp};
use crate::document::{AxisScale, PlotType};

#[derive(Clone, Copy, Debug, PartialEq)]
struct OrderedFloat(f64);

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

pub(super) fn cartesian_frame(
    series: &[PlotSeries], plot_type: Option<&PlotType>, x_scale: &AxisScale, y_scale: &AxisScale,
) -> Result<Option<PlotCartesianFrame>, String> {
    let Some(mode) = plot_type.and_then(cartesian_mode) else { return Ok(None) };
    match mode {
        PlotCartesianMode::Aligned => aligned_cartesian_frame(series, x_scale, y_scale).map(Some),
        PlotCartesianMode::Faceted => faceted_cartesian_frame(series, x_scale, y_scale).map(Some),
    }
}

fn cartesian_point(point: &super::PlotPoint, index: usize, x_scale: &AxisScale, y_scale: &AxisScale) -> Option<(f64, f64)> {
    let x = plot_point_x_value(&point.x, index)?;
    if matches!(x_scale, AxisScale::Log) && x <= 0.0 {
        return None;
    }
    Some((x, plot_y_value(point.y, y_scale)?))
}

fn reserved<T>(count: usize) -> Result<Vec<T>, String> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|error| error.to_string())?;
    Ok(values)
}

fn aligned_cartesian_frame(series: &[PlotSeries], x_scale: &AxisScale, y_scale: &AxisScale) -> Result<PlotCartesianFrame, String> {
    let total = series
        .iter()
        .try_fold(0usize, |total, entry| {
            total.checked_add(
                entry.points.iter().enumerate().filter(|(index, point)| cartesian_point(point, *index, x_scale, y_scale).is_some()).count(),
            )
        })
        .ok_or_else(|| "plot point count overflows".to_owned())?;
    let mut x_values = reserved(total)?;
    let mut series_maps = reserved(series.len())?;
    for series in series {
        let mut points = BTreeMap::<OrderedFloat, Option<f64>>::new();
        for (index, point) in series.points.iter().enumerate() {
            let Some((x, y)) = cartesian_point(point, index, x_scale, y_scale) else {
                continue;
            };
            x_values.push(x);
            points.insert(OrderedFloat(x), Some(y));
        }
        series_maps.push(points);
    }
    x_values.sort_unstable_by(f64::total_cmp);
    x_values.dedup_by(|left, right| left.total_cmp(right).is_eq());
    let mut frames = reserved(series_maps.len())?;
    for points in &series_maps {
        let mut x = reserved(x_values.len())?;
        x.extend_from_slice(&x_values);
        let mut y = reserved(x_values.len())?;
        y.extend(x_values.iter().map(|x| points.get(&OrderedFloat(*x)).copied().flatten()));
        frames.push(PlotCartesianSeriesFrame { x, y });
    }
    cartesian_frame_from_series(PlotCartesianMode::Aligned, frames)
}

fn faceted_cartesian_frame(series: &[PlotSeries], x_scale: &AxisScale, y_scale: &AxisScale) -> Result<PlotCartesianFrame, String> {
    let mut frames = reserved(series.len())?;
    for series in series {
        let count =
            series.points.iter().enumerate().filter(|(index, point)| cartesian_point(point, *index, x_scale, y_scale).is_some()).count();
        let mut x = reserved(count)?;
        let mut y = reserved(count)?;
        for (index, point) in series.points.iter().enumerate() {
            let Some((value, ordinate)) = cartesian_point(point, index, x_scale, y_scale) else {
                continue;
            };
            x.push(value);
            y.push(Some(ordinate));
        }
        frames.push(PlotCartesianSeriesFrame { x, y });
    }
    cartesian_frame_from_series(PlotCartesianMode::Faceted, frames)
}

fn cartesian_frame_from_series(mode: PlotCartesianMode, series: Vec<PlotCartesianSeriesFrame>) -> Result<PlotCartesianFrame, String> {
    let point_count = series.iter().map(|series| series.y.iter().filter(|value| value.is_some()).count()).sum();
    let signature = cartesian_signature(&series)?;
    Ok(PlotCartesianFrame {
        mode,
        x_domain: plot_domain(series.iter().flat_map(|series| series.x.iter().copied())),
        y_domain: plot_domain(series.iter().flat_map(|series| series.y.iter().flatten().copied())),
        series,
        point_count,
        signature,
    })
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

fn cartesian_signature(series: &[PlotCartesianSeriesFrame]) -> Result<String, String> {
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
    use std::fmt::Write;
    let mut signature = String::new();
    signature.try_reserve_exact(16).map_err(|error| error.to_string())?;
    write!(&mut signature, "{:016x}", hash.finish()).map_err(|error| error.to_string())?;
    Ok(signature)
}
