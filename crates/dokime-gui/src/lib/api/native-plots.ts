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

import * as rpc from "@/lib/rpc/generated/client";
import * as plotWire from "../../../.generated/dokime/v1/PlotsMessages/QuerySnapshots/messages_pb";
import type { Row3 as WatchFrame } from "../../../.generated/dokime/v1/PlotsMessages/Watch/messages_pb";
import type { Wire } from "@/lib/rpc/canonical";
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
  PlotPayloadCursor,
  PlotPayloadDiagnostics,
  PlotPayloadEnvelope,
  PlotPayloadStreamMessage,
  PlotPoint,
  PlotSeries,
  RecordScope,
  TelemetrySampleRecord,
} from "@/lib/types";
import { mapParameter } from "./native-catalog";
import {
  jsonToNativeValue,
  mapNativeValueFormat,
  NativeApiLimitError,
  nativeEvidenceScope,
  nativeValueObjectToJson,
  nativeValueToJson,
  recordScopeFromNative,
  safeNumber,
  timestampIso,
} from "./native-values";

/* === Canonical Types === */

type DataExpression = Wire<plotWire.Row8>;
type DataTransform = Wire<plotWire.Row43>;
type NativeAnalysisComputation = Wire<plotWire.Row58>;
type NativeAnalysisDefinition = Wire<plotWire.Row93>;
type NativePlotAnalysis = Wire<plotWire.Row57>;
type NativePlotCartesianFrame = Wire<plotWire.Row54>;
type NativePlotMarker = Wire<plotWire.Row55>;
type NativePlotPayload = Wire<plotWire.Row16>;
type NativePlotPayloadDiagnostics = Wire<plotWire.Row17>;
type NativePlotPoint = Wire<plotWire.Row87>;
type NativePlotSeries = Wire<plotWire.Row53>;
type PlotCursor = Wire<plotWire.Row10>;
type PlotEnvelope = Wire<plotWire.Row4>;
type PlotRequest = Wire<plotWire.Row3>;
type PlotType = Wire<plotWire.Row5>;
type AxisScale = Wire<plotWire.Row6>;
type DownsampleTechnique = Wire<plotWire.Row11>;
type FilterOperator = Wire<plotWire.Row40>;
type PlotCartesianMode = Wire<plotWire.Row88>;
const AnalysisSelectionSchema = plotWire.Row15Schema;
const ArtifactDataSourceSchema = plotWire.Row101Schema;
const DataExpressionSchema = plotWire.Row8Schema;
const DataTransformSchema = plotWire.Row43Schema;
const DimensionDataSourceSchema = plotWire.Row99Schema;
const ParameterDataSourceSchema = plotWire.Row98Schema;
const PlotCursorSchema = plotWire.Row10Schema;
const PlotFftOptionsSchema = plotWire.Row12Schema;
const PlotFilterSchema = plotWire.Row7Schema;
const PlotHistogramOptionsSchema = plotWire.Row14Schema;
const PlotIntervalOptionsSchema = plotWire.Row13Schema;
const PlotRequestSchema = plotWire.Row3Schema;
const RecordDataSourceSchema = plotWire.Row100Schema;
const TelemetryDataSourceSchema = plotWire.Row97Schema;
type PlotStreamFrame = Wire<WatchFrame>;
type PlotStreamKind = PlotStreamFrame["kind"];
type NativeTelemetrySample = Wire<
  Awaited<ReturnType<typeof rpc.telemetryListSamples>>
>["samples"][number];

/* === Constants === */

export const NATIVE_PLOT_MAX_REQUESTS = 32;
export const NATIVE_PLOT_MAX_POINTS = 100_000;
export const NATIVE_TELEMETRY_MAX_LIMIT = 5_000;
const MAX_PLOT_TRANSFORMS = 32;

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
  /** Absent is the host default (`lttb`). */
  technique?: DocumentDownsampleTechnique | null;
  windowS?: number | null;
  scope?: RecordScope | null;
  /** Legacy cursors are accepted only when they carry native epoch identity. */
  cursor?: PlotPayloadCursor | null;
};

/** Renderer envelope plus the exact native cursor used for a follow-up watch. */
export type NativePlotPayloadEnvelope = PlotPayloadEnvelope & {
  nativeCursor: PlotCursor;
};

/** Renderer stream message plus its exact native cursor. */
export type NativePlotStreamMessage = PlotPayloadStreamMessage & {
  nativeCursor: PlotCursor;
};

export interface NativePlotStream extends AsyncIterable<NativePlotStreamMessage> {
  cancel(): Promise<void>;
}

export type NativeTelemetrySamples = {
  samples: TelemetrySampleRecord[];
  throughStreamCursors: Record<string, bigint>;
  /** Continuation is unavailable for an explicit latest tail window. */
  continuation: Record<string, bigint> | null;
  /** False means a forward page may have more samples. */
  complete: boolean;
  /** True only when the request explicitly asked for the latest tail window. */
  latestWindow: boolean;
};

export type NativeTelemetrySamplesOptions = {
  limit?: number;
  scope?: RecordScope | null;
  signal?: AbortSignal;
  latest?: boolean;
  afterStreamCursors?: Readonly<Record<string, bigint>>;
};

export type NativePlotReadOptions = {
  signal?: AbortSignal;
  /** Exact cursors returned by an earlier native snapshot, indexed by request. */
  cursors?: readonly (PlotCursor | undefined)[];
};

export type NativePlotWatchOptions = NativePlotReadOptions & {
  /** Exact cursors returned by an earlier native snapshot, indexed by request. */
  cursors?: readonly (PlotCursor | undefined)[];
};

/* === Public API === */

/** Build one typed protobuf plot request from the document-facing input. */
export function toNativePlotRequest(
  input: PlotPayloadRequestInput,
  cursor?: PlotCursor,
): PlotRequest {
  const nativeCursor = cursor ?? nativeCursorFromInput(input.cursor);
  const nativeScope = nativeEvidenceScope(input.scope);
  return create(PlotRequestSchema, {
    ...(input.plotId == null ? {} : { plotId: input.plotId }),
    analysis: input.analysis
      ? create(AnalysisSelectionSchema, {
          id: input.analysis.id,
          version: input.analysis.version,
          parameters: jsonToNativeValue(input.analysis.parameters),
        })
      : undefined,
    plotType: plotType(input.plotType),
    xScale: axisScale(input.xScale),
    yScale: axisScale(input.yScale),
    ...(input.zExpression == null ? {} : { zExpression: dataExpression(input.zExpression) }),
    ...(input.fft == null
      ? {}
      : {
          fft: create(PlotFftOptionsSchema, {
            sampleRateHz: input.fft.sample_rate_hz ?? undefined,
            window:
              input.fft.window === "rectangular"
                ? { HarmosVariant: { case: "Rectangular" as const, value: {} } }
                : { HarmosVariant: { case: "Hann" as const, value: {} } },
            detrend:
              input.fft.detrend === "none"
                ? { HarmosVariant: { case: "None" as const, value: {} } }
                : { HarmosVariant: { case: "Mean" as const, value: {} } },
            levelUnit: {
              linear: { HarmosVariant: { case: "Linear" as const, value: {} } },
              dB: { HarmosVariant: { case: "Db" as const, value: {} } },
              dBu: { HarmosVariant: { case: "Dbu" as const, value: {} } },
              dBm: { HarmosVariant: { case: "Dbm" as const, value: {} } },
            }[input.fft.level_unit],
            includeDc: input.fft.include_dc,
            minFrequencyHz: input.fft.min_frequency_hz ?? undefined,
            maxFrequencyHz: input.fft.max_frequency_hz ?? undefined,
          }),
        }),
    ...(input.interval == null
      ? {}
      : {
          interval: create(PlotIntervalOptionsSchema, {
            binWidth: input.interval.bin_width,
            sigma: input.interval.sigma,
            clipToObserved: input.interval.clip_to_observed,
          }),
        }),
    ...(input.histogram == null
      ? {}
      : {
          histogram: create(PlotHistogramOptionsSchema, {
            bins: boundedUint64(input.histogram.bins, "histogram bins"),
            unit: input.histogram.unit ?? undefined,
          }),
        }),
    stream: requiredText(input.stream, "plot stream"),
    yKeys: input.yKeys,
    markerRecordTypes: input.markerRecordTypes ?? [],
    filters: (input.filters ?? []).map(plotFilter),
    ...(input.xKey == null ? {} : { xKey: input.xKey }),
    ...(input.seriesBy == null ? {} : { seriesBy: input.seriesBy }),
    ...(input.xExpression == null ? {} : { xExpression: dataExpression(input.xExpression) }),
    yExpressions: (input.yExpressions ?? []).map(dataExpression),
    ...(input.seriesByExpression == null
      ? {}
      : { seriesByExpression: dataExpression(input.seriesByExpression) }),
    ...(input.zStream == null ? {} : { zStream: input.zStream }),
    ...(input.zField == null ? {} : { zField: input.zField }),
    ...(input.unit == null ? {} : { unit: input.unit }),
    ...(input.maxPoints == null ? {} : { maxPoints: boundedPlotPoints(input.maxPoints) }),
    ...(input.windowS == null ? {} : { windowS: boundedWindow(input.windowS) }),
    ...(nativeScope ? { scope: nativeScope } : {}),
    ...(nativeCursor ? { cursor: nativeCursor } : {}),
    technique: downsampleTechnique(input.technique),
  });
}

/** Read bounded plot snapshots over the generation-owned native transport. */
export async function queryNativePlotPayloads(
  session: SessionIdentity,
  requests: readonly PlotPayloadRequestInput[],
  options: NativePlotReadOptions = {},
): Promise<NativePlotPayloadEnvelope[]> {
  boundedRequestCount(requests.length, "Plots.QuerySnapshots");
  ensureActive(session, options.signal);
  try {
    const reply = await rpc.plotsQuerySnapshots(session, {
      requests: requests.map((request, index) =>
        toNativePlotRequest(request, options.cursors?.[index]),
      ),
    });
    ensureActive(session, options.signal);
    return reply.envelopes.map(mapNativePlotEnvelope);
  } catch (error) {
    throw normalizeCancellation(error, session, options.signal);
  }
}

/** Read one plot payload while preserving the native snapshot cursor internally. */
export async function fetchNativePlotPayload(
  session: SessionIdentity,
  request: PlotPayloadRequestInput,
  options: NativePlotReadOptions = {},
): Promise<PlotPayload> {
  const [envelope] = await queryNativePlotPayloads(session, [request], options);
  if (!envelope) {
    throw new Error("Native plot snapshot response was empty.");
  }
  return envelope.payload;
}

export async function fetchNativePlotPayloadBatch(
  session: SessionIdentity,
  requests: readonly PlotPayloadRequestInput[],
  options: NativePlotReadOptions = {},
): Promise<PlotPayload[]> {
  return (await queryNativePlotPayloads(session, requests, options)).map(
    (envelope) => envelope.payload,
  );
}

/**
 * Open the subscribed plot watch. Cursorless requests receive the server's
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
  const nativeRequests = requests.map((request, index) =>
    toNativePlotRequest(request, options.cursors?.[index]),
  );

  let source: Awaited<ReturnType<typeof rpc.plotsWatch>> | null = null;
  try {
    source = await rpc.plotsWatch(session, { requests: nativeRequests });
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
        return { done: false, value: mapNativePlotStreamFrame(result.value) };
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

/** Read typed telemetry samples with explicit forward-page and tail-window semantics. */
export async function listNativeTelemetrySamples(
  session: SessionIdentity,
  stream: string,
  options: NativeTelemetrySamplesOptions = {},
): Promise<NativeTelemetrySamples> {
  const limit = boundedTelemetryLimit(options.limit ?? 500);
  const latest = options.latest ?? false;
  const afterStreamCursors = { ...(options.afterStreamCursors ?? {}) };
  if (latest && Object.keys(afterStreamCursors).length > 0) {
    throw new Error("Latest telemetry windows cannot be continued with after cursors.");
  }
  ensureActive(session, options.signal);
  try {
    const reply = await rpc.telemetryListSamples(session, {
      streams: [requiredText(stream, "telemetry stream")],
      limit,
      schemaVersions: [],
      sources: [],
      signals: [],
      ...(nativeEvidenceScope(options.scope) ? { scope: nativeEvidenceScope(options.scope) } : {}),
      afterStreamCursors: Object.entries(afterStreamCursors).map(([key, value]) => ({
        key,
        value,
      })),
      latest,
    });
    ensureActive(session, options.signal);
    const throughStreamCursors = Object.fromEntries(
      reply.throughStreamCursors.map(({ key, value }) => [key, value]),
    );
    return {
      samples: reply.samples.map(mapNativeTelemetrySample),
      throughStreamCursors,
      continuation: latest || reply.complete ? null : throughStreamCursors,
      complete: reply.complete,
      latestWindow: latest,
    };
  } catch (error) {
    throw normalizeCancellation(error, session, options.signal);
  }
}

/** Compatibility-shaped sample projection for callers that only need samples. */
export async function fetchNativeTelemetrySamples(
  session: SessionIdentity,
  stream: string,
  options: NativeTelemetrySamplesOptions = {},
): Promise<TelemetrySampleRecord[]> {
  return (await listNativeTelemetrySamples(session, stream, options)).samples;
}

/** Map one protobuf envelope into the payload family used by the renderer. */
export function mapNativePlotEnvelope(envelope: PlotEnvelope): NativePlotPayloadEnvelope {
  if (!envelope.cursor) throw new Error("Native plot envelope has no cursor.");
  if (!envelope.payload) throw new Error("Native plot envelope has no payload.");
  if (!envelope.diagnostics) throw new Error("Native plot envelope has no diagnostics.");
  return {
    plot_id: envelope.plotId,
    revision: safeNumber(envelope.revision, "Plots envelope revision"),
    cursor: mapPlotCursor(envelope.cursor),
    nativeCursor: envelope.cursor,
    payload: mapNativePlotPayload(envelope.payload),
    diagnostics: mapNativePlotDiagnostics(envelope.diagnostics),
    warnings: [...envelope.warnings],
  };
}

/**
 * Map one protobuf stream frame into the renderer's append/replace model.
 *
 * An error frame reports one plot's failure while the watch stays open, so it
 * may carry no payload, cursor or diagnostics; it maps to an empty payload
 * with its warnings rather than ending the whole stream on a mapping throw.
 */
export function mapNativePlotStreamFrame(frame: PlotStreamFrame): NativePlotStreamMessage {
  if (!frame.envelope) throw new Error("Native plot stream frame has no envelope.");
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
  if (envelope.cursor && envelope.payload && envelope.diagnostics) {
    return mapNativePlotEnvelope(envelope);
  }
  const cursor = envelope.cursor ?? create(PlotCursorSchema, {});
  return {
    plot_id: envelope.plotId,
    revision: safeNumber(envelope.revision, "Plots envelope revision"),
    cursor: mapPlotCursor(cursor),
    nativeCursor: cursor,
    payload: envelope.payload
      ? mapNativePlotPayload(envelope.payload)
      : { stream: "", x_key: "", max_points: 0, generated_at: "", series: [] },
    ...(envelope.diagnostics
      ? { diagnostics: mapNativePlotDiagnostics(envelope.diagnostics) }
      : {}),
    warnings: envelope.warnings.length
      ? [...envelope.warnings]
      : ["Plot stream returned an error."],
  };
}

/* === Request mapping === */

function dataExpression(value: DocumentDataExpression): DataExpression {
  const input = object(value, "data expression");
  const source = object(input.source, "data expression source");
  const sourceType = requiredText(source.type, "data expression source type");
  const wireSource: NonNullable<DataExpression["source"]>["HarmosVariant"] = (() => {
    switch (sourceType) {
      case "telemetry":
        return {
          case: "Telemetry",
          value: {
            field0: create(TelemetryDataSourceSchema, {
              stream: requiredText(source.stream, "telemetry expression stream"),
              field: requiredText(source.field, "telemetry expression field"),
            }),
          },
        };
      case "parameter":
        return {
          case: "Parameter",
          value: {
            field0: create(ParameterDataSourceSchema, {
              name: requiredText(source.name, "parameter name"),
            }),
          },
        };
      case "dimension":
        return {
          case: "Dimension",
          value: {
            field0: create(DimensionDataSourceSchema, {
              name: requiredText(source.name, "dimension name"),
            }),
          },
        };
      case "record":
        return {
          case: "Record",
          value: {
            field0: create(RecordDataSourceSchema, {
              recordType: requiredText(source.record_type, "record type"),
            }),
          },
        };
      case "artifact":
        return {
          case: "Artifact",
          value: {
            field0: create(ArtifactDataSourceSchema, {
              artifact: requiredText(source.artifact, "artifact"),
            }),
          },
        };
      default:
        throw new Error(`Unsupported data expression source: ${sourceType}`);
    }
  })();
  const rawTransforms = input.transforms;
  if (rawTransforms !== undefined && !Array.isArray(rawTransforms)) {
    throw new Error("Data expression transforms must be an array.");
  }
  const transforms = (rawTransforms ?? []).map(dataTransform);
  if (transforms.length > MAX_PLOT_TRANSFORMS) {
    throw new NativeApiLimitError(
      "Plots expression",
      `more than ${MAX_PLOT_TRANSFORMS} transforms`,
    );
  }
  return create(DataExpressionSchema, { source: { HarmosVariant: wireSource }, transforms });
}

function dataTransform(value: unknown): DataTransform {
  const input = object(value, "data transform");
  const kind = requiredText(input.type, "data transform type");
  switch (kind) {
    case "count":
      return create(DataTransformSchema, {
        kind: { HarmosVariant: { case: "Count", value: { field0: {} } } },
      });
    case "mean":
      return create(DataTransformSchema, {
        kind: { HarmosVariant: { case: "Mean", value: { field0: {} } } },
      });
    case "median":
      return create(DataTransformSchema, {
        kind: { HarmosVariant: { case: "Median", value: { field0: {} } } },
      });
    case "min":
      return create(DataTransformSchema, {
        kind: { HarmosVariant: { case: "Min", value: { field0: {} } } },
      });
    case "max":
      return create(DataTransformSchema, {
        kind: { HarmosVariant: { case: "Max", value: { field0: {} } } },
      });
    case "time_between":
      return create(DataTransformSchema, {
        kind: {
          HarmosVariant: {
            case: "TimeBetween",
            value: {
              field0: {
                previousRecordType: requiredText(
                  input.previous_record_type,
                  "previous record type",
                ),
              },
            },
          },
        },
      });
    case "telemetry_at_or_before":
      return create(DataTransformSchema, {
        kind: {
          HarmosVariant: {
            case: "TelemetryAtOrBefore",
            value: {
              field0: {
                stream: requiredText(input.stream, "telemetry-at-or-before stream"),
                field: requiredText(input.field, "telemetry-at-or-before field"),
              },
            },
          },
        },
      });
    default:
      throw new Error(`Unsupported data transform: ${kind}`);
  }
}

function plotFilter(value: NonNullable<DocumentViewPlot["filters"]>[number]) {
  return create(PlotFilterSchema, {
    expression: dataExpression(value.expression),
    operator: filterOperator(value.operator),
    ...(value.value === undefined ? {} : { value: jsonToNativeValue(value.value) }),
  });
}

function nativeCursorFromInput(cursor?: PlotPayloadCursor | null): PlotCursor | undefined {
  if (!cursor) return undefined;
  if (!("epoch" in cursor) || !("selection" in cursor)) {
    throw new Error("Native plot cursors must come from a current native envelope.");
  }
  const candidate = cursor;
  if (typeof candidate.epoch !== "string" || typeof candidate.selection !== "string") {
    throw new Error("Native plot cursors must come from a current native envelope.");
  }
  return create(PlotCursorSchema, {
    telemetryStreams: Object.entries(cursor.telemetry_streams ?? {}).map(([key, value]) => ({
      key,
      value: boundedUint64(value, "plot cursor telemetry sequence"),
    })),
    ...(cursor.record_sequence == null
      ? {}
      : { recordSequence: boundedUint64(cursor.record_sequence, "plot cursor record sequence") }),
    epoch: candidate.epoch,
    selection: candidate.selection,
  });
}

function plotType(value?: DocumentPlotType | null): PlotType {
  switch (value) {
    case undefined:
    case null:
      return { HarmosVariant: { case: "Unspecified" as const, value: {} } };
    case "time_series":
      return { HarmosVariant: { case: "TimeSeries" as const, value: {} } };
    case "bar":
      return { HarmosVariant: { case: "Bar" as const, value: {} } };
    case "xy_scatter":
      return { HarmosVariant: { case: "XyScatter" as const, value: {} } };
    case "xy_line":
      return { HarmosVariant: { case: "XyLine" as const, value: {} } };
    case "xy_interval":
      return { HarmosVariant: { case: "XyInterval" as const, value: {} } };
    case "trajectory":
      return { HarmosVariant: { case: "Trajectory" as const, value: {} } };
    case "heatmap":
      return { HarmosVariant: { case: "Heatmap" as const, value: {} } };
    case "xy_value_heatmap":
      return { HarmosVariant: { case: "XyValueHeatmap" as const, value: {} } };
    case "wifi_channel_occupancy":
      return { HarmosVariant: { case: "WifiChannelOccupancy" as const, value: {} } };
    case "frequency_series":
      return { HarmosVariant: { case: "FrequencySeries" as const, value: {} } };
    case "histogram":
      return { HarmosVariant: { case: "Histogram" as const, value: {} } };
    case "box_summary":
      return { HarmosVariant: { case: "BoxSummary" as const, value: {} } };
    default:
      throw new Error(`Unsupported document plot type: ${String(value)}`);
  }
}

function downsampleTechnique(value?: DocumentDownsampleTechnique | null): DownsampleTechnique {
  switch (value) {
    case "raw":
      return { HarmosVariant: { case: "Raw" as const, value: {} } };
    case "min_max":
      return { HarmosVariant: { case: "MinMax" as const, value: {} } };
    default:
      return { HarmosVariant: { case: "Lttb" as const, value: {} } };
  }
}

function axisScale(value?: DocumentAxisScale | null): AxisScale {
  switch (value) {
    case undefined:
    case null:
    case "linear":
      return { HarmosVariant: { case: "Linear" as const, value: {} } };
    case "log":
      return { HarmosVariant: { case: "Log" as const, value: {} } };
    default:
      throw new Error(`Unsupported plot axis scale: ${String(value)}`);
  }
}

function filterOperator(
  value: NonNullable<DocumentViewPlot["filters"]>[number]["operator"],
): FilterOperator {
  switch (value) {
    case "eq":
      return { HarmosVariant: { case: "Eq" as const, value: {} } };
    case "ne":
      return { HarmosVariant: { case: "Ne" as const, value: {} } };
    case "gt":
      return { HarmosVariant: { case: "Gt" as const, value: {} } };
    case "gte":
      return { HarmosVariant: { case: "Gte" as const, value: {} } };
    case "lt":
      return { HarmosVariant: { case: "Lt" as const, value: {} } };
    case "lte":
      return { HarmosVariant: { case: "Lte" as const, value: {} } };
    case "contains":
      return { HarmosVariant: { case: "Contains" as const, value: {} } };
    case "exists":
      return { HarmosVariant: { case: "Exists" as const, value: {} } };
    default:
      throw new Error(`Unsupported plot filter operator: ${String(value)}`);
  }
}

/* === Response mapping === */

function mapPlotCursor(cursor: PlotCursor): PlotPayloadCursor {
  return {
    telemetry_streams: Object.fromEntries(
      cursor.telemetryStreams.map(({ key, value }) => [
        key,
        safeNumber(value, "Plots cursor telemetry sequence"),
      ]),
    ),
    record_sequence:
      cursor.recordSequence === undefined
        ? null
        : safeNumber(cursor.recordSequence, "Plots cursor record sequence"),
  };
}

function mapNativePlotPayload(payload: NativePlotPayload): PlotPayload {
  return {
    stream: payload.stream,
    x_key: payload.xKey,
    series_by: payload.seriesBy ?? null,
    context: nativeValueObjectToJson(payload.context) as PlotPayload["context"],
    series: payload.series.map(mapNativePlotSeries),
    analysis: mapNativePlotAnalysis(payload.analysis),
    computed: mapNativeComputation(payload.computed),
    cartesian: mapNativeCartesianFrame(payload.cartesian),
    markers: payload.markers.map(mapNativePlotMarker),
    max_points: safeNumber(payload.maxPoints, "Plots payload max_points"),
    window_s: payload.windowS ?? null,
    generated_at: payload.generatedAt,
    x_format: mapNativeValueFormat(payload.xFormat),
    run_started_at: payload.runStartedAt ?? null,
    cycle_started_at: payload.cycleStartedAt ?? null,
  };
}

function mapNativePlotAnalysis(analysis?: NativePlotAnalysis): PlotAnalysis | null {
  const result = analysis?.result?.HarmosVariant;
  if (!result?.case) return null;
  if (!result.value.field0) throw new Error("Native plot analysis is missing its result.");

  switch (result.case) {
    case "Error":
      if (!result.value.field0) throw new Error("Native plot analysis is missing its result.");
      return { kind: "error", message: result.value.field0.message };
    case "Histogram":
      if (!result.value.field0) throw new Error("Native plot analysis is missing its result.");
      return {
        kind: "histogram",
        series: result.value.field0.series.map((series) => ({
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
    case "BoxSummary":
      if (!result.value.field0) throw new Error("Native plot analysis is missing its result.");
      return {
        kind: "box_summary",
        series: result.value.field0.series.map((series) => ({
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
      };
    case "SigmaInterval":
      if (!result.value.field0) throw new Error("Native plot analysis is missing its result.");
      return {
        kind: "sigma_interval",
        series: result.value.field0.series.map((series) => ({
          key: series.key,
          bins: series.bins.map((bin) => ({
            x: bin.x,
            count: safeNumber(bin.count, "interval bin count"),
            mean: bin.mean,
            lower: bin.lower,
            upper: bin.upper,
          })),
        })),
      };
  }
}

function mapNativePlotSeries(series: NativePlotSeries): PlotSeries {
  return {
    key: series.key,
    label: series.label,
    unit: series.unit ?? null,
    points: series.points.map(mapNativePlotPoint),
    raw_points: safeNumber(series.rawPoints, "Plots series raw_points"),
    downsampled: series.downsampled,
    format: mapNativeValueFormat(series.format),
  };
}

function mapNativePlotPoint(point: NativePlotPoint): PlotPoint {
  return {
    x: nativeValueToJson(point.x) ?? null,
    y: point.y ?? null,
    z: point.z ?? null,
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

function mapNativeCartesianFrame(frame?: NativePlotCartesianFrame): PlotCartesianFrame | null {
  if (!frame || frame.series.some((series) => series.x.some((value) => !Number.isFinite(value)))) {
    return null;
  }
  return {
    mode: cartesianMode(frame.mode),
    series: frame.series.map((series) => ({
      x: [...series.x],
      y: series.y.map((value) => value.value ?? null),
    })),
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

function mapNativePlotDiagnostics(
  diagnostics: NativePlotPayloadDiagnostics,
): PlotPayloadDiagnostics {
  return {
    series_count: safeNumber(diagnostics.seriesCount, "Plots diagnostics series_count"),
    point_count: safeNumber(diagnostics.pointCount, "Plots diagnostics point_count"),
    raw_point_count: safeNumber(diagnostics.rawPointCount, "Plots diagnostics raw_point_count"),
    downsampled: diagnostics.downsampled,
    delta: diagnostics.delta,
    max_points: safeNumber(diagnostics.maxPoints, "Plots diagnostics max_points"),
    window_s: diagnostics.windowS ?? null,
  };
}

function mapNativeTelemetrySample(sample: NativeTelemetrySample): TelemetrySampleRecord {
  const timestamp = timestampIso(sample.timestamp);
  return {
    stream: sample.stream,
    sequence: safeNumber(sample.sequence, "Telemetry sample sequence"),
    sample: {
      ...(timestamp ? { timestamp } : {}),
      ...nativeValueObjectToJson(sample.values),
      ...nativeValueObjectToJson(sample.summary),
    },
    scope: recordScopeFromNative(sample.scope),
  };
}

function plotStreamKind(value: PlotStreamKind): PlotPayloadStreamMessage["kind"] {
  switch (value?.HarmosVariant.case) {
    case "Snapshot":
      return "snapshot";
    case "Append":
      return "append";
    case "Replace":
      return "replace";
    case "Error":
      return "error";
    default:
      throw new Error("Native plot stream returned an unspecified kind.");
  }
}

function cartesianMode(value: PlotCartesianMode | undefined): PlotCartesianFrame["mode"] {
  switch (value?.HarmosVariant.case) {
    case "Aligned":
      return "aligned";
    case "Faceted":
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

function boundedTelemetryLimit(value: number): number {
  if (!Number.isSafeInteger(value) || value < 0 || value > NATIVE_TELEMETRY_MAX_LIMIT) {
    throw new NativeApiLimitError(
      "Telemetry.ListSamples",
      `limit must be an integer between 0 and ${NATIVE_TELEMETRY_MAX_LIMIT}`,
    );
  }
  return value;
}

function boundedUint64(value: number, label: string): bigint {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new NativeApiLimitError(label, String(value));
  }
  return BigInt(value);
}

function boundedPlotPoints(value: number): bigint {
  if (!Number.isSafeInteger(value) || value < 0 || value > NATIVE_PLOT_MAX_POINTS) {
    throw new NativeApiLimitError(
      "Plots request",
      `max_points must be an integer between 0 and ${NATIVE_PLOT_MAX_POINTS}`,
    );
  }
  return BigInt(value);
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

function normalizeCancellation(
  error: unknown,
  session: SessionIdentity,
  signal?: AbortSignal,
): unknown {
  if (signal?.aborted || !session.isCurrent()) return abortError();
  return error;
}

function abortError(): DOMException {
  return new DOMException("Native plot request cancelled.", "AbortError");
}

function mapNativeAnalysisDefinition(value: NativeAnalysisDefinition): EvidenceAnalysisDefinition {
  return {
    id: value.id,
    version: value.version,
    label: value.label,
    description: value.description,
    parameters: value.parameters.map(mapParameter),
    outputs: value.outputs.map((output) => ({
      key: output.key,
      label: output.label,
      unit: output.unit,
      axis: output.axis ? { key: output.axis.key, unit: output.axis.unit } : undefined,
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
      parameters: nativeValueObjectToJson(value.selection.parameters),
    },
    source: {
      epoch: value.source.epoch,
      stream: value.source.stream,
      scope: recordScopeFromNative(value.source.scope),
      telemetryStreams: Object.fromEntries(
        value.source.telemetryStreams.map(({ key, value: cursor }) => [key, cursor.toString()]),
      ),
      recordSequence: value.source.recordSequence?.toString(),
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
