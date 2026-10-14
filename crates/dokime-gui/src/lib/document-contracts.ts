/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Shared document serialization contracts.
 *
 * A generated report embeds its payload as serde JSON in the HTML, so these are
 * the TypeScript shapes of captured declarations, binding results, and selections.
 * They belong to shared frontend document assembly. Keep them aligned with
 * the Rust serde types they mirror. `_Serialize` marks a
 * serde type whose read and write shapes differ; a report only ever reads that
 * side, so only that side is modelled here.
 */

export type ArtifactListDescriptor = {
  scope: RecordScope;
  artifact_count: number;
  completed_count: number;
};

export type AnalysisSelection = {
  id: Identifier;
  version: number;
  parameters: Record<string, Value>;
};

export type AxisScale = "linear" | "log";

export type CaseId = string;

export type CriterionId = string;

export type DataExpression = {
  label?: string | null;
  source: DataSource;
  transforms?: DataTransform[];
};

export type DataSource =
  | { type: "telemetry"; stream: string; field: string }
  | { type: "parameter"; name: string }
  | { type: "dimension"; name: string }
  | { type: "record"; record_type: string }
  | { type: "artifact"; artifact: string };

/* One member of the Harmos aggregate vocabulary by its display name; a
   percentile is `p` and a whole rank from 0 to 100. */
export type Aggregate =
  | "count"
  | "sum"
  | "mean"
  | "median"
  | "min"
  | "max"
  | "rms"
  | "std"
  | `p${number}`;

export type DataTransform =
  | { type: "aggregate"; aggregate: Aggregate }
  | { type: "time_between"; previous_record_type: string }
  | { type: "telemetry_at_or_before"; stream: string; field: string }
  | { type: "detail"; name: string }
  | { type: "per_case" };

export type DocumentCardBody_Serialize =
  | {
      type: "text";
      data: {
        markdown: string;
      };
    }
  | {
      type: "telemetry_summary";
      data: {
        summary: TelemetrySummaryDefinition_Serialize;
      };
    }
  | {
      type: "plot";
      data: {
        plot: PlotDefinition;
      };
    }
  | {
      type: "table";
      data: {
        rows: { [key in string]: Value }[];
      };
    }
  | ({ type: "steps" } & { data?: never })
  | ({ type: "criteria" } & { data?: never })
  | ({ type: "traceability" } & { data?: never })
  | ({ type: "notes" } & { data?: never })
  | ({ type: "records" } & { data?: never })
  | ({ type: "artifacts" } & { data?: never })
  | ({ type: "media" } & { data?: never })
  | ({ type: "health" } & { data?: never })
  | ({ type: "telemetry_context" } & { data?: never });

export type DocumentCardDensity = "compact" | "normal" | "spacious";

export type DocumentCardLayout = {
  col_span?: ResponsiveSpan;
  row_span?: number;
  order?: number;
  density?: DocumentCardDensity;
  rail?: DocumentCardRail;
};

export type DocumentCardPlacementPayload = {
  card_id: Identifier;
  layout?: DocumentCardLayout;
};

export type DocumentCardRail = "main" | "right" | "left";

export type DocumentCard_Serialize = {
  metadata: Metadata;
  layout: DocumentCardLayout;
  filters: DocumentFilter[];
} & DocumentCardBody_Serialize;

export type DocumentFilter = {
  expression: DataExpression;
  operator: FilterOperator;
  value?: Value | null;
};

export type DocumentId = string;

export type DocumentSelection = {
  scope: RecordScope;
};

export type DocumentWarning = {
  code: Identifier;
  message: string;
};

export type ElapsedUnit = "seconds" | "milliseconds" | "minutes";

export type FftDetrend = "none" | "mean";

export type FftOptions = {
  sample_rate_hz?: number | null;
  window?: FftWindow;
  detrend?: FftDetrend;
  level_unit?: FrequencyLevelUnit;
  include_dc?: boolean;
  min_frequency_hz?: number | null;
  max_frequency_hz?: number | null;
};

export type FftWindow = "hann" | "rectangular";

export type FilterOperator = "eq" | "ne" | "gt" | "gte" | "lt" | "lte" | "contains" | "exists";

export type FrequencyLevelUnit = "linear" | "dB" | "dBu" | "dBm";

export type HistogramOptions = {
  bins?: number;
  unit?: string | null;
};

export type Identifier = string;

export type Metadata = {
  uuid?: string;
  created_at?: Timestamp;
  id?: string;
  identifier: Identifier;
  label: string;
  description?: string | null;
  tags?: string[];
};

/**
 * What a plot's x axis measures, as the plot definition declares it.
 *
 * A declaration, never an inference: a trace whose first sample is 0.1 is 0.1
 * of something, and only the author says of what. `wall_time` draws a date and
 * a clock face; `elapsed` draws a duration counted from the start of the run,
 * in `unit`.
 */
export type PlotAxisKind = { kind: "wall_time" } | { kind: "elapsed"; unit: string };

export type PlotDefinition = {
  metadata: Metadata;
  plot_type: PlotType;
  stream?: string | null;
  x?: string | null;
  y?: string[];
  z_stream?: string | null;
  z_field?: string | null;
  series_by?: string | null;
  x_expression?: DataExpression | null;
  y_expressions?: DataExpression[];
  z_expression?: DataExpression | null;
  series_by_expression?: DataExpression | null;
  filters?: DocumentFilter[];
  group?: string;
  unit?: string | null;
  style?: PlotStyle;
  x_scale?: AxisScale;
  y_scale?: AxisScale;
  fft?: FftOptions;
  analysis?: AnalysisSelection | null;
  interval?: SigmaIntervalOptions;
  histogram?: HistogramOptions;
  marker_record_types?: string[];
  reference_lines?: ReferenceLine[];
  reference_bands?: ReferenceBand[];
  live_window_s?: number | null;
  x_format?: ValueFormat | null;
  y_format?: ValueFormat | null;
  x_axis?: PlotAxisKind | null;
  display_points?: number | null;
  downsample?: DownsampleTechnique | null;
};

export type PlotDescriptor = {
  plot_id: Identifier;
  metadata: Metadata;
  plot_type: PlotType;
  input: PlotInput;
};

export type PlotInput = {
  plot_id: Identifier;
  scope: RecordScope;
  plot_type: PlotType;
  x_field: string | null;
  series_fields: string[];
  z_stream: string | null;
  z_field: string | null;
  stream: string | null;
  series_by: string | null;
  x_expression?: DataExpression | null;
  y_expressions?: DataExpression[];
  z_expression?: DataExpression | null;
  series_by_expression?: DataExpression | null;
  filters?: DocumentFilter[];
  unit: string | null;
  style: PlotStyle;
  x_scale: AxisScale;
  y_scale: AxisScale;
  fft: FftOptions;
  analysis?: AnalysisSelection | null;
  interval: SigmaIntervalOptions;
  histogram: HistogramOptions;
  marker_record_types: string[];
  reference_lines: ReferenceLine[];
  reference_bands: ReferenceBand[];
  live_window_s: number | null;
  available_streams: Identifier[];
  marker_count: number;
  x_format?: ValueFormat | null;
  y_format?: ValueFormat | null;
  x_axis?: PlotAxisKind | null;
  display_points?: number | null;
  downsample?: DownsampleTechnique | null;
};

export type PlotStyle = "line" | "points" | "line_points" | "step";

export type DownsampleTechnique = "lttb" | "min_max" | "raw";

export type PlotType =
  | "time_series"
  | "bar"
  | "grouped_bar"
  | "stacked_bar"
  | "pie"
  | "donut"
  | "pareto"
  | "xy_scatter"
  | "xy_line"
  | "xy_interval"
  | "trajectory"
  | "heatmap"
  | "xy_value_heatmap"
  | "wifi_channel_occupancy"
  | "frequency_series"
  | "histogram"
  | "box_summary"
  | "aggregate_table";

export type RecordMarker = {
  sequence: number;
  timestamp: Timestamp;
  record_type: Identifier;
  label: string;
  severity: RecordSeverity;
  scope: RecordScope | null;
};

export type RecordScope = {
  session_id?: SessionId | null;
  test_cycle_id?: TestCycleId | null;
  test_case_id?: CaseId | null;
  test_execution_id?: TestExecutionId | null;
  step_id?: StepId | null;
  criterion_id?: CriterionId | null;
  template_id?: TemplateId | null;
  suite_id?: SuiteId | null;
};

export type RecordSeverity = "debug" | "info" | "warning" | "error";

export type ReferenceAxis = "x" | "y";

export type ReferenceBand = {
  axis: ReferenceAxis;
  min?: number | null;
  max?: number | null;
  label?: string | null;
  color?: string;
  center?: DataExpression | null;
  tolerance?: DataExpression | null;
};

export type ReferenceLine = {
  axis: ReferenceAxis;
  value?: number | null;
  source?: DataExpression | null;
  source_scale?: number | null;
  label?: string | null;
  color?: string;
};

export type DocumentCardDefinition_Serialize = {
  metadata: Metadata;
  filters?: DocumentFilter[];
} & DocumentCardBody_Serialize;

export type DocumentDefinition_Serialize = {
  document_id: DocumentId;
  metadata: Metadata;
  cards: DocumentCardDefinition_Serialize[];
  layout: { nodes: DocumentNode[] };
};

export type DocumentNode =
  | {
      type: "section";
      data: {
        node_id: Identifier;
        metadata: Metadata;
        repeat?: DocumentSectionRepeat;
        children: DocumentNode[];
      };
    }
  | { type: "card"; data: DocumentCardPlacementPayload };

export type DocumentSectionRepeat = {
  value: DataExpression;
  label_template: string;
  id_template?: string | null;
  order_by?: DataExpression | null;
  filters?: DocumentFilter[];
};

export type CardBinding = {
  source_id: Identifier;
  metadata: Metadata;
  result:
    | { type: "static" }
    | { type: "summary"; data: { groups: TelemetrySummaryGroupPayload_Serialize[] } }
    | { type: "plot"; data: { descriptor: PlotDescriptor } };
};

export type SectionBinding = {
  metadata: Metadata;
  unavailable?: string | null;
};

export type RepeatInstance = {
  suffix: string;
  node_id: Identifier;
  filter: DocumentFilter;
};

export type DocumentCapture_Serialize = {
  definition: DocumentDefinition_Serialize;
  selection: DocumentSelection;
  cards: CardBinding[];
  repeats: Record<Identifier, RepeatInstance[]>;
  sections: Record<Identifier, SectionBinding>;
  warnings: DocumentWarning[];
};

export type ResponsiveSpan = {
  base?: number;
  sm?: number | null;
  md?: number | null;
  lg?: number | null;
  xl?: number | null;
};

export type SessionId = string;

export type SigmaIntervalOptions = {
  bin_width?: number | null;
  sigma?: number | null;
  clip_to_observed?: boolean;
};

export type StepId = string;

export type SuiteId = string;

export type TelemetrySummaryCondition = {
  operator: FilterOperator;
  value: Value;
  state: TelemetrySummaryState;
};

export type TelemetrySummaryDefinition_Serialize = {
  groups: TelemetrySummaryGroupDefinition_Serialize[];
};

export type TelemetrySummaryFormat =
  | "auto"
  | "number"
  | "percent"
  | "duration"
  | "distance"
  | "status"
  | "text";

export type TelemetrySummaryGroupDefinition_Serialize = {
  metadata: Metadata;
  tiles: TelemetrySummaryTileDefinition_Serialize[];
};

export type TelemetrySummaryGroupPayload_Serialize = {
  metadata: Metadata;
  tiles: TelemetrySummaryTilePayload_Serialize[];
};

export type TelemetrySummaryPresentation = "value" | "progress" | "badge";

export type TelemetrySummaryStat = "latest";

export type TelemetrySummaryState = "ok" | "warning" | "critical" | "muted";

export type TelemetrySummaryTileDefinition_Serialize = {
  metadata: Metadata;
  value: DataExpression;
  filters: DocumentFilter[];
  stat: TelemetrySummaryStat;
  format: TelemetrySummaryFormat;
  presentation: TelemetrySummaryPresentation;
  unit?: string | null;
  conditions: TelemetrySummaryCondition[];
};

export type TelemetrySummaryTilePayload_Serialize = {
  metadata: Metadata;
  value: Value | null;
  unavailable?: string | null;
  stat: TelemetrySummaryStat;
  format: TelemetrySummaryFormat;
  presentation: TelemetrySummaryPresentation;
  unit?: string | null;
  state?: TelemetrySummaryState | null;
};

export type TemplateId = string;

export type TestCycleId = string;

export type TestExecutionId = string;

export type Timestamp = string;

export type Value =
  | { type: "null" }
  | { type: "bool"; value: boolean }
  | { type: "integer" | "unsigned" | "number"; value: string }
  | { type: "text" | "datetime"; value: string }
  | { type: "array"; value: Value[] }
  | { type: "object"; value: Record<string, Value> };

export type ValueFormat =
  | { kind: "normal"; max_fraction_digits?: number | null }
  | { kind: "fixed"; decimals: number }
  | { kind: "scientific"; significant_digits: number }
  | { kind: "engineering"; significant_digits: number }
  | { kind: "si_prefix"; significant_digits: number }
  | { kind: "absolute_time"; pattern: string }
  | { kind: "elapsed_since_run"; unit: ElapsedUnit }
  | { kind: "elapsed_since_cycle"; unit: ElapsedUnit };
