/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { StatisticValue } from "@/lib/statistics";
import { formatNumber } from "@/lib/format";
import { plotColor } from "@/plots/colors";
import type { PlotCardSpec } from "@/plots/types";

/* === Types === */

export type CategoryTotal = { key: string; value: number; exact?: StatisticValue; order: number; color: string };
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
      entries.push({ key, value: point.y, exact: point.exact_y, order: entries.length, color: plotColor(entries.length) });
    }
  }
  return { entries };
}

/** Category Labels Use The Statistic, Never Its Drawing Coordinate. */
export function formatCategoryTotal(entry: { value: number; exact?: StatisticValue }, format: Parameters<typeof formatNumber>[1]): string {
  return entry.exact !== undefined && typeof entry.exact !== "number" ? entry.exact.decimal : formatNumber(entry.value, format);
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
