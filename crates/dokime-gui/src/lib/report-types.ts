/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { DocumentCapturePayload } from "./document-normalizer";
import type {
  HubSnapshot,
  PlotPayload,
  RecordScope,
  ResolvedCycleSummary,
  StepExecutionRecord,
  TestCyclePlan,
  TestExecution,
  TraceabilitySnapshot,
} from "./types";

/* === Types === */

export type ReportArtifactLink = {
  artifact_id: string;
  label: string;
  href: string;
  path: string;
  media_type?: string | null;
  description?: string | null;
  size_bytes?: number | null;
  group_id?: string | null;
  group_label?: string | null;
  scope?: RecordScope | null;
};

export type DocumentHistogramPayload = {
  histogram_id: string;
  title: string;
  stream: string;
  field: string;
  group: string;
  unit?: string | null;
  display_unit: string;
  bins: number;
  values: number[];
};

export type DocumentBoxSummaryPayload = {
  box_summary_id: string;
  title: string;
  stream: string;
  field: string;
  group: string;
  unit?: string | null;
  display_unit: string;
  sample_count: number;
  minimum?: number | null;
  q1?: number | null;
  median?: number | null;
  q3?: number | null;
  maximum?: number | null;
  outliers: number[];
};

export type ReportCaseRunPayload = {
  queue_item_id: string;
  case_id: string;
  test_execution_id?: string | null;
  traceability?: TraceabilitySnapshot | null;
  documents: Record<string, DocumentCapturePayload>;
  plots: Record<string, PlotPayload>;
  histograms: Record<string, DocumentHistogramPayload>;
  box_summaries: Record<string, DocumentBoxSummaryPayload>;
};

export type ReportHubInfo = {
  name?: string | null;
  version?: string | null;
  generator?: string | null;
};

export type ReportPayload = {
  schema_version: number;
  generated_at?: string | null;
  hub?: ReportHubInfo | null;
  snapshot?: HubSnapshot | null;
  documents: Record<string, DocumentCapturePayload>;
  test_cycle_plan?: TestCyclePlan | null;
  test_executions: TestExecution[];
  test_steps: StepExecutionRecord[];
  plots: Record<string, PlotPayload>;
  histograms: Record<string, DocumentHistogramPayload>;
  box_summaries: Record<string, DocumentBoxSummaryPayload>;
  case_runs: ReportCaseRunPayload[];
  cycle_summary?: ResolvedCycleSummary | null;
  artifact_links: ReportArtifactLink[];
  /** The hub appearance frame captured at generation time (theme contract for the frozen report). */
  appearance?: unknown;
  warnings: string[];
};
