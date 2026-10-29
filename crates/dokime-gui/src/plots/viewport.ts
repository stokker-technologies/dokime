/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * Box zoom and held-viewport panning on a cartesian plot.
 *
 * A drag draws uPlot's selection box; on release the box becomes the held
 * viewport. A mostly horizontal drag zooms x alone, a mostly vertical one y
 * alone, a diagonal one both: uPlot expands a one-axis selection across the
 * whole other axis, and an axis the box spans fully is not held, so it keeps
 * following the run (x) or auto-ranging (y). This module has no application
 * imports so a node test can load it directly.
 */

/* === Imports === */

import type uPlot from "uplot";

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

type PanOptions = { logX: boolean; logY: boolean; panY: boolean; onPan: (viewport: HeldViewport) => void };

type PanStart = {
  left: number;
  top: number;
  width: number;
  height: number;
  buttons: number;
  x: ViewportRange;
  y: ViewportRange | null;
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

/** Translate A Visible Range Without Changing Its Linear Width Or Logarithmic Ratio. */
export function pannedRange(range: ViewportRange, fraction: number, logarithmic: boolean): ViewportRange | null {
  if (!Number.isFinite(fraction) || !Number.isFinite(range.min) || !Number.isFinite(range.max) || range.min >= range.max) return null;
  if (logarithmic && range.min <= 0) return null;
  const first = logarithmic ? Math.log(range.min) : range.min;
  const last = logarithmic ? Math.log(range.max) : range.max;
  const shift = (last - first) * fraction;
  const min = logarithmic ? Math.exp(first + shift) : first + shift;
  const max = logarithmic ? Math.exp(last + shift) : last + shift;
  return Number.isFinite(min) && Number.isFinite(max) && min < max ? { min, max } : null;
}

/** The Chart Owns Its Gesture Listeners; Only A Drag Owns Window Listeners. */
export function attachPan(
  plot: Pick<uPlot, "over" | "scales" | "setScale" | "batch" | "cursor" | "setCursor">,
  options: PanOptions,
): () => void {
  const view = plot.over.ownerDocument.defaultView;
  if (!view) return () => undefined;
  let start: PanStart | null = null;
  const stop = () => {
    start = null;
    view.removeEventListener("mousemove", move);
    view.removeEventListener("mouseup", stop);
    view.removeEventListener("blur", stop);
  };
  const move = (event: MouseEvent) => {
    if (!start) return;
    if (!(event.buttons & start.buttons)) {
      stop();
      return;
    }
    const x = pannedRange(start.x, (start.left - event.clientX) / start.width, options.logX);
    const y = start.y ? pannedRange(start.y, (event.clientY - start.top) / start.height, options.logY) : null;
    if (!x || (start.y && !y)) return;
    event.preventDefault();
    options.onPan({ x, y });
    plot.batch(() => {
      plot.setScale("x", x);
      if (y) plot.setScale("y", y);
    });
    const { left, top } = plot.cursor;
    if (left != null && top != null) plot.setCursor({ left, top });
  };
  const down = (event: MouseEvent) => {
    if (!((event.button === 0 && event.shiftKey) || event.button === 1)) return;
    const { min, max } = plot.scales.x;
    if (min == null || max == null || !(plot.over.clientWidth > 0) || !(plot.over.clientHeight > 0)) return;
    const x = orderedRange(min, max);
    if (!x) return;
    stop();
    const yScale = plot.scales.y;
    const y = options.panY && yScale.min != null && yScale.max != null ? orderedRange(yScale.min, yScale.max) : null;
    start = {
      left: event.clientX,
      top: event.clientY,
      width: plot.over.clientWidth,
      height: plot.over.clientHeight,
      buttons: event.button === 1 ? 4 : 1,
      x,
      y,
    };
    event.preventDefault();
    event.stopImmediatePropagation();
    view.addEventListener("mousemove", move);
    view.addEventListener("mouseup", stop);
    view.addEventListener("blur", stop);
  };
  plot.over.addEventListener("mousedown", down, true);
  return () => {
    stop();
    plot.over.removeEventListener("mousedown", down, true);
  };
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
