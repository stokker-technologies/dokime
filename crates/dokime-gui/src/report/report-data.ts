/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { serializedValueObjectToJson } from "@/lib/api/native-values";
import type { Value as SerializedValue } from "@/lib/document-contracts";
import type { ReportPayload } from "@/lib/report-types";
import type { PlotPayload } from "@/lib/types";

/* === Constants === */

const REPORT_DATA_ID = "hub-report-data";

const EMPTY_REPORT_PAYLOAD: ReportPayload = {
  generated_at: null,
  snapshot: null,
  documents: {},
  test_cycle_plan: null,
  test_executions: [],
  plots: {},
  case_runs: [],
  cycle_summary: null,
  warnings: ["Report data is unavailable."],
};

/* === Public Functions === */

export function readReportPayload(): ReportPayload {
  const node = document.getElementById(REPORT_DATA_ID);
  const text = node?.textContent?.trim();
  if (!text) {
    return EMPTY_REPORT_PAYLOAD;
  }

  try {
    const parsed = JSON.parse(text) as Partial<ReportPayload>;
    return {
      ...EMPTY_REPORT_PAYLOAD,
      ...parsed,
      test_executions: parsed.test_executions ?? [],
      documents: parsed.documents ?? {},
      plots: capturedPlots(parsed.plots),
      case_runs: (parsed.case_runs ?? []).map((caseRun) => ({
        ...caseRun,
        documents: caseRun.documents ?? {},
        plots: capturedPlots(caseRun.plots),
      })),
      cycle_summary: parsed.cycle_summary ?? null,
      warnings: parsed.warnings ?? [],
    };
  } catch (exc) {
    return {
      ...EMPTY_REPORT_PAYLOAD,
      warnings: [`Report data could not be parsed: ${String(exc)}`],
    };
  }
}

/* === Private Functions === */

function capturedPlots(plots?: Record<string, PlotPayload>): Record<string, PlotPayload> {
  return Object.fromEntries(
    Object.entries(plots ?? {}).map(([key, payload]) => {
      const context = serializedValueObjectToJson((payload.context ?? {}) as Record<string, SerializedValue>);
      return [key, { ...payload, context }];
    }),
  );
}
