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
  ResolvedCycleSummary,
  TestCyclePlan,
  TestExecution,
  TraceabilitySnapshot,
} from "./types";

/* === Types === */

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
  /** Notes and health arrive once per subject, artifacts with their download target. */
  snapshot?: HubSnapshot | null;
  documents: Record<string, DocumentCapturePayload>;
  test_cycle_plan?: TestCyclePlan | null;
  /** Each execution already carries its own step and criterion rows. */
  test_executions: TestExecution[];
  plots: Record<string, PlotPayload>;
  histograms: Record<string, DocumentHistogramPayload>;
  box_summaries: Record<string, DocumentBoxSummaryPayload>;
  case_runs: ReportCaseRunPayload[];
  cycle_summary?: ResolvedCycleSummary | null;
  /** The hub appearance frame captured at generation time (theme contract for the frozen report). */
  appearance?: unknown;
  warnings: string[];
};
