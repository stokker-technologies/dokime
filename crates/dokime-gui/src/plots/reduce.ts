/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * Pixel-window point reduction for live plots and saved reports.
 *
 * The renderer keeps each pixel column's first, lowest, highest and last
 * sample over the visible x range. Buckets span equal intervals of x, so
 * uneven sampling does not distort the drawn envelope.
 */

/* === Constants === */

const DOCUMENTS_DEFAULT_PLOT_POINTS = 2_000;
const DOCUMENTS_MAXIMUM_PLOT_POINTS = 100_000;

/** The host display budget when a plot declares none. */
export const DEFAULT_PLOT_MAX_POINTS = DOCUMENTS_DEFAULT_PLOT_POINTS;

/** The host's per-plot point ceiling; it also bounds a plot that keeps every sample. */
export const NATIVE_PLOT_MAX_POINTS = DOCUMENTS_MAXIMUM_PLOT_POINTS;

/* === Types === */

type Values = readonly (number | null | undefined)[];

/* === Public Functions === */

/**
 * The indices one x axis draws over the visible `range`, ascending, for a
 * plot `columns` CSS pixels wide; `null` means every sample is drawn.
 *
 * M4 over the window: each pixel column keeps its first, lowest, highest and
 * last finite sample per series, so the drawn envelope is the data's envelope
 * at any zoom. The sample on each side of the window is kept so the line runs
 * to the frame edges, and the series' own first and last samples are kept so
 * the data extent (what a reset fits to) never shrinks to the window.
 */
export function windowIndices(
  x: readonly number[],
  series: readonly Values[],
  range: { min: number; max: number } | null,
  columns: number,
): number[] | null {
  const length = x.length;
  const buckets = Math.max(1, Math.floor(columns));
  if (length <= 4 * buckets) return null;

  const from = range ? Math.max(0, lowerBound(x, range.min) - 1) : 0;
  const to = range ? Math.min(length, upperBound(x, range.max) + 1) : length;
  const picked = new Set<number>([0, length - 1]);
  if (to - from <= 4 * buckets) {
    for (let index = from; index < to; index += 1) picked.add(index);
  } else {
    for (const y of series) {
      for (const index of m4Indices(x, y, from, to, buckets)) picked.add(index);
    }
  }
  return [...picked].sort((left, right) => left - right);
}

/**
 * M4 over the indices `[from, to)` in `buckets` equal spans of x: each bucket
 * keeps its first, lowest, highest and last finite sample, in index order.
 */
export function m4Indices(x: readonly number[], y: Values, from: number, to: number, buckets: number): number[] {
  if (to <= from || buckets <= 0) return [];

  const bounds = spanBuckets(x, from, to, buckets);
  const kept: number[] = [];
  for (let bucket = 0; bucket < buckets; bucket += 1) {
    let first = -1;
    let last = -1;
    let lowest = -1;
    let highest = -1;
    for (let index = bounds[bucket]; index < bounds[bucket + 1]; index += 1) {
      const value = y[index];
      if (typeof value !== "number" || !Number.isFinite(value)) continue;
      if (first < 0) first = index;
      last = index;
      if (lowest < 0 || value < (y[lowest] as number)) lowest = index;
      if (highest < 0 || value > (y[highest] as number)) highest = index;
    }
    if (first < 0) continue;
    const corners = [...new Set([first, lowest, highest, last])].sort((left, right) => left - right);
    kept.push(...corners);
  }
  return kept;
}

/* === Private Functions === */

/** The first index whose x is at or above `value`; x ascends. */
function lowerBound(x: readonly number[], value: number): number {
  let low = 0;
  let high = x.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if (x[middle] < value) low = middle + 1;
    else high = middle;
  }
  return low;
}

/** The first index whose x is above `value`; x ascends. */
function upperBound(x: readonly number[], value: number): number {
  let low = 0;
  let high = x.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if (x[middle] <= value) low = middle + 1;
    else high = middle;
  }
  return low;
}

/**
 * Boundaries of `count` equal spans of x over the indices `[from, to)`:
 * bucket `i` holds the indices `[bounds[i], bounds[i + 1])`. x ascends; a
 * degenerate span falls back to equal counts.
 */
function spanBuckets(x: readonly number[], from: number, to: number, count: number): number[] {
  const bounds = new Array<number>(count + 1);
  const first = x[from];
  const width = (x[to - 1] - first) / count;
  if (!(width > 0) || !Number.isFinite(width)) {
    for (let bucket = 0; bucket <= count; bucket += 1) {
      bounds[bucket] = from + Math.floor((bucket * (to - from)) / count);
    }
    return bounds;
  }

  let index = from;
  bounds[0] = from;
  for (let bucket = 1; bucket < count; bucket += 1) {
    const edge = first + bucket * width;
    while (index < to && x[index] < edge) index += 1;
    bounds[bucket] = index;
  }
  bounds[count] = to;
  return bounds;
}
