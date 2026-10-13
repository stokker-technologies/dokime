/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type {
  DocumentStatistic,
  JsonObject,
  Metadata,
  MetricPresentation,
  MetricState,
} from "../types";
import type { ValueFormat } from "./format";
import type {
  DocumentAxisScale,
  DocumentDownsampleTechnique,
  DocumentFftOptions,
  DocumentHistogramOptions,
  DocumentPlotAxisKind,
  DocumentPlotStyle,
  DocumentPlotType,
  DocumentReferenceBand,
  DocumentReferenceLine,
  DocumentSigmaInterval,
} from "./plot";

/* === Document Presentation Types === */

export type DocumentDataExpression = JsonObject;

export type DocumentFilterOperator =
  | "eq"
  | "ne"
  | "gt"
  | "gte"
  | "lt"
  | "lte"
  | "contains"
  | "exists";

export type DocumentFilter = {
  expression: DocumentDataExpression;
  operator: DocumentFilterOperator;
  value?: unknown;
};

export type DocumentValueSource = {
  kind: "static" | "result_metric" | "telemetry_value" | "record_count" | "run_field";
  value?: unknown;
  stream?: string | null;
  field?: string | null;
  label?: string | null;
  aliases: string[];
  record_types: string[];
  run_field?: string | null;
  unit?: string | null;
  format_key?: string | null;
  live_stat: DocumentStatistic;
  report_stat: DocumentStatistic;
};

export type DocumentCardRail = "main" | "right" | "left";

export type DocumentCardDensity = "compact" | "normal" | "spacious";

export type DocumentCardLayout = {
  col_span: {
    base: number;
    sm?: number | null;
    md?: number | null;
    lg?: number | null;
    xl?: number | null;
  };
  row_span: number;
  order: number;
  density: DocumentCardDensity;
  rail: DocumentCardRail;
};

export type DocumentViewMetric = {
  metric_id: string;
  label: string;
  value?: unknown;
  unit?: string | null;
  formatted: string;
  presentation: MetricPresentation;
  state?: MetricState | null;
  aggregation?: DocumentStatistic | null;
  detail?: string | null;
};

export type DocumentViewMetricGroup = {
  group_id: string;
  title: string;
  metrics: DocumentViewMetric[];
};

export type DocumentViewPlot = {
  plot_id: string;
  title: string;
  stream: string;
  plot_type: DocumentPlotType;
  style: DocumentPlotStyle;
  x_scale: DocumentAxisScale;
  y_scale: DocumentAxisScale;
  fft: DocumentFftOptions;
  analysis?: import("./plot").EvidenceAnalysisSelection | null;
  interval?: DocumentSigmaInterval | null;
  histogram?: DocumentHistogramOptions | null;
  y: string[];
  x_expression?: DocumentDataExpression | null;
  y_expressions?: DocumentDataExpression[];
  series_by_expression?: DocumentDataExpression | null;
  z_expression?: DocumentDataExpression | null;
  z_stream?: string | null;
  z_field?: string | null;
  x?: string | null;
  series_by?: string | null;
  unit?: string | null;
  group: string;
  description?: string | null;
  marker_record_types?: string[];
  filters?: DocumentFilter[];
  reference_lines?: DocumentReferenceLine[];
  reference_bands?: DocumentReferenceBand[];
  live_window_s?: number | null;
  x_format?: ValueFormat | null;
  y_format?: ValueFormat | null;
  /** Absent for a payload written before the axis was declared on the wire. */
  x_axis?: DocumentPlotAxisKind | null;
  /** Displayed points; absent or `null` is the host default, `0` keeps every sample. */
  display_points?: number | null;
  /** Reduction technique; absent is the host default (`lttb`). */
  downsample?: DocumentDownsampleTechnique | null;
};

export type DocumentViewHistogram = {
  histogram_id: string;
  title: string;
  stream: string;
  field: string;
  group: string;
  unit?: string | null;
  description?: string | null;
  bins: number;
  live_window_s?: number | null;
};

export type DocumentViewBoxSummary = {
  box_summary_id: string;
  title: string;
  stream: string;
  field: string;
  group: string;
  unit?: string | null;
  description?: string | null;
  live_window_s?: number | null;
};

/** Presentation-ready table row: cells are already formatted for display, so no
    renderer needs to know whether the row came off the wire or out of a report. */
export type DocumentViewTableRow = {
  row_id: string;
  cells: Record<string, string>;
};

export type DocumentViewCard = {
  card_id: string;
  kind:
    | "result"
    | "overview"
    | "table"
    | "traceability"
    | "telemetry_context"
    | "criteria"
    | "steps"
    | "plots"
    | "histograms"
    | "box_summaries"
    | "notes"
    | "records"
    | "health"
    | "artifacts"
    | "media";
  title: string;
  metadata?: {
    uuid?: string | null;
    created_at?: string | null;
    id?: string | null;
    identifier: string;
    label: string;
    description?: string | null;
    tags?: string[];
  };
  layout: DocumentCardLayout;
  region: "main" | "sidebar";
  groups: DocumentViewMetricGroup[];
  plots: DocumentViewPlot[];
  histograms: DocumentViewHistogram[];
  box_summaries: DocumentViewBoxSummary[];
  rows: DocumentViewTableRow[];
  show_report_outcome: boolean;
  max_items?: number | null;
  config: JsonObject;
};

export type DocumentViewCardPlacement = {
  type: "card";
  card_id: string;
  layout: DocumentCardLayout;
};

export type DocumentViewSectionNode = {
  type: "section";
  node_id: string;
  metadata: Metadata;
  unavailable?: string;
  children: DocumentViewNode[];
};

export type DocumentViewNode = DocumentViewSectionNode | DocumentViewCardPlacement;

export type DocumentViewLayout = {
  nodes: DocumentViewNode[];
};

export type DocumentViewSection = {
  metadata: Metadata;
  unavailable?: string;
  cards: DocumentViewCard[];
};

export type DocumentViewPayload = {
  schema_version: number;
  document_id: string;
  metadata: Metadata;
  title: string;
  subtitle?: string | null;
  desktop_sidebar_min_px: number;
  desktop_sidebar_max_px: number;
  cards: DocumentViewCard[];
  layout: DocumentViewLayout;
};
