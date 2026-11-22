/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Native Response Projection
 */

/* === Imports === */

import { nativeValueToJson } from "@/lib/host/values";

import { required, type Wire } from "@/lib/rpc/canonical";

import type { StatisticValue } from "@/lib/statistics";

import type {
  DocumentPlotType,
  PlotAnalysis,
  PlotCartesianFrame,
  PlotMarker,
  PlotPayload,
  PlotPayloadEnvelope,
  PlotSeries,
} from "@/lib/types";

import * as hostModel from "@generated/host/model";

import { PlotKind } from "@generated/host/model";

import { safeNumber } from "@/lib/api/native-values";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type PlotRequest = Wire<hostModel.PlotRequest>;

export type PlotEnvelope = Wire<hostModel.SnapshotsListResponse>["envelopes"][number];

type NativePlotPayload = NonNullable<PlotEnvelope["payload"]>;

type NativePlotAnalysis = NonNullable<NativePlotPayload["analysis"]>;

type NativePlotFrame = NonNullable<NativePlotPayload["frame"]>;

type NativePlotMarker = NativePlotPayload["markers"][number];

type NativePlotSeries = NativePlotPayload["series"][number];

type PlotType = NonNullable<PlotRequest["plotType"]>;

type PlotCartesianMode = NonNullable<NativePlotFrame["mode"]>;

/** Renderer envelope for one storage query. */

/* === Classes === */

/* === Public Functions === */

/** Map one protobuf envelope into the payload family used by the renderer. */
export function mapNativePlotEnvelope(envelope: PlotEnvelope, plotType?: DocumentPlotType | null): PlotPayloadEnvelope {
  if (!envelope.payload) {
    throw new Error("Read Plot Envelope: Restore Its Payload");
  }
  if (!envelope.diagnostics) {
    throw new Error("Read Plot Envelope: Restore Its Diagnostics");
  }
  return {
    plot_id: envelope.plotId,
    revision: safeNumber(envelope.revision, "Plots envelope revision"),
    epoch: requiredText(envelope.epoch, "plot runtime epoch"),
    selection: requiredText(envelope.selection, "plot selection"),
    payload: mapNativePlotPayload(envelope.payload, plotType),
    diagnostics: envelope.diagnostics,
    warnings: [...envelope.warnings],
  };
}

export function plotType(value?: DocumentPlotType | null): PlotType {
  const kinds: Record<DocumentPlotType, PlotKind> = {
    time_series: PlotKind.TIME_SERIES,
    bar: PlotKind.BAR,
    xy: PlotKind.XY,
    trajectory: PlotKind.TRAJECTORY,
    contour: PlotKind.CONTOUR,
    grid_chart: PlotKind.GRID_CHART,
    heat_strip: PlotKind.HEAT_STRIP,
    frequency_series: PlotKind.FREQUENCY_SERIES,
    histogram: PlotKind.HISTOGRAM,
    box_plot: PlotKind.BOX_PLOT,
    pie: PlotKind.PIE,
  };
  if (value == null) {
    return PlotKind.UNSPECIFIED;
  }
  const kind = kinds[value];
  if (kind === undefined) {
    throw new Error(`Unsupported document plot type: ${String(value)}`);
  }
  return kind;
}

export function mapNativePlotPayload(payload: NativePlotPayload, plotType?: DocumentPlotType | null): PlotPayload {
  const points = plotType === "trajectory" || plotType === "bar" || plotType === "pie" || plotType === "heat_strip";
  return {
    stream: payload.stream,
    x_key: payload.xKey,
    series_by: payload.seriesBy ?? null,
    context: payload.context ? { value: nativeValueToJson(payload.context) } : {},
    series: payload.series.map((series, index) => {
      const metadata = mapNativePlotSeries(series);
      if (payload.frame?.mode && !points) {
        return metadata;
      }
      const column = payload.frame?.series[index];
      const timed = plotType === "trajectory" || plotType === "heat_strip";
      if (column && (column.x.length !== column.y.length || (timed && column.instants.length !== column.y.length))) {
        throw new Error("Read Plot Columns: Restore Matching Coordinates and Source Instants");
      }
      return {
        ...metadata,
        points: (column?.x ?? []).map((x, row) => ({
          x: plotType === "bar" || plotType === "pie" ? categoryValue(x) : (nativeValueToJson(x) ?? null),
          y: column?.y[row]?.value ?? null,
          ...(column?.totals[row] ? { exact_y: statisticValue(column.totals[row]) } : {}),
          z: column?.z[row]?.value ?? null,
          instant: column?.instants[row] ?? null,
        })),
      };
    }),
    analysis: mapNativePlotAnalysis(payload.analysis),
    cartesian: points ? null : mapNativeCartesianFrame(payload.frame),
    markers: payload.markers.map(mapNativePlotMarker),
    max_points: safeNumber(payload.maxPoints, "Plots payload max_points"),
    window_s: payload.windowS ?? null,
    generated_at: payload.generatedAt,
    x_format: payload.xFormat ?? null,
    run_started_at: payload.runStartedAt ?? null,
    cycle_started_at: payload.cycleStartedAt ?? null,
    bucket_width_s: payload.bucketWidthS ?? null,
    bucket_window: payload.bucketWindow ? { from: payload.bucketWindow.from, to: payload.bucketWindow.to } : null,
    replaced_buckets: payload.replacedBuckets,
    summaries: payload.summaries.map((summary) => ({
      series_id: summary.seriesId,
      count: safeNumber(summary.count, "plot summary count"),
      min: statisticValue(summary.min) ?? null,
      max: statisticValue(summary.max) ?? null,
      mean: statisticValue(summary.mean) ?? null,
      median: statisticValue(summary.median) ?? null,
      std: summary.std ?? null,
      rms: summary.rms ?? null,
      window: summary.window ? { from: summary.window.from, to: summary.window.to } : null,
    })),
  };
}

export function requiredText(value: unknown, label: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`Native ${label} is required.`);
  }
  return value;
}

/* === Private Functions === */

function statisticValue(value?: NativePlotFrame["series"][number]["x"][number]): StatisticValue | undefined {
  if (!value || value.kind.case === "null") {
    return undefined;
  }
  if (value.kind.case === "number") {
    return value.kind.value;
  }
  if (["integer", "unsignedInteger", "exact"].includes(value.kind.case ?? "")) {
    return nativeValueToJson(value) as StatisticValue;
  }
  throw new Error("Read Statistic: Restore Its Numeric or Exact Value");
}

function categoryValue(value: NativePlotFrame["series"][number]["x"][number]): string | number | boolean | null {
  const kind = value.kind;
  switch (kind.case) {
    case "integer":
    case "unsignedInteger":
      return kind.value.toString();
    case "number":
    case "text":
    case "boolean":
      return kind.value;
    default:
      return null;
  }
}

function mapNativePlotAnalysis(analysis?: NativePlotAnalysis): PlotAnalysis | null {
  const result = analysis?.result?.kind;
  if (!result?.case) {
    return null;
  }
  if (!result.value) {
    throw new Error("Native plot analysis is missing its result.");
  }

  switch (result.case) {
    case "error":
      if (!result.value) {
        throw new Error("Native plot analysis is missing its result.");
      }
      return { kind: "error", message: result.value.message };
    case "histogram":
      if (!result.value) {
        throw new Error("Native plot analysis is missing its result.");
      }
      return {
        kind: "histogram",
        series: result.value.series.map((series) => ({
          key: series.key,
          count: safeNumber(series.count, "histogram count"),
          ...(series.statistics
            ? {
                statistics: {
                  mean: statisticValue(series.statistics.mean),
                  median: statisticValue(series.statistics.median),
                  std: series.statistics.std,
                  rms: series.statistics.rms,
                  min: statisticValue(series.statistics.min),
                  max: statisticValue(series.statistics.max),
                },
              }
            : {}),
          bins: series.bins.map((bin) => ({
            lower: bin.lower,
            upper: bin.upper,
            count: safeNumber(bin.count, "histogram bin count"),
          })),
        })),
      };
    case "boxPlot":
      if (!result.value) {
        throw new Error("Native plot analysis is missing its result.");
      }
      return {
        kind: "box_plot",
        series: result.value.series.map((series) => ({
          key: series.key,
          label: series.label,
          count: safeNumber(series.count, "box summary count"),
          min: required(statisticValue(series.min), "Read Box Summary: Restore min"),
          q1: required(statisticValue(series.q1), "Read Box Summary: Restore q1"),
          median: required(statisticValue(series.median), "Read Box Summary: Restore median"),
          q3: required(statisticValue(series.q3), "Read Box Summary: Restore q3"),
          max: required(statisticValue(series.max), "Read Box Summary: Restore max"),
          lower_whisker: series.lowerWhisker,
          upper_whisker: series.upperWhisker,
          outliers: [...series.outliers],
        })),
        total: safeNumber(result.value.total, "box summary total"),
      };
    case "sigmaInterval":
      if (!result.value) {
        throw new Error("Native plot analysis is missing its result.");
      }
      return {
        kind: "sigma_interval",
        series: result.value.series.map((series) => ({
          key: series.key,
          bins: series.bins.map((bin) => ({
            x: bin.x,
            count: safeNumber(bin.count, "interval bin count"),
            mean: bin.mean,
            lower: bin.lower,
            upper: bin.upper,
          })),
          raw_points: safeNumber(series.rawPoints, "interval raw_points"),
        })),
      };
    case "contour":
      return { kind: "contour", grid: result.value };
    case "gridChart":
      return { kind: "grid_chart", data: result.value };
  }
}

function mapNativePlotSeries(series: NativePlotSeries): PlotSeries {
  return {
    key: series.key,
    label: series.label,
    unit: series.unit ?? null,
    points: [],
    raw_points: safeNumber(series.rawPoints, "Plots series raw_points"),
    downsampled: series.downsampled,
    format: series.format ?? null,
  };
}

function mapNativePlotMarker(marker: NativePlotMarker): PlotMarker {
  return {
    x: nativeValueToJson(marker.x) ?? null,
    label: marker.label,
    record_type: marker.recordType,
    color: marker.color,
  };
}

function mapNativeCartesianFrame(frame?: NativePlotFrame): PlotCartesianFrame | null {
  if (!frame?.mode) {
    return null;
  }
  const mode = cartesianMode(frame.mode);
  let valid = true;
  const series = frame.series.map((column) => {
    const x = mode === "aligned" ? frame.time : column.x.map((value) => Number(nativeValueToJson(value)));
    const y: (number | null)[] = [];
    if (x.length !== column.y.length) {
      valid = false;
    }
    for (let row = 0; row < column.y.length; row += 1) {
      y.push(column.y[row].value ?? null);
      if (!Number.isFinite(x[row])) {
        valid = false;
      }
    }
    return { x, y };
  });
  if (!valid) {
    return null;
  }
  return {
    mode,
    series,
    x_domain: nativeDomain(frame.xDomain),
    y_domain: nativeDomain(frame.yDomain),
    point_count: safeNumber(frame.pointCount, "Plots Cartesian point_count"),
    signature: frame.signature,
  };
}

function nativeDomain(value?: { min: number; max: number }): { min: number; max: number } | null {
  if (!value || !Number.isFinite(value.min) || !Number.isFinite(value.max)) {
    return null;
  }
  return value;
}

function cartesianMode(value: PlotCartesianMode | undefined): PlotCartesianFrame["mode"] {
  switch (value) {
    case hostModel.PlotCartesianMode.ALIGNED:
      return "aligned";
    case hostModel.PlotCartesianMode.FACETED:
      return "faceted";
    default:
      throw new Error("Native Cartesian frame returned an unspecified mode.");
  }
}

/* === Entry Point === */
