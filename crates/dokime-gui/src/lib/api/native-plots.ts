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
import type { EvidenceScope } from "@host/model";
import * as hostModel from "@host/model";
import {
  NativePlotPayloadEnvelope,
  PlotEnvelope,
  PlotRequest,
  mapNativePlotEnvelope,
  mapNativePlotPayload,
  plotType,
  requiredText,
} from "./native-plot-projection";

/* === Local Imports === */

import * as rpc from "@/lib/host/client";
import { jsonToNativeValue } from "@/lib/host/values";
import { cutKey, required, type Wire } from "@/lib/rpc/canonical";
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
  PlotPayloadStreamMessage,
} from "@/lib/types";
import { NATIVE_PLOT_MAX_POINTS } from "@/plots/reduce";
import type { SnapshotsWatchResponse } from "@host/model";
import {
  Aggregate,
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
  type TransformExpression,
} from "@host/model";
import { NATIVE_MAX_PAGES, NativeApiLimitError, safeNumber } from "./native-values";

/* === Canonical Types === */

type DataExpression = NonNullable<PlotRequest["xExpression"]>;

type AxisScale = NonNullable<PlotRequest["xScale"]>;
type DownsampleTechnique = NonNullable<PlotRequest["technique"]>;
type FilterOperator = NonNullable<PlotRequest["filters"][number]["operator"]>;

type PlotWatchPage = Wire<SnapshotsWatchResponse>;
type PlotStreamUpdate = PlotWatchPage["updates"][number];
type PlotStreamKind = PlotStreamUpdate["kind"];
type PlotTypes = ReadonlyMap<string, DocumentPlotType | null | undefined>;

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
  scope?: Wire<EvidenceScope> | null;
};

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
  const nativeScope = input.scope ?? undefined;
  return create(PlotRequestSchema, {
    arrangement: input.arrangement ? (input.arrangement === "grouped" ? BarArrangement.GROUPED : BarArrangement.STACKED) : undefined,
    pareto: input.pareto ?? false,
    hole: input.hole ?? undefined,
    xyInterval: input.xyInterval ? XyInterval.SIGMA : undefined,
    render: input.render === "bands" ? ContourRender.BANDS : ContourRender.CELLS,
    reduction: input.reduction ? { mean: Reduction.MEAN, min: Reduction.MIN, max: Reduction.MAX }[input.reduction] : undefined,
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
          summaryLabel: input.grid.summaryLabel,
        }
      : undefined,
    panelHeight: input.panelHeight,
    yViewport: input.yViewport,

    ...(input.plotId == null ? {} : { plotId: input.plotId }),
    plotType: plotType(input.plotType),
    style: {
      line: LineStyle.LINE,
      points: LineStyle.POINTS,
      line_points: LineStyle.LINE_POINTS,
      step: LineStyle.STEP,
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
            window: input.fft.window === "rectangular" ? FftWindow.RECTANGULAR : FftWindow.HANN,
            detrend: input.fft.detrend === "none" ? FftDetrend.NONE : FftDetrend.MEAN,
            levelUnit: {
              linear: FrequencyLevelUnit.LINEAR,
              dB: FrequencyLevelUnit.DB,
              dBu: FrequencyLevelUnit.DBU,
              dBm: FrequencyLevelUnit.DBM,
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
    const plotTypes: PlotTypes = new Map(requests.map((request) => [request.plotId ?? request.stream, request.plotType]));
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
      envelopes.push(...response.envelopes.map((envelope) => mapNativePlotEnvelope(envelope, plotTypes.get(envelope.plotId))));
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
  const plotTypes: PlotTypes = new Map(requests.map((request) => [request.plotId ?? request.stream, request.plotType]));

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
        return { done: false, value: mapNativePlotWatchPage(result.value, plotTypes) };
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

/**
 * Map one protobuf storage-query snapshot into the renderer.
 *
 * An error frame reports one plot's failure while the watch stays open, so it
 * may carry no payload or diagnostics; it maps to an empty payload
 * with its warnings rather than ending the whole stream on a mapping throw.
 */
export function mapNativePlotWatchPage(frame: PlotWatchPage, plotTypes?: PlotTypes): NativePlotStreamMessage {
  if (frame.resyncRequired) throw new Error("Watch Plot Snapshots: Resynchronization Required");
  if (frame.updates.length === 0) throw new Error("Native plot revision has no updates.");
  return { updates: frame.updates.map((update) => mapNativePlotStreamUpdate(update, plotTypes?.get(update.envelope?.plotId ?? ""))) };
}

function mapNativePlotStreamUpdate(frame: PlotStreamUpdate, plotType?: DocumentPlotType | null): NativePlotStreamUpdate {
  if (!frame.envelope) throw new Error("Native plot stream frame has no envelope.");
  // An unspecified frame that names a plot and carries nothing is the watch's pending notice.
  if (frame.kind === hostModel.PlotStreamKind.UNSPECIFIED && !frame.envelope.payload) {
    return { kind: "pending", plot_id: frame.envelope.plotId };
  }
  const kind = plotStreamKind(frame.kind);
  if (kind === "error") {
    return {
      ...mapNativePlotErrorEnvelope(frame.envelope, plotType),
      kind,
    };
  }
  const envelope = mapNativePlotEnvelope(frame.envelope, plotType);
  return {
    ...envelope,
    kind,
  };
}

function mapNativePlotErrorEnvelope(envelope: PlotEnvelope, plotType?: DocumentPlotType | null): NativePlotPayloadEnvelope {
  if (envelope.payload && envelope.diagnostics) {
    return mapNativePlotEnvelope(envelope, plotType);
  }
  return {
    plot_id: envelope.plotId,
    revision: safeNumber(envelope.revision, "Plots envelope revision"),
    epoch: envelope.epoch,
    selection: envelope.selection,
    payload: envelope.payload
      ? mapNativePlotPayload(envelope.payload, plotType)
      : { stream: "", x_key: "", max_points: 0, generated_at: "", series: [] },
    ...(envelope.diagnostics ? { diagnostics: envelope.diagnostics } : {}),
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
      const kinds: Record<string, Aggregate> = {
        count: Aggregate.COUNT,
        sum: Aggregate.SUM,
        min: Aggregate.MIN,
        max: Aggregate.MAX,
        mean: Aggregate.MEAN,
        std: Aggregate.STDDEV,
        median: Aggregate.MEDIAN,
        rms: Aggregate.RMS,
        first: Aggregate.FIRST,
        last: Aggregate.LAST,
      };
      const rank = /^p(\d{1,3})$/.exec(name)?.[1];
      const percentile = rank === undefined ? 0 : Number(rank);
      const kind = rank !== undefined && percentile <= 100 ? Aggregate.PERCENTILE : kinds[name];
      if (kind === undefined) throw new Error(`Unsupported aggregate: ${name}`);
      expression = { label: "", kind: { case: "aggregate", value: { input: expression, kind, percentile } } };
      continue;
    }
    let kind: Wire<TransformExpression>["kind"];
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

function downsampleTechnique(value?: DocumentDownsampleTechnique | null): DownsampleTechnique {
  return value === "raw" ? Downsample.RAW : value === "min_max" ? Downsample.MIN_MAX : value === "lttb" ? Downsample.LTTB : Downsample.AUTO;
}

function axisScale(value?: DocumentAxisScale | null): AxisScale {
  if (value === "log") return hostModel.AxisScale.LOG;
  if (value === "linear" || value == null) return hostModel.AxisScale.LINEAR;
  throw new Error(`Unsupported plot axis scale: ${String(value)}`);
}

function filterOperator(value: NonNullable<DocumentViewPlot["filters"]>[number]["operator"]): FilterOperator {
  const operator = {
    eq: hostModel.FilterOperator.EQ,
    ne: hostModel.FilterOperator.NE,
    gt: hostModel.FilterOperator.GT,
    gte: hostModel.FilterOperator.GE,
    lt: hostModel.FilterOperator.LT,
    lte: hostModel.FilterOperator.LE,
    contains: hostModel.FilterOperator.CONTAINS,
    exists: hostModel.FilterOperator.EXISTS,
  }[value];
  if (operator === undefined) throw new Error(`Unsupported plot filter operator: ${String(value)}`);
  return operator;
}

/* === Response mapping === */

function plotStreamKind(value: PlotStreamKind): PlotPayloadStreamMessage["kind"] {
  switch (value) {
    case hostModel.PlotStreamKind.SNAPSHOT:
      return "snapshot";
    case hostModel.PlotStreamKind.REPLACE:
      return "replace";
    case hostModel.PlotStreamKind.DELTA:
      return "delta";
    case hostModel.PlotStreamKind.STATISTICS:
      return "statistics";
    case hostModel.PlotStreamKind.ERROR:
      return "error";
    default:
      throw new Error("Native plot stream returned an unspecified kind.");
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
