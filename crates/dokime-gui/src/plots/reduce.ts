/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * Time-aware point reduction for live plots.
 *
 * The host reduces a snapshot with `harmos_signal::lttb`; the live append path
 * reduces the same way here so a frame that grew past its budget between two
 * server snapshots keeps the shape of the signal instead of its last N samples.
 * This module has no application imports so a node test can load it directly.
 */

/* === Constants === */

/** The host display budget when a plot declares none. */
export const DEFAULT_PLOT_MAX_POINTS = 2_000;

/** Raw samples retained behind the display budget before reduction. */
export const RAW_RING_FACTOR = 20;

/* === Types === */

export type ReduceTechnique = "auto" | "lttb" | "min_max" | "raw";

export type ReduceOptions = {
  /** Display budget; `null` keeps every retained sample. */
  budget: number | null;
  technique: ReduceTechnique;
};

/* === Public Functions === */

/**
 * The first index kept once the trailing window and the raw cap apply.
 *
 * The window is applied before the cap on purpose: a count-first cut evicts
 * the oldest samples of a decimated snapshot and the visible span drifts
 * below `windowS`. The cap only guards memory for an unbounded window.
 */
export function retainedStart(x: readonly number[], windowS: number | null, cap: number): number {
  const length = x.length;
  if (length === 0) return 0;
  let start = 0;
  if (windowS !== null && windowS > 0 && length > 1) {
    const latest = finiteMax(x);
    if (latest !== null) {
      const cutoff = latest - windowS;
      start = firstAtOrAfter(x, cutoff);
      if (start >= length) start = length - 1;
    }
  }
  if (cap > 0 && length - start > cap) start = length - cap;
  return start;
}

/** Indices kept by the requested technique, in ascending order. */
export function reduceIndices(
  x: readonly number[],
  y: readonly (number | null | undefined)[],
  options: ReduceOptions,
): number[] | null {
  const length = Math.min(x.length, y.length);
  if (options.budget === null || options.technique === "raw" || length <= options.budget) {
    return null;
  }
  if (
    options.technique === "min_max" ||
    (options.technique === "auto" && length > 4 * options.budget)
  ) {
    return minMaxIndices(y, options.budget);
  }
  return lttbIndices(x, y, options.budget);
}

/**
 * Largest-Triangle-Three-Buckets, the same bucketing as the host's `lttb`:
 * first and last samples are kept, every interior bucket keeps the sample
 * that spans the largest triangle with the previous pick and the next
 * bucket's mean. Samples without a finite y are never picked.
 */
export function lttbIndices(
  x: readonly number[],
  y: readonly (number | null | undefined)[],
  count: number,
): number[] {
  const length = Math.min(x.length, y.length);
  if (length <= count || count < 3) return ascending(length);

  const kept: number[] = [0];
  const bucket = (length - 2) / (count - 2);
  let previous = 0;

  for (let index = 0; index < count - 2; index += 1) {
    const start = 1 + Math.floor(index * bucket);
    const end = Math.min(1 + Math.floor((index + 1) * bucket), length - 1);
    const ahead =
      index < count - 3 ? Math.min(1 + Math.floor((index + 2) * bucket), length) : length;

    const mean = meanOf(x, y, end, ahead) ?? [x[length - 1], finite(y[length - 1])];
    const previousY = finite(y[previous]);

    let picked = start;
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

    kept.push(picked);
    previous = picked;
  }

  kept.push(length - 1);
  return kept;
}

/**
 * Bucketed extrema, the host's `extrema` over `count / 2` buckets: each bucket
 * contributes its lowest and highest finite sample in index order.
 */
export function minMaxIndices(y: readonly (number | null | undefined)[], count: number): number[] {
  const length = y.length;
  if (length === 0 || count <= 0) return [];
  const buckets = Math.max(1, Math.floor(count / 2));
  const span = Math.max(1, Math.ceil(length / buckets));
  const kept: number[] = [];

  for (let base = 0; base < length; base += span) {
    const limit = Math.min(base + span, length);
    let lowest = -1;
    let highest = -1;
    for (let index = base; index < limit; index += 1) {
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

function ascending(length: number): number[] {
  return Array.from({ length }, (_, index) => index);
}

function finite(value: number | null | undefined): number {
  return typeof value === "number" && Number.isFinite(value) ? value : 0;
}

function finiteMax(values: readonly number[]): number | null {
  let latest: number | null = null;
  for (const value of values) {
    if (!Number.isFinite(value)) continue;
    if (latest === null || value > latest) latest = value;
  }
  return latest;
}

/** The first index whose x is at or after `cutoff`; x is ascending apart from gaps. */
function firstAtOrAfter(x: readonly number[], cutoff: number): number {
  for (let index = 0; index < x.length; index += 1) {
    const value = x[index];
    if (Number.isFinite(value) && value >= cutoff) return index;
  }
  return x.length;
}

function meanOf(
  x: readonly number[],
  y: readonly (number | null | undefined)[],
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
