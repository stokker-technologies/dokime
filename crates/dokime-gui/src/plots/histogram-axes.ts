/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * The axes a histogram draws.
 *
 * A histogram declares the value it bins on x; the host counts that value into
 * bins, so y is always the count. The x title, unit and format come from the
 * declaration and the column it names. This module has no application imports
 * so a node test can load it directly.
 */

/* === Local Imports === */

import type { DocumentViewPlot, TelemetryColumnSpec, ValueFormat } from "@/lib/types";

/* === Constants === */

/** The y axis title of every histogram: the host counts the binned value. */
export const HISTOGRAM_COUNT_LABEL = "Count";

/* === Types === */

export type HistogramValueAxis = {
  label: string;
  unit: string;
  format: ValueFormat | null;
};

export type HistogramAxes = {
  x: HistogramValueAxis;
  y: { label: string };
};

export type HistogramDeclaration = Pick<
  DocumentViewPlot,
  "title" | "x" | "unit" | "histogram" | "x_format"
>;

export type HistogramColumn = Pick<TelemetryColumnSpec, "label" | "unit" | "format">;

/* === Public Functions === */

/** The histogram's axes: the declared value on x, the host's count on y. */
export function histogramAxes(
  plot: HistogramDeclaration,
  column?: HistogramColumn | null,
): HistogramAxes {
  const field = plot.x && plot.x !== "timestamp" ? plot.x : null;
  return {
    x: {
      label: column?.label?.trim() || field || plot.title,
      unit: plot.histogram?.unit ?? plot.unit ?? column?.unit ?? "",
      format: plot.x_format ?? column?.format ?? null,
    },
    y: { label: HISTOGRAM_COUNT_LABEL },
  };
}

/** An axis title with its unit, `Voltage (V)`, or the bare label without one. */
export function histogramAxisTitle(axis: { label: string; unit?: string | null }): string {
  return axis.unit ? `${axis.label} (${axis.unit})` : axis.label;
}
