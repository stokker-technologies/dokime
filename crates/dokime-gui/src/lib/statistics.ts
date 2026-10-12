/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Types === */

export type StatsSummaryRow = {
  label: string;
  unit?: string;
  values: number[];
  statistics?: StatsSummary | null;
  count?: number;
};

export type StatsSummary = {
  mean?: number;
  median?: number;
  std?: number;
  rms?: number;
  min?: number;
  max?: number;
};

/* === Public Functions === */

export function describeValues(values: number[]): StatsSummary {
  if (!values.length) {
    return {};
  }
  const ordered = [...values].sort((left, right) => left - right);
  const mean = values.reduce((total, value) => total + value, 0) / values.length;
  const variance = values.reduce((total, value) => total + (value - mean) ** 2, 0) / values.length;
  const middle = Math.floor(ordered.length / 2);
  const median =
    ordered.length % 2
      ? ordered[middle]
      : ((ordered[middle - 1] ?? 0) + (ordered[middle] ?? 0)) / 2;

  return {
    mean,
    median,
    std: Math.sqrt(variance),
    rms: Math.sqrt(values.reduce((total, value) => total + value ** 2, 0) / values.length),
    min: ordered[0],
    max: ordered[ordered.length - 1],
  };
}

export function formatStatValue(value: number | undefined, unit?: string): string {
  if (value === undefined || !Number.isFinite(value)) {
    return "-";
  }
  const precision = Math.abs(value) >= 100 ? 0 : 2;
  return unit ? `${value.toFixed(precision)} ${unit}` : value.toFixed(precision);
}
