/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * Time-bucketed point reduction for retained live plots.
 *
 * The host reduces every snapshot and append to the display budget; the client
 * only compacts the frame it retains once appends have grown it past twice its
 * capacity. Buckets are equal spans of x, not equal counts, so a frame that is
 * compacted again and again keeps an even density over time instead of thinning
 * its oldest samples each time. This module has no application imports so a
 * node test can load it directly.
 */

/* === Constants === */

/** The host display budget when a plot declares none. */
export const DEFAULT_PLOT_MAX_POINTS = 2_000;

/** The host's per-plot point ceiling; it also bounds a plot that keeps every sample. */
export const NATIVE_PLOT_MAX_POINTS = 100_000;

/* === Types === */

export type ReduceTechnique = "auto" | "lttb" | "min_max" | "raw";

type Values = readonly (number | null | undefined)[];

/* === Public Functions === */

/**
 * The indices one x axis keeps for every series drawn on it, ascending.
 *
 * Each series is reduced to `budget` on its own and the picks are united with
 * the first and last samples, so every series stays aligned on the shared x
 * and the drawn span never shrinks. `null` means nothing is dropped.
 */
export function reduceIndices(
  x: readonly number[],
  series: readonly Values[],
  budget: number,
  technique: ReduceTechnique,
): number[] | null {
  const length = x.length;
  if (length <= budget) return null;
  if (technique === "raw") return strideIndices(x, budget);

  const envelope = technique === "min_max" || (technique === "auto" && length > 4 * budget);
  // The ends always stay, so compaction never moves where the trace starts or stops.
  const picked = new Set<number>([0, length - 1]);
  for (const y of series) {
    // The envelope leaves room for the two ends; LTTB keeps them itself.
    const indices = envelope ? minMaxIndices(x, y, budget - 2) : lttbIndices(x, y, budget);
    for (const index of indices) picked.add(index);
  }
  return [...picked].sort((left, right) => left - right);
}

/**
 * Largest-Triangle-Three-Buckets over equal spans of x: the first and last
 * samples are kept, every interior bucket keeps the sample that spans the
 * largest triangle with the previous pick and the next bucket's mean. Samples
 * without a finite y are never picked; a bucket without one keeps nothing.
 */
export function lttbIndices(x: readonly number[], y: Values, count: number): number[] {
  const length = Math.min(x.length, y.length);
  if (length <= count || count < 3) return ascending(length);

  const bounds = spanBuckets(x, 1, length - 1, count - 2);
  const kept: number[] = [0];
  let previous = 0;

  for (let bucket = 0; bucket < count - 2; bucket += 1) {
    const start = bounds[bucket];
    const end = bounds[bucket + 1];
    if (start >= end) continue;

    const mean = meanOf(x, y, end, bounds[bucket + 2] ?? length) ?? [
      x[length - 1],
      finite(y[length - 1]),
    ];
    const previousY = finite(y[previous]);

    let picked = -1;
    let largest = -1;
    for (let candidate = start; candidate < end; candidate += 1) {
      const candidateY = y[candidate];
      if (typeof candidateY !== "number" || !Number.isFinite(candidateY)) continue;
      const measured = area(x[previous], previousY, x[candidate], candidateY, mean[0], mean[1]);
      if (measured > largest) {
        largest = measured;
        picked = candidate;
      }
    }
    if (picked < 0) continue;

    kept.push(picked);
    previous = picked;
  }

  kept.push(length - 1);
  return kept;
}

/**
 * Bucketed extrema over `count / 2` equal spans of x: each bucket contributes
 * its lowest and highest finite sample in index order.
 */
export function minMaxIndices(x: readonly number[], y: Values, count: number): number[] {
  const length = Math.min(x.length, y.length);
  if (length === 0 || count <= 0) return [];

  const buckets = Math.max(1, Math.floor(count / 2));
  const bounds = spanBuckets(x, 0, length, buckets);
  const kept: number[] = [];

  for (let bucket = 0; bucket < buckets; bucket += 1) {
    let lowest = -1;
    let highest = -1;
    for (let index = bounds[bucket]; index < bounds[bucket + 1]; index += 1) {
      const value = y[index];
      if (typeof value !== "number" || !Number.isFinite(value)) continue;
      if (lowest < 0 || value < (y[lowest] as number)) lowest = index;
      if (highest < 0 || value > (y[highest] as number)) highest = index;
    }
    if (lowest < 0) continue;
    kept.push(Math.min(lowest, highest));
    if (lowest !== highest) kept.push(Math.max(lowest, highest));
  }

  return kept;
}

/* === Private Functions === */

/** The first sample of each of `count` equal spans of x, plus the last sample. */
function strideIndices(x: readonly number[], count: number): number[] {
  const length = x.length;
  const buckets = Math.max(1, count - 1);
  const bounds = spanBuckets(x, 0, length - 1, buckets);
  const kept: number[] = [];
  for (let bucket = 0; bucket < buckets; bucket += 1) {
    if (bounds[bucket] < bounds[bucket + 1]) kept.push(bounds[bucket]);
  }
  kept.push(length - 1);
  return kept;
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

function ascending(length: number): number[] {
  return Array.from({ length }, (_, index) => index);
}

function finite(value: number | null | undefined): number {
  return typeof value === "number" && Number.isFinite(value) ? value : 0;
}

function meanOf(
  x: readonly number[],
  y: Values,
  start: number,
  end: number,
): [number, number] | null {
  let count = 0;
  let sumX = 0;
  let sumY = 0;
  for (let index = start; index < end; index += 1) {
    const value = y[index];
    if (typeof value !== "number" || !Number.isFinite(value) || !Number.isFinite(x[index])) {
      continue;
    }
    count += 1;
    sumX += x[index];
    sumY += value;
  }
  return count === 0 ? null : [sumX / count, sumY / count];
}

function area(x0: number, y0: number, x1: number, y1: number, x2: number, y2: number): number {
  return Math.abs((x0 - x2) * (y1 - y0) - (x0 - x1) * (y2 - y0));
}
