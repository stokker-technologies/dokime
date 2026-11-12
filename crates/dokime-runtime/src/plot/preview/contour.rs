/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Contour Capture on an Expanding, Aligned Bin Lattice.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::Limits;
use crate::document::{AxisScale, Reduction};
use crate::plot::{PlotPayloadRequest, PlotPoint};
use dokime_domain::model::plots::wire::ContourGrid;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default, Serialize, Deserialize)]
pub(super) struct Contour {
    width: Option<[f64; 2]>,
    bounds: Option<[f64; 4]>,
    single: [bool; 2],
    cells: BTreeMap<i64, BTreeMap<i64, Cell>>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Cell {
    value: f64,
    count: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Contour {
    pub(super) fn push(&mut self, point: &PlotPoint, request: &PlotPayloadRequest, limits: Limits) {
        let (Some(x), Some(y), Some(z)) = (point.x.as_f64(), point.y, point.z) else { return };
        let (Some(x), Some(y)) = (coordinate(x, &request.x_scale), coordinate(y, &request.y_scale)) else { return };
        if !z.is_finite() {
            return;
        }
        self.single = [limits.grid_columns == 1, limits.grid_rows == 1];
        self.bounds =
            Some(self.bounds.map_or([x, x, y, y], |bounds| [bounds[0].min(x), bounds[1].max(x), bounds[2].min(y), bounds[3].max(y)]));
        self.width.get_or_insert_with(|| [quantum(x), quantum(y)]);
        for (axis, value) in [x, y].into_iter().enumerate() {
            while (value / self.width.expect("initialized width")[axis]).abs() > i64::MAX as f64 / 2.0 {
                self.coarsen(axis, request.reduction.unwrap_or_default());
            }
        }
        let width = self.width.expect("initialized width");
        let x = if self.single[0] { 0 } else { (x / width[0]).floor() as i64 };
        let y = if self.single[1] { 0 } else { (y / width[1]).floor() as i64 };
        self.insert(x, y, Cell { value: z, count: 1 }, request.reduction.unwrap_or_default());
        self.fit(request, limits);
    }

    pub(super) fn merge(&mut self, mut other: Self, request: &PlotPayloadRequest, limits: Limits) {
        let Some(other_width) = other.width else { return };
        if self.width.is_none() {
            *self = other;
            return;
        }
        if let Some(bounds) = other.bounds {
            self.bounds = Some(self.bounds.map_or(bounds, |current| {
                [current[0].min(bounds[0]), current[1].max(bounds[1]), current[2].min(bounds[2]), current[3].max(bounds[3])]
            }));
        }
        for (axis, other_width) in other_width.into_iter().enumerate() {
            let width = self.width.expect("initialized width")[axis].max(other_width);
            while self.width.expect("initialized width")[axis] < width {
                self.coarsen(axis, request.reduction.unwrap_or_default());
            }
            while other.width.expect("initialized width")[axis] < width {
                other.coarsen(axis, request.reduction.unwrap_or_default());
            }
        }
        for (x, rows) in other.cells {
            for (y, cell) in rows {
                self.insert(x, y, cell, request.reduction.unwrap_or_default());
            }
        }
        self.fit(request, limits);
    }

    fn insert(&mut self, x: i64, y: i64, cell: Cell, reduction: Reduction) {
        self.cells
            .entry(x)
            .or_default()
            .entry(y)
            .and_modify(|current| {
                let total = current.count.saturating_add(cell.count);
                current.value = match reduction {
                    Reduction::Mean => {
                        current.value * (current.count as f64 / total as f64) + cell.value * (cell.count as f64 / total as f64)
                    }
                    Reduction::Min => current.value.min(cell.value),
                    Reduction::Max => current.value.max(cell.value),
                };
                current.count = total;
            })
            .or_insert(cell);
    }

    fn extent(&self) -> Option<[i64; 4]> {
        let first = *self.cells.first_key_value()?.0;
        let last = *self.cells.last_key_value()?.0;
        let low = self.cells.values().filter_map(|rows| rows.first_key_value().map(|(y, _)| *y)).min()?;
        let high = self.cells.values().filter_map(|rows| rows.last_key_value().map(|(y, _)| *y)).max()?;
        Some([first, last, low, high])
    }

    fn fit(&mut self, request: &PlotPayloadRequest, limits: Limits) {
        for (axis, maximum) in [limits.grid_columns.max(1), limits.grid_rows.max(1)].into_iter().enumerate() {
            if self.single[axis] {
                continue;
            }
            while self.extent().is_some_and(|extent| i128::from(extent[axis * 2 + 1]) - i128::from(extent[axis * 2]) + 1 > maximum as i128)
            {
                self.coarsen(axis, request.reduction.unwrap_or_default());
            }
        }
    }

    fn coarsen(&mut self, axis: usize, reduction: Reduction) {
        self.width.as_mut().expect("initialized width")[axis] *= 2.0;
        // Aligned Powers Of Two Merge Complete Cells; No Missing Cell Is Invented.
        for (x, rows) in std::mem::take(&mut self.cells) {
            for (y, cell) in rows {
                self.insert(if axis == 0 { x.div_euclid(2) } else { x }, if axis == 1 { y.div_euclid(2) } else { y }, cell, reduction);
            }
        }
    }

    pub(super) fn grid(&self, request: &PlotPayloadRequest) -> (ContourGrid, Vec<u64>) {
        let [x0, x1, y0, y1] = self.extent().unwrap_or([0, 0, 0, 0]);
        let width = self.width.unwrap_or([1.0, 1.0]);
        let columns = (i128::from(x1) - i128::from(x0) + 1) as u32;
        let rows = (i128::from(y1) - i128::from(y0) + 1) as u32;
        let mut values = vec![0.0; columns as usize * rows as usize];
        let mut valid = vec![false; values.len()];
        let mut counts = vec![0; values.len()];
        for (x, entries) in &self.cells {
            for (y, cell) in entries {
                let index = (*y - y0) as usize * columns as usize + (*x - x0) as usize;
                values[index] = cell.value;
                valid[index] = true;
                counts[index] = cell.count;
            }
        }
        let grid = ContourGrid {
            columns,
            rows,
            values,
            valid,
            x_edges: if self.single[0] {
                single_edges(self.bounds.map(|bounds| [bounds[0], bounds[1]]), &request.x_scale)
            } else {
                (0..=columns).map(|index| edge((x0 as f64 + f64::from(index)) * width[0], &request.x_scale)).collect()
            },
            y_edges: if self.single[1] {
                single_edges(self.bounds.map(|bounds| [bounds[2], bounds[3]]), &request.y_scale)
            } else {
                (0..=rows).map(|index| edge((y0 as f64 + f64::from(index)) * width[1], &request.y_scale)).collect()
            },
        };
        (grid, counts)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn coordinate(value: f64, scale: &AxisScale) -> Option<f64> {
    (value.is_finite() && (*scale != AxisScale::Log || value > 0.0)).then(|| if *scale == AxisScale::Log { value.ln() } else { value })
}

fn quantum(value: f64) -> f64 {
    2.0_f64.powf((value.abs().max(1.0) * 1e-9).log2().floor())
}

fn edge(value: f64, scale: &AxisScale) -> f64 {
    if *scale == AxisScale::Log { value.exp() } else { value }
}

fn single_edges(bounds: Option<[f64; 2]>, scale: &AxisScale) -> Vec<f64> {
    let [mut low, mut high] = bounds.unwrap_or([0.0, 1.0]);
    if low == high {
        let pad = (low.abs() * 0.01).max(0.5);
        low -= pad;
        high += pad;
    }
    vec![edge(low, scale), edge(high, scale)]
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_preserves_reductions_gaps_and_checkpoint_merges() {
        let request = PlotPayloadRequest::default();
        let limits = Limits {
            exact_points: 4,
            grid_columns: 4,
            grid_rows: 4,
            groups: 8,
            buckets: 4,
            interval_nanos: 1,
        };
        let point = |x, y, z| PlotPoint {
            instant: None,
            x: serde_json::json!(x),
            y: Some(y),
            z: Some(z),
        };
        let mut left = Contour::default();
        for z in [2.0, 6.0] {
            left.push(&point(1.0, 1.0, z), &request, limits);
        }
        let mut right = Contour::default();
        for _ in 0..100 {
            right.push(&point(3.0, 3.0, 0.0), &request, limits);
        }
        right.push(&point(3.0, 3.0, f64::NAN), &request, limits);
        left.merge(right, &request, limits);
        let checkpoint = serde_json::to_vec(&left).unwrap();
        let reopened: Contour = serde_json::from_slice(&checkpoint).unwrap();
        let (grid, _) = reopened.grid(&request);
        assert!(grid.columns <= 4 && grid.rows <= 4);
        assert_eq!(grid.valid.iter().filter(|valid| **valid).count(), 2);
        assert!(grid.valid.iter().any(|valid| !valid));
        assert!(grid.values.iter().zip(&grid.valid).any(|(value, valid)| *valid && *value == 0.0));
        assert!(grid.values.contains(&4.0));
        left.push(&point(-20.0, -30.0, 8.0), &request, limits);
        let (grid, _) = left.grid(&request);
        assert!(grid.columns <= 4 && grid.rows <= 4);
        assert!(grid.x_edges.windows(2).all(|edge| edge[0] < edge[1]));
        let single = Limits { grid_columns: 1, grid_rows: 1, ..limits };
        let mut capture = Contour::default();
        for (x, value) in [(-10.0, 2.0), (100.0, 6.0)] {
            capture.push(&point(x, x, value), &request, single);
        }
        let (grid, _) = capture.grid(&request);
        assert_eq!((grid.columns, grid.rows), (1, 1));
        assert_eq!(grid.values, [4.0]);
        assert_eq!(grid.valid, [true]);
        assert_eq!(grid.x_edges, [-10.0, 100.0]);
    }
}
