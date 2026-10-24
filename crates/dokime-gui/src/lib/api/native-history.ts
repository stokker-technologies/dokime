/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native cycle-history projection for the original routed UI. */

/* === External Imports === */

/* === Local Imports === */

import type { Wire } from "@/lib/rpc/canonical";
import type * as rpc from "@/lib/rpc/generated/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { CycleHistorySummary } from "@/lib/types";
import { listNativeCycles, mapCyclePhase } from "./native-planning";
import { durationSeconds, safeNumber, timestampIso } from "./native-values";

/* === Types === */

type CycleSummary = Wire<Awaited<ReturnType<typeof rpc.executionGetCycle>>>;
type CycleStatus = CycleSummary["status"];
type ResultStatus = NonNullable<CycleSummary["result"]>["status"];

/* === Public API === */

/**
 * Read the native cycle index and project its summary fields, running and
 * queued counts included, into the view model consumed by the original cycle
 * selector/sidebar.
 */
export async function fetchNativeCycleHistory(session: SessionIdentity): Promise<CycleHistorySummary[]> {
  const cycles = await listNativeCycles(session);
  return cycles.map(mapCycleHistorySummary).sort(compareCycleHistorySummary);
}

/* === Mapping === */

function mapCycleHistorySummary(cycle: CycleSummary): CycleHistorySummary {
  const caseCount = safeNumber(cycle.caseCount, "Execution.ListCycles case count");
  const executionCount = safeNumber(cycle.executionCount, "Execution.ListCycles execution count");
  const result = cycle.result;
  const terminal = ["Completed", "Failed", "Blocked", "Stopped"].includes(cycle.status?.HarmosVariant.case ?? "");
  const startedAt = timestampIso(cycle.startedAt);
  const updatedAt = timestampIso(cycle.updatedAt) ?? timestampIso(cycle.createdAt) ?? "";
  const finishedAt = terminal ? updatedAt || null : null;

  return {
    test_cycle_id: cycle.cycleId,
    session_id: null,
    title: cycle.metadata?.label || cycle.cycleId,
    phase: mapCyclePhase(cycle.phase),
    lifecycle: cycleLifecycle(cycle.status),
    result_status: resultStatus(result?.status),
    case_count: caseCount,
    execution_count: executionCount,
    queued_count: safeNumber(cycle.queuedCount, "Execution.ListCycles queued count"),
    running_count: safeNumber(cycle.runningCount, "Execution.ListCycles running count"),
    passed_count: result?.passed ?? 0,
    failed_count: result?.failed ?? 0,
    blocked_count: result?.blocked ?? 0,
    created_at: timestampIso(cycle.createdAt) ?? updatedAt,
    started_at: startedAt,
    finished_at: finishedAt,
    updated_at: updatedAt,
    duration_s: durationSeconds(cycle.startedAt, cycle.updatedAt),
  };
}

function cycleLifecycle(status: CycleStatus): string {
  switch (status?.HarmosVariant.case) {
    case "Ready":
      return "ready";
    case "Running":
      return "running";
    case "Paused":
      return "paused";
    case "Stopping":
      return "stopping";
    case "Completed":
      return "completed";
    case "Failed":
      return "failed";
    case "Blocked":
      return "blocked";
    case "Stopped":
      return "stopped";
    default:
      return "unspecified";
  }
}

function resultStatus(status?: ResultStatus): string | null {
  switch (status?.HarmosVariant.case) {
    case "Pending":
      return "pending";
    case "Passed":
      return "passed";
    case "Failed":
      return "failed";
    case "Blocked":
      return "blocked";
    case "Cancelled":
      return "cancelled";
    default:
      return null;
  }
}

function compareCycleHistorySummary(left: CycleHistorySummary, right: CycleHistorySummary): number {
  return (
    (right.started_at ?? right.updated_at).localeCompare(left.started_at ?? left.updated_at) ||
    right.updated_at.localeCompare(left.updated_at) ||
    right.created_at.localeCompare(left.created_at)
  );
}
