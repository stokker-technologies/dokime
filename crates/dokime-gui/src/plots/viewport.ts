/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * Box zoom on a cartesian plot.
 *
 * A drag draws uPlot's selection box; on release the box becomes the held
 * viewport. A mostly horizontal drag zooms x alone, a mostly vertical one y
 * alone, a diagonal one both: uPlot expands a one-axis selection across the
 * whole other axis, and an axis the box spans fully is not held, so it keeps
 * following the run (x) or auto-ranging (y). This module has no application
 * imports so a node test can load it directly.
 */

/* === Constants === */

/**
 * The drag thresholds, in CSS pixels: a drag shorter than `dist` on both
 * axes is a click, and a drag within `uni` on one axis zooms the other alone.
 */
export const PLOT_DRAG_OPTIONS = {
  x: true,
  y: true,
  uni: 24,
  dist: 4,
  setScale: true,
} as const;

/* === Types === */

export type ViewportRange = {
  min: number;
  max: number;
};

export type HeldViewport = {
  /** `null` leaves the axis to its live follow or auto range. */
  x: ViewportRange | null;
  y: ViewportRange | null;
};

export type SelectionBox = {
  left: number;
  top: number;
  width: number;
  height: number;
};

export type PlotArea = {
  width: number;
  height: number;
};

/* === Public Functions === */

/**
 * The viewport a released selection box holds, or `null` for an empty box.
 *
 * `posToVal` maps a CSS pixel on an axis to its value, as `uPlot.posToVal`
 * does; ranges come back ordered whichever way the axis runs.
 */
export function heldViewport(
  select: SelectionBox,
  area: PlotArea,
  posToVal: (position: number, axis: "x" | "y") => number,
): HeldViewport | null {
  if (!(select.width > 0) || !(select.height > 0)) return null;
  const x = spansAxis(select.left, select.width, area.width)
    ? null
    : orderedRange(posToVal(select.left, "x"), posToVal(select.left + select.width, "x"));
  const y = spansAxis(select.top, select.height, area.height)
    ? null
    : orderedRange(posToVal(select.top, "y"), posToVal(select.top + select.height, "y"));
  if (!x && !y) return null;
  return { x, y };
}

/* === Private Functions === */

/** Whether the box covers the whole axis: uPlot's one-axis drag fills the other. */
function spansAxis(start: number, extent: number, full: number): boolean {
  return full > 0 && start <= 0.5 && extent >= full - 0.5;
}

function orderedRange(first: number, second: number): ViewportRange | null {
  if (!Number.isFinite(first) || !Number.isFinite(second) || first === second) return null;
  return { min: Math.min(first, second), max: Math.max(first, second) };
}
