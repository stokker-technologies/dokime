/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Document Presentation Assembly
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import type {
  DocumentCardLayout,
  TelemetrySummaryTilePayload_Serialize,
} from "./document-contracts";
import { formatRecordValue } from "./record-display";
import type {
  DocumentViewCard,
  DocumentDataExpression,
  DocumentViewLayout,
  DocumentViewMetric,
  DocumentViewPayload,
  DocumentViewPlot,
  DocumentViewTableRow,
  Metadata,
} from "./types";
import { isRecord, numericValue } from "./utils";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type SummaryTile = Omit<TelemetrySummaryTilePayload_Serialize, "value" | "stat" | "metadata"> & {
  metadata: Metadata;
  value: unknown;
  stat?: TelemetrySummaryTilePayload_Serialize["stat"];
};

type PlotDeclaration = Omit<
  Partial<DocumentViewPlot>,
  "fft" | "interval" | "histogram" | "reference_lines" | "reference_bands"
> &
  Pick<DocumentViewPlot, "plot_id" | "plot_type"> & {
    fft?: Partial<DocumentViewPlot["fft"]> | null;
    interval?: {
      bin_width?: number | null;
      sigma?: number | null;
      clip_to_observed?: boolean;
    } | null;
    histogram?: Partial<NonNullable<DocumentViewPlot["histogram"]>> | null;
    available_streams?: string[];
    reference_lines?: (Omit<NonNullable<DocumentViewPlot["reference_lines"]>[number], "color"> & {
      color?: string | null;
    })[];
    reference_bands?: (Omit<NonNullable<DocumentViewPlot["reference_bands"]>[number], "color"> & {
      color?: string | null;
    })[];
  };

/* === Classes === */

/* === Public Functions === */

export function documentView(
  documentId: string,
  metadata: Metadata,
  cards: DocumentViewCard[],
  layout: DocumentViewLayout,
): DocumentViewPayload {
  return {
    schema_version: 1,
    document_id: documentId,
    metadata,
    title: metadata.label,
    subtitle: metadata.description ?? null,
    desktop_sidebar_min_px: 300,
    desktop_sidebar_max_px: 420,
    cards,
    layout,
  };
}

export function documentCard(
  metadata: Metadata,
  kind: DocumentViewCard["kind"],
  content: Partial<Pick<DocumentViewCard, "groups" | "plots" | "rows">> = {},
): DocumentViewCard {
  return {
    card_id: metadata.identifier,
    kind,
    metadata,
    layout: documentCardLayout(),
    title: metadata.label,
    region: "main",
    groups: [],
    plots: [],
    histograms: [],
    box_summaries: [],
    rows: [],
    show_report_outcome: true,
    max_items: null,
    config: {},
    ...content,
  };
}

export function documentCardLayout(layout?: DocumentCardLayout): DocumentViewCard["layout"] {
  return {
    col_span: {
      base: layout?.col_span?.base ?? 12,
      sm: layout?.col_span?.sm ?? null,
      md: layout?.col_span?.md ?? null,
      lg: layout?.col_span?.lg ?? null,
      xl: layout?.col_span?.xl ?? null,
    },
    row_span: layout?.row_span ?? 1,
    order: layout?.order ?? 0,
    density: layout?.density ?? "normal",
    rail: layout?.rail ?? "main",
  };
}

export function documentPlot(input: PlotDeclaration): DocumentViewPlot {
  const { available_streams, ...declaration } = input;
  const xExpression = input.x_expression ?? null;
  const yExpressions = input.y_expressions ?? [];
  const seriesByExpression = input.series_by_expression ?? null;
  const bins = input.histogram?.bins;
  const stream =
    nonEmptyString(input.stream) ??
    expressionStream(xExpression) ??
    yExpressions.map(expressionStream).find(Boolean) ??
    expressionStream(seriesByExpression) ??
    available_streams?.map(nonEmptyString).find(Boolean) ??
    nonEmptyString(input.z_stream) ??
    input.plot_id;
  const fields = input.y?.length
    ? input.y
    : yExpressions.map(expressionField).filter((field): field is string => field !== null);

  return {
    ...declaration,
    title: nonEmptyString(input.title) ?? input.plot_id,
    stream,
    style: input.style ?? "line",
    x_scale: input.x_scale ?? "linear",
    y_scale: input.y_scale ?? "linear",
    analysis: input.analysis ?? null,
    fft: {
      ...input.fft,
      window: input.fft?.window ?? "hann",
      detrend: input.fft?.detrend ?? "mean",
      level_unit: input.fft?.level_unit ?? "linear",
      include_dc: input.fft?.include_dc ?? false,
    },
    interval: {
      bin_width: input.interval?.bin_width ?? 1,
      sigma: input.interval?.sigma ?? 3,
      clip_to_observed: input.interval?.clip_to_observed ?? true,
    },
    histogram: {
      ...input.histogram,
      bins: bins !== undefined && Number.isSafeInteger(bins) && bins > 0 ? bins : 12,
    },
    y: fields,
    x: nonEmptyString(input.x) ?? telemetryField(xExpression) ?? "timestamp",
    series_by: nonEmptyString(input.series_by) ?? telemetryField(seriesByExpression),
    x_expression: xExpression,
    y_expressions: yExpressions,
    series_by_expression: seriesByExpression,
    z_expression: input.z_expression ?? null,
    z_stream: input.z_stream ?? null,
    z_field: input.z_field ?? null,
    unit: input.unit ?? null,
    group: nonEmptyString(input.group) ?? "Plots",
    description: input.description ?? null,
    marker_record_types: input.marker_record_types ?? [],
    filters: input.filters ?? [],
    reference_lines: (input.reference_lines ?? []).map((line) => ({
      ...line,
      color: nonEmptyString(line.color) ?? "#60706a",
      source_scale: line.source_scale ?? 1,
    })),
    reference_bands: (input.reference_bands ?? []).map((band) => ({
      ...band,
      center: band.center ?? null,
      tolerance: band.tolerance ?? null,
      color: nonEmptyString(band.color) ?? "#60706a",
    })),
    live_window_s: input.live_window_s ?? null,
    x_format: input.x_format ?? null,
    y_format: input.y_format ?? null,
    x_axis: input.x_axis ?? null,
    display_points: input.display_points ?? null,
    downsample: input.downsample ?? null,
  };
}

export function documentText(
  metadata: Metadata,
  markdown: string,
): DocumentViewCard["groups"][number] {
  return {
    group_id: `${metadata.identifier}-text`,
    title: metadata.label,
    metrics: [
      {
        metric_id: `${metadata.identifier}-text`,
        label: "Content",
        value: markdown,
        formatted: markdown,
        presentation: "value",
      },
    ],
  };
}

export function validateSummaryResult(tile: {
  value?: unknown;
  state?: unknown;
  unavailable?: string | null;
}): void {
  const reason = unavailableReason(tile.unavailable);
  if (
    reason &&
    ((tile.value !== undefined && tile.value !== null) ||
      (tile.state !== undefined && tile.state !== null))
  ) {
    throw new Error("Invalid summary unavailable result");
  }
}

export function unavailableReason(value: unknown): string | undefined {
  if (value === undefined || value === null) return undefined;
  if (
    typeof value !== "string" ||
    !value.trim() ||
    value.length > 512 ||
    new TextEncoder().encode(value).byteLength > 512
  ) {
    throw new Error("Invalid document unavailable reason");
  }
  return value;
}

export function documentSummaryTile(tile: SummaryTile): DocumentViewMetric {
  validateSummaryResult(tile);
  const unavailable = tile.unavailable;
  if (unavailable !== undefined && unavailable !== null) {
    return {
      metric_id: tile.metadata.identifier,
      label: tile.metadata.label,
      value: null,
      unit: null,
      formatted: "Unavailable",
      presentation: "value",
      state: "muted",
      aggregation: null,
      detail: unavailable,
    };
  }
  return {
    metric_id: tile.metadata.identifier,
    label: tile.metadata.label,
    value: tile.value,
    unit: tile.unit ?? null,
    formatted: summaryValue(tile.value, tile.format, tile.unit),
    presentation: tile.presentation ?? "value",
    state: tile.state ?? null,
    aggregation: tile.stat === "latest" ? "last" : null,
    detail: tile.metadata.description ?? null,
  };
}

export function documentTableRows(
  cardId: string,
  rows: readonly Record<string, unknown>[],
): DocumentViewTableRow[] {
  return rows.map((row, index) => ({
    row_id: `${cardId}-${index}`,
    cells: Object.fromEntries(
      Object.entries(row).map(([field, value]) => [field, documentValue(value)]),
    ),
  }));
}

/* === Private Functions === */

function nonEmptyString(value: string | null | undefined): string | null {
  const trimmed = value?.trim();
  return trimmed ? trimmed : null;
}

function expressionStream(expression?: DocumentDataExpression | null): string | null {
  const source = expression?.source;
  return isRecord(source) && source.type === "telemetry" && typeof source.stream === "string"
    ? nonEmptyString(source.stream)
    : null;
}

function expressionField(expression: DocumentDataExpression): string | null {
  const source = expression.source;
  if (!isRecord(source)) return null;
  for (const key of ["field", "name", "record_type", "artifact"]) {
    const field = source[key];
    if (typeof field === "string" && field) return field;
  }
  return null;
}

function telemetryField(expression?: DocumentDataExpression | null): string | null {
  return expression && isRecord(expression.source) && expression.source.type === "telemetry"
    ? expressionField(expression)
    : null;
}

function documentValue(value: unknown): string {
  value = numericValue(value) ?? value;
  if (typeof value === "number") return Number.isFinite(value) ? value.toLocaleString() : "-";
  return formatRecordValue(value, null);
}

function summaryValue(value: unknown, format: SummaryTile["format"], unit?: string | null): string {
  value = numericValue(value) ?? value;
  const rendered = documentValue(value);
  const number = typeof value === "number" && Number.isFinite(value) ? value : null;
  const integer = isRecord(value) && value.__dokime_native_integer__ === true;
  if (number === null && !integer) return rendered;
  if (format === "percent") {
    return `${number !== null ? number.toFixed(Math.abs(number) >= 10 ? 0 : 1) : rendered}%`;
  }
  if (format === "duration") return `${rendered}${unit ?? "s"}`;
  return `${rendered}${unit ? ` ${unit}` : ""}`;
}

/* === Entry Point === */
