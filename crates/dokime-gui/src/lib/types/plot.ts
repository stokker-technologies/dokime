/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { StatsSummary } from "../statistics";
import type { DocumentDataExpression, DocumentValueSource } from "../types";
import type { ParameterDefinition } from "./test";
import type {
  PlotCartesianFrame as NativePlotCartesianFrame,
  PlotCartesianSeriesFrame as NativePlotCartesianSeriesFrame,
  PlotDomain as NativePlotDomain,
  PlotMarker as NativePlotMarker,
  PlotPayload_Serialize as NativePlotPayload,
  PlotPayloadEnvelope_Serialize as NativePlotPayloadEnvelope,
  PlotPoint_Serialize as NativePlotPoint,
  PlotSeries_Serialize as NativePlotSeries,
} from "./view-contracts";

export type { PlotPayloadCursor, PlotPayloadDiagnostics } from "./view-contracts";

import type { PlotPayloadDiagnostics, PlotPayloadStreamKind } from "./view-contracts";

/* === Plot & telemetry types ===
   PlotPoint/Series/Domain/Cartesian/Marker, the document plot-config types
   (Fft/Axis/Reference/Histogram/Sigma) and the PlotPayload* frame family. */

/**
 * What a plot's x axis measures, as the document declares it.
 *
 * The renderer-side mirror of `PlotAxisKind` in `lib/document-contracts.ts`
 * and of `documents.proto`'s `PlotPresentation.x_axis` plus its `x_axis_unit`:
 * `wall_time` is an absolute instant, `elapsed` a duration counted from the
 * start of the run in `unit`. An absent declaration is not a statement, and
 * only then does the x column's own name get a say.
 */
export type DocumentPlotAxisKind = { kind: "wall_time" } | { kind: "elapsed"; unit: string };

// Renderer points may contain derived statistics and non-wire axis values.
export type PlotPoint = Omit<NativePlotPoint, "x"> & {
  x: unknown;
  low?: number | null;
  high?: number | null;
  mean?: number | null;
  stddev?: number | null;
  minimum?: number | null;
  maximum?: number | null;
  sample_count?: number | null;
};

export type PlotSeries = Omit<NativePlotSeries, "points" | "unit"> &
  Partial<Pick<NativePlotSeries, "unit">> & { points: PlotPoint[] };

export type PlotDomain = { [Key in keyof NativePlotDomain]: NonNullable<NativePlotDomain[Key]> };

export type PlotCartesianFrame = Omit<
  NativePlotCartesianFrame,
  "series" | "x_domain" | "y_domain"
> & {
  series: Array<Omit<NativePlotCartesianSeriesFrame, "x"> & { x: number[] }>;
  x_domain?: PlotDomain | null;
  y_domain?: PlotDomain | null;
};

export type PlotMarker = Omit<NativePlotMarker, "x"> & { x: unknown };

export type DocumentPlotType =
  | "time_series"
  | "bar"
  | "xy_scatter"
  | "xy_line"
  | "xy_interval"
  | "trajectory"
  | "heatmap"
  | "xy_value_heatmap"
  | "wifi_channel_occupancy"
  | "frequency_series"
  | "histogram"
  | "box_summary";

export type DocumentPlotStyle = "line" | "points" | "line_points" | "step";

export type DocumentAxisScale = "linear" | "log";

/** How a plot is reduced to its display budget; the host default is `auto`. */
export type DocumentDownsampleTechnique = "auto" | "lttb" | "min_max" | "raw";

export type DocumentFrequencyLevelUnit = "linear" | "dB" | "dBu" | "dBm";

export type DocumentFftWindow = "hann" | "rectangular";

export type DocumentFftDetrend = "none" | "mean";

export type DocumentFftOptions = {
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

export type DocumentReferenceAxis = "x" | "y";

export type DocumentReferenceLine = {
  axis: DocumentReferenceAxis;
  value?: number | null;
  source?: DocumentDataExpression | DocumentValueSource | null;
  source_scale?: number | null;
  label?: string | null;
  color: string;
};

export type DocumentReferenceBand = {
  axis: DocumentReferenceAxis;
  min?: number | null;
  max?: number | null;
  min_source?: DocumentValueSource | null;
  max_source?: DocumentValueSource | null;
  label?: string | null;
  color: string;
  center?: DocumentDataExpression | null;
  tolerance?: DocumentDataExpression | null;
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
      kind: "box_summary";
      series: {
        key: string;
        label: string;
        count: number;
        min: number;
        q1: number;
        median: number;
        q3: number;
        max: number;
        lower_whisker: number;
        upper_whisker: number;
        outliers: number[];
      }[];
    }
  | {
      kind: "sigma_interval";
      series: {
        key: string;
        bins: { x: number; count: number; mean: number; lower: number; upper: number }[];
      }[];
    };

// Renderer payloads also represent locally materialized, partial plot data.
export type PlotPayload = Pick<
  NativePlotPayload,
  "stream" | "x_key" | "max_points" | "generated_at"
> &
  Partial<
    Pick<
      NativePlotPayload,
      "series_by" | "context" | "window_s" | "x_format" | "run_started_at" | "cycle_started_at"
    >
  > & {
    series: PlotSeries[];
    computed?: EvidenceAnalysisResult | null;
    analysis?: PlotAnalysis | null;
    cartesian?: PlotCartesianFrame | null;
    markers?: PlotMarker[];
  };

export type PlotPayloadEnvelope = Omit<NativePlotPayloadEnvelope, "payload" | "diagnostics"> & {
  payload: PlotPayload;
  diagnostics?: PlotPayloadDiagnostics;
};

export type PlotPayloadStreamMessage = PlotPayloadEnvelope & {
  kind: PlotPayloadStreamKind;
};

/** GUI projections of the admitted numerical analysis catalog and results. */
export type EvidenceAnalysisDefinition = {
  id: string;
  version: number;
  label: string;
  description: string;
  parameters: ParameterDefinition[];
  outputs: { key: string; label: string; unit?: string; axis?: { key: string; unit?: string } }[];
  maxSamples: number;
  maxOutputValues: number;
};

export type EvidenceAnalysisSelection = {
  id: string;
  version: number;
  parameters: Record<string, unknown>;
};

export type EvidenceAnalysisResult = {
  definition: Omit<EvidenceAnalysisDefinition, "parameters"> & { parameters?: unknown[] };
  selection: EvidenceAnalysisSelection;
  source: {
    epoch: string;
    stream: string;
    scope: import("../types").RecordScope | null;
    telemetryStreams: Record<string, string>;
    recordSequence?: string;
    windowS?: number;
    fields: string[];
    xKey: string;
  };
  series: {
    key: string;
    label: string;
    origin?: string;
    inputSamples: number;
    firstAxis: number;
    lastAxis: number;
    outputs: {
      key: string;
      unit: string;
      axisUnit: string;
      displayIndices?: number[];
      axis: number[];
      values: number[];
    }[];
  }[];
};
