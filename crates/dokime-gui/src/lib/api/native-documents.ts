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

import { toBinary } from "@bufbuild/protobuf";

/* === Local Imports === */

import * as documentWire from "@generated/dokime/v1/DocumentsMessages/CaptureCurrentDocument/messages_pb";
import { assembleCapture, type DeclaredNode } from "@/lib/document-assembly";
import {
  documentCard,
  documentCardLayout,
  documentPlot,
  documentSummaryTile,
  documentTableRows,
  documentText,
  unavailableReason,
  validateSummaryResult,
} from "@/lib/document-presentation";
import { failureOf } from "@/lib/notification-details";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { DocumentDataExpression, DocumentDownsampleTechnique, DocumentViewCard, DocumentViewPayload, Metadata } from "@/lib/types";
import { aggregateName } from "./native-aggregate";
import { enumTail, mapNativeValueFormat, nativeValueObjectToJson, nativeValueToJson, safeNumber } from "./native-values";

const DOCUMENTS_ENCODED_DEPTH = 32;
const DOCUMENTS_EXPANDED_NODES = 4_096;
const DOCUMENTS_MATERIALIZED_BYTES = 33_554_432;
const GUI_PLOT_TRANSFORMS = 32;

/* === Canonical Types === */

type DocumentFrame = documentWire.Row3;
const DocumentFrameSchema = documentWire.Row3Schema;
type VariantValue<T, K extends string> =
  Extract<NonNullable<NonNullable<T> extends { HarmosVariant: infer V } ? V : never>, { case: K }> extends { value: { field0?: infer F } }
    ? NonNullable<F>
    : never;
type FrameValue<K extends string> = VariantValue<Wire<DocumentFrame>["frame"], K>;
type DocumentCard = FrameValue<"Card">;
type DocumentHeader = FrameValue<"Header">;
type DocumentBinding = FrameValue<"Binding">;
type DocumentLayoutNode = FrameValue<"LayoutNode">;
type DocumentPlacement = VariantValue<DocumentLayoutNode["node"], "Placement">;
type DocumentDensity = NonNullable<DocumentPlacement["density"]>;
type DocumentRail = NonNullable<DocumentPlacement["rail"]>;
type DocumentSummaryGroupDefinition = FrameValue<"GroupDefinition">;
type DocumentSummaryTileDefinition = FrameValue<"TileDefinition">;
type DocumentRepeatSelection = FrameValue<"RepeatSelection">;
type DocumentRepeatInstance = FrameValue<"RepeatInstance">;
type DocumentSectionIdentity = FrameValue<"SectionIdentity">;
type DocumentScope = NonNullable<Wire<documentWire.Row1>["scope"]>;
type DocumentSummaryGroup = FrameValue<"SummaryGroup">;
type DocumentSummaryTile = FrameValue<"SummaryTile">;
type DocumentSummaryPresentation = NonNullable<DocumentSummaryTile["presentation"]>;
type DocumentSummaryState = NonNullable<DocumentSummaryTile["state"]>;
type DocumentTableRow = FrameValue<"TableRow">;
type DocumentWarning = FrameValue<"Warning">;
type NativePlotDefinition = VariantValue<DocumentBinding["result"], "Plot">;
type NativePlotPresentation = NonNullable<NativePlotDefinition["presentation"]>;
type NativePlotAxisKind = NonNullable<NativePlotPresentation["xAxis"]>;
type PlotStyle = NonNullable<NativePlotPresentation["style"]>;
type DataExpression = NonNullable<DocumentSummaryTileDefinition["value"]>;
type PlotRequest = NonNullable<NativePlotDefinition["request"]>;
type AxisScale = NonNullable<PlotRequest["yScale"]>;
type FftWindow = NonNullable<NonNullable<PlotRequest["fft"]>["window"]>;
type FftDetrend = NonNullable<NonNullable<PlotRequest["fft"]>["detrend"]>;
type FrequencyLevelUnit = NonNullable<NonNullable<PlotRequest["fft"]>["levelUnit"]>;
type PlotType = NonNullable<PlotRequest["plotType"]>;
type FilterOperator = NonNullable<DocumentCard["filters"][number]["operator"]>;
type DocumentCardKind = NonNullable<VariantValue<DocumentCard["body"], "Owner">["kind"]>;
type NativeDownsampleTechnique = NonNullable<PlotRequest["technique"]>;

/** Indexes reference canonical protobuf messages; no alternate document DTO contract. */
export interface CompletedDocument {
  complete?: boolean;
  page?: { current: string; next?: string };
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
  structureOnly?: boolean;
  cardIds?: string[];
}

export interface NativeDocumentRequest {
  cycleId: string;
  executionId: string;
  documentId: string;
  epoch: string;
  structureOnly?: boolean;
  cardIds?: string[];
  page?: string;
}

export interface NativeDocumentSession extends SessionIdentity {
  /** Keep the stream attached to the owning generation until it settles. */
  own(dispose: () => void): () => void;
}

/** One operator edit of a plot's display settings on a running cycle's frozen document. */
export interface PlotSettingsUpdate {
  cycleId: string;
  executionId: string;
  documentId: string;
  plotId: string;
  /** `null` restores the host default; `0` keeps every sample. */
  displayPoints: number | null;
  downsample: DocumentDownsampleTechnique;
}

/**
 * Persist a plot's display settings into the execution's frozen document, so
 * the live view and the sealed report render the same reduction.
 *
 */
export async function updatePlotSettings(session: SessionIdentity, update: PlotSettingsUpdate): Promise<void> {
  if (!session.isCurrent()) throw new Error("The backend connection was replaced.");
  if (!update.cycleId || !update.executionId || !update.documentId || !update.plotId) {
    throw new Error("An execution document and a plot are required.");
  }
  const variants = { auto: "Auto", raw: "Raw", min_max: "MinMax", lttb: "Lttb" } as const;
  await rpc.documentsUpdatePlotSettings(session, {
    scope: { cycleId: update.cycleId, executionId: update.executionId },
    documentId: update.documentId,
    plotId: update.plotId,
    displayPoints: update.displayPoints ?? undefined,
    downsample: { HarmosVariant: { case: variants[update.downsample], value: {} } },
  });
  if (!session.isCurrent()) throw new Error("The backend connection was replaced.");
}

/** Open one generation-owned document stream and assemble it after clean EOF. */
export async function fetchNativeDocument(
  session: NativeDocumentSession,
  request: NativeDocumentRequest,
  signal?: AbortSignal,
  previous?: CompletedDocument | null,
  publish?: (document: CompletedDocument) => void,
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
      structureOnly: request.structureOnly ?? false,
      cardIds: request.cardIds ?? [],
      page: request.page,
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
      { scope, documentId: request.documentId, epoch: request.epoch, structureOnly: request.structureOnly, cardIds: request.cardIds },
      () => !signal?.aborted && session.isCurrent(),
      publish,
      async (header) => {
        if (
          previous &&
          previous.complete !== false &&
          ![...previous.bindings].some(
            ([id, binding]) =>
              (!request.cardIds?.length || request.cardIds.includes(id)) && binding.result?.HarmosVariant.case === "Failure",
          ) &&
          sameFrontiers(header.frontiers, previous.header.frontiers)
        ) {
          await reader?.cancel();
          return previous;
        }
        return undefined;
      },
    );
  } finally {
    signal?.removeEventListener("abort", abort);
    dispose?.();
  }
}

export function mapNativeDocumentCard(document: CompletedDocument, id: string): DocumentViewCard {
  const binding = document.bindings.get(id);
  const source = binding && document.cards.get(binding.sourceId);
  if (!binding || !source) invalid("requested card is missing");
  return mapNativeCard(source, binding, document);
}

export function nativeDocumentCardKey(document: CompletedDocument, id: string): string {
  const binding = document.bindings.get(id);
  const source = binding && document.cards.get(binding.sourceId);
  if (!binding || !source) invalid("requested card declaration is missing");
  const groups = document.definitions.get(binding.sourceId);
  const tiles = [...(document.definitionTiles.get(binding.sourceId) ?? [])];
  return JSON.stringify([source, binding.metadata, groups, tiles], (_key, value) => (typeof value === "bigint" ? value.toString() : value));
}

/** Whether two captures read the same inputs: same state revision, telemetry cut and artifacts. */
export function sameFrontiers(left: DocumentHeader["frontiers"], right: DocumentHeader["frontiers"]): boolean {
  if (!left || !right) return false;
  return frontierKey(left) === frontierKey(right);
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
    pendingRepeats: new Set([...document.repeats].filter(([, repeat]) => repeat.pending).map(([id]) => id)),
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

function mapNativeCard(card: DocumentCard, binding: DocumentBinding, document: CompletedDocument): DocumentViewCard {
  const id = binding.metadata?.identifier;
  if (!id) invalid("binding metadata is missing");
  const metadata = metadataFromNative(binding.metadata, id);
  const result = binding.result?.HarmosVariant;
  const body = card.body?.HarmosVariant;
  if (!result?.case || !body?.case) invalid("card body or binding result is missing");
  if (result.case === "Pending" || result.case === "Failure") {
    const kind =
      body.case === "Table" ? "table" : body.case === "Owner" ? (mapCardKind(body.value.field0?.kind) ?? "overview") : "overview";
    if (result.case === "Pending") return documentCard(metadata, kind, { loading: true });
    const failure = result.value.field0;
    if (!failure?.code || !failure.message.trim()) invalid("card failure is missing");
    return documentCard(metadata, kind, { failure: failureOf(failure, metadata.label) });
  }
  if (result.case === "Summary" && body.case === "Summary") {
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
            format: enumTail(undefined, tile.format, "auto"),
            presentation: mapSummaryPresentation(tile.presentation),
            state: tile.state === undefined ? null : mapSummaryState(tile.state),
            stat: tile.stat?.HarmosVariant.case === "Latest" ? "latest" : undefined,
          }),
        ),
      })),
    });
  }
  if (result.case === "Plot" && body.case === "Owner" && body.value.field0?.kind?.HarmosVariant.case === "Plot") {
    if (!result.value.field0) invalid("plot binding is missing");
    const plot = mapNativePlot(result.value.field0, id);
    return documentCard(metadata, "plots", { plots: [plot] });
  }
  if (result.case !== "Literal" || !result.value.field0 || body.case === "Summary") invalid("binding result differs from its declaration");
  switch (body.case) {
    case "Markdown":
      return documentCard(metadata, "overview", {
        groups: [documentText(metadata, body.value.field0)],
      });
    case "Table":
      return documentCard(metadata, "table", {
        rows: documentTableRows(
          id,
          (document.rows.get(binding.sourceId) ?? []).map((row) => nativeValueObjectToJson(row.values)),
        ),
      });
    case "Owner": {
      const kind = mapCardKind(body.value.field0?.kind);
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
      const kind = node.node?.HarmosVariant;
      if (!kind?.case || !kind.value.field0) invalid("layout node kind is missing");
      switch (kind.case) {
        case "Section":
          if (!kind.value.field0) invalid("layout node payload is missing");
          return {
            type: "section",
            metadata: metadataFromNative(kind.value.field0.metadata, kind.value.field0.nodeId),
            node_id: kind.value.field0.nodeId,
            repeat: Boolean(kind.value.field0.repeat),
            children: mapNodes(node.ordinal.toString()),
          };
        case "Placement":
          if (!kind.value.field0) invalid("layout node payload is missing");
          return {
            type: "card",
            card_id: kind.value.field0.cardId,
            layout: mapNativePlacementLayout(kind.value.field0),
          };
        default:
          return invalid("layout node kind is missing");
      }
    });
  return mapNodes("root");
}

function metadataFromNative(
  metadata: { identifier: string; label: string; description?: string; tags?: string[] } | undefined,
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
  return enumTail(undefined, value, "normal");
}

function mapRail(value?: DocumentRail): "main" | "right" | "left" {
  return enumTail(undefined, value, "main");
}

function mapCardKind(value: DocumentCardKind | undefined): DocumentViewCard["kind"] | null {
  switch (value?.HarmosVariant.case) {
    case "Plot":
      return "plots";
    case "Steps":
      return "steps";
    case "Criteria":
      return "criteria";
    case "Traceability":
      return "traceability";
    case "Notes":
      return "notes";
    case "Records":
      return "records";
    case "Artifacts":
      return "artifacts";
    case "Media":
      return "media";
    case "Health":
      return "health";
    case "TelemetryContext":
      return "telemetry_context";
    default:
      return null;
  }
}

function mapSummaryPresentation(value: DocumentSummaryPresentation | undefined): "value" | "progress" | "badge" {
  switch (value?.HarmosVariant.case) {
    case "Progress":
      return "progress";
    case "Badge":
      return "badge";
    default:
      return "value";
  }
}

function mapSummaryState(value: DocumentSummaryState | undefined): "ok" | "warning" | "critical" | "muted" | null {
  switch (value?.HarmosVariant.case) {
    case "Ok":
      return "ok";
    case "Warning":
      return "warning";
    case "Critical":
      return "critical";
    case "Muted":
      return "muted";
    default:
      return null;
  }
}

function mapNativePlot(definition: NativePlotDefinition, fallbackId: string): DocumentViewCard["plots"][number] {
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
    scale: presentation?.scale,
    description: metadata?.description ?? null,
    marker_record_types: [...request.markerRecordTypes],
    filters: request.filters.map(mapNativeFilter),
    reference_lines: (presentation?.referenceLines ?? []).map((line) => ({
      value: mapNativeReferenceValue(line.value),
      label: line.label ?? null,
      color: line.color,
    })),
    reference_bands: (presentation?.referenceBands ?? []).map((band) => ({
      min: band.min ? mapNativeReferenceValue(band.min) : null,
      max: band.max ? mapNativeReferenceValue(band.max) : null,
      center: band.center ? mapNativeReferenceValue(band.center) : null,
      tolerance: band.tolerance ? mapNativeReferenceValue(band.tolerance) : null,
      label: band.label ?? null,
      color: band.color,
    })),
    x_format: mapNativeValueFormat(presentation?.xFormat),
    y_format: mapNativeValueFormat(presentation?.yFormat),
    x_axis: mapNativePlotAxis(presentation?.xAxis, presentation?.xAxisUnit),
    display_points: request.maxPoints === undefined ? null : safeNumber(request.maxPoints, "plot display points"),
    downsample: mapDownsampleTechnique(request.technique),
  });
}

/**
 * The reduction the definition asks for, or `null` for the host default.
 *
 * Keyed by variant name so a technique the frozen-document wire gains later
 * (`Lttb`) maps without an edit here; `Unspecified` stays the default.
 */
function mapDownsampleTechnique(value: NativeDownsampleTechnique | undefined): DocumentDownsampleTechnique | null {
  const techniques: Record<string, DocumentDownsampleTechnique> = {
    Auto: "auto",
    Raw: "raw",
    MinMax: "min_max",
    Lttb: "lttb",
  };
  return techniques[value?.HarmosVariant.case ?? ""] ?? null;
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
function mapNativePlotAxis(kind: NativePlotAxisKind | undefined, unit: string | undefined): DocumentViewCard["plots"][number]["x_axis"] {
  switch (kind?.HarmosVariant.case) {
    case "WallTime":
      return { kind: "wall_time" };
    case "Elapsed":
      return { kind: "elapsed", unit: unit?.trim() || "s" };
    default:
      return null;
  }
}

function mapNativePlotType(value: PlotType | undefined): DocumentViewCard["plots"][number]["plot_type"] {
  switch (value?.HarmosVariant.case) {
    case "Bar":
      return "bar";
    case "GroupedBar":
    case "StackedBar":
    case "Pie":
    case "Donut":
    case "Pareto":
    case "AggregateTable":
      throw new Error(`Unsupported document plot type: ${value?.HarmosVariant.case}`);
    case "XyScatter":
      return "xy_scatter";
    case "XyLine":
      return "xy_line";
    case "XyInterval":
      return "xy_interval";
    case "Trajectory":
      return "trajectory";
    case "Heatmap":
      return "heatmap";
    case "XyValueHeatmap":
      return "xy_value_heatmap";
    case "WifiChannelOccupancy":
      return "wifi_channel_occupancy";
    case "FrequencySeries":
      return "frequency_series";
    case "Histogram":
      return "histogram";
    case "BoxSummary":
      return "box_summary";
    case "TimeSeries":
      return "time_series";
    default:
      throw new Error(`Unsupported document plot type: ${value}`);
  }
}

function mapPlotStyle(value?: PlotStyle): "line" | "points" | "line_points" | "step" {
  return enumTail(undefined, value, "line");
}

function mapAxisScale(value?: AxisScale): "linear" | "log" {
  return enumTail(undefined, value, "linear");
}

function mapFftWindow(value?: FftWindow): "hann" | "rectangular" {
  return enumTail(undefined, value, "hann");
}

function mapFftDetrend(value?: FftDetrend): "none" | "mean" {
  return enumTail(undefined, value, "mean");
}

/* The level units keep their proto casing, so every member is an override. */
function mapFrequencyLevelUnit(value?: FrequencyLevelUnit): "linear" | "dB" | "dBu" | "dBm" {
  return enumTail(undefined, value, "linear", {
    Db: "dB",
    Dbu: "dBu",
    Dbm: "dBm",
  });
}

function mapNativeFilter(filter: PlotRequest["filters"][number]): {
  expression: DocumentDataExpression;
  operator: "eq" | "ne" | "gt" | "gte" | "lt" | "lte" | "contains" | "exists";
  value?: unknown;
} {
  if (!filter.expression) throw new Error("Native plot filter has no expression.");
  const expression = mapNativeExpression(filter.expression);
  if (!expression) invalid("plot filter expression is missing");
  return {
    expression,
    operator: mapFilterOperator(filter.operator),
    ...(filter.value === undefined ? {} : { value: nativeValueToJson(filter.value) }),
  };
}

function mapFilterOperator(value: FilterOperator | undefined): "eq" | "ne" | "gt" | "gte" | "lt" | "lte" | "contains" | "exists" {
  switch (value?.HarmosVariant.case) {
    case "Ne":
      return "ne";
    case "Gt":
      return "gt";
    case "Gte":
      return "gte";
    case "Lt":
      return "lt";
    case "Lte":
      return "lte";
    case "Contains":
      return "contains";
    case "Exists":
      return "exists";
    default:
      return "eq";
  }
}

function mapNativeReferenceValue(value: { constant?: number; expression?: Parameters<typeof mapNativeExpression>[0] } | undefined) {
  if (value?.constant !== undefined) return value.constant;
  const expression = mapNativeExpression(value?.expression);
  if (!expression) throw new Error("plot reference value is absent");
  return expression;
}

function mapNativeExpression(value?: DataExpression): import("../document-contracts").DataExpression | null {
  if (!value) return null;
  const sourceKind = value.source?.HarmosVariant;
  if (!sourceKind?.case || !sourceKind.value.field0 || value.transforms.length > GUI_PLOT_TRANSFORMS)
    invalid("expression source or transform bound is invalid");
  const source = (() => {
    switch (sourceKind.case) {
      case "Telemetry":
        if (!sourceKind.value.field0) invalid("expression source is missing");
        return {
          type: "telemetry" as const,
          stream: sourceKind.value.field0.stream,
          field: sourceKind.value.field0.field,
        };
      case "Parameter":
        if (!sourceKind.value.field0) invalid("expression source is missing");
        return { type: "parameter" as const, name: sourceKind.value.field0.name };
      case "Dimension":
        if (!sourceKind.value.field0) invalid("expression source is missing");
        return { type: "dimension" as const, name: sourceKind.value.field0.name };
      case "Record":
        if (!sourceKind.value.field0) invalid("expression source is missing");
        return { type: "record" as const, record_type: sourceKind.value.field0.recordType };
      case "Artifact":
        if (!sourceKind.value.field0) invalid("expression source is missing");
        return { type: "artifact" as const, artifact: sourceKind.value.field0.artifact };
      default:
        return invalid("expression kind is missing");
    }
  })();

  const transforms = value.transforms.map((transform) => {
    const kind = transform.kind?.HarmosVariant;
    if (!kind?.case || !kind.value.field0) invalid("expression transform is missing");
    switch (kind.case) {
      case "Aggregate":
        return {
          type: "aggregate" as const,
          aggregate: aggregateName(kind.value.field0.aggregate),
        };
      case "TimeBetween":
        if (!kind.value.field0) invalid("expression transform is missing");
        return {
          type: "time_between" as const,
          previous_record_type: kind.value.field0.previousRecordType,
        };
      case "Detail":
        if (!kind.value.field0) invalid("expression transform is missing");
        return { type: "detail" as const, name: kind.value.field0.name };
      case "PerCase":
        if (!kind.value.field0) invalid("expression transform is missing");
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
  return {
    cycleId: request.cycleId,
    executionId: request.executionId,
  };
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
  publish?: (document: CompletedDocument) => void,
  reuse?: (header: DocumentHeader) => Promise<CompletedDocument | undefined>,
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
  let count = 0;
  let complete = false;
  let structured = false;
  let evidence = false;
  let page: CompletedDocument["page"];
  const committed = new Map<string, DocumentBinding>();
  const bytes = new Map<string, number>();
  const progress = (): CompletedDocument => {
    if (!header) invalid("header is missing");
    return {
      complete,
      page,
      header,
      layout: [...layout],
      cards: new Map(cards),
      bindings: new Map(committed),
      rows: new Map([...rows].map(([id, items]) => [id, [...items]])),
      groups: new Map([...groups].map(([id, items]) => [id, [...items]])),
      tiles: new Map([...tiles].map(([id, groups]) => [id, new Map([...groups].map(([ordinal, items]) => [ordinal, [...items]]))])),
      definitions: new Map(definitions),
      definitionTiles: new Map(definitionTiles),
      repeats: new Map(repeats),
      instances: new Map([...instances].map(([id, items]) => [id, [...items]])),
      sections: new Map(sections),
      warnings: [...warnings],
    };
  };
  const notify = () => {
    if (!active()) invalid("selection changed or transfer cancelled");
    publish?.(progress());
  };
  const validateStructure = () => {
    for (const [id, card] of cards) {
      if (card.body?.HarmosVariant.case === "Table" && card.body.HarmosVariant.value.field0?.rowCount !== BigInt(rows.get(id)?.length ?? 0))
        invalid("table row count is incomplete");
      if (card.body?.HarmosVariant.case === "Summary") {
        const entries = definitions.get(id) ?? [];
        if (card.body.HarmosVariant.value.field0?.groupCount !== BigInt(entries.length))
          invalid("summary definition group count is incomplete");
        for (const group of entries)
          if (group.tileCount !== BigInt(definitionTiles.get(id)?.get(group.ordinal)?.length ?? 0))
            invalid("summary definition tile count is incomplete");
      }
    }
    for (const node of layout)
      if (
        node.node?.HarmosVariant.case === "Placement" &&
        (!node.node.HarmosVariant.value.field0 || !cards.has(node.node.HarmosVariant.value.field0.cardId))
      )
        invalid("layout references a missing source card");
  };
  const validateBinding = (id: string) => {
    const binding = bindings.get(id);
    if (!binding || binding.result?.HarmosVariant.case === "Pending") invalid("card is incomplete");
    if (binding.result?.HarmosVariant.case !== "Summary") return;
    const entries = groups.get(id) ?? [];
    if (binding.result.HarmosVariant.value.field0?.groupCount !== BigInt(entries.length)) invalid("summary group count is incomplete");
    for (const group of entries)
      if (group.tileCount !== BigInt(tiles.get(id)?.get(group.ordinal)?.length ?? 0)) invalid("summary tile count is incomplete");
  };
  for await (const item of frames) {
    if (!active()) invalid("selection changed or transfer cancelled");
    if (complete) invalid("a frame followed completion");
    count++;
    const frame = item.frame?.HarmosVariant;
    if (!frame?.case) invalid("frame payload is missing");
    const payload = frame.value.field0;
    const cardId =
      payload && typeof payload === "object" && "cardId" in payload
        ? String(payload.cardId)
        : frame.case === "Binding"
          ? frame.value.field0?.metadata?.identifier
          : undefined;
    const key = structured && cardId ? `card:${cardId}` : "structure";
    const reached = (bytes.get(key) ?? 0) + toBinary(DocumentFrameSchema, item).byteLength;
    if (reached > DOCUMENTS_MATERIALIZED_BYTES)
      invalid(
        `budget materialized_bytes exceeded (limit ${DOCUMENTS_MATERIALIZED_BYTES}, reached ${reached}, card ${cardId ?? "structure"})`,
      );
    bytes.set(key, reached);
    if (!header && frame.case !== "Header") invalid("header must be first");
    switch (frame.case) {
      case "Header": {
        if (header) invalid("duplicate header");
        const value = frame.value.field0;
        if (!value) invalid("frame payload is missing");
        if (
          value.documentId !== expected.documentId ||
          value.scope?.cycleId !== expected.scope.cycleId ||
          // A cycle's own document carries no execution in its scope.
          (value.scope?.executionId ?? "") !== expected.scope.executionId ||
          value.frontiers?.processEpoch !== expected.epoch ||
          !value.metadata
        )
          invalid("document identity or scope does not match the request");
        header = value;
        break;
      }
      case "LayoutNode": {
        const node = frame.value.field0;
        if (!node) invalid("frame payload is missing");
        if (node.ordinal !== BigInt(layout.length) || !node.node?.HarmosVariant.case || layout.length >= DOCUMENTS_EXPANDED_NODES)
          invalid("layout ordinals are not contiguous or exceed the budget");
        if (node.parentOrdinal === undefined) path = [];
        else {
          const parent = byOrdinal.get(node.parentOrdinal);
          const ancestor = path.indexOf(node.parentOrdinal);
          if (!parent || parent.node?.HarmosVariant.case === "Placement" || ancestor < 0)
            invalid("layout parent is missing or outside the pre-order path");
          path = path.slice(0, ancestor + 1);
        }
        path.push(node.ordinal);
        if (path.length > DOCUMENTS_ENCODED_DEPTH) invalid("layout depth budget exceeded");
        if (
          node.node?.HarmosVariant.case === "Section" &&
          (!node.node.HarmosVariant.value.field0?.nodeId ||
            !node.node.HarmosVariant.value.field0?.metadata ||
            (node.node.HarmosVariant.value.field0?.repeat && !node.node.HarmosVariant.value.field0?.repeat.value))
        )
          invalid("section declaration is incomplete");
        byOrdinal.set(node.ordinal, node);
        layout.push(node);
        break;
      }
      case "Card": {
        const card = frame.value.field0;
        if (!card) invalid("frame payload is missing");
        const id = card.metadata?.identifier;
        if (!id || cards.has(id) || !card.body?.HarmosVariant.case) invalid("missing or duplicate card identity");
        cards.set(id, card);
        if (card.body?.HarmosVariant.case === "Table") rows.set(id, []);
        if (card.body?.HarmosVariant.case === "Summary") {
          definitions.set(id, []);
          definitionTiles.set(id, new Map());
        }
        break;
      }
      case "Binding": {
        const binding = frame.value.field0;
        if (!binding) invalid("frame payload is missing");
        const id = binding.metadata?.identifier;
        const source = cards.get(binding.sourceId);
        if (
          !id ||
          (bindings.has(id) && (!structured || bindings.get(id)?.result?.HarmosVariant.case !== "Pending")) ||
          !source ||
          !binding.result?.HarmosVariant.case
        )
          invalid("binding source or identity is invalid");
        const expectedResult =
          source.body?.HarmosVariant.case === "Summary"
            ? "Summary"
            : source.body?.HarmosVariant.case === "Owner" && source.body.HarmosVariant.value.field0?.kind?.HarmosVariant.case === "Plot"
              ? "Plot"
              : "Literal";
        if (
          (!["Pending", "Failure"].includes(binding.result?.HarmosVariant.case ?? "") &&
            binding.result?.HarmosVariant.case !== expectedResult) ||
          (binding.result?.HarmosVariant.case === "Literal" && !binding.result.HarmosVariant.value.field0)
        )
          invalid("binding result differs from its declaration");
        bindings.set(id, binding);
        if (binding.result?.HarmosVariant.case === "Summary") {
          groups.set(id, []);
          tiles.set(id, new Map());
        }
        break;
      }
      case "TableRow": {
        const row = frame.value.field0;
        if (!row) invalid("frame payload is missing");
        const target = rows.get(row.cardId);
        if (!target || row.ordinal !== BigInt(target.length) || !row.values)
          invalid("table row has no matching card or contiguous ordinal");
        target.push(row);
        break;
      }
      case "GroupDefinition": {
        const group = frame.value.field0;
        if (!group) invalid("frame payload is missing");
        const target = definitions.get(group.cardId);
        if (!target || group.ordinal !== BigInt(target.length) || !group.metadata) invalid("summary definition group is invalid");
        target.push(group);
        definitionTiles.get(group.cardId)?.set(group.ordinal, []);
        break;
      }
      case "TileDefinition": {
        const tile = frame.value.field0;
        if (!tile) invalid("frame payload is missing");
        const target = definitionTiles.get(tile.cardId)?.get(tile.groupOrdinal);
        if (!target || tile.ordinal !== BigInt(target.length) || !tile.metadata || !tile.value)
          invalid("summary definition tile is invalid");
        target.push(tile);
        break;
      }
      case "SummaryGroup": {
        const group = frame.value.field0;
        if (!group) invalid("frame payload is missing");
        const target = groups.get(group.cardId);
        if (!target || group.ordinal !== BigInt(target.length) || !group.metadata)
          invalid("summary group has no matching binding or contiguous ordinal");
        target.push(group);
        tiles.get(group.cardId)?.set(group.ordinal, []);
        break;
      }
      case "SummaryTile": {
        const tile = frame.value.field0;
        if (!tile) invalid("frame payload is missing");
        const target = tiles.get(tile.cardId)?.get(tile.groupOrdinal);
        if (!target || tile.ordinal !== BigInt(target.length) || !tile.metadata)
          invalid("summary tile has no matching group or contiguous ordinal");
        validateSummaryResult(tile);
        target.push(tile);
        break;
      }
      case "RepeatSelection": {
        const repeat = frame.value.field0;
        if (!repeat) invalid("frame payload is missing");
        if (!repeat.nodeId || (repeats.has(repeat.nodeId) && !repeats.get(repeat.nodeId)?.pending))
          invalid("repeat selection identity is invalid");
        repeats.set(repeat.nodeId, repeat);
        instances.set(repeat.nodeId, []);
        break;
      }
      case "RepeatInstance": {
        const instance = frame.value.field0;
        if (!instance) invalid("frame payload is missing");
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
      case "SectionRemoval": {
        if (!structured || !frame.value.field0 || !sections.delete(frame.value.field0)) invalid("section removal is invalid");
        break;
      }
      case "SectionIdentity": {
        const section = frame.value.field0;
        if (!section) invalid("frame payload is missing");
        if (!section.nodeId || !section.metadata || sections.has(section.nodeId)) invalid("section identity is invalid");
        unavailableReason(section.unavailable);
        sections.set(section.nodeId, section);
        break;
      }
      case "Warning":
        if (!frame.value.field0) invalid("warning payload is missing");
        warnings.push(frame.value.field0);
        break;
      case "StructureComplete": {
        if (structured) invalid("duplicate structure completion");
        const value = frame.value.field0;
        if (
          !value ||
          value.cardCount !== BigInt(cards.size) ||
          value.layoutNodeCount !== BigInt(layout.length) ||
          value.bindingCount !== BigInt(bindings.size) ||
          value.repeatCount !== BigInt(repeats.size) ||
          value.sectionCount !== BigInt(sections.size) ||
          value.frameCount !== BigInt(count)
        )
          invalid("structure counts differ");
        validateStructure();
        for (const [id, binding] of bindings) {
          if (binding.result?.HarmosVariant.case !== "Pending") validateBinding(id);
          committed.set(id, binding);
        }
        structured = true;
        notify();
        break;
      }
      case "EvidenceCut": {
        if (!structured || evidence || !header?.frontiers) invalid("evidence cut is misplaced");
        const value = frame.value.field0;
        if (!value?.cut?.store || !value.cut.epoch) invalid("evidence cut is incomplete");
        header = { ...header, frontiers: { ...header.frontiers, evidence: value.cut, telemetry: value.telemetry ? value.cut : undefined } };
        evidence = true;
        const held = value.cacheable ? await reuse?.(header) : undefined;
        if (held) return held;
        break;
      }
      case "ArtifactCut": {
        if (!structured || !evidence || !header?.frontiers || !frame.value.field0 || header.frontiers.artifactsSha256)
          invalid("artifact frontier is misplaced");
        header = { ...header, frontiers: { ...header.frontiers, artifactsSha256: frame.value.field0 } };
        const held = await reuse?.(header);
        if (held) return held;
        break;
      }
      case "Page": {
        const value = frame.value.field0;
        if (!structured || !evidence || !value?.current || page) invalid("document page is misplaced");
        page = { current: value.current, next: value.next };
        break;
      }
      case "ExpansionComplete": {
        if (!structured || frame.value.field0 === undefined) invalid("expansion completion is invalid");
        for (const [id, repeat] of repeats)
          if (!repeat.pending && repeat.instanceCount !== BigInt(instances.get(id)?.length ?? 0))
            invalid("repeat instance count is incomplete");
        if (frame.value.field0 && header) {
          const held = await reuse?.(header);
          if (held) return held;
        }
        for (const [id, binding] of bindings)
          if (!committed.has(id)) {
            if (binding.result?.HarmosVariant.case !== "Pending") validateBinding(id);
            committed.set(id, binding);
          }
        notify();
        break;
      }
      case "CardComplete": {
        if (!structured || !evidence) invalid("card completed before evidence");
        const id = frame.value.field0;
        if (!id || committed.get(id)?.result?.HarmosVariant.case !== "Pending") invalid("card completion is duplicated");
        validateBinding(id);
        const binding = bindings.get(id);
        if (!binding) invalid("completed card binding is missing");
        committed.set(id, binding);
        notify();
        break;
      }
      case "Completion": {
        const value = frame.value.field0;
        if (!value) invalid("frame payload is missing");
        if (
          value.frameCount !== BigInt(count) ||
          value.cardCount !== BigInt(cards.size) ||
          value.layoutNodeCount !== BigInt(layout.length) ||
          value.bindingCount !== BigInt(bindings.size) ||
          value.repeatCount !== BigInt(repeats.size) ||
          value.sectionCount !== BigInt(sections.size)
        )
          invalid("completion counts do not match the received capture");
        if (!structured || !evidence) invalid("completion preceded evidence");
        const partial = Boolean(expected.structureOnly || expected.cardIds?.length);
        if (value.partial !== partial) invalid("completion projection differs from the request");
        if (!partial) for (const id of bindings.keys()) validateBinding(id);
        for (const id of expected.cardIds ?? []) validateBinding(id);
        if (!expected.cardIds?.length && [...repeats.values()].some((repeat) => repeat.pending)) invalid("repeat expansion is incomplete");
        complete = true;
        break;
      }
      default:
        invalid("unknown frame");
    }
  }
  if (!active()) invalid("selection changed or transfer cancelled");
  if (!complete || !header) invalid("transfer ended before completion");
  validateStructure();
  for (const [id, binding] of bindings) {
    if (binding.result?.HarmosVariant.case !== "Summary") continue;
    const entries = groups.get(id) ?? [];
    if (binding.result.HarmosVariant.value.field0?.groupCount !== BigInt(entries.length)) invalid("summary group count is incomplete");
    for (const group of entries)
      if (group.tileCount !== BigInt(tiles.get(id)?.get(group.ordinal)?.length ?? 0)) invalid("summary tile count is incomplete");
  }
  for (const [id, repeat] of repeats)
    if (!repeat.pending && repeat.instanceCount !== BigInt(instances.get(id)?.length ?? 0)) invalid("repeat instance count is incomplete");
  notify();
  return progress();
}

function frontierKey(frontiers: NonNullable<DocumentHeader["frontiers"]>): string {
  return JSON.stringify(frontiers, (_key, value) => (typeof value === "bigint" ? value.toString() : value));
}
