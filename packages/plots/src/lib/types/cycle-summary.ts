/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { DocumentStatistic, ExecutionFact, TestExecution } from "../types";
import type { PlotPayload } from "./plot";

/* === Cycle-summary types ===
   CycleSummary* definitions + datasets/aggregates + ResolvedCycleSummary* views. */

export type CycleSummarySource = {
  kind:
    | "case_parameter"
    | "execution_field"
    | "environment_field"
    | "telemetry_stat"
    | "telemetry_field"
    | "telemetry_elapsed_minutes"
    | "telemetry_regression_slope"
    | "traceability_field";
  path?: string | null;
  stream?: string | null;
  field?: string | null;
  x_stream?: string | null;
  x_field?: string | null;
  stat?: DocumentStatistic | null;
};

export type CycleSummaryMetricItem = {
  item_type: "metric";
  metric_id: string;
  label: string;
  measure: CycleSummarySource;
  reduce: DocumentStatistic;
  unit?: string | null;
  group: string;
  scale: number;
};

export type CycleSummaryPlotItem = {
  item_type: "plot";
  plot_id: string;
  title: string;
  plot_type: "bar" | "box" | "xy";
  x: CycleSummarySource;
  y: CycleSummarySource;
  series_by?: CycleSummarySource | null;
  mode: "scatter" | "interval" | "line";
  x_bin_size?: number | null;
  reduce: DocumentStatistic;
  unit?: string | null;
  group: string;
  description?: string | null;
  scale: number;
};

export type CycleSummaryDefinition = {
  schema_version: number;
  title?: string | null;
  group?: string | null;
  items: Array<CycleSummaryMetricItem | CycleSummaryPlotItem>;
};

export type ResolvedCycleSummaryMetric = {
  metric_id: string;
  label: string;
  value?: unknown;
  formatted: string;
  unit?: string | null;
  group: string;
  sample_count: number;
};

export type ResolvedCycleSummaryPlot = {
  plot_id: string;
  title: string;
  cycle_summary_plot_type: "bar" | "box" | "xy";
  mode: "scatter" | "interval" | "line";
  group: string;
  description?: string | null;
  unit?: string | null;
  payload: PlotPayload;
};

export type ResolvedCycleSummarySection = {
  template_id: string;
  title: string;
  group?: string | null;
  metrics: ResolvedCycleSummaryMetric[];
  plots: ResolvedCycleSummaryPlot[];
};

export type ResolvedCycleSummary = {
  schema_version: number;
  test_cycle_id?: string;
  generated_at?: string;
  selection?: CycleSummarySelection;
  executions?: CycleSummaryExecutionRow[];
  aggregates?: CycleSummaryMetricAggregate[];
  cycle_aggregates?: CycleSummaryMetricAggregate[];
  facts?: ExecutionFact[];
  sections?: ResolvedCycleSummarySection[];
};

export type CycleSummaryTelemetryFieldSource = "values" | "summary";

export type CycleSummaryScopeLevel = "execution" | "cycle";

export type CycleSummarySelection = {
  execution_ids: string[];
};

export type CycleSummaryDimensionValue = {
  source: "execution_field" | "case_parameter" | "traceability_field";
  key: string;
  label: string;
  value: unknown;
};

export type CycleSummaryExecutionRow = {
  test_execution_id: string;
  queue_item_id: string;
  case_id: string;
  template_id: string;
  ordinal: number;
  status: TestExecution["status"];
  dimensions: CycleSummaryDimensionValue[];
};

export type CycleSummaryAggregateStats = {
  count: number;
  first: number;
  last: number;
  min: number;
  max: number;
  mean: number;
  median: number;
  std: number;
  rms: number;
  max_abs: number;
  sum: number;
  p95: number;
  p99: number;
};

export type CycleSummaryMetricAggregate = {
  scope_level: CycleSummaryScopeLevel;
  test_execution_id?: string | null;
  stream: string;
  schema_version: number;
  field_source: CycleSummaryTelemetryFieldSource;
  field: string;
  stats: CycleSummaryAggregateStats;
};
