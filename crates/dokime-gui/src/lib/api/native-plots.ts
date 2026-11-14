/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Typed native plot and telemetry adapters for the original routed UI. */

/* === External Imports === */

import { create } from "@bufbuild/protobuf";

/* === Local Imports === */

import * as protocolV1Plots from "@host/dokime/v1/plots_pb.js";
import * as protocolClientV1PlotsSnapshots from "@host/dokime/client/v1/plots/snapshots_pb.js";
import * as protocolV1Expressions from "@host/dokime/v1/expressions_pb.js";
import { cutKey, required, type Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/host/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type {
  DocumentAxisScale,
  DocumentDataExpression,
  DocumentDownsampleTechnique,
  DocumentFftOptions,
  DocumentHistogramOptions,
  DocumentPlotType,
  DocumentSigmaInterval,
  DocumentViewPlot,
  EvidenceAnalysisDefinition,
  EvidenceAnalysisResult,
  EvidenceAnalysisSelection,
  PlotAnalysis,
  PlotCartesianFrame,
  PlotMarker,
  PlotPayload,
  PlotPayloadDiagnostics,
  PlotPayloadEnvelope,
  PlotPayloadStreamMessage,
  PlotSeries,
  RecordScope,
} from "@/lib/types";
import { NATIVE_PLOT_MAX_POINTS } from "@/plots/reduce";
import { mapParameter } from "@/lib/host/suites";
import {
  jsonToNativeValue,
  mapNativeValueFormat,
  nativeEvidenceScope,
  nativeValuesToJson,
  nativeValueToJson,
  recordScopeFromNative,
} from "@/lib/host/values";
import { NativeApiLimitError, NATIVE_MAX_PAGES, safeNumber } from "./native-values";

/* === Canonical Types === */

type PlotRequest = Wire<protocolV1Plots.PlotRequest>;
type PlotEnvelope = Wire<Awaited<ReturnType<typeof rpc.plotsSnapshotsList>>>["envelopes"][number];
type DataExpression = NonNullable<PlotRequest["xExpression"]>;
type NativePlotPayload = NonNullable<PlotEnvelope["payload"]>;
type NativeAnalysisComputation = NonNullable<NativePlotPayload["computed"]>;
type NativeAnalysisDefinition = NonNullable<NativeAnalysisComputation["definition"]>;
type NativePlotAnalysis = NonNullable<NativePlotPayload["analysis"]>;
type NativePlotFrame = NonNullable<NativePlotPayload["frame"]>;
type NativePlotMarker = NativePlotPayload["markers"][number];
type NativePlotPayloadDiagnostics = NonNullable<PlotEnvelope["diagnostics"]>;
type NativePlotSeries = NativePlotPayload["series"][number];

type PlotType = NonNullable<PlotRequest["plotType"]>;
type AxisScale = NonNullable<PlotRequest["xScale"]>;
type DownsampleTechnique = NonNullable<PlotRequest["technique"]>;
type FilterOperator = NonNullable<PlotRequest["filters"][number]["operator"]>;
type PlotCartesianMode = NonNullable<NativePlotFrame["mode"]>;

const PlotRequestSchema = protocolV1Plots.PlotRequestSchema;
type PlotWatchPage = Wire<protocolClientV1PlotsSnapshots.SnapshotsWatchResponse>;
type PlotStreamUpdate = PlotWatchPage["updates"][number];
type PlotStreamKind = PlotStreamUpdate["kind"];

/* === Constants === */

const GUI_PLOT_REQUESTS = 32;
const GUI_PLOT_TRANSFORMS = 32;

export const NATIVE_PLOT_MAX_REQUESTS = GUI_PLOT_REQUESTS;
const MAX_PLOT_TRANSFORMS = GUI_PLOT_TRANSFORMS;

/* === Types === */

/** Input shape retained for the original plot renderer and document model. */
export type PlotPayloadRequestInput = {
  plotId?: string | null;
  stream: string;
  yKeys: string[];
  markerRecordTypes?: string[];
  filters?: DocumentViewPlot["filters"];
  xKey?: string | null;
  plotType?: DocumentPlotType | null;
  arrangement?: "grouped" | "stacked" | null;
  pareto?: boolean;
  hole?: number | null;
  xyInterval?: "sigma" | null;
  reduction?: "mean" | "min" | "max" | null;
  render?: "cells" | "bands";
  grid?: DocumentViewPlot["grid"];
  panelHeight?: number;
  yViewport?: { from: number; to: number };
  style?: DocumentViewPlot["style"];
  xScale?: DocumentAxisScale | null;
  yScale?: DocumentAxisScale | null;
  fft?: DocumentFftOptions;
  analysis?: EvidenceAnalysisSelection | null;
  interval?: DocumentSigmaInterval;
  histogram?: DocumentHistogramOptions;
  zExpression?: DocumentDataExpression | null;
  seriesBy?: string | null;
  xExpression?: DocumentDataExpression | null;
  yExpressions?: DocumentDataExpression[];
  seriesByExpression?: DocumentDataExpression | null;
  zStream?: string | null;
  zField?: string | null;
  unit?: string | null;
  maxPoints?: number;
  /** Absent is the host default (`auto`). */
  technique?: DocumentDownsampleTechnique | null;
  windowS?: number | null;
  /** The x range a zoomed time series or scatter plot summarizes. */
  summaryWindow?: { from: number; to: number } | null;
  viewport?: { from: number; to: number } | null;
  panelWidth?: number;
  scope?: RecordScope | null;
};

/** Renderer envelope for one storage query. */
export type NativePlotPayloadEnvelope = PlotPayloadEnvelope;

/**
 * The watch's word that a plot is subscribed and still reading its first
 * frame: sent at once when the watch opens and as a keepalive while the watch
 * is quiet, even after the plot's data arrived. It carries no data and no
 * revision.
 */
export type NativePlotPendingMessage = {
  kind: "pending";
  plot_id: string;
};

/** One Plot Within An Ordered Watch Revision. */
export type NativePlotStreamUpdate = PlotPayloadStreamMessage | NativePlotPendingMessage;

/** One Complete Dirty Set, Or Payload-Free Keepalive Notices. */
export type NativePlotStreamMessage = { updates: NativePlotStreamUpdate[] };

export interface NativePlotStream extends AsyncIterable<NativePlotStreamMessage> {
  cancel(): Promise<void>;
}

export type NativePlotReadOptions = {
  signal?: AbortSignal;
};

export type NativePlotWatchOptions = NativePlotReadOptions;

/* === Public API === */

/** Build one typed protobuf plot request from the document-facing input. */
export function toNativePlotRequest(input: PlotPayloadRequestInput): PlotRequest {
  const nativeScope = nativeEvidenceScope(input.scope);
  return create(PlotRequestSchema, {
    arrangement: input.arrangement
      ? input.arrangement === "grouped"
        ? protocolV1Plots.BarArrangement.GROUPED
        : protocolV1Plots.BarArrangement.STACKED
      : undefined,
    pareto: input.pareto ?? false,
    hole: input.hole ?? undefined,
    xyInterval: input.xyInterval ? protocolV1Plots.XyInterval.SIGMA : undefined,
    render: input.render === "bands" ? protocolV1Plots.ContourRender.BANDS : protocolV1Plots.ContourRender.CELLS,
    reduction: input.reduction
      ? { mean: protocolV1Plots.Reduction.MEAN, min: protocolV1Plots.Reduction.MIN, max: protocolV1Plots.Reduction.MAX }[input.reduction]
      : undefined,
    grid: input.grid
      ? {
          row: input.grid.row ? dataExpression(input.grid.row) : undefined,
          column: input.grid.column ? dataExpression(input.grid.column) : undefined,
          frame: input.grid.frame ? dataExpression(input.grid.frame) : undefined,
          rows: input.grid.rows.map((entry) => ({ ...entry, group: entry.group ?? undefined })),
          columns: input.grid.columns.map((entry) => ({ ...entry, group: entry.group ?? undefined })),
          highlights: input.grid.highlights.map((highlight) => ({
            row: dataExpression(highlight.row),
            column: dataExpression(highlight.column),
            label: highlight.label,
          })),
          summaryLabel: input.grid.summary_label,
        }
      : undefined,
    panelHeight: input.panelHeight,
    yViewport: input.yViewport,

    ...(input.plotId == null ? {} : { plotId: input.plotId }),
    analysis: input.analysis
      ? {
          id: input.analysis.id,
          version: input.analysis.version,
          parameters: {
            values: Object.fromEntries(Object.entries(input.analysis.parameters).map(([key, value]) => [key, jsonToNativeValue(value)])),
          },
        }
      : undefined,
    plotType: plotType(input.plotType),
    style: {
      line: protocolV1Plots.LineStyle.LINE,
      points: protocolV1Plots.LineStyle.POINTS,
      line_points: protocolV1Plots.LineStyle.LINE_POINTS,
      step: protocolV1Plots.LineStyle.STEP,
    }[input.style ?? "line"],
    xScale: axisScale(input.xScale),
    yScale: axisScale(input.yScale),
    ...(input.zExpression == null ? {} : { zExpression: dataExpression(input.zExpression) }),
    ...(input.fft == null
      ? {}
      : {
          fft: {
            inputSamples: input.fft.input_samples ?? undefined,
            sampleRateHz: input.fft.sample_rate_hz ?? undefined,
            window: input.fft.window === "rectangular" ? protocolV1Plots.FftWindow.RECTANGULAR : protocolV1Plots.FftWindow.HANN,
            detrend: input.fft.detrend === "none" ? protocolV1Plots.FftDetrend.NONE : protocolV1Plots.FftDetrend.MEAN,
            levelUnit: {
              linear: protocolV1Plots.FrequencyLevelUnit.LINEAR,
              dB: protocolV1Plots.FrequencyLevelUnit.DB,
              dBu: protocolV1Plots.FrequencyLevelUnit.DBU,
              dBm: protocolV1Plots.FrequencyLevelUnit.DBM,
            }[input.fft.level_unit],
            includeDc: input.fft.include_dc,
            minFrequencyHz: input.fft.min_frequency_hz ?? undefined,
            maxFrequencyHz: input.fft.max_frequency_hz ?? undefined,
          },
        }),
    ...(input.interval == null
      ? {}
      : {
          interval: {
            binWidth: input.interval.bin_width,
            sigma: input.interval.sigma,
            clipToObserved: input.interval.clip_to_observed,
          },
        }),
    ...(input.histogram == null
      ? {}
      : {
          histogram: {
            bins: boundedUint64(input.histogram.bins, "histogram bins"),
            unit: input.histogram.unit ?? undefined,
          },
        }),
    stream: requiredText(input.stream, "plot stream"),
    yKeys: input.yKeys,
    markerRecordTypes: input.markerRecordTypes ?? [],
    filters: (input.filters ?? []).map(plotFilter),
    ...(input.xKey == null ? {} : { xKey: input.xKey }),
    ...(input.seriesBy == null ? {} : { seriesBy: input.seriesBy }),
    ...(input.xExpression == null ? {} : { xExpression: dataExpression(input.xExpression) }),
    yExpressions: (input.yExpressions ?? []).map(dataExpression),
    ...(input.seriesByExpression == null ? {} : { seriesByExpression: dataExpression(input.seriesByExpression) }),
    ...(input.zStream == null ? {} : { zStream: input.zStream }),
    ...(input.zField == null ? {} : { zField: input.zField }),
    ...(input.unit == null ? {} : { unit: input.unit }),
    ...(input.maxPoints == null ? {} : { maxPoints: boundedPlotPoints(input.maxPoints) }),
    ...(input.windowS == null ? {} : { windowS: boundedWindow(input.windowS) }),
    ...(input.summaryWindow == null ? {} : { summaryWindow: summaryWindow(input.summaryWindow) }),
    ...(input.viewport == null ? {} : { viewport: summaryWindow(input.viewport) }),
    ...(input.panelWidth == null ? {} : { panelWidth: boundedPanelWidth(input.panelWidth) }),
    ...(nativeScope ? { scope: nativeScope } : {}),
    technique: downsampleTechnique(input.technique),
  });
}

/** Read one finite batch without subscribing to subsequent runtime changes. */
export async function fetchNativePlotPayloads(
  session: SessionIdentity,
  requests: readonly PlotPayloadRequestInput[],
  options: NativePlotReadOptions = {},
): Promise<NativePlotPayloadEnvelope[]> {
  boundedRequestCount(requests.length, "Plots.Snapshots.List");
  ensureActive(session, options.signal);
  try {
    const envelopes: NativePlotPayloadEnvelope[] = [];
    const nativeRequests = requests.map(toNativePlotRequest);
    let at: Wire<Awaited<ReturnType<typeof rpc.plotsSnapshotsList>>>["resolvedAt"];
    let token = "";
    const seen = new Set<string>();
    for (let page = 0; page < NATIVE_MAX_PAGES; page += 1) {
      const response = await rpc.plotsSnapshotsList(session, {
        requests: nativeRequests,
        at,
        page: { pageSize: NATIVE_PLOT_MAX_REQUESTS, pageToken: token },
      });
      ensureActive(session, options.signal);
      const resolved = required(response.resolvedAt, "Read Plot Snapshots: Missing Read Cut");
      if (at && cutKey(at) !== cutKey(resolved)) throw new Error("Read Plot Snapshots: Read Cut Changed");
      at = resolved;
      envelopes.push(...response.envelopes.map(mapNativePlotEnvelope));
      token = response.nextPageToken;
      if (!token) return envelopes;
      if (seen.has(token)) throw new Error("Read Plot Snapshots: Repeated Continuation");
      seen.add(token);
    }
    throw new NativeApiLimitError("Read Plot Snapshots", "page budget exhausted");
  } catch (error) {
    throw normalizeCancellation(error, session, options.signal);
  }
}

/**
 * Open the subscribed plot watch. Each request receives the server's
 * initial snapshots after subscription, so callers cannot miss a racing
 * evidence commit between a separate snapshot and watch setup.
 */
export async function openNativePlotStream(
  session: SessionIdentity,
  requests: readonly PlotPayloadRequestInput[],
  options: NativePlotWatchOptions = {},
): Promise<NativePlotStream> {
  boundedRequestCount(requests.length, "Plots.Watch");
  ensureActive(session, options.signal);
  const nativeRequests = requests.map((request) => toNativePlotRequest(request));

  let source: Awaited<ReturnType<typeof rpc.plotsSnapshotsWatch>> | null = null;
  try {
    source = await rpc.plotsSnapshotsWatch(session, { requests: nativeRequests });
    ensureActive(session, options.signal);
  } catch (error) {
    await source?.cancel().catch(() => undefined);
    throw normalizeCancellation(error, session, options.signal);
  }
  if (!source) throw new Error("Native plot watch returned no stream.");

  let ended = false;
  const iterator = source[Symbol.asyncIterator]();
  let removeAbortListener: (() => void) | undefined;
  const cancel = async (): Promise<void> => {
    if (ended) return;
    ended = true;
    removeAbortListener?.();
    removeAbortListener = undefined;
    await source.cancel();
  };
  const onAbort = () => {
    void cancel().catch(() => undefined);
  };
  if (options.signal) {
    options.signal.addEventListener("abort", onAbort, { once: true });
    removeAbortListener = () => options.signal?.removeEventListener("abort", onAbort);
  }
  if (options.signal?.aborted || !session.isCurrent()) {
    await cancel().catch(() => undefined);
    throw abortError();
  }

  const mapped: AsyncIterator<NativePlotStreamMessage> = {
    async next() {
      if (ended) return { done: true, value: undefined };
      try {
        ensureActive(session, options.signal);
        const result = await iterator.next();
        if (result.done) {
          await cancel();
          return { done: true, value: undefined };
        }
        ensureActive(session, options.signal);
        return { done: false, value: mapNativePlotWatchPage(result.value) };
      } catch (error) {
        await cancel().catch(() => undefined);
        throw normalizeCancellation(error, session, options.signal);
      }
    },
    async return() {
      await cancel();
      return { done: true, value: undefined };
    },
  };

  return {
    cancel,
    [Symbol.asyncIterator]: () => mapped,
  };
}

/** Map one protobuf envelope into the payload family used by the renderer. */
export function mapNativePlotEnvelope(envelope: PlotEnvelope): NativePlotPayloadEnvelope {
  if (!envelope.payload) throw new Error("Native plot envelope has no payload.");
  if (!envelope.diagnostics) throw new Error("Native plot envelope has no diagnostics.");
  return {
    plot_id: envelope.plotId,
    revision: safeNumber(envelope.revision, "Plots envelope revision"),
    epoch: requiredText(envelope.epoch, "plot runtime epoch"),
    selection: requiredText(envelope.selection, "plot selection"),
    payload: mapNativePlotPayload(envelope.payload),
    diagnostics: mapNativePlotDiagnostics(envelope.diagnostics),
    warnings: [...envelope.warnings],
  };
}

/**
 * Map one protobuf storage-query snapshot into the renderer.
 *
 * An error frame reports one plot's failure while the watch stays open, so it
 * may carry no payload or diagnostics; it maps to an empty payload
 * with its warnings rather than ending the whole stream on a mapping throw.
 */
export function mapNativePlotWatchPage(frame: PlotWatchPage): NativePlotStreamMessage {
  if (frame.resyncRequired) throw new Error("Watch Plot Snapshots: Resynchronization Required");
  if (frame.updates.length === 0) throw new Error("Native plot revision has no updates.");
  return { updates: frame.updates.map(mapNativePlotStreamUpdate) };
}

function mapNativePlotStreamUpdate(frame: PlotStreamUpdate): NativePlotStreamUpdate {
  if (!frame.envelope) throw new Error("Native plot stream frame has no envelope.");
  // An unspecified frame that names a plot and carries nothing is the watch's pending notice.
  if (frame.kind === protocolClientV1PlotsSnapshots.PlotStreamKind.UNSPECIFIED && !frame.envelope.payload) {
    return { kind: "pending", plot_id: frame.envelope.plotId };
  }
  const kind = plotStreamKind(frame.kind);
  if (kind === "error") {
    return {
      ...mapNativePlotErrorEnvelope(frame.envelope),
      kind,
    };
  }
  const envelope = mapNativePlotEnvelope(frame.envelope);
  return {
    ...envelope,
    kind,
  };
}

function mapNativePlotErrorEnvelope(envelope: PlotEnvelope): NativePlotPayloadEnvelope {
  if (envelope.payload && envelope.diagnostics) {
    return mapNativePlotEnvelope(envelope);
  }
  return {
    plot_id: envelope.plotId,
    revision: safeNumber(envelope.revision, "Plots envelope revision"),
    epoch: envelope.epoch,
    selection: envelope.selection,
    payload: envelope.payload
      ? mapNativePlotPayload(envelope.payload)
      : { stream: "", x_key: "", max_points: 0, generated_at: "", series: [] },
    ...(envelope.diagnostics ? { diagnostics: mapNativePlotDiagnostics(envelope.diagnostics) } : {}),
    warnings: envelope.warnings.length ? [...envelope.warnings] : ["Plot stream returned an error."],
  };
}

/* === Request mapping === */

function dataExpression(value: DocumentDataExpression): DataExpression {
  const input = object(value, "data expression");
  const source = object(input.source, "data expression source");
  const sourceType = requiredText(source.type, "data expression source type");
  let expression: DataExpression;
  switch (sourceType) {
    case "telemetry":
      expression = {
        label: "",
        kind: {
          case: "stream",
          value: { identifier: requiredText(source.stream, "telemetry stream"), field: requiredText(source.field, "telemetry field") },
        },
      };
      break;
    case "record":
      expression = {
        label: "",
        kind: { case: "record", value: { identifier: requiredText(source.record_type, "record type"), field: "" } },
      };
      break;
    case "parameter":
      expression = { label: "", kind: { case: "parameter", value: requiredText(source.name, "parameter name") } };
      break;
    case "dimension":
      expression = { label: "", kind: { case: "dimension", value: requiredText(source.name, "dimension name") } };
      break;
    case "literal":
      expression = { label: "", kind: { case: "literal", value: jsonToNativeValue(source.value) } };
      break;
    case "artifact":
      expression = { label: "", kind: { case: "artifact", value: requiredText(source.artifact, "artifact") } };
      break;
    default:
      throw new Error(`Unsupported data expression source: ${sourceType}`);
  }
  const rawTransforms = input.transforms;
  if (rawTransforms !== undefined && !Array.isArray(rawTransforms)) throw new Error("Data expression transforms must be an array.");
  if (Array.isArray(rawTransforms) && rawTransforms.length > MAX_PLOT_TRANSFORMS)
    throw new NativeApiLimitError("Plots Expression", "Transform Budget Reached");
  for (const raw of rawTransforms ?? []) {
    const transform = object(raw, "data transform");
    const type = requiredText(transform.type, "data transform type");
    if (type === "aggregate") {
      const name = requiredText(transform.aggregate, "aggregate");
      const kinds: Record<string, protocolV1Expressions.Aggregate> = {
        count: protocolV1Expressions.Aggregate.COUNT,
        sum: protocolV1Expressions.Aggregate.SUM,
        min: protocolV1Expressions.Aggregate.MIN,
        max: protocolV1Expressions.Aggregate.MAX,
        mean: protocolV1Expressions.Aggregate.MEAN,
        std: protocolV1Expressions.Aggregate.STDDEV,
        median: protocolV1Expressions.Aggregate.MEDIAN,
        rms: protocolV1Expressions.Aggregate.RMS,
        first: protocolV1Expressions.Aggregate.FIRST,
        last: protocolV1Expressions.Aggregate.LAST,
      };
      const rank = /^p(\d{1,3})$/.exec(name)?.[1];
      const percentile = rank === undefined ? 0 : Number(rank);
      const kind = rank !== undefined && percentile <= 100 ? protocolV1Expressions.Aggregate.PERCENTILE : kinds[name];
      if (kind === undefined) throw new Error(`Unsupported aggregate: ${name}`);
      expression = { label: "", kind: { case: "aggregate", value: { input: expression, kind, percentile } } };
      continue;
    }
    let kind: Wire<protocolV1Expressions.TransformExpression>["kind"];
    switch (type) {
      case "scale":
      case "offset": {
        const value = type === "scale" ? transform.factor : transform.value;
        if (typeof value !== "number" || !Number.isFinite(value)) throw new Error(`${type} requires a finite number`);
        kind = { case: type, value };
        break;
      }
      case "detail":
        kind = { case: "detail", value: requiredText(transform.name, "detail field") };
        break;
      case "time_between":
        kind = { case: "timeBetween", value: requiredText(transform.previous_record_type, "previous record type") };
        break;
      case "per_case":
        kind = { case: "perCase", value: true };
        break;
      default:
        throw new Error(`Unsupported data transform: ${type}`);
    }
    expression = { label: "", kind: { case: "transform", value: { input: expression, kind } } };
  }
  return expression;
}

function plotFilter(value: NonNullable<DocumentViewPlot["filters"]>[number]): PlotRequest["filters"][number] {
  return {
    expression: dataExpression(value.expression),
    operator: filterOperator(value.operator),
    ...(value.value === undefined ? {} : { value: jsonToNativeValue(value.value) }),
  };
}

function plotType(value?: DocumentPlotType | null): PlotType {
  const kinds: Record<DocumentPlotType, protocolV1Plots.PlotKind> = {
    time_series: protocolV1Plots.PlotKind.TIME_SERIES,
    bar: protocolV1Plots.PlotKind.BAR,
    xy: protocolV1Plots.PlotKind.XY,
    trajectory: protocolV1Plots.PlotKind.TRAJECTORY,
    contour: protocolV1Plots.PlotKind.CONTOUR,
    grid_chart: protocolV1Plots.PlotKind.GRID_CHART,
    frequency_series: protocolV1Plots.PlotKind.FREQUENCY_SERIES,
    histogram: protocolV1Plots.PlotKind.HISTOGRAM,
    box_plot: protocolV1Plots.PlotKind.BOX_PLOT,
    pie: protocolV1Plots.PlotKind.PIE,
  };
  if (value == null) return protocolV1Plots.PlotKind.UNSPECIFIED;
  const kind = kinds[value];
  if (kind === undefined) throw new Error(`Unsupported document plot type: ${String(value)}`);
  return kind;
}

function downsampleTechnique(value?: DocumentDownsampleTechnique | null): DownsampleTechnique {
  return value === "raw"
    ? protocolV1Plots.Downsample.RAW
    : value === "min_max"
      ? protocolV1Plots.Downsample.MIN_MAX
      : value === "lttb"
        ? protocolV1Plots.Downsample.LTTB
        : protocolV1Plots.Downsample.AUTO;
}

function axisScale(value?: DocumentAxisScale | null): AxisScale {
  if (value === "log") return protocolV1Plots.AxisScale.LOG;
  if (value === "linear" || value == null) return protocolV1Plots.AxisScale.LINEAR;
  throw new Error(`Unsupported plot axis scale: ${String(value)}`);
}

function filterOperator(value: NonNullable<DocumentViewPlot["filters"]>[number]["operator"]): FilterOperator {
  const operator = {
    eq: protocolV1Expressions.FilterOperator.EQ,
    ne: protocolV1Expressions.FilterOperator.NE,
    gt: protocolV1Expressions.FilterOperator.GT,
    gte: protocolV1Expressions.FilterOperator.GE,
    lt: protocolV1Expressions.FilterOperator.LT,
    lte: protocolV1Expressions.FilterOperator.LE,
    contains: protocolV1Expressions.FilterOperator.CONTAINS,
    exists: protocolV1Expressions.FilterOperator.EXISTS,
  }[value];
  if (operator === undefined) throw new Error(`Unsupported plot filter operator: ${String(value)}`);
  return operator;
}

/* === Response mapping === */

function mapNativePlotPayload(payload: NativePlotPayload): PlotPayload {
  return {
    stream: payload.stream,
    x_key: payload.xKey,
    series_by: payload.seriesBy ?? null,
    context: payload.context ? { value: nativeValueToJson(payload.context) } : {},
    series: payload.series.map((series, index) => {
      const metadata = mapNativePlotSeries(series);
      if (payload.frame?.mode) return metadata;
      const column = payload.frame?.series[index];
      return {
        ...metadata,
        points: (column?.x ?? []).map((x, row) => ({
          x: nativeValueToJson(x) ?? null,
          y: column?.y[row]?.value ?? null,
          z: column?.z[row]?.value ?? null,
          instant: column?.instants[row] ?? null,
        })),
      };
    }),
    analysis: mapNativePlotAnalysis(payload.analysis),
    computed: mapNativeComputation(payload.computed),
    cartesian: mapNativeCartesianFrame(payload.frame),
    markers: payload.markers.map(mapNativePlotMarker),
    max_points: safeNumber(payload.maxPoints, "Plots payload max_points"),
    window_s: payload.windowS ?? null,
    generated_at: payload.generatedAt,
    x_format: mapNativeValueFormat(payload.xFormat),
    run_started_at: payload.runStartedAt ?? null,
    cycle_started_at: payload.cycleStartedAt ?? null,
    bucket_width_s: payload.bucketWidthS ?? null,
    bucket_window: payload.bucketWindow ? { from: payload.bucketWindow.from, to: payload.bucketWindow.to } : null,
    replaced_buckets: payload.replacedBuckets,
    summaries: payload.summaries.map((summary) => ({
      series_id: summary.seriesId,
      count: safeNumber(summary.count, "plot summary count"),
      min: summary.min ?? null,
      max: summary.max ?? null,
      mean: summary.mean ?? null,
      median: summary.median ?? null,
      std: summary.std ?? null,
      rms: summary.rms ?? null,
      window: summary.window ? { from: summary.window.from, to: summary.window.to } : null,
    })),
  };
}

function mapNativePlotAnalysis(analysis?: NativePlotAnalysis): PlotAnalysis | null {
  const result = analysis?.result?.kind;
  if (!result?.case) return null;
  if (!result.value) throw new Error("Native plot analysis is missing its result.");

  switch (result.case) {
    case "error":
      if (!result.value) throw new Error("Native plot analysis is missing its result.");
      return { kind: "error", message: result.value.message };
    case "histogram":
      if (!result.value) throw new Error("Native plot analysis is missing its result.");
      return {
        kind: "histogram",
        series: result.value.series.map((series) => ({
          key: series.key,
          count: safeNumber(series.count, "histogram count"),
          ...(series.statistics
            ? {
                statistics: {
                  mean: series.statistics.mean,
                  median: series.statistics.median,
                  std: series.statistics.std,
                  rms: series.statistics.rms,
                  min: series.statistics.min,
                  max: series.statistics.max,
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
      if (!result.value) throw new Error("Native plot analysis is missing its result.");
      return {
        kind: "box_plot",
        series: result.value.series.map((series) => ({
          key: series.key,
          label: series.label,
          count: safeNumber(series.count, "box summary count"),
          min: series.min,
          q1: series.q1,
          median: series.median,
          q3: series.q3,
          max: series.max,
          lower_whisker: series.lowerWhisker,
          upper_whisker: series.upperWhisker,
          outliers: [...series.outliers],
        })),
        total: safeNumber(result.value.total, "box summary total"),
      };
    case "sigmaInterval":
      if (!result.value) throw new Error("Native plot analysis is missing its result.");
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
    case "contour": {
      const grid = result.value;
      return {
        kind: "contour",
        grid: {
          columns: grid.columns,
          rows: grid.rows,
          x_edges: grid.xEdges,
          y_edges: grid.yEdges,
          values: grid.values,
          valid: grid.valid,
        },
      };
    }
    case "gridChart": {
      const data = result.value;
      return {
        kind: "grid_chart",
        data: {
          rows: data.rows.map((entry) => ({ ...entry })),
          columns: data.columns.map((entry) => ({ ...entry })),
          summary_label: data.summaryLabel,
          frames: data.frames.map((frame) => ({
            instant: frame.instant,
            values: frame.values,
            valid: frame.valid,
            highlights: frame.highlights.map((cell) => ({ ...cell })),
          })),
        },
      };
    }
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
    format: mapNativeValueFormat(series.format),
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
    if (x.length !== column.y.length) valid = false;
    for (let row = 0; row < column.y.length; row += 1) {
      y.push(column.y[row].value ?? null);
      if (!Number.isFinite(x[row])) valid = false;
    }
    return { x, y };
  });
  if (!valid) return null;
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
  return { min: value.min, max: value.max };
}

function mapNativePlotDiagnostics(diagnostics: NativePlotPayloadDiagnostics): PlotPayloadDiagnostics {
  return {
    series_count: safeNumber(diagnostics.seriesCount, "Plots diagnostics series_count"),
    point_count: safeNumber(diagnostics.pointCount, "Plots diagnostics point_count"),
    raw_point_count: safeNumber(diagnostics.rawPointCount, "Plots diagnostics raw_point_count"),
    downsampled: diagnostics.downsampled,
    max_points: safeNumber(diagnostics.maxPoints, "Plots diagnostics max_points"),
    window_s: diagnostics.windowS ?? null,
  };
}

function plotStreamKind(value: PlotStreamKind): PlotPayloadStreamMessage["kind"] {
  switch (value) {
    case protocolClientV1PlotsSnapshots.PlotStreamKind.SNAPSHOT:
      return "snapshot";
    case protocolClientV1PlotsSnapshots.PlotStreamKind.REPLACE:
      return "replace";
    case protocolClientV1PlotsSnapshots.PlotStreamKind.DELTA:
      return "delta";
    case protocolClientV1PlotsSnapshots.PlotStreamKind.STATISTICS:
      return "statistics";
    case protocolClientV1PlotsSnapshots.PlotStreamKind.ERROR:
      return "error";
    default:
      throw new Error("Native plot stream returned an unspecified kind.");
  }
}

function cartesianMode(value: PlotCartesianMode | undefined): PlotCartesianFrame["mode"] {
  switch (value) {
    case protocolClientV1PlotsSnapshots.PlotCartesianMode.ALIGNED:
      return "aligned";
    case protocolClientV1PlotsSnapshots.PlotCartesianMode.FACETED:
      return "faceted";
    default:
      throw new Error("Native Cartesian frame returned an unspecified mode.");
  }
}

/* === Telemetry and lifecycle helpers === */

function boundedRequestCount(count: number, operation: string): void {
  if (count > NATIVE_PLOT_MAX_REQUESTS) {
    throw new NativeApiLimitError(operation, `more than ${NATIVE_PLOT_MAX_REQUESTS} plots`);
  }
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

function requiredText(value: unknown, label: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`Native ${label} is required.`);
  }
  return value;
}

function object(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`Native ${label} must be an object.`);
  }
  return value as Record<string, unknown>;
}

function ensureActive(session: SessionIdentity, signal?: AbortSignal): void {
  if (signal?.aborted || !session.isCurrent()) throw abortError();
}

function normalizeCancellation(error: unknown, session: SessionIdentity, signal?: AbortSignal): unknown {
  if (signal?.aborted || !session.isCurrent()) return abortError();
  return error;
}

function abortError(): DOMException {
  return new DOMException("Native plot request cancelled.", "AbortError");
}

function mapNativeAnalysisDefinition(value: NativeAnalysisDefinition): EvidenceAnalysisDefinition {
  return {
    id: value.metadata?.identifier ?? "",
    version: value.version,
    label: value.metadata?.label ?? "",
    description: value.metadata?.description ?? "",
    parameters: value.parameters.map(mapParameter),
    outputs: value.outputs.map((output) => ({
      key: output.metadata?.identifier ?? "",
      label: output.metadata?.label ?? "",
      unit: output.unit,
      axis: output.axis ? { key: output.axis, unit: output.axisUnit } : undefined,
    })),
    maxSamples: value.maxSamples,
    maxOutputValues: value.maxOutputValues,
  };
}

function mapNativeComputation(value?: NativeAnalysisComputation): EvidenceAnalysisResult | null {
  if (!value) return null;
  if (!value.definition || !value.selection || !value.source) {
    throw new Error("Native analysis result is missing its definition or provenance.");
  }
  return {
    definition: mapNativeAnalysisDefinition(value.definition),
    selection: {
      id: value.selection.id,
      version: value.selection.version,
      parameters: nativeValuesToJson(value.selection.parameters?.values),
    },
    source: {
      store: value.source.store,
      epoch: value.source.epoch,
      position: value.source.position.toString(),
      stream: value.source.stream,
      scope: recordScopeFromNative(value.source.scope),
      windowS: value.source.windowS,
      fields: [...value.source.fields],
      xKey: value.source.xKey,
    },
    series: value.series.map((series) => ({
      key: series.key,
      label: series.label,
      origin: series.origin,
      inputSamples: safeNumber(series.inputSamples, "analysis input samples"),
      firstAxis: series.firstAxis,
      lastAxis: series.lastAxis,
      outputs: series.outputs.map((output) => ({
        key: output.key,
        unit: output.unit,
        axisUnit: output.axisUnit,
        displayIndices: [...output.displayIndices],
        axis: [...output.axis],
        values: [...output.values],
      })),
    })),
  };
}
