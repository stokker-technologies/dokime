/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { plotColor } from "@/plots/colors";
import type { PlotCardSpec } from "@/plots/types";

/* === Types === */

export type CategoryTotal = { key: string; value: number; order: number; color: string };
export type CategoryTotals = { entries: CategoryTotal[]; error?: undefined } | { entries?: undefined; error: string };

/* === Constants === */

/* === Public Functions === */

export function categoryTotals(plot: PlotCardSpec): CategoryTotals {
  const entries: CategoryTotal[] = [];
  const keys = new Set<string>();
  for (const series of plot.series) {
    for (const point of series.points) {
      if (!["string", "number", "boolean"].includes(typeof point.x) || (typeof point.x === "number" && !Number.isFinite(point.x))) {
        return { error: "Supply a category for every total." };
      }
      const key = String(point.x);
      if (keys.has(key)) return { error: `Duplicate category total ${key}; aggregate samples explicitly.` };
      if (typeof point.y !== "number" || !Number.isFinite(point.y) || point.y < 0)
        return { error: `Category ${key} requires a finite nonnegative total.` };
      keys.add(key);
      entries.push({ key, value: point.y, order: entries.length, color: plotColor(entries.length) });
    }
  }
  return { entries };
}

/** Normalize Before Summing So Finite Magnitudes Cannot Overflow the Total. */
export function categoryFractions(entries: readonly CategoryTotal[]): number[] {
  let maximum = 0;
  for (const entry of entries) maximum = Math.max(maximum, entry.value);
  if (maximum === 0) return entries.map(() => 0);
  const total = entries.reduce((sum, entry) => sum + entry.value / maximum, 0);
  return entries.map((entry) => entry.value / maximum / total);
}

/* === Private Functions === */
