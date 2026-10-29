/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { DocumentCapturePayload } from "./document-normalizer";
import type { HubSnapshot, PlotPayload, ResolvedCycleSummary, TestCyclePlan, TestExecution, TraceabilitySnapshot } from "./types";

export type ReportCaseRunPayload = {
  queue_item_id: string;
  case_id: string;
  test_execution_id?: string | null;
  traceability?: TraceabilitySnapshot | null;
  documents: Record<string, DocumentCapturePayload>;
  plots: Record<string, PlotPayload>;
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
  case_runs: ReportCaseRunPayload[];
  cycle_summary?: ResolvedCycleSummary | null;
  /** The hub appearance frame captured at generation time (theme contract for the frozen report). */
  appearance?: unknown;
  warnings: string[];
};
