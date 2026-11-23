/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import type { EvidenceScope } from "@generated/host/model";

import { create } from "@bufbuild/protobuf";
import { validatedExpression } from "@/plots/expression";
import { plotType } from "@/lib/api/native-plot-projection";
import { jsonToNativeValue } from "@/lib/host/values";
import {
  AxisScale,
  FilterOperator,
  type PlotRequest,
  BarArrangement,
  ContourRender,
  Downsample,
  FftDetrend,
  FftWindow,
  FrequencyLevelUnit,
  LineStyle,
  PlotRequestSchema,
  Reduction,
  XyInterval,
} from "@generated/host/model";

import { NativeApiLimitError } from "@/lib/api/native-values";
import type { DocumentAxisScale, DocumentDownsampleTechnique, DocumentViewPlot } from "@/lib/types";
import { DEFAULT_PLOT_MAX_POINTS, NATIVE_PLOT_MAX_POINTS } from "@/plots/reduce";
import type { SummaryWindow } from "@/stores/summary-window-store";
import type { Wire } from "@/lib/rpc/canonical";

/* === Public Functions === */

export function plotRequestForDocumentPlot(
  plot: DocumentViewPlot,
  options: {
    scope: Wire<EvidenceScope> | null;
    windowS?: number | null;
    summaryWindow?: SummaryWindow | null;
    panelWidth?: number;
    panelHeight?: number;
  },
): PlotRequest {
  const declared = displayPointBudget(plot.display_points);
  const maximum = options.windowS == null && declared === 0 ? DEFAULT_PLOT_MAX_POINTS : declared;
  return create(PlotRequestSchema, {
    arrangement: plot.arrangement ? (plot.arrangement === "grouped" ? BarArrangement.GROUPED : BarArrangement.STACKED) : undefined,
    pareto: plot.pareto ?? false,
    hole: plot.hole ?? undefined,
    xyInterval: plot.xy_interval ? XyInterval.SIGMA : undefined,
    render: plot.render === "bands" ? ContourRender.BANDS : ContourRender.CELLS,
    reduction: plot.reduction
      ? { mean: Reduction.MEAN, min: Reduction.MIN, max: Reduction.MAX, last: Reduction.LAST, count: Reduction.COUNT }[plot.reduction]
      : undefined,
    cellExpression: plot.cell_expression ? validatedExpression(plot.cell_expression) : undefined,
    facetExpression: plot.facet_expression ? validatedExpression(plot.facet_expression) : undefined,
    grid: plot.grid
      ? {
          row: plot.grid.row ? validatedExpression(plot.grid.row) : undefined,
          column: plot.grid.column ? validatedExpression(plot.grid.column) : undefined,
          frame: plot.grid.frame ? validatedExpression(plot.grid.frame) : undefined,
          rows: plot.grid.rows.map((entry) => ({ ...entry, group: entry.group ?? undefined })),
          columns: plot.grid.columns.map((entry) => ({ ...entry, group: entry.group ?? undefined })),
          highlights: plot.grid.highlights.map((highlight) => ({
            row: validatedExpression(highlight.row),
            column: validatedExpression(highlight.column),
            label: highlight.label,
          })),
          summaryLabel: plot.grid.summaryLabel,
        }
      : undefined,
    panelHeight: options.panelHeight,

    ...(plot.plot_id == null ? {} : { plotId: plot.plot_id }),
    plotType: plotType(plot.plot_type),
    style: {
      line: LineStyle.LINE,
      points: LineStyle.POINTS,
      line_points: LineStyle.LINE_POINTS,
      step: LineStyle.STEP,
    }[plot.style ?? "line"],
    xScale: axisScale(plot.x_scale),
    yScale: axisScale(plot.y_scale),
    ...(plot.z_expression == null ? {} : { zExpression: validatedExpression(plot.z_expression) }),
    ...(plot.fft == null
      ? {}
      : {
          fft: {
            inputSamples: plot.fft.input_samples ?? undefined,
            sampleRateHz: plot.fft.sample_rate_hz ?? undefined,
            window: plot.fft.window === "rectangular" ? FftWindow.RECTANGULAR : FftWindow.HANN,
            detrend: plot.fft.detrend === "none" ? FftDetrend.NONE : FftDetrend.MEAN,
            levelUnit: {
              linear: FrequencyLevelUnit.LINEAR,
              dB: FrequencyLevelUnit.DB,
              dBu: FrequencyLevelUnit.DBU,
              dBm: FrequencyLevelUnit.DBM,
            }[plot.fft.level_unit],
            includeDc: plot.fft.include_dc,
            minFrequencyHz: plot.fft.min_frequency_hz ?? undefined,
            maxFrequencyHz: plot.fft.max_frequency_hz ?? undefined,
          },
        }),
    ...(plot.interval == null
      ? {}
      : {
          interval: {
            binWidth: plot.interval.bin_width,
            sigma: plot.interval.sigma,
            clipToObserved: plot.interval.clip_to_observed,
          },
        }),
    ...(plot.histogram == null
      ? {}
      : {
          histogram: {
            bins: boundedUint64(plot.histogram.bins, "histogram bins"),
            unit: plot.histogram.unit ?? undefined,
          },
        }),
    stream: plot.stream,
    yKeys: plot.y,
    markerRecordTypes: plot.marker_record_types ?? [],
    filters: (plot.filters ?? []).map(plotFilter),
    ...(plot.x == null ? {} : { xKey: plot.x }),
    ...(plot.series_by == null ? {} : { seriesBy: plot.series_by }),
    ...(plot.x_expression == null ? {} : { xExpression: validatedExpression(plot.x_expression) }),
    yExpressions: (plot.y_expressions ?? []).map(validatedExpression),
    ...(plot.series_by_expression == null ? {} : { seriesByExpression: validatedExpression(plot.series_by_expression) }),
    ...(plot.z_stream == null ? {} : { zStream: plot.z_stream }),
    ...(plot.z_field == null ? {} : { zField: plot.z_field }),
    ...(plot.unit == null ? {} : { unit: plot.unit }),
    ...(maximum == null ? {} : { maxPoints: boundedPlotPoints(maximum) }),
    ...(options.windowS == null ? {} : { windowS: boundedWindow(options.windowS) }),
    ...(options.summaryWindow == null ? {} : { summaryWindow: summaryWindow(options.summaryWindow) }),
    ...(options.summaryWindow == null ? {} : { viewport: summaryWindow(options.summaryWindow) }),
    ...(options.panelWidth == null ? {} : { panelWidth: boundedPanelWidth(options.panelWidth) }),
    scope: options.scope ?? undefined,
    technique: downsampleTechnique(plot.downsample),
  });
}

/** The request budget a plot's declared `display_points` resolves to. */
export function displayPointBudget(displayPoints: number | null | undefined): number {
  if (typeof displayPoints !== "number" || !Number.isFinite(displayPoints)) {
    return DEFAULT_PLOT_MAX_POINTS;
  }
  return Math.min(Math.max(0, Math.trunc(displayPoints)), NATIVE_PLOT_MAX_POINTS);
}

/* === Private Functions === */

function plotFilter(value: NonNullable<DocumentViewPlot["filters"]>[number]): Wire<PlotRequest>["filters"][number] {
  return {
    expression: validatedExpression(value.expression),
    operator: filterOperator(value.operator),
    ...(value.value === undefined ? {} : { value: jsonToNativeValue(value.value) }),
  };
}

function downsampleTechnique(value?: DocumentDownsampleTechnique | null): Downsample {
  return value === "raw" ? Downsample.RAW : value === "min_max" ? Downsample.MIN_MAX : value === "lttb" ? Downsample.LTTB : Downsample.AUTO;
}

function axisScale(value?: DocumentAxisScale | null): AxisScale {
  if (value === "log") {
    return AxisScale.LOG;
  }
  if (value === "linear" || value == null) {
    return AxisScale.LINEAR;
  }
  throw new Error(`Unsupported plot axis scale: ${String(value)}`);
}

function filterOperator(value: NonNullable<DocumentViewPlot["filters"]>[number]["operator"]): FilterOperator {
  const operator = {
    eq: FilterOperator.EQ,
    ne: FilterOperator.NE,
    gt: FilterOperator.GT,
    gte: FilterOperator.GE,
    lt: FilterOperator.LT,
    lte: FilterOperator.LE,
    contains: FilterOperator.CONTAINS,
    exists: FilterOperator.EXISTS,
  }[value];
  if (operator === undefined) {
    throw new Error(`Unsupported plot filter operator: ${String(value)}`);
  }
  return operator;
}

function boundedUint64(value: number, label: string): bigint {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new NativeApiLimitError(label, String(value));
  }
  return BigInt(value);
}

function boundedPanelWidth(value: number): number {
  if (!Number.isSafeInteger(value) || value < 1 || value > 0xffff_ffff) {
    throw new Error("Native plot panel_width must be a positive 32-bit integer.");
  }
  return value;
}

function boundedPlotPoints(value: number): bigint {
  if (!Number.isSafeInteger(value) || value < 0 || value > NATIVE_PLOT_MAX_POINTS) {
    throw new NativeApiLimitError("Plots request", `max_points must be an integer between 0 and ${NATIVE_PLOT_MAX_POINTS}`);
  }
  return BigInt(value);
}

function summaryWindow(window: { from: number; to: number }): { from: number; to: number } {
  if (!Number.isFinite(window.from) || !Number.isFinite(window.to) || window.from > window.to) {
    throw new Error("Native plot summary_window must be a finite range with from <= to.");
  }
  return { from: window.from, to: window.to };
}

function boundedWindow(value: number): number {
  if (!Number.isFinite(value) || value < 0) {
    throw new Error("Native plot window_s must be finite and non-negative.");
  }
  return value;
}
