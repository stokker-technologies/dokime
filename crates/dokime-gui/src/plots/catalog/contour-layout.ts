// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ContourGrid } from "@generated/host/model";
import { contours } from "d3-contour";
import { geoPath, geoTransform } from "d3-geo";
import { scaleLinear, scaleLog } from "d3-scale";
import type { Wire } from "../../lib/rpc/canonical";
import { sequentialColorScale } from "../colors";
import type { PlotCardSpec } from "../types";

export const WIDTH = 720;
export const HEIGHT = 420;
export const MARGIN = { left: 58, right: 24, top: 20, bottom: 52 };

export function contourLayout(grid: Wire<ContourGrid>, plot: PlotCardSpec) {
  const width = WIDTH - MARGIN.left - MARGIN.right;
  const height = HEIGHT - MARGIN.top - MARGIN.bottom;
  const values = grid.values.filter((_, index) => grid.valid[index]);
  const { color } = sequentialColorScale(values);
  const x = (plot.xScale === "log" ? scaleLog : scaleLinear)(
    [grid.xEdges[0], grid.xEdges[grid.columns]],
    [MARGIN.left, MARGIN.left + width],
  );
  const y = (plot.yScale === "log" ? scaleLog : scaleLinear)([grid.yEdges[0], grid.yEdges[grid.rows]], [MARGIN.top + height, MARGIN.top]);
  const path = geoPath(
    geoTransform({
      point(column, row) {
        this.stream.point(gridCoordinate(column, grid.xEdges, x), gridCoordinate(row, grid.yEdges, y));
      },
    }),
  );
  const shapes =
    plot.render === "bands"
      ? contours().size([grid.columns, grid.rows]).smooth(false).thresholds(8)(
          grid.values.map((value, index) => (grid.valid[index] ? value : Number.NaN)),
        )
      : [];
  const cells = grid.values.flatMap((value, index) => {
    if (!grid.valid[index]) {
      return [];
    }
    const column = index % grid.columns;
    const row = Math.floor(index / grid.columns);
    return [
      {
        value,
        index,
        column,
        row,
        left: x(grid.xEdges[column]),
        right: x(grid.xEdges[column + 1]),
        top: y(grid.yEdges[row + 1]),
        bottom: y(grid.yEdges[row]),
      },
    ];
  });
  return { width, height, color, x, y, path, shapes, cells };
}

export function validContourGrid(grid: Wire<ContourGrid>): boolean {
  const count = grid.columns * grid.rows;
  return (
    Number.isSafeInteger(count) &&
    grid.columns > 0 &&
    grid.rows > 0 &&
    grid.values.length === count &&
    grid.valid.length === count &&
    grid.xEdges.length === grid.columns + 1 &&
    grid.yEdges.length === grid.rows + 1 &&
    grid.xEdges.every(Number.isFinite) &&
    grid.yEdges.every(Number.isFinite) &&
    grid.values.every((value, index) => !grid.valid[index] || Number.isFinite(value))
  );
}

/* === Private Functions === */

function gridCoordinate(position: number, edges: number[], scale: (value: number) => number): number {
  const bounded = Math.max(0, Math.min(position, edges.length - 1));
  const cell = Math.min(Math.floor(bounded), edges.length - 2);
  const from = scale(edges[cell]);
  const to = scale(edges[cell + 1]);
  return from + (to - from) * (bounded - cell);
}
