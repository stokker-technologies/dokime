/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native document stream transport and bounded assembly for the current document. */

/* === External Imports === */

import { create, toBinary } from "@bufbuild/protobuf";

/* === Local Imports === */

import {
  documentCard,
  documentCardLayout,
  documentPlot,
  documentSummaryTile,
  validateSummaryResult,
  unavailableReason,
  documentTableRows,
  documentText,
} from "@/lib/document-presentation";
import { assembleCapture, type DeclaredNode } from "@/lib/document-assembly";
import * as rpc from "@/lib/rpc/generated/client";
import {
  type DocumentCard,
  DocumentCardKind,
  type DocumentDensity,
  DocumentDensitySchema,
  type DocumentFrame,
  DocumentFrameSchema,
  type DocumentLayoutNode,
  type DocumentHeader,
  type DocumentBinding,
  type DocumentSummaryGroupDefinition,
  type DocumentSummaryTileDefinition,
  type DocumentRepeatSelection,
  type DocumentRepeatInstance,
  type DocumentSectionIdentity,
  type DocumentRail,
  DocumentRailSchema,
  type DocumentScope,
  DocumentScopeSchema,
  DocumentSummaryFormatSchema,
  type DocumentSummaryGroup,
  DocumentSummaryPresentation,
  DocumentSummaryStat,
  DocumentSummaryState,
  type DocumentSummaryTile,
  type DocumentTableRow,
  type DocumentWarning,
  PlotAxisKind as NativePlotAxisKind,
  type PlotStyle,
  PlotStyleSchema,
  ReferenceAxis,
} from "@/lib/rpc/generated/dokime/v1/documents_pb";
import {
  type AxisScale,
  AxisScaleSchema,
  type DataExpression,
  type FftDetrend,
  FftDetrendSchema,
  type FftWindow,
  FftWindowSchema,
  FilterOperator,
  FrequencyLevelUnit,
  FrequencyLevelUnitSchema,
  type PlotRequest,
  PlotType,
} from "@/lib/rpc/generated/dokime/v1/plots_pb";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type {
  DocumentDataExpression,
  DocumentViewCard,
  DocumentViewPayload,
  Metadata,
} from "@/lib/types";
import {
  enumTail,
  mapNativeValueFormat,
  nativeValueObjectToJson,
  nativeValueToJson,
} from "./native-values";

export const documentLimits = {
  bytes: 32 * 1024 * 1024,
  frameBytes: 512 * 1024,
  frames: 20_000,
  depth: 32,
} as const;
/** Indexes reference canonical protobuf messages; no alternate document DTO contract. */
export interface CompletedDocument {
  header: DocumentHeader;
  bindings: Map<string, DocumentBinding>;
  definitions: Map<string, DocumentSummaryGroupDefinition[]>;
  definitionTiles: Map<string, Map<bigint, DocumentSummaryTileDefinition[]>>;
  repeats: Map<string, DocumentRepeatSelection>;
  instances: Map<string, DocumentRepeatInstance[]>;
  sections: Map<string, DocumentSectionIdentity>;
  layout: DocumentLayoutNode[];
  cards: Map<string, DocumentCard>;
  rows: Map<string, DocumentTableRow[]>;
  groups: Map<string, DocumentSummaryGroup[]>;
  tiles: Map<string, Map<bigint, DocumentSummaryTile[]>>;
  warnings: DocumentWarning[];
}
export interface DocumentSelection {
  scope: DocumentScope;
  documentId: string;
  epoch: string;
}

export interface NativeDocumentRequest {
  cycleId: string;
  executionId: string;
  documentId: string;
  epoch: string;
}

export interface NativeDocumentSession extends SessionIdentity {
  /** Keep the stream attached to the owning generation until it settles. */
  own(dispose: () => void): () => void;
}

/** Open one generation-owned document stream and assemble it after clean EOF. */
export async function fetchNativeDocument(
  session: NativeDocumentSession,
  request: NativeDocumentRequest,
  signal?: AbortSignal,
): Promise<CompletedDocument> {
  const scope = createDocumentScope(request);
  let reader: Awaited<ReturnType<typeof rpc.documentsCaptureCurrentDocument>> | undefined;
  let dispose: (() => void) | undefined;
  const abort = () => {
    void reader?.cancel().catch(() => undefined);
  };
  signal?.addEventListener("abort", abort, { once: true });
  try {
    reader = await rpc.documentsCaptureCurrentDocument(session, {
      scope,
      documentId: request.documentId,
    });
    if (signal?.aborted || !session.isCurrent()) {
      await reader.cancel();
      throw new Error("Document transfer cancelled.");
    }
    dispose = session.own(() => {
      void reader?.cancel().catch(() => undefined);
    });
    return await assembleDocument(
      reader,
      {
        scope,
        documentId: request.documentId,
        epoch: request.epoch,
      },
      () => !signal?.aborted && session.isCurrent(),
    );
  } finally {
    signal?.removeEventListener("abort", abort);
    dispose?.();
  }
}

/** Decode Transport Values And Delegate Layout Assembly To The Shared Renderer. */
export function mapNativeDocument(document: CompletedDocument): DocumentViewPayload {
  return assembleCapture({
    document_id: document.header.documentId,
    metadata: metadataFromNative(document.header.metadata, document.header.documentId),
    sources: [...document.cards.keys()],
    bindings: [...document.bindings.values()].map((binding) => {
      const source = document.cards.get(binding.sourceId);
      if (!source) invalid("binding source is missing");
      return { source_id: binding.sourceId, card: mapNativeCard(source, binding, document) };
    }),
    nodes: mapNativeLayout(document.layout),
    repeats: new Map(
      [...document.instances].map(([id, instances]) => [
        id,
        instances.map((instance) => ({ suffix: instance.suffix, node_id: instance.nodeId })),
      ]),
    ),
    sections: new Map(
      [...document.sections].map(([id, section]) => [
        id,
        {
          metadata: metadataFromNative(section.metadata, id),
          unavailable: section.unavailable,
        },
      ]),
    ),
  });
}

function mapNativeCard(
  card: DocumentCard,
  binding: DocumentBinding,
  document: CompletedDocument,
): DocumentViewCard {
  const id = binding.metadata?.identifier;
  if (!id) invalid("binding metadata is missing");
  const metadata = metadataFromNative(binding.metadata, id);
  if (binding.result.case === "summary" && card.body.case === "summary") {
    return documentCard(metadata, "overview", {
      groups: (document.groups.get(id) ?? []).map((group) => ({
        group_id: group.metadata?.identifier ?? `${id}-${group.ordinal}`,
        title: group.metadata?.label ?? "Summary",
        metrics: (document.tiles.get(id)?.get(group.ordinal) ?? []).map((tile) =>
          documentSummaryTile({
            metadata: metadataFromNative(tile.metadata, `${id}-${group.ordinal}-${tile.ordinal}`),
            value: nativeValueToJson(tile.value),
            unavailable: tile.unavailable,
            unit: tile.unit,
            format: enumTail(DocumentSummaryFormatSchema, tile.format, "auto"),
            presentation: mapSummaryPresentation(tile.presentation),
            state: tile.state === undefined ? null : mapSummaryState(tile.state),
            stat: tile.stat === DocumentSummaryStat.LATEST ? "latest" : undefined,
          }),
        ),
      })),
    });
  }
  if (
    binding.result.case === "plot" &&
    card.body.case === "owner" &&
    card.body.value.kind === DocumentCardKind.PLOT
  ) {
    const plot = mapNativePlot(binding.result.value, id);
    plot.group = document.header.metadata?.label || document.header.documentId;
    return documentCard(metadata, "plots", { plots: [plot] });
  }
  if (binding.result.case !== "literal" || !binding.result.value || card.body.case === "summary")
    invalid("binding result differs from its declaration");
  switch (card.body.case) {
    case "markdown":
      return documentCard(metadata, "overview", {
        groups: [documentText(metadata, card.body.value)],
      });
    case "table":
      return documentCard(metadata, "table", {
        rows: documentTableRows(
          id,
          (document.rows.get(binding.sourceId) ?? []).map((row) =>
            nativeValueObjectToJson(row.values),
          ),
        ),
      });
    case "owner": {
      const kind = mapCardKind(card.body.value.kind);
      if (!kind || kind === "plots") invalid("literal owner kind is invalid");
      return documentCard(metadata, kind);
    }
    default:
      return invalid("card body is missing");
  }
}

/** Rebuild The Authored Tree From Its Bounded Pre-Order Wire List. */
function mapNativeLayout(nodes: readonly DocumentLayoutNode[]): DeclaredNode[] {
  const children = new Map<string, DocumentLayoutNode[]>();
  for (const node of nodes) {
    const key = node.parentOrdinal?.toString() ?? "root";
    const entries = children.get(key) ?? [];
    entries.push(node);
    children.set(key, entries);
  }
  const mapNodes = (key: string): DeclaredNode[] =>
    (children.get(key) ?? []).map((node) => {
      switch (node.node.case) {
        case "section":
          return {
            type: "section",
            node_id: node.node.value.nodeId,
            repeat: Boolean(node.node.value.repeat),
            children: mapNodes(node.ordinal.toString()),
          };
        case "placement":
          return {
            type: "card",
            card_id: node.node.value.cardId,
            layout: mapNativePlacementLayout(node.node.value),
          };
        default:
          return invalid("layout node kind is missing");
      }
    });
  return mapNodes("root");
}

function metadataFromNative(
  metadata:
    | { identifier: string; label: string; description?: string; tags?: string[] }
    | undefined,
  fallback: string,
): Metadata {
  return {
    identifier: metadata?.identifier || fallback,
    label: metadata?.label || metadata?.identifier || fallback,
    description: metadata?.description ?? null,
    ...(metadata?.tags?.length ? { tags: metadata.tags } : {}),
  };
}

function mapNativePlacementLayout(placement?: {
  columnSpan?: { base: number; sm?: number; md?: number; lg?: number; xl?: number };
  rowSpan?: number;
  order?: number;
  density?: DocumentDensity;
  rail?: DocumentRail;
}) {
  return documentCardLayout({
    col_span: {
      base: placement?.columnSpan?.base || 12,
      sm: placement?.columnSpan?.sm,
      md: placement?.columnSpan?.md,
      lg: placement?.columnSpan?.lg,
      xl: placement?.columnSpan?.xl,
    },
    row_span: placement?.rowSpan || 1,
    order: placement?.order,
    density: mapDensity(placement?.density),
    rail: mapRail(placement?.rail),
  });
}

function mapDensity(value?: DocumentDensity): "compact" | "normal" | "spacious" {
  return enumTail(DocumentDensitySchema, value, "normal");
}

function mapRail(value?: DocumentRail): "main" | "right" | "left" {
  return enumTail(DocumentRailSchema, value, "main");
}

function mapCardKind(value: DocumentCardKind): DocumentViewCard["kind"] | null {
  switch (value) {
    case DocumentCardKind.PLOT:
      return "plots";
    case DocumentCardKind.STEPS:
      return "steps";
    case DocumentCardKind.CRITERIA:
      return "criteria";
    case DocumentCardKind.TRACEABILITY:
      return "traceability";
    case DocumentCardKind.NOTES:
      return "notes";
    case DocumentCardKind.RECORDS:
      return "records";
    case DocumentCardKind.ARTIFACTS:
      return "artifacts";
    case DocumentCardKind.MEDIA:
      return "media";
    case DocumentCardKind.HEALTH:
      return "health";
    case DocumentCardKind.TELEMETRY_CONTEXT:
      return "telemetry_context";
    default:
      return null;
  }
}

function mapSummaryPresentation(
  value: DocumentSummaryPresentation,
): "value" | "progress" | "badge" {
  switch (value) {
    case DocumentSummaryPresentation.PROGRESS:
      return "progress";
    case DocumentSummaryPresentation.BADGE:
      return "badge";
    default:
      return "value";
  }
}

function mapSummaryState(
  value: DocumentSummaryState,
): "ok" | "warning" | "critical" | "muted" | null {
  switch (value) {
    case DocumentSummaryState.OK:
      return "ok";
    case DocumentSummaryState.WARNING:
      return "warning";
    case DocumentSummaryState.CRITICAL:
      return "critical";
    case DocumentSummaryState.MUTED:
      return "muted";
    default:
      return null;
  }
}

function mapNativePlot(
  definition: NonNullable<Extract<DocumentCard["body"], { case: "owner" }>["value"]["plot"]>,
  fallbackId: string,
): DocumentViewCard["plots"][number] {
  const request = definition.request;
  if (!request) throw new Error(`Native plot ${fallbackId} has no request definition.`);
  const metadata = definition.metadata;
  const presentation = definition.presentation;
  const plotId = request.plotId || metadata?.identifier || fallbackId;
  const xExpression = mapNativeExpression(request.xExpression);
  const yExpressions = request.yExpressions.map((expression) => {
    const mapped = mapNativeExpression(expression);
    if (!mapped) invalid("plot y expression is missing");
    return mapped;
  });
  const seriesByExpression = mapNativeExpression(request.seriesByExpression);
  const fft = presentation?.fft ?? request.fft;
  const interval = presentation?.interval ?? request.interval;
  const histogram = presentation?.histogram ?? request.histogram;
  return documentPlot({
    plot_id: plotId,
    title: metadata?.label,
    stream: request.stream,
    plot_type: mapNativePlotType(request.plotType),
    style: mapPlotStyle(presentation?.style),
    x_scale: mapAxisScale(request.xScale),
    y_scale: mapAxisScale(request.yScale),
    analysis: request.analysis
      ? {
          id: request.analysis.id,
          version: request.analysis.version,
          parameters: nativeValueObjectToJson(request.analysis.parameters),
        }
      : null,
    fft: {
      ...(fft?.sampleRateHz === undefined ? {} : { sample_rate_hz: fft.sampleRateHz }),
      window: mapFftWindow(fft?.window),
      detrend: mapFftDetrend(fft?.detrend),
      level_unit: mapFrequencyLevelUnit(fft?.levelUnit),
      include_dc: fft?.includeDc,
      ...(fft?.minFrequencyHz === undefined ? {} : { min_frequency_hz: fft.minFrequencyHz }),
      ...(fft?.maxFrequencyHz === undefined ? {} : { max_frequency_hz: fft.maxFrequencyHz }),
    },
    interval: {
      bin_width: interval?.binWidth,
      sigma: interval?.sigma,
      clip_to_observed: interval?.clipToObserved,
    },
    histogram: {
      bins: histogram?.bins === undefined ? undefined : Number(histogram.bins),
      ...(histogram?.unit === undefined ? {} : { unit: histogram.unit }),
    },
    y: [...request.yKeys],
    x_expression: xExpression,
    y_expressions: yExpressions,
    series_by_expression: seriesByExpression,
    z_expression: mapNativeExpression(definition.zExpression ?? request.zExpression),
    z_stream: request.zStream ?? null,
    z_field: request.zField ?? null,
    x: request.xKey,
    series_by: request.seriesBy,
    unit: request.unit ?? null,
    group: presentation?.group,
    description: metadata?.description ?? null,
    marker_record_types: [...request.markerRecordTypes],
    filters: request.filters.map(mapNativeFilter),
    reference_lines: (presentation?.referenceLines ?? []).map((line) => ({
      axis: line.axis === ReferenceAxis.X ? "x" : "y",
      value: line.value ?? null,
      source: mapNativeExpression(line.source),
      source_scale: line.sourceScale,
      label: line.label ?? null,
      color: line.color,
    })),
    reference_bands: (presentation?.referenceBands ?? []).map((band) => ({
      axis: band.axis === ReferenceAxis.X ? "x" : "y",
      min: band.min ?? null,
      max: band.max ?? null,
      center: mapNativeExpression(band.center),
      tolerance: mapNativeExpression(band.tolerance),
      label: band.label ?? null,
      color: band.color,
    })),
    live_window_s: presentation?.liveWindowS ?? request.windowS ?? null,
    x_format: mapNativeValueFormat(presentation?.xFormat),
    y_format: mapNativeValueFormat(presentation?.yFormat),
    x_axis: mapNativePlotAxis(presentation?.xAxis, presentation?.xAxisUnit),
  });
}

/**
 * What the plot says its x axis measures, or nothing when it says nothing.
 *
 * `PLOT_AXIS_KIND_UNSPECIFIED` is a payload written before the declaration
 * existed, so it stays `null` and the renderer's column-name fallback still
 * decides. The contract declares `x_axis_unit` present for an elapsed axis;
 * seconds is the row unit the runtime's own elapsed columns carry, so a writer
 * that declared the axis and omitted the unit still gets a labelled duration
 * rather than an epoch date.
 */
function mapNativePlotAxis(
  kind: NativePlotAxisKind | undefined,
  unit: string | undefined,
): DocumentViewCard["plots"][number]["x_axis"] {
  switch (kind) {
    case NativePlotAxisKind.WALL_TIME:
      return { kind: "wall_time" };
    case NativePlotAxisKind.ELAPSED:
      return { kind: "elapsed", unit: unit?.trim() || "s" };
    default:
      return null;
  }
}

function mapNativePlotType(value: PlotType): DocumentViewCard["plots"][number]["plot_type"] {
  switch (value) {
    case PlotType.BAR:
      return "bar";
    case PlotType.GROUPED_BAR:
    case PlotType.STACKED_BAR:
    case PlotType.PIE:
    case PlotType.DONUT:
    case PlotType.PARETO:
    case PlotType.AGGREGATE_TABLE:
      throw new Error(`Unsupported document plot type: ${PlotType[value].toLowerCase()}`);
    case PlotType.XY_SCATTER:
      return "xy_scatter";
    case PlotType.XY_LINE:
      return "xy_line";
    case PlotType.XY_INTERVAL:
      return "xy_interval";
    case PlotType.TRAJECTORY:
      return "trajectory";
    case PlotType.HEATMAP:
      return "heatmap";
    case PlotType.XY_VALUE_HEATMAP:
      return "xy_value_heatmap";
    case PlotType.WIFI_CHANNEL_OCCUPANCY:
      return "wifi_channel_occupancy";
    case PlotType.FREQUENCY_SERIES:
      return "frequency_series";
    case PlotType.HISTOGRAM:
      return "histogram";
    case PlotType.BOX_SUMMARY:
      return "box_summary";
    case PlotType.TIME_SERIES:
      return "time_series";
    default:
      throw new Error(`Unsupported document plot type: ${value}`);
  }
}

function mapPlotStyle(value?: PlotStyle): "line" | "points" | "line_points" | "step" {
  return enumTail(PlotStyleSchema, value, "line");
}

function mapAxisScale(value?: AxisScale): "linear" | "log" {
  return enumTail(AxisScaleSchema, value, "linear");
}

function mapFftWindow(value?: FftWindow): "hann" | "rectangular" {
  return enumTail(FftWindowSchema, value, "hann");
}

function mapFftDetrend(value?: FftDetrend): "none" | "mean" {
  return enumTail(FftDetrendSchema, value, "mean");
}

/* The level units keep their proto casing, so every member is an override. */
function mapFrequencyLevelUnit(value?: FrequencyLevelUnit): "linear" | "dB" | "dBu" | "dBm" {
  return enumTail(FrequencyLevelUnitSchema, value, "linear", {
    [FrequencyLevelUnit.DB]: "dB",
    [FrequencyLevelUnit.DBU]: "dBu",
    [FrequencyLevelUnit.DBM]: "dBm",
  });
}

function mapNativeFilter(filter: PlotRequest["filters"][number]): {
  expression: DocumentDataExpression;
  operator: "eq" | "ne" | "gt" | "gte" | "lt" | "lte" | "contains" | "exists";
  value?: unknown;
} {
  if (!filter.expression) throw new Error("Native plot filter has no expression.");
  return {
    expression: mapNativeExpression(filter.expression) ?? {},
    operator: mapFilterOperator(filter.operator),
    ...(filter.value === undefined ? {} : { value: nativeValueToJson(filter.value) }),
  };
}

function mapFilterOperator(
  value: FilterOperator,
): "eq" | "ne" | "gt" | "gte" | "lt" | "lte" | "contains" | "exists" {
  switch (value) {
    case FilterOperator.NE:
      return "ne";
    case FilterOperator.GT:
      return "gt";
    case FilterOperator.GTE:
      return "gte";
    case FilterOperator.LT:
      return "lt";
    case FilterOperator.LTE:
      return "lte";
    case FilterOperator.CONTAINS:
      return "contains";
    case FilterOperator.EXISTS:
      return "exists";
    default:
      return "eq";
  }
}

function mapNativeExpression(value?: DataExpression): DocumentDataExpression | null {
  if (!value) return null;
  if (!value.source.case || value.transforms.length > 32)
    invalid("expression source or transform bound is invalid");
  const source = (() => {
    switch (value.source.case) {
      case "telemetry":
        return {
          type: "telemetry" as const,
          stream: value.source.value.stream,
          field: value.source.value.field,
        };
      case "parameter":
        return { type: "parameter" as const, name: value.source.value.name };
      case "dimension":
        return { type: "dimension" as const, name: value.source.value.name };
      case "record":
        return { type: "record" as const, record_type: value.source.value.recordType };
      case "artifact":
        return { type: "artifact" as const, artifact: value.source.value.artifact };
      default:
        return invalid("expression kind is missing");
    }
  })();

  const transforms = value.transforms.map((transform) => {
    switch (transform.kind.case) {
      case "count":
      case "mean":
      case "median":
      case "min":
      case "max":
        return { type: transform.kind.case };
      case "timeBetween":
        return {
          type: "time_between" as const,
          previous_record_type: transform.kind.value.previousRecordType,
        };
      case "telemetryAtOrBefore":
        return {
          type: "telemetry_at_or_before" as const,
          stream: transform.kind.value.stream,
          field: transform.kind.value.field,
        };
      case "detail":
        return { type: "detail" as const, name: transform.kind.value.name };
      case "perCase":
        return { type: "per_case" as const };
      default:
        return invalid("expression kind is missing");
    }
  });
  return {
    source,
    ...(value.label === undefined ? {} : { label: value.label }),
    ...(transforms.length ? { transforms } : {}),
  };
}

function createDocumentScope(request: NativeDocumentRequest) {
  return create(DocumentScopeSchema, {
    cycleId: request.cycleId,
    executionId: request.executionId,
  });
}

/** Keep this adapter independent from the React session provider. */
function invalid(message: string): never {
  throw new Error(`Document transfer rejected: ${message}`);
}
/** Consumes through successful iterator termination. A completion frame alone is insufficient. */
export async function assembleDocument(
  frames: AsyncIterable<DocumentFrame>,
  expected: DocumentSelection,
  active: () => boolean,
): Promise<CompletedDocument> {
  let header: DocumentHeader | undefined;
  const layout: DocumentLayoutNode[] = [];
  const byOrdinal = new Map<bigint, DocumentLayoutNode>();
  let path: bigint[] = [];
  const cards = new Map<string, DocumentCard>();
  const bindings = new Map<string, DocumentBinding>();
  const rows = new Map<string, DocumentTableRow[]>();
  const groups = new Map<string, DocumentSummaryGroup[]>();
  const tiles = new Map<string, Map<bigint, DocumentSummaryTile[]>>();
  const definitions = new Map<string, DocumentSummaryGroupDefinition[]>();
  const definitionTiles = new Map<string, Map<bigint, DocumentSummaryTileDefinition[]>>();
  const repeats = new Map<string, DocumentRepeatSelection>();
  const instances = new Map<string, DocumentRepeatInstance[]>();
  const sections = new Map<string, DocumentSectionIdentity>();
  const warnings: DocumentWarning[] = [];
  let bytes = 0;
  let count = 0;
  let complete = false;
  for await (const item of frames) {
    if (!active()) invalid("selection changed or transfer cancelled");
    if (complete) invalid("a frame followed completion");
    if (++count > documentLimits.frames) invalid("frame budget exceeded");
    const length = toBinary(DocumentFrameSchema, item).byteLength;
    bytes += length;
    if (length > documentLimits.frameBytes || bytes > documentLimits.bytes)
      invalid("capture byte budget exceeded");
    const frame = item.frame;
    if (!header && frame.case !== "header") invalid("header must be first");
    switch (frame.case) {
      case "header": {
        if (header) invalid("duplicate header");
        const value = frame.value;
        if (
          value.documentId !== expected.documentId ||
          value.scope?.cycleId !== expected.scope.cycleId ||
          value.scope?.executionId !== expected.scope.executionId ||
          value.frontiers?.processEpoch !== expected.epoch ||
          !value.metadata
        )
          invalid("document identity or scope does not match the request");
        header = value;
        break;
      }
      case "layoutNode": {
        const node = frame.value;
        if (node.ordinal !== BigInt(layout.length) || !node.node.case || layout.length >= 4096)
          invalid("layout ordinals are not contiguous or exceed the budget");
        if (node.parentOrdinal === undefined) path = [];
        else {
          const parent = byOrdinal.get(node.parentOrdinal);
          const ancestor = path.indexOf(node.parentOrdinal);
          if (!parent || parent.node.case === "placement" || ancestor < 0)
            invalid("layout parent is missing or outside the pre-order path");
          path = path.slice(0, ancestor + 1);
        }
        path.push(node.ordinal);
        if (path.length > documentLimits.depth) invalid("layout depth budget exceeded");
        if (
          node.node.case === "section" &&
          (!node.node.value.nodeId ||
            !node.node.value.metadata ||
            (node.node.value.repeat && !node.node.value.repeat.value))
        )
          invalid("section declaration is incomplete");
        byOrdinal.set(node.ordinal, node);
        layout.push(node);
        break;
      }
      case "card": {
        const card = frame.value;
        const id = card.metadata?.identifier;
        if (!id || cards.has(id) || !card.body.case || cards.size >= 4096)
          invalid("missing or duplicate card identity");
        cards.set(id, card);
        if (card.body.case === "table") rows.set(id, []);
        if (card.body.case === "summary") {
          definitions.set(id, []);
          definitionTiles.set(id, new Map());
        }
        break;
      }
      case "binding": {
        const binding = frame.value;
        const id = binding.metadata?.identifier;
        const source = cards.get(binding.sourceId);
        if (!id || bindings.has(id) || !source || !binding.result.case || bindings.size >= 4096)
          invalid("binding source or identity is invalid");
        const expectedResult =
          source.body.case === "summary"
            ? "summary"
            : source.body.case === "owner" && source.body.value.kind === DocumentCardKind.PLOT
              ? "plot"
              : "literal";
        if (
          binding.result.case !== expectedResult ||
          (binding.result.case === "literal" && !binding.result.value)
        )
          invalid("binding result differs from its declaration");
        bindings.set(id, binding);
        if (binding.result.case === "summary") {
          groups.set(id, []);
          tiles.set(id, new Map());
        }
        break;
      }
      case "tableRow": {
        const row = frame.value;
        const target = rows.get(row.cardId);
        if (!target || row.ordinal !== BigInt(target.length) || !row.values)
          invalid("table row has no matching card or contiguous ordinal");
        target.push(row);
        break;
      }
      case "groupDefinition": {
        const group = frame.value;
        const target = definitions.get(group.cardId);
        if (!target || group.ordinal !== BigInt(target.length) || !group.metadata)
          invalid("summary definition group is invalid");
        target.push(group);
        definitionTiles.get(group.cardId)?.set(group.ordinal, []);
        break;
      }
      case "tileDefinition": {
        const tile = frame.value;
        const target = definitionTiles.get(tile.cardId)?.get(tile.groupOrdinal);
        if (!target || tile.ordinal !== BigInt(target.length) || !tile.metadata || !tile.value)
          invalid("summary definition tile is invalid");
        target.push(tile);
        break;
      }
      case "summaryGroup": {
        const group = frame.value;
        const target = groups.get(group.cardId);
        if (!target || group.ordinal !== BigInt(target.length) || !group.metadata)
          invalid("summary group has no matching binding or contiguous ordinal");
        target.push(group);
        tiles.get(group.cardId)?.set(group.ordinal, []);
        break;
      }
      case "summaryTile": {
        const tile = frame.value;
        const target = tiles.get(tile.cardId)?.get(tile.groupOrdinal);
        if (!target || tile.ordinal !== BigInt(target.length) || !tile.metadata)
          invalid("summary tile has no matching group or contiguous ordinal");
        validateSummaryResult(tile);
        target.push(tile);
        break;
      }
      case "repeatSelection": {
        const repeat = frame.value;
        if (!repeat.nodeId || repeats.has(repeat.nodeId) || repeats.size >= 4096)
          invalid("repeat selection identity is invalid");
        repeats.set(repeat.nodeId, repeat);
        instances.set(repeat.nodeId, []);
        break;
      }
      case "repeatInstance": {
        const instance = frame.value;
        const target = instances.get(instance.selectionId);
        if (
          !target ||
          instance.ordinal !== BigInt(target.length) ||
          !instance.nodeId ||
          !instance.suffix ||
          !instance.filter?.expression ||
          !instance.filter.value
        )
          invalid("repeat instance is incomplete");
        target.push(instance);
        break;
      }
      case "sectionIdentity": {
        const section = frame.value;
        if (
          !section.nodeId ||
          !section.metadata ||
          sections.has(section.nodeId) ||
          sections.size >= 4096
        )
          invalid("section identity is invalid");
        unavailableReason(section.unavailable);
        sections.set(section.nodeId, section);
        break;
      }
      case "warning":
        warnings.push(frame.value);
        break;
      case "completion": {
        const value = frame.value;
        if (
          value.frameCount !== BigInt(count) ||
          value.cardCount !== BigInt(cards.size) ||
          value.layoutNodeCount !== BigInt(layout.length) ||
          value.bindingCount !== BigInt(bindings.size) ||
          value.repeatCount !== BigInt(repeats.size) ||
          value.sectionCount !== BigInt(sections.size)
        )
          invalid("completion counts do not match the received capture");
        complete = true;
        break;
      }
      default:
        invalid("unknown frame");
    }
  }
  if (!active()) invalid("selection changed or transfer cancelled");
  if (!complete || !header) invalid("transfer ended before completion");
  for (const [id, card] of cards) {
    if (
      card.body.case === "table" &&
      card.body.value.rowCount !== BigInt(rows.get(id)?.length ?? 0)
    )
      invalid("table row count is incomplete");
    if (card.body.case === "summary") {
      const entries = definitions.get(id) ?? [];
      if (card.body.value.groupCount !== BigInt(entries.length))
        invalid("summary definition group count is incomplete");
      for (const group of entries)
        if (group.tileCount !== BigInt(definitionTiles.get(id)?.get(group.ordinal)?.length ?? 0))
          invalid("summary definition tile count is incomplete");
    }
  }
  for (const [id, binding] of bindings) {
    if (binding.result.case !== "summary") continue;
    const entries = groups.get(id) ?? [];
    if (binding.result.value.groupCount !== BigInt(entries.length))
      invalid("summary group count is incomplete");
    for (const group of entries)
      if (group.tileCount !== BigInt(tiles.get(id)?.get(group.ordinal)?.length ?? 0))
        invalid("summary tile count is incomplete");
  }
  for (const [id, repeat] of repeats)
    if (repeat.instanceCount !== BigInt(instances.get(id)?.length ?? 0))
      invalid("repeat instance count is incomplete");
  for (const node of layout)
    if (node.node.case === "placement" && !cards.has(node.node.value.cardId))
      invalid("layout references a missing source card");
  return {
    header,
    layout,
    cards,
    bindings,
    rows,
    groups,
    tiles,
    definitions,
    definitionTiles,
    repeats,
    instances,
    sections,
    warnings,
  };
}
