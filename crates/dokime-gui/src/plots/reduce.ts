/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * Plot point budgets and M4 point reduction.
 *
 * uPlot decimates line paths itself; M4 thins dense point marks to each
 * pixel column's first, lowest, highest and last sample. Buckets span equal
 * intervals of x, so uneven sampling does not distort the drawn envelope.
 */

/* === Constants === */

/** The host display budget when a plot declares none: four points per column of a 4K-wide plot draw every sample exactly. */
export const DEFAULT_PLOT_MAX_POINTS = 15_360;

/** The host's per-plot point ceiling; it also bounds a plot that keeps every sample. */
export const NATIVE_PLOT_MAX_POINTS = 100_000;

/* === Types === */

type Values = readonly (number | null | undefined)[];

/* === Public Functions === */

/**
 * M4 over the indices `[from, to)` in `buckets` equal spans of x: each bucket
 * keeps its first, lowest, highest and last finite sample, in index order.
 */
export function m4Indices(x: readonly number[], y: Values, from: number, to: number, buckets: number): number[] {
  if (to <= from || buckets <= 0) {
    return [];
  }

  const bounds = spanBuckets(x, from, to, buckets);
  const kept: number[] = [];
  for (let bucket = 0; bucket < buckets; bucket += 1) {
    let first = -1;
    let last = -1;
    let lowest = -1;
    let highest = -1;
    for (let index = bounds[bucket]; index < bounds[bucket + 1]; index += 1) {
      const value = y[index];
      if (typeof value !== "number" || !Number.isFinite(value)) {
        continue;
      }
      if (first < 0) {
        first = index;
      }
      last = index;
      if (lowest < 0 || value < (y[lowest] as number)) {
        lowest = index;
      }
      if (highest < 0 || value > (y[highest] as number)) {
        highest = index;
      }
    }
    if (first < 0) {
      continue;
    }
    const corners = [...new Set([first, lowest, highest, last])].sort((left, right) => left - right);
    kept.push(...corners);
  }
  return kept;
}

/* === Private Functions === */

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
    while (index < to && x[index] < edge) {
      index += 1;
    }
    bounds[bucket] = index;
  }
  bounds[count] = to;
  return bounds;
}
