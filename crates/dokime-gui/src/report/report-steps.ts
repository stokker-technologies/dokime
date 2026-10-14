/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { ReportPayload } from "@/lib/report-types";

/* === Public Functions === */

/** The payload carries step and criterion rows beside the executions, as the
    live adapters read them from their own call; each execution gets its own
    rows back in payload order, which is the order the run recorded them. */
export function executionsWithSteps(
  payload: Pick<ReportPayload, "test_executions" | "test_steps">,
): ReportPayload["test_executions"] {
  const steps = new Map<string, ReportPayload["test_steps"]>();
  for (const step of payload.test_steps) {
    const owned = steps.get(step.test_execution_id);
    if (owned) owned.push(step);
    else steps.set(step.test_execution_id, [step]);
  }
  return payload.test_executions.map((execution) => ({
    ...execution,
    steps: steps.get(execution.test_execution_id) ?? [],
  }));
}
