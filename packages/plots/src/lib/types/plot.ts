/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type * as model from "@host/model";
import type { PlotDomain, ContourGrid, GridFrames, GridAxisEntry, PlotPayloadDiagnostics, ValueFormat } from "@host/model";
import type { Wire } from "../rpc/canonical";
import type { SeriesSummary, StatisticValue, StatsSummary } from "../statistics";
import type { JsonObject } from "../types";
import type { DocumentDataExpression } from "./document";

export type PlotPayloadStreamKind = "snapshot" | "replace" | "delta" | "statistics" | "error";

/* === Plot & telemetry types ===
   PlotPoint/Series/Domain/Cartesian/Marker, the document plot-config types
   (Fft/Axis/Reference/Histogram/Sigma) and the PlotPayload* frame family. */

/**
 * What a plot's x axis measures, as the document declares it.
 *
 * Local axis state derived from the generated plot presentation:
 * `wall_time` is an absolute instant, `elapsed` a duration counted from the
 * start of the run in `unit`. An absent declaration is not a statement, and
 * only then does the x column's own name get a say.
 */
export type DocumentPlotAxisKind = { kind: "wall_time" } | { kind: "elapsed"; unit: string };

// Renderer points may contain derived statistics and non-wire axis values.
export type PlotPoint = {
  y: number | null;
  z?: number | null;
  x: unknown;
  instant?: string | null;
  exact_y?: StatisticValue;
  low?: number | null;
  high?: number | null;
  mean?: number | null;
  stddev?: number | null;
  minimum?: number | null;
  maximum?: number | null;
  sample_count?: number | null;
};

export type PlotSeries = Pick<Wire<model.PlotSeries>, "key" | "label" | "downsampled"> & {
  unit?: Wire<model.PlotSeries>["unit"] | null;
  format?: Wire<ValueFormat> | null;
  raw_points: number;
  points: PlotPoint[];
};

export type PlotCartesianMode = "aligned" | "faceted";

export type PlotCartesianFrame = Pick<Wire<model.PlotFrame>, "signature"> & {
  mode: PlotCartesianMode;
  series: { x: number[]; y: (number | null)[] }[];
  point_count: number;
  x_domain?: Wire<PlotDomain> | null;
  y_domain?: Wire<PlotDomain> | null;
};

export type PlotMarker = Pick<Wire<model.PlotMarker>, "label" | "color"> & { x: unknown; record_type: model.PlotMarker["recordType"] };

export type DocumentPlotType =
  | "time_series"
  | "frequency_series"
  | "xy"
  | "trajectory"
  | "contour"
  | "grid_chart"
  | "heat_strip"
  | "bar"
  | "pie"
  | "histogram"
  | "box_plot";

export type DocumentPlotStyle = "line" | "points" | "line_points" | "step";

export type DocumentAxisScale = "linear" | "log";

/** How a plot is reduced to its display budget; the host default is `auto`. */
export type DocumentDownsampleTechnique = "auto" | "lttb" | "min_max" | "raw";

export type DocumentFrequencyLevelUnit = "linear" | "dB" | "dBu" | "dBm";

export type DocumentFftWindow = "hann" | "rectangular";

export type DocumentFftDetrend = "none" | "mean";

export type DocumentFftOptions = {
  input_samples?: number | null;
  sample_rate_hz?: number | null;
  window: DocumentFftWindow;
  detrend: DocumentFftDetrend;
  level_unit: DocumentFrequencyLevelUnit;
  include_dc: boolean;
  min_frequency_hz?: number | null;
  max_frequency_hz?: number | null;
};

export type DocumentSigmaInterval = {
  bin_width: number;
  sigma: number;
  clip_to_observed: boolean;
};

export type DocumentHistogramOptions = {
  bins: number;
  unit?: string | null;
};

export type DocumentReferenceLine = {
  value?: number | null;
  label?: string | null;
  color: string;
};

export type DocumentReferenceBand = {
  min?: number | null;
  max?: number | null;
  label?: string | null;
  color: string;
};

export type PlotAnalysis =
  | { kind: "error"; message: string }
  | {
      kind: "histogram";
      series: {
        key: string;
        count: number;
        statistics?: StatsSummary | null;
        bins: { lower: number; upper: number; count: number }[];
      }[];
    }
  | {
      kind: "box_plot";
      series: {
        key: string;
        label: string;
        count: number;
        min: StatisticValue;
        q1: StatisticValue;
        median: StatisticValue;
        q3: StatisticValue;
        max: StatisticValue;
        lower_whisker: number;
        upper_whisker: number;
        outliers: number[];
      }[];
      /** Samples summarized across every box. */
      total: number;
    }
  | {
      kind: "sigma_interval";
      series: {
        key: string;
        bins: { x: number; count: number; mean: number; lower: number; upper: number }[];
        /** Samples the bins summarize. */
        raw_points: number;
      }[];
    }
  | { kind: "contour"; grid: Wire<ContourGrid> }
  | { kind: "grid_chart"; data: Wire<GridFrames> };

export type GridHighlight = {
  row: DocumentDataExpression;
  column: DocumentDataExpression;
  label: string;
};
export type GridDefinition = {
  row?: DocumentDataExpression | null;
  column?: DocumentDataExpression | null;
  frame?: DocumentDataExpression | null;
  rows: Wire<GridAxisEntry>[];
  columns: Wire<GridAxisEntry>[];
  highlights: GridHighlight[];
  summaryLabel: string;
};

// Renderer payloads also represent locally materialized, partial plot data.
export type PlotPayload = Pick<Wire<model.PlotPayload>, "stream"> & {
  x_key: model.PlotPayload["xKey"];
  max_points: number;
  generated_at: model.PlotPayload["generatedAt"];
  series_by?: model.PlotPayload["seriesBy"] | null;
  window_s?: model.PlotPayload["windowS"] | null;
  x_format?: Wire<ValueFormat> | null;
  run_started_at?: model.PlotPayload["runStartedAt"] | null;
  cycle_started_at?: model.PlotPayload["cycleStartedAt"] | null;
  context?: JsonObject | null;
  series: PlotSeries[];
  analysis?: PlotAnalysis | null;
  cartesian?: PlotCartesianFrame | null;
  markers?: PlotMarker[];
  /** One per series of a time series or scatter plot, over the shown window. */
  summaries?: SeriesSummary[];
  bucket_width_s?: number | null;
  bucket_window?: { from: number; to: number } | null;
  replaced_buckets?: number[];
};

export type PlotPayloadEnvelope = Pick<Wire<model.PlotEnvelope>, "epoch" | "selection" | "warnings"> & {
  plot_id: model.PlotEnvelope["plotId"];
  revision: number;
  payload: PlotPayload;
  diagnostics?: Wire<PlotPayloadDiagnostics>;
};

export type PlotPayloadStreamMessage = PlotPayloadEnvelope & {
  kind: PlotPayloadStreamKind;
};
