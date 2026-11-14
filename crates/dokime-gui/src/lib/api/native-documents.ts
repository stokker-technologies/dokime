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

import * as protocolV1Plots from "@host/dokime/v1/plots_pb.js";
import * as protocolClientV1Documents from "@host/dokime/client/v1/documents_pb.js";
import type * as protocolV1Documents from "@host/dokime/v1/documents_pb.js";
import * as protocolV1Expressions from "@host/dokime/v1/expressions_pb.js";
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
import * as rpc from "@/lib/host/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { DocumentDataExpression, DocumentDownsampleTechnique, DocumentViewCard, DocumentViewPayload, Metadata } from "@/lib/types";
import { required, cutKey } from "@/lib/rpc/canonical";
import { safeNumber } from "./native-values";
import { mapNativeValueFormat, nativeValuesToJson, nativeValueToJson } from "@/lib/host/values";

const DOCUMENTS_ENCODED_DEPTH = 32;
const DOCUMENTS_EXPANDED_NODES = 4_096;
const DOCUMENTS_MATERIALIZED_BYTES = 33_554_432;
const GUI_PLOT_TRANSFORMS = 32;
const DOCUMENT_FRAME_PAGE_SIZE = 32;

/* === Canonical Types === */

type DocumentFrame = protocolClientV1Documents.DocumentFrame;
const DocumentFrameSchema = protocolClientV1Documents.DocumentFrameSchema;
type VariantValue<T, K extends string> =
  Extract<NonNullable<T> extends { kind: infer V } ? V : never, { case: K }> extends { value: infer F } ? NonNullable<F> : never;
type FrameValue<K extends string> = VariantValue<Wire<DocumentFrame>["frame"], K>;
type DocumentCard = FrameValue<"card">;
type DocumentHeader = FrameValue<"header">;
type DocumentBinding = FrameValue<"binding">;
type DocumentLayoutNode = FrameValue<"layoutNode">;
type DocumentPlacement = VariantValue<DocumentLayoutNode["node"], "placement">;
type DocumentDensity = NonNullable<DocumentPlacement["density"]>;
type DocumentRail = NonNullable<DocumentPlacement["rail"]>;
type DocumentSummaryGroupDefinition = FrameValue<"groupDefinition">;
type DocumentSummaryTileDefinition = FrameValue<"tileDefinition">;
type DocumentRepeatSelection = FrameValue<"repeatSelection">;
type DocumentRepeatInstance = FrameValue<"repeatInstance">;
type DocumentSectionIdentity = FrameValue<"sectionIdentity">;
type DocumentScope = Wire<protocolV1Documents.DocumentScope>;
type DocumentSummaryGroup = FrameValue<"summaryGroup">;
type DocumentSummaryTile = FrameValue<"summaryTile">;
type DocumentSummaryPresentation = NonNullable<DocumentSummaryTile["presentation"]>;
type DocumentSummaryState = NonNullable<DocumentSummaryTile["state"]>;
type DocumentTableRow = FrameValue<"tableRow">;
type DocumentWarning = FrameValue<"warning">;
type NativePlotDefinition = VariantValue<DocumentBinding["result"], "plot">;
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
type DocumentCardKind = NonNullable<VariantValue<DocumentCard["body"], "owner">["kind"]>;
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
  at?: Parameters<typeof rpc.documentsGet>[1]["at"];
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
  const variants = {
    auto: protocolV1Plots.Downsample.AUTO,
    raw: protocolV1Plots.Downsample.RAW,
    min_max: protocolV1Plots.Downsample.MIN_MAX,
    lttb: protocolV1Plots.Downsample.LTTB,
  } as const;
  const identity = {
    scope: { cycleId: update.cycleId, executionId: update.executionId },
    documentId: update.documentId,
    plotId: update.plotId,
  };
  const current = await rpc.plotsSettingsGet(session, identity);
  await rpc.plotsSettingsSet(session, {
    ...identity,
    expectedRevision: current.revision,
    requestId: crypto.randomUUID(),
    value: { displayPoints: update.displayPoints ?? undefined, downsample: variants[update.downsample] },
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
  const documentId = request.executionId && request.documentId === "default" ? "" : request.documentId;
  let cancelled = false;
  const dispose = session.own(() => {
    cancelled = true;
  });
  const active = () => !cancelled && !signal?.aborted && session.isCurrent();
  const frames = (async function* () {
    let pageToken = request.page ?? "";
    let at: Parameters<typeof rpc.documentsGet>[1]["at"] = request.at;
    const tokens = new Set<string>();
    for (let index = 0; index < 4096; index++) {
      if (!active()) throw new Error("Read Document: Selection Changed or Transfer Cancelled");
      const reply = await rpc.documentsGet(session, {
        scope,
        documentId,
        structureOnly: request.structureOnly ?? false,
        cardIds: request.cardIds ?? [],
        pageToken,
        maxFrames: DOCUMENT_FRAME_PAGE_SIZE,
        at,
      });
      const cut = required(reply.resolvedAt, "Read Document Cut");
      if (at && cutKey(at) !== cutKey(cut)) invalid("retained document cut changed");
      at = cut;
      for (const frame of reply.frames) yield frame;
      if (!reply.nextPageToken) return;
      if (tokens.has(reply.nextPageToken)) invalid("document continuation did not advance");
      pageToken = reply.nextPageToken;
      tokens.add(pageToken);
    }
    invalid("document page budget exceeded");
  })();
  try {
    return await assembleDocument(
      frames,
      { scope, documentId, epoch: request.epoch, structureOnly: request.structureOnly, cardIds: request.cardIds },
      active,
      publish,
      async (header) => {
        if (
          previous &&
          previous.complete !== false &&
          ![...previous.bindings].some(
            ([id, binding]) => (!request.cardIds?.length || request.cardIds.includes(id)) && binding.result?.kind.case === "failure",
          ) &&
          sameFrontiers(header.frontiers, previous.header.frontiers)
        )
          return previous;
        return undefined;
      },
    );
  } finally {
    dispose();
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
  const result = binding.result?.kind;
  const body = card.body?.kind;
  if (!result?.case || !body?.case) invalid("card body or binding result is missing");
  if (result.case === "pending" || result.case === "failure") {
    const kind = body.case === "table" ? "table" : body.case === "owner" ? (mapCardKind(body.value?.kind) ?? "overview") : "overview";
    if (result.case === "pending") return documentCard(metadata, kind, { loading: true });
    const failure = result.value;
    if (!failure?.code || !failure.message.trim()) invalid("card failure is missing");
    return documentCard(metadata, kind, { failure: failureOf(failure, metadata.label) });
  }
  if (result.case === "summary" && body.case === "summary") {
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
            format: summaryFormat(tile.format),
            presentation: mapSummaryPresentation(tile.presentation),
            state: tile.state === undefined ? null : mapSummaryState(tile.state),
            stat: tile.stat === protocolClientV1Documents.DocumentSummaryStat.LATEST ? "latest" : undefined,
          }),
        ),
      })),
    });
  }
  if (result.case === "plot" && body.case === "owner" && body.value?.kind === protocolClientV1Documents.DocumentCardKind.PLOT) {
    if (!result.value) invalid("plot binding is missing");
    const plot = mapNativePlot(result.value, id);
    return documentCard(metadata, "plots", { plots: [plot] });
  }
  if (result.case !== "literal" || !result.value || body.case === "summary") invalid("binding result differs from its declaration");
  switch (body.case) {
    case "markdown":
      return documentCard(metadata, "overview", {
        groups: [documentText(metadata, body.value)],
      });
    case "table":
      return documentCard(metadata, "table", {
        rows: documentTableRows(
          id,
          (document.rows.get(binding.sourceId) ?? []).map((row) => nativeValuesToJson(row.values?.cells)),
        ),
      });
    case "owner": {
      const kind = mapCardKind(body.value?.kind);
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
      const kind = node.node?.kind;
      if (!kind?.case || !kind.value) invalid("layout node kind is missing");
      switch (kind.case) {
        case "section":
          if (!kind.value) invalid("layout node payload is missing");
          return {
            type: "section",
            metadata: metadataFromNative(kind.value.metadata, kind.value.nodeId),
            node_id: kind.value.nodeId,
            repeat: Boolean(kind.value.repeat),
            children: mapNodes(node.ordinal.toString()),
          };
        case "placement":
          if (!kind.value) invalid("layout node payload is missing");
          return {
            type: "card",
            card_id: kind.value.cardId,
            layout: mapNativePlacementLayout(kind.value),
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
  return value === protocolClientV1Documents.DocumentDensity.COMPACT
    ? "compact"
    : value === protocolClientV1Documents.DocumentDensity.SPACIOUS
      ? "spacious"
      : "normal";
}
function mapRail(value?: DocumentRail): "main" | "right" | "left" {
  return value === protocolClientV1Documents.DocumentRail.LEFT
    ? "left"
    : value === protocolClientV1Documents.DocumentRail.RIGHT
      ? "right"
      : "main";
}
function mapCardKind(value: DocumentCardKind | undefined): DocumentViewCard["kind"] | null {
  switch (value) {
    case protocolClientV1Documents.DocumentCardKind.PLOT:
      return "plots";
    case protocolClientV1Documents.DocumentCardKind.STEPS:
      return "steps";
    case protocolClientV1Documents.DocumentCardKind.CRITERIA:
      return "criteria";
    case protocolClientV1Documents.DocumentCardKind.TRACEABILITY:
      return "traceability";
    case protocolClientV1Documents.DocumentCardKind.NOTES:
      return "notes";
    case protocolClientV1Documents.DocumentCardKind.RECORDS:
      return "records";
    case protocolClientV1Documents.DocumentCardKind.ARTIFACTS:
      return "artifacts";
    case protocolClientV1Documents.DocumentCardKind.MEDIA:
      return "media";
    case protocolClientV1Documents.DocumentCardKind.HEALTH:
      return "health";
    case protocolClientV1Documents.DocumentCardKind.TELEMETRY_CONTEXT:
      return "telemetry_context";
    default:
      return null;
  }
}
function summaryFormat(
  value: protocolClientV1Documents.DocumentSummaryFormat,
): "auto" | "number" | "percent" | "duration" | "distance" | "status" | "text" {
  switch (value) {
    case protocolClientV1Documents.DocumentSummaryFormat.NUMBER:
      return "number";
    case protocolClientV1Documents.DocumentSummaryFormat.PERCENT:
      return "percent";
    case protocolClientV1Documents.DocumentSummaryFormat.DURATION:
      return "duration";
    case protocolClientV1Documents.DocumentSummaryFormat.DISTANCE:
      return "distance";
    case protocolClientV1Documents.DocumentSummaryFormat.STATUS:
      return "status";
    case protocolClientV1Documents.DocumentSummaryFormat.TEXT:
      return "text";
    default:
      return "auto";
  }
}
function mapSummaryPresentation(value: DocumentSummaryPresentation | undefined): "value" | "progress" | "badge" {
  return value === protocolClientV1Documents.DocumentSummaryPresentation.PROGRESS
    ? "progress"
    : value === protocolClientV1Documents.DocumentSummaryPresentation.BADGE
      ? "badge"
      : "value";
}
function mapSummaryState(value: DocumentSummaryState | undefined): "ok" | "warning" | "critical" | "muted" | null {
  return value === protocolClientV1Documents.DocumentSummaryState.OK
    ? "ok"
    : value === protocolClientV1Documents.DocumentSummaryState.WARNING
      ? "warning"
      : value === protocolClientV1Documents.DocumentSummaryState.CRITICAL
        ? "critical"
        : value === protocolClientV1Documents.DocumentSummaryState.MUTED
          ? "muted"
          : null;
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
          parameters: nativeValuesToJson(request.analysis.parameters?.values),
        }
      : null,
    fft: {
      ...(fft?.inputSamples === undefined ? {} : { input_samples: fft.inputSamples }),
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
    arrangement: request.arrangement ? (request.arrangement === protocolV1Plots.BarArrangement.GROUPED ? "grouped" : "stacked") : null,
    pareto: request.pareto,
    hole: request.hole ?? null,
    xy_interval: request.xyInterval ? "sigma" : null,
    reduction: request.reduction
      ? ({
          [protocolV1Plots.Reduction.MEAN]: "mean" as const,
          [protocolV1Plots.Reduction.MIN]: "min" as const,
          [protocolV1Plots.Reduction.MAX]: "max" as const,
        }[request.reduction] ?? null)
      : null,
    render: request.render === protocolV1Plots.ContourRender.BANDS ? "bands" : "cells",
    grid: request.grid
      ? {
          row: mapNativeExpression(request.grid.row),
          column: mapNativeExpression(request.grid.column),
          frame: mapNativeExpression(request.grid.frame),
          rows: request.grid.rows.map((entry) => ({ ...entry })),
          columns: request.grid.columns.map((entry) => ({ ...entry })),
          highlights: request.grid.highlights.map((highlight) => ({
            row: mapNativeExpression(highlight.row) ?? invalid("grid highlight row is missing"),
            column: mapNativeExpression(highlight.column) ?? invalid("grid highlight column is missing"),
            label: highlight.label,
          })),
          summary_label: request.grid.summaryLabel,
        }
      : null,
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
  return value === protocolV1Plots.Downsample.AUTO
    ? "auto"
    : value === protocolV1Plots.Downsample.RAW
      ? "raw"
      : value === protocolV1Plots.Downsample.MIN_MAX
        ? "min_max"
        : value === protocolV1Plots.Downsample.LTTB
          ? "lttb"
          : null;
}
function mapNativePlotAxis(kind: NativePlotAxisKind | undefined, unit: string | undefined): DocumentViewCard["plots"][number]["x_axis"] {
  return kind === protocolClientV1Documents.PlotAxisKind.WALL_TIME
    ? { kind: "wall_time" }
    : kind === protocolClientV1Documents.PlotAxisKind.ELAPSED
      ? { kind: "elapsed", unit: unit?.trim() || "s" }
      : null;
}
function mapNativePlotType(value?: PlotType): DocumentViewCard["plots"][number]["plot_type"] {
  switch (value) {
    case protocolV1Plots.PlotKind.TIME_SERIES:
      return "time_series";
    case protocolV1Plots.PlotKind.FREQUENCY_SERIES:
      return "frequency_series";
    case protocolV1Plots.PlotKind.XY:
      return "xy";
    case protocolV1Plots.PlotKind.TRAJECTORY:
      return "trajectory";
    case protocolV1Plots.PlotKind.CONTOUR:
      return "contour";
    case protocolV1Plots.PlotKind.GRID_CHART:
      return "grid_chart";
    case protocolV1Plots.PlotKind.BAR:
      return "bar";
    case protocolV1Plots.PlotKind.PIE:
      return "pie";
    case protocolV1Plots.PlotKind.HISTOGRAM:
      return "histogram";
    case protocolV1Plots.PlotKind.BOX_PLOT:
      return "box_plot";
    default:
      throw new Error(`Unsupported document plot type: ${value}`);
  }
}
function mapPlotStyle(value?: PlotStyle): "line" | "points" | "line_points" | "step" {
  return value === protocolV1Plots.LineStyle.POINTS
    ? "points"
    : value === protocolV1Plots.LineStyle.LINE_POINTS
      ? "line_points"
      : value === protocolV1Plots.LineStyle.STEP
        ? "step"
        : "line";
}
function mapAxisScale(value?: AxisScale): "linear" | "log" {
  return value === protocolV1Plots.AxisScale.LOG ? "log" : "linear";
}
function mapFftWindow(value?: FftWindow): "hann" | "rectangular" {
  return value === protocolV1Plots.FftWindow.RECTANGULAR ? "rectangular" : "hann";
}
function mapFftDetrend(value?: FftDetrend): "none" | "mean" {
  return value === protocolV1Plots.FftDetrend.NONE ? "none" : "mean";
}
function mapFrequencyLevelUnit(value?: FrequencyLevelUnit): "linear" | "dB" | "dBu" | "dBm" {
  return value === protocolV1Plots.FrequencyLevelUnit.DB
    ? "dB"
    : value === protocolV1Plots.FrequencyLevelUnit.DBU
      ? "dBu"
      : value === protocolV1Plots.FrequencyLevelUnit.DBM
        ? "dBm"
        : "linear";
}
function mapNativeFilter(filter: PlotRequest["filters"][number]): {
  expression: DocumentDataExpression;
  operator: "eq" | "ne" | "gt" | "gte" | "lt" | "lte" | "contains" | "exists";
  value?: unknown;
} {
  const expression = mapNativeExpression(filter.expression);
  if (!expression) invalid("plot filter expression is missing");
  return {
    expression,
    operator: mapFilterOperator(filter.operator),
    ...(filter.value === undefined ? {} : { value: nativeValueToJson(filter.value) }),
  };
}
function mapFilterOperator(value: FilterOperator | undefined): "eq" | "ne" | "gt" | "gte" | "lt" | "lte" | "contains" | "exists" {
  switch (value) {
    case protocolV1Expressions.FilterOperator.NE:
      return "ne";
    case protocolV1Expressions.FilterOperator.GT:
      return "gt";
    case protocolV1Expressions.FilterOperator.GE:
      return "gte";
    case protocolV1Expressions.FilterOperator.LT:
      return "lt";
    case protocolV1Expressions.FilterOperator.LE:
      return "lte";
    case protocolV1Expressions.FilterOperator.CONTAINS:
      return "contains";
    case protocolV1Expressions.FilterOperator.EXISTS:
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

function mapNativeExpression(value?: DataExpression, depth = 0): import("../document-contracts").DataExpression | null {
  if (!value) return null;
  if (depth > GUI_PLOT_TRANSFORMS) invalid("expression transform bound is invalid");
  const kind = value.kind;
  switch (kind.case) {
    case "literal":
      return { source: { type: "literal", value: nativeValueToJson(kind.value) }, label: value.label };
    case "stream":
      return { source: { type: "telemetry", stream: kind.value.identifier, field: kind.value.field }, label: value.label };
    case "record":
      return {
        source: { type: "record", record_type: kind.value.identifier },
        transforms: kind.value.field ? [{ type: "detail", name: kind.value.field }] : [],
        label: value.label,
      };
    case "parameter":
      return { source: { type: "parameter", name: kind.value }, label: value.label };
    case "dimension":
      return { source: { type: "dimension", name: kind.value }, label: value.label };
    case "artifact":
      return { source: { type: "artifact", artifact: kind.value }, label: value.label };
    case "aggregate": {
      const input = mapNativeExpression(kind.value.input, depth + 1);
      if (!input) invalid("aggregate input is missing");
      const names: Partial<Record<protocolV1Expressions.Aggregate, import("../document-contracts").Aggregate>> = {
        [protocolV1Expressions.Aggregate.FIRST]: "first",
        [protocolV1Expressions.Aggregate.LAST]: "last",
        [protocolV1Expressions.Aggregate.COUNT]: "count",
        [protocolV1Expressions.Aggregate.SUM]: "sum",
        [protocolV1Expressions.Aggregate.MIN]: "min",
        [protocolV1Expressions.Aggregate.MAX]: "max",
        [protocolV1Expressions.Aggregate.MEAN]: "mean",
        [protocolV1Expressions.Aggregate.STDDEV]: "std",
        [protocolV1Expressions.Aggregate.MEDIAN]: "median",
        [protocolV1Expressions.Aggregate.RMS]: "rms",
      };
      const aggregate =
        kind.value.kind === protocolV1Expressions.Aggregate.PERCENTILE ? (`p${kind.value.percentile}` as const) : names[kind.value.kind];
      if (!aggregate) invalid("aggregate kind is missing");
      return { ...input, label: value.label || input.label, transforms: [...(input.transforms ?? []), { type: "aggregate", aggregate }] };
    }
    case "transform": {
      const input = mapNativeExpression(kind.value.input, depth + 1);
      if (!input) invalid("transform input is missing");
      const transform = kind.value.kind;
      let mapped: import("../document-contracts").DataTransform;
      switch (transform.case) {
        case "scale":
          mapped = { type: "scale", factor: transform.value };
          break;
        case "offset":
          mapped = { type: "offset", value: transform.value };
          break;
        case "detail":
          mapped = { type: "detail", name: transform.value };
          break;
        case "timeBetween":
          mapped = { type: "time_between", previous_record_type: transform.value };
          break;
        case "perCase":
          mapped = { type: "per_case" };
          break;
        default:
          return invalid("expression transform is missing");
      }
      return { ...input, label: value.label || input.label, transforms: [...(input.transforms ?? []), mapped] };
    }
    default:
      return invalid("expression source is missing");
  }
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
      if (card.body?.kind.case === "summary") {
        const entries = definitions.get(id) ?? [];
        if (card.body.kind.value?.groupCount !== BigInt(entries.length)) invalid("summary definition group count is incomplete");
        for (const group of entries)
          if (group.tileCount !== BigInt(definitionTiles.get(id)?.get(group.ordinal)?.length ?? 0))
            invalid("summary definition tile count is incomplete");
      }
    }
    for (const node of layout)
      if (node.node?.kind.case === "placement" && (!node.node.kind.value || !cards.has(node.node.kind.value.cardId)))
        invalid("layout references a missing source card");
  };
  const validateBinding = (id: string) => {
    const binding = bindings.get(id);
    if (!binding || binding.result?.kind.case === "pending") invalid("card is incomplete");
    const card = cards.get(binding.sourceId);
    if (card?.body?.kind.case === "table" && card.body.kind.value?.rowCount !== BigInt(rows.get(id)?.length ?? 0))
      invalid("table row count is incomplete");
    if (binding.result?.kind.case !== "summary") return;
    const entries = groups.get(id) ?? [];
    if (binding.result.kind.value?.groupCount !== BigInt(entries.length)) invalid("summary group count is incomplete");
    for (const group of entries)
      if (group.tileCount !== BigInt(tiles.get(id)?.get(group.ordinal)?.length ?? 0)) invalid("summary tile count is incomplete");
  };
  for await (const item of frames) {
    if (!active()) invalid("selection changed or transfer cancelled");
    if (complete) invalid("a frame followed completion");
    count++;
    const frame = item.frame?.kind;
    if (!frame?.case) invalid("frame payload is missing");
    const payload = frame.value;
    const cardId =
      payload && typeof payload === "object" && "cardId" in payload
        ? String(payload.cardId)
        : frame.case === "binding"
          ? frame.value?.metadata?.identifier
          : undefined;
    const key = structured && cardId ? `card:${cardId}` : "structure";
    const reached = (bytes.get(key) ?? 0) + toBinary(DocumentFrameSchema, item).byteLength;
    if (reached > DOCUMENTS_MATERIALIZED_BYTES)
      invalid(
        `budget materialized_bytes exceeded (limit ${DOCUMENTS_MATERIALIZED_BYTES}, reached ${reached}, card ${cardId ?? "structure"})`,
      );
    bytes.set(key, reached);
    if (!header && frame.case !== "header") invalid("header must be first");
    switch (frame.case) {
      case "header": {
        if (header) invalid("duplicate header");
        const value = frame.value;
        if (!value) invalid("frame payload is missing");
        if (
          !value.documentId ||
          (expected.documentId !== "" && value.documentId !== expected.documentId) ||
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
      case "layoutNode": {
        const node = frame.value;
        if (!node) invalid("frame payload is missing");
        if (node.ordinal !== BigInt(layout.length) || !node.node?.kind.case || layout.length >= DOCUMENTS_EXPANDED_NODES)
          invalid("layout ordinals are not contiguous or exceed the budget");
        if (node.parentOrdinal === undefined) path = [];
        else {
          const parent = byOrdinal.get(node.parentOrdinal);
          const ancestor = path.indexOf(node.parentOrdinal);
          if (!parent || parent.node?.kind.case === "placement" || ancestor < 0)
            invalid("layout parent is missing or outside the pre-order path");
          path = path.slice(0, ancestor + 1);
        }
        path.push(node.ordinal);
        if (path.length > DOCUMENTS_ENCODED_DEPTH) invalid("layout depth budget exceeded");
        if (
          node.node?.kind.case === "section" &&
          (!node.node.kind.value?.nodeId ||
            !node.node.kind.value?.metadata ||
            (node.node.kind.value?.repeat && !node.node.kind.value?.repeat.value))
        )
          invalid("section declaration is incomplete");
        byOrdinal.set(node.ordinal, node);
        layout.push(node);
        break;
      }
      case "card": {
        const card = frame.value;
        if (!card) invalid("frame payload is missing");
        const id = card.metadata?.identifier;
        if (!id || cards.has(id) || !card.body?.kind.case) invalid("missing or duplicate card identity");
        cards.set(id, card);
        if (card.body?.kind.case === "table") rows.set(id, []);
        if (card.body?.kind.case === "summary") {
          definitions.set(id, []);
          definitionTiles.set(id, new Map());
        }
        break;
      }
      case "binding": {
        const binding = frame.value;
        if (!binding) invalid("frame payload is missing");
        const id = binding.metadata?.identifier;
        const source = cards.get(binding.sourceId);
        if (
          !id ||
          (bindings.has(id) && (!structured || bindings.get(id)?.result?.kind.case !== "pending")) ||
          !source ||
          !binding.result?.kind.case
        )
          invalid("binding source or identity is invalid");
        const expectedResult =
          source.body?.kind.case === "summary"
            ? "summary"
            : source.body?.kind.case === "owner" && source.body.kind.value?.kind === protocolClientV1Documents.DocumentCardKind.PLOT
              ? "plot"
              : "literal";
        if (
          (!["pending", "failure"].includes(binding.result?.kind.case ?? "") && binding.result?.kind.case !== expectedResult) ||
          (binding.result?.kind.case === "literal" && !binding.result.kind.value)
        )
          invalid("binding result differs from its declaration");
        bindings.set(id, binding);
        if (binding.result?.kind.case === "summary") {
          groups.set(id, []);
          tiles.set(id, new Map());
        }
        break;
      }
      case "tableRow": {
        const row = frame.value;
        if (!row) invalid("frame payload is missing");
        const target = rows.get(row.cardId);
        if (!target || row.ordinal !== BigInt(target.length) || !row.values)
          invalid("table row has no matching card or contiguous ordinal");
        target.push(row);
        break;
      }
      case "groupDefinition": {
        const group = frame.value;
        if (!group) invalid("frame payload is missing");
        const target = definitions.get(group.cardId);
        if (!target || group.ordinal !== BigInt(target.length) || !group.metadata) invalid("summary definition group is invalid");
        target.push(group);
        definitionTiles.get(group.cardId)?.set(group.ordinal, []);
        break;
      }
      case "tileDefinition": {
        const tile = frame.value;
        if (!tile) invalid("frame payload is missing");
        const target = definitionTiles.get(tile.cardId)?.get(tile.groupOrdinal);
        if (!target || tile.ordinal !== BigInt(target.length) || !tile.metadata || !tile.value)
          invalid("summary definition tile is invalid");
        target.push(tile);
        break;
      }
      case "summaryGroup": {
        const group = frame.value;
        if (!group) invalid("frame payload is missing");
        const target = groups.get(group.cardId);
        if (!target || group.ordinal !== BigInt(target.length) || !group.metadata)
          invalid("summary group has no matching binding or contiguous ordinal");
        target.push(group);
        tiles.get(group.cardId)?.set(group.ordinal, []);
        break;
      }
      case "summaryTile": {
        const tile = frame.value;
        if (!tile) invalid("frame payload is missing");
        const target = tiles.get(tile.cardId)?.get(tile.groupOrdinal);
        if (!target || tile.ordinal !== BigInt(target.length) || !tile.metadata)
          invalid("summary tile has no matching group or contiguous ordinal");
        validateSummaryResult(tile);
        target.push(tile);
        break;
      }
      case "repeatSelection": {
        const repeat = frame.value;
        if (!repeat) invalid("frame payload is missing");
        if (!repeat.nodeId || (repeats.has(repeat.nodeId) && !repeats.get(repeat.nodeId)?.pending))
          invalid("repeat selection identity is invalid");
        repeats.set(repeat.nodeId, repeat);
        instances.set(repeat.nodeId, []);
        break;
      }
      case "repeatInstance": {
        const instance = frame.value;
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
      case "sectionRemoval": {
        if (!structured || !frame.value || !sections.delete(frame.value)) invalid("section removal is invalid");
        break;
      }
      case "sectionIdentity": {
        const section = frame.value;
        if (!section) invalid("frame payload is missing");
        if (!section.nodeId || !section.metadata || sections.has(section.nodeId)) invalid("section identity is invalid");
        unavailableReason(section.unavailable);
        sections.set(section.nodeId, section);
        break;
      }
      case "warning":
        if (!frame.value) invalid("warning payload is missing");
        warnings.push(frame.value);
        break;
      case "structureComplete": {
        if (structured) invalid("duplicate structure completion");
        const value = frame.value;
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
          if (binding.result?.kind.case !== "pending") validateBinding(id);
          committed.set(id, binding);
        }
        structured = true;
        notify();
        break;
      }
      case "evidenceCut": {
        if (!structured || evidence || !header?.frontiers) invalid("evidence cut is misplaced");
        const value = frame.value;
        if (!value?.cut?.store || !value.cut.epoch) invalid("evidence cut is incomplete");
        header = { ...header, frontiers: { ...header.frontiers, evidence: value.cut, telemetry: value.telemetry ? value.cut : undefined } };
        evidence = true;
        const held = value.cacheable ? await reuse?.(header) : undefined;
        if (held) return held;
        break;
      }
      case "artifactCut": {
        if (!structured || !evidence || !header?.frontiers || !frame.value || header.frontiers.artifactsSha256)
          invalid("artifact frontier is misplaced");
        header = { ...header, frontiers: { ...header.frontiers, artifactsSha256: frame.value } };
        const held = await reuse?.(header);
        if (held) return held;
        break;
      }
      case "page": {
        const value = frame.value;
        if (!structured || !evidence || !value?.current || page) invalid("document page is misplaced");
        page = { current: value.current, next: value.next };
        break;
      }
      case "expansionComplete": {
        if (!structured || frame.value === undefined) invalid("expansion completion is invalid");
        for (const [id, repeat] of repeats)
          if (!repeat.pending && repeat.instanceCount !== BigInt(instances.get(id)?.length ?? 0))
            invalid("repeat instance count is incomplete");
        if (frame.value && header) {
          const held = await reuse?.(header);
          if (held) return held;
        }
        for (const [id, binding] of bindings)
          if (!committed.has(id)) {
            if (binding.result?.kind.case !== "pending") validateBinding(id);
            committed.set(id, binding);
          }
        notify();
        break;
      }
      case "cardComplete": {
        if (!structured || !evidence) invalid("card completed before evidence");
        const id = frame.value;
        if (!id || committed.get(id)?.result?.kind.case !== "pending") invalid("card completion is duplicated");
        validateBinding(id);
        const binding = bindings.get(id);
        if (!binding) invalid("completed card binding is missing");
        committed.set(id, binding);
        notify();
        break;
      }
      case "completion": {
        const value = frame.value;
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
    if (binding.result?.kind.case !== "summary") continue;
    const entries = groups.get(id) ?? [];
    if (binding.result.kind.value?.groupCount !== BigInt(entries.length)) invalid("summary group count is incomplete");
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
