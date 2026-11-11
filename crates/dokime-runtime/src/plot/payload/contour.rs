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

// Third Party Imports

// Local Imports
use super::{PlotPayloadRequest, PlotSeries, PlotWindow};
use crate::document::{AxisScale, Reduction};
use dokime_domain::model::plots::wire::ContourGrid;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn bin(series: &[PlotSeries], request: &PlotPayloadRequest) -> Result<ContourGrid, String> {
    let columns = request.panel_width.unwrap_or(36).clamp(1, 512);
    let rows = request.panel_height.unwrap_or(28).clamp(1, 512);
    let count = (columns * rows) as usize;
    let points = series.iter().flat_map(|series| &series.points).filter_map(|point| {
        let x = transformed(point.x.as_f64()?, &request.x_scale)?;
        let y = transformed(point.y?, &request.y_scale)?;
        let z = point.z.filter(|z| z.is_finite())?;
        Some((x, y, z))
    });
    let x = bounds(request.viewport, points.clone().map(|(x, _, _)| x), &request.x_scale)?;
    let y = bounds(request.y_viewport, points.clone().map(|(_, y, _)| y), &request.y_scale)?;
    let mut values = vec![0.0; count];
    let mut counts = vec![0_u64; count];
    let reduction = request.reduction.unwrap_or_default();
    for (px, py, z) in points {
        let Some(column) = index(px, x, columns) else { continue };
        let Some(row) = index(py, y, rows) else { continue };
        let cell = (row * columns + column) as usize;
        values[cell] = match (reduction, counts[cell]) {
            (_, 0) => z,
            (Reduction::Mean, n) => values[cell] * (n as f64 / (n + 1) as f64) + z / (n + 1) as f64,
            (Reduction::Min, _) => values[cell].min(z),
            (Reduction::Max, _) => values[cell].max(z),
        };
        counts[cell] += 1;
    }
    Ok(ContourGrid {
        columns,
        rows,
        x_edges: edges(x, columns, &request.x_scale),
        y_edges: edges(y, rows, &request.y_scale),
        values,
        valid: counts.into_iter().map(|count| count > 0).collect(),
    })
}

/* === Private Functions === */

fn transformed(value: f64, scale: &AxisScale) -> Option<f64> {
    if !value.is_finite() || (*scale == AxisScale::Log && value <= 0.0) {
        return None;
    }
    Some(if *scale == AxisScale::Log { value.ln() } else { value })
}

fn bounds(viewport: Option<PlotWindow>, values: impl Iterator<Item = f64>, scale: &AxisScale) -> Result<PlotWindow, String> {
    if let Some(viewport) = viewport {
        let from = transformed(viewport.from, scale).ok_or("contour viewport bounds must be finite and valid for the axis scale")?;
        let to = transformed(viewport.to, scale).ok_or("contour viewport bounds must be finite and valid for the axis scale")?;
        if from >= to {
            return Err("contour viewport bounds must have from < to".into());
        }
        return Ok(PlotWindow { from, to });
    }
    let extent = values.fold(None::<PlotWindow>, |extent, value| {
        Some(extent.map_or(PlotWindow { from: value, to: value }, |extent| PlotWindow {
            from: extent.from.min(value),
            to: extent.to.max(value),
        }))
    });
    let mut extent = extent.unwrap_or(PlotWindow { from: 0.0, to: 1.0 });
    if extent.from == extent.to {
        let padding = (extent.from.abs() * 0.01).max(0.5);
        extent.from -= padding;
        extent.to += padding;
    }
    Ok(extent)
}

fn index(value: f64, bounds: PlotWindow, bins: u32) -> Option<u32> {
    if value < bounds.from || value > bounds.to {
        return None;
    }
    Some((((value - bounds.from) / (bounds.to - bounds.from) * f64::from(bins)).floor() as u32).min(bins - 1))
}

fn edges(bounds: PlotWindow, bins: u32, scale: &AxisScale) -> Vec<f64> {
    (0..=bins).map(|index| {
        let value = bounds.from + (bounds.to - bounds.from) * f64::from(index) / f64::from(bins);
        if *scale == AxisScale::Log { value.exp() } else { value }
    }).collect()
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::PlotPoint;

    #[test]
    fn sparse_duplicates_invalid_samples_and_zero() {
        let series = vec![PlotSeries {
            key: "z".into(), label: "Z".into(), unit: None, raw_points: 5,
            downsampled: false, format: None,
            points: [(0.1, 0.1, 2.0), (0.2, 0.2, 6.0), (1.5, 1.5, 0.0), (0.1, 0.1, f64::NAN), (f64::INFINITY, 1.0, 7.0)]
                .into_iter().map(|(x, y, z)| PlotPoint { x: serde_json::json!(x), y: Some(y), z: Some(z) }).collect(),
        }];
        let mut request = PlotPayloadRequest {
            panel_width: Some(2), panel_height: Some(2),
            viewport: Some(PlotWindow { from: 0.0, to: 2.0 }),
            y_viewport: Some(PlotWindow { from: 0.0, to: 2.0 }),
            ..Default::default()
        };
        for (reduction, expected) in [(Reduction::Mean, 4.0), (Reduction::Min, 2.0), (Reduction::Max, 6.0)] {
            request.reduction = Some(reduction);
            let grid = bin(&series, &request).unwrap();
            assert_eq!(grid.valid, vec![true, false, false, true]);
            assert_eq!(grid.values, vec![expected, 0.0, 0.0, 0.0]);
            assert_eq!(grid.x_edges, vec![0.0, 1.0, 2.0]);
        }
        request.x_scale = AxisScale::Log;
        request.viewport = Some(PlotWindow { from: 1.0, to: 100.0 });
        let grid = bin(&series, &request).unwrap();
        assert!((grid.x_edges[1] - 10.0).abs() < 1e-12);
    }
}
