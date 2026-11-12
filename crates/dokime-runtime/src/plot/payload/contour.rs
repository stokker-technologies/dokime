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

pub(super) fn bin(series: &[PlotSeries], request: &PlotPayloadRequest, maximum: usize) -> Result<(ContourGrid, Vec<u64>), String> {
    let dimensions = [request.panel_width.unwrap_or(36), request.panel_height.unwrap_or(28)];
    if dimensions.contains(&0) {
        return Err("Refresh contour: panel dimensions must be positive".into());
    }
    let maximum = if maximum == 0 { 262_144 } else { maximum.min(262_144) };
    let mut strides = [1_u64; 2];
    let mut bins = dimensions.map(u64::from);
    while bins[0] * bins[1] > maximum as u64 {
        let axis = usize::from(bins[1] > bins[0]);
        strides[axis] *= 2;
        bins[axis] = u64::from(dimensions[axis]).div_ceil(strides[axis]);
    }
    let [columns, rows] = bins.map(|value| value as u32);
    let count = columns as usize * rows as usize;
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
        let Some(column) = index(px, x, dimensions[0]) else { continue };
        let Some(row) = index(py, y, dimensions[1]) else { continue };
        let column = (u64::from(column) / strides[0]) as u32;
        let row = (u64::from(row) / strides[1]) as u32;
        let cell = (row * columns + column) as usize;
        values[cell] = match (reduction, counts[cell]) {
            (_, 0) => z,
            (Reduction::Mean, n) => values[cell] * (n as f64 / (n + 1) as f64) + z / (n + 1) as f64,
            (Reduction::Min, _) => values[cell].min(z),
            (Reduction::Max, _) => values[cell].max(z),
        };
        counts[cell] += 1;
    }
    let grid = ContourGrid {
        columns,
        rows,
        x_edges: edges(x, dimensions[0], strides[0], &request.x_scale),
        y_edges: edges(y, dimensions[1], strides[1], &request.y_scale),
        values,
        valid: counts.iter().map(|count| *count > 0).collect(),
    };
    Ok((grid, counts))
}

pub(super) fn fit(grid: &mut ContourGrid, counts: &mut Vec<u64>, reduction: Reduction, maximum: usize) {
    while maximum > 0 && grid.values.len() > maximum {
        let axis = usize::from(grid.rows > grid.columns);
        let columns = if axis == 0 { grid.columns.div_ceil(2) } else { grid.columns };
        let rows = if axis == 1 { grid.rows.div_ceil(2) } else { grid.rows };
        let mut values = vec![0.0; columns as usize * rows as usize];
        let mut merged = vec![0_u64; values.len()];
        for (index, count) in counts.iter().copied().enumerate() {
            if count == 0 {
                continue;
            }
            let column = index % grid.columns as usize;
            let row = index / grid.columns as usize;
            let column = if axis == 0 { column / 2 } else { column };
            let row = if axis == 1 { row / 2 } else { row };
            let cell = row * columns as usize + column;
            let total = merged[cell] + count;
            values[cell] = match (reduction, merged[cell]) {
                (_, 0) => grid.values[index],
                (Reduction::Mean, n) => values[cell] * (n as f64 / total as f64) + grid.values[index] * (count as f64 / total as f64),
                (Reduction::Min, _) => values[cell].min(grid.values[index]),
                (Reduction::Max, _) => values[cell].max(grid.values[index]),
            };
            merged[cell] = total;
        }
        let edges = if axis == 0 { &mut grid.x_edges } else { &mut grid.y_edges };
        let last = *edges.last().expect("grid has edges");
        *edges = edges.iter().step_by(2).copied().collect();
        let edge_count = if axis == 0 { columns } else { rows } as usize + 1;
        if edges.len() != edge_count {
            edges.push(last);
        }
        grid.columns = columns;
        grid.rows = rows;
        grid.values = values;
        grid.valid = merged.iter().map(|count| *count > 0).collect();
        *counts = merged;
    }
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

fn edges(bounds: PlotWindow, bins: u32, stride: u64, scale: &AxisScale) -> Vec<f64> {
    (0..=u64::from(bins).div_ceil(stride))
        .map(|index| {
            let index = (index * stride).min(u64::from(bins));
            let value = bounds.from + (bounds.to - bounds.from) * index as f64 / f64::from(bins);
            if *scale == AxisScale::Log { value.exp() } else { value }
        })
        .collect()
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::super::PlotPoint;
    use super::*;

    #[test]
    fn sparse_duplicates_invalid_samples_and_zero() {
        let series = vec![PlotSeries {
            key: "z".into(),
            label: "Z".into(),
            unit: None,
            raw_points: 5,
            downsampled: false,
            format: None,
            points: [(0.1, 0.1, 2.0), (0.2, 0.2, 6.0), (1.5, 1.5, 0.0), (0.1, 0.1, f64::NAN), (f64::INFINITY, 1.0, 7.0)]
                .into_iter()
                .map(|(x, y, z)| PlotPoint {
                    instant: None,
                    x: serde_json::json!(x),
                    y: Some(y),
                    z: Some(z),
                })
                .collect(),
        }];
        let mut request = PlotPayloadRequest {
            panel_width: Some(2),
            panel_height: Some(2),
            viewport: Some(PlotWindow { from: 0.0, to: 2.0 }),
            y_viewport: Some(PlotWindow { from: 0.0, to: 2.0 }),
            ..Default::default()
        };
        for (reduction, expected) in [(Reduction::Mean, 4.0), (Reduction::Min, 2.0), (Reduction::Max, 6.0)] {
            request.reduction = Some(reduction);
            let (mut grid, mut counts) = bin(&series, &request, 0).unwrap();
            assert_eq!(grid.valid, vec![true, false, false, true]);
            assert_eq!(grid.values, vec![expected, 0.0, 0.0, 0.0]);
            assert_eq!(grid.x_edges, vec![0.0, 1.0, 2.0]);
            fit(&mut grid, &mut counts, reduction, 1);
            let expected = match reduction {
                Reduction::Mean => 8.0 / 3.0,
                Reduction::Min => 0.0,
                Reduction::Max => 6.0,
            };
            assert_eq!((grid.columns, grid.rows), (1, 1));
            assert!((grid.values[0] - expected).abs() < 1e-12);
            assert_eq!(grid.valid, [true]);
            assert_eq!(counts, [3]);
            assert_eq!(grid.x_edges, [0.0, 2.0]);
        }
        request.x_scale = AxisScale::Log;
        request.viewport = Some(PlotWindow { from: 1.0, to: 100.0 });
        let (grid, _) = bin(&series, &request, 0).unwrap();
        assert!((grid.x_edges[1] - 10.0).abs() < 1e-12);
    }
}
