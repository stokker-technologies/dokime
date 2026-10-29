/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Live Execution Selection And Honest Progress.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import type { TestExecution, TestCyclePlan } from "@/lib/types";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */
export type RunFocus = { cycleLabel: string; executionLabel: string; execution: TestExecution; percent: number | null };

/* === Classes === */

/* === Public Functions === */
export function activeRunFocus(details: readonly { plan: TestCyclePlan | null; executions: TestExecution[] }[]): RunFocus[] {
  const active: RunFocus[] = [];
  for (const { plan, executions } of details) {
    if (!plan) continue;
    for (const execution of executions) {
      if (!["running", "waiting_for_prompt", "finalizing"].includes(execution.status)) continue;
      const owner = plan.cases.find((item) => item.case_id === execution.case_id);
      const template = plan.templates.find((item) => item.template_id === execution.template_id);
      const total = template?.steps.length ?? 0;
      // A summary with no step records cannot report how much has completed.
      const steps = execution.steps.filter((item) => item.record_type === "step");
      const planned = new Set(template?.steps.map((step) => step.step_id));
      const completed = new Set(
        steps.filter((item) => planned.has(item.step_id) && !["pending", "running"].includes(item.status)).map((item) => item.step_id),
      ).size;
      active.push({
        cycleLabel: plan.test_cycle.title || plan.test_cycle.test_cycle_id,
        executionLabel:
          typeof execution.metadata.label === "string" && execution.metadata.label
            ? execution.metadata.label
            : `${owner?.label ?? execution.case_id} · ${execution.attempt_index + 1}`,
        execution,
        percent: total > 0 && steps.length > 0 ? Math.min(100, Math.round((completed / total) * 100)) : null,
      });
    }
  }
  return active.sort(
    (a, b) =>
      (Date.parse(b.execution.started_at ?? "") || 0) - (Date.parse(a.execution.started_at ?? "") || 0) ||
      b.execution.test_execution_id.localeCompare(a.execution.test_execution_id),
  );
}
export function elapsedMinutes(startedAt: string | null | undefined, now: number): number | null {
  const start = Date.parse(startedAt ?? "");
  return Number.isFinite(start) ? Math.max(0, Math.floor((now - start) / 60000)) : null;
}
export function runElapsed(minutes: number | null): string {
  if (minutes === null) return "";
  const days = Math.floor(minutes / 1440);
  const hours = Math.floor((minutes % 1440) / 60);
  return `${days}d ${hours}h ${minutes % 60}m`;
}
export function runDestination(focus: readonly RunFocus[]) {
  const first = focus[0];
  if (!first) return null;
  return focus.length > 1
    ? { to: "/cycles" as const, hash: "active" }
    : {
        to: "/cycles/$cycleId" as const,
        params: { cycleId: first.execution.test_cycle_id },
        search: { executionId: first.execution.test_execution_id },
      };
}

/* === Private Functions === */

/* === Entry Point === */
