// Dokime — SPDX-License-Identifier: Apache-2.0
import { scaleLinear } from "d3-scale";
import type { StatisticValue } from "../../lib/statistics";
import { categoryFractions, type categoryTotals } from "../categories";
import type { PlotCardSpec } from "../types";

type BarReading = {
  key: string;
  color?: string;
  label: string;
  value: number;
  exact?: StatisticValue;
};

export function arrangedBars(plot: PlotCardSpec) {
  const categories = new Map<string, Map<number, number>>();
  const exact = new Map<string, StatisticValue>();
  plot.series.forEach((series, seriesIndex) => {
    for (const point of series.points) {
      if (typeof point.y !== "number" || !Number.isFinite(point.y)) {
        continue;
      }
      const key = String(point.x);
      if (!categories.has(key)) {
        categories.set(key, new Map());
      }
      categories.get(key)?.set(seriesIndex, point.y);
      if (point.exact_y !== undefined) {
        exact.set(JSON.stringify([key, seriesIndex]), point.exact_y);
      }
    }
  });
  if (!categories.size) {
    return null;
  }
  const stacked = plot.arrangement === "stacked";
  let minimum = 0;
  let maximum = 0;
  for (const values of categories.values()) {
    let negative = 0;
    let positive = 0;
    for (const value of values.values()) {
      if (stacked) {
        negative += Math.min(0, value);
        positive += Math.max(0, value);
      } else {
        minimum = Math.min(minimum, value);
        maximum = Math.max(maximum, value);
      }
    }
    if (stacked) {
      minimum = Math.min(minimum, negative);
      maximum = Math.max(maximum, positive);
    }
  }
  const x = scaleLinear([minimum, maximum === minimum ? minimum + 1 : maximum], [150, 650]);
  const rowHeight = stacked ? 36 : Math.max(36, plot.series.length * 24);
  const height = 44 + categories.size * rowHeight + 32;
  const rows = [...categories].map(([category, values], rowIndex) => {
    let positive = 0;
    let negative = 0;
    const bars = [...values].map(([seriesIndex, value]) => {
      const start = stacked ? (value < 0 ? negative : positive) : 0;
      const end = start + value;
      if (stacked) {
        if (value < 0) {
          negative = end;
        } else {
          positive = end;
        }
      }
      return {
        seriesIndex,
        value,
        exact: exact.get(JSON.stringify([category, seriesIndex])),
        start,
        end,
        top: 20 + rowIndex * rowHeight + (stacked ? 0 : seriesIndex * 24),
      };
    });
    return { category, rowIndex, bars };
  });
  return { rows, x, height, rowHeight };
}

export function paretoBars(entries: NonNullable<ReturnType<typeof categoryTotals>["entries"]>) {
  const fractions = categoryFractions(entries);
  const maximum = entries[0].value;
  const valueAxis = scaleLinear([0, maximum || 1], [280, 20]);
  let cumulative = 0;
  const bars = entries.map((entry, index) => {
    cumulative += fractions[index] * 100;
    return {
      ...entry,
      x: 50 + ((index + 0.5) * 600) / entries.length,
      height: maximum > 0 ? (entry.value / maximum) * 260 : 0,
      cumulative: Math.min(100, cumulative),
    };
  });
  return { bars, maximum, valueAxis };
}

export function latestBars(plot: PlotCardSpec): BarReading[] {
  return plot.series
    .flatMap((series, index) => {
      const values = new Map<string, { value: number; exact?: StatisticValue }>();
      series.points.forEach((point) => {
        if (Number.isFinite(point.y)) {
          values.set(String(point.x), { value: Number(point.y), exact: point.exact_y });
        }
      });

      return [...values.entries()].map(([category, value]) => ({
        key: JSON.stringify([series.key, category]),
        label: plot.series.length > 1 ? `${series.label} / ${category}` : category,
        ...value,
        color: plot.colors[index] ?? `var(--chart-${(index % 8) + 1})`,
      }));
    })
    .sort(compareBarLabels);
}

function compareBarLabels(left: BarReading, right: BarReading): number {
  const leftNumber = Number(left.label);
  const rightNumber = Number(right.label);
  if (Number.isFinite(leftNumber) && Number.isFinite(rightNumber)) {
    return leftNumber - rightNumber;
  }
  return left.label.localeCompare(right.label, undefined, { numeric: true });
}
