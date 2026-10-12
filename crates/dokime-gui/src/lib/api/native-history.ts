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

import {
  CycleStatus,
  type CycleSummary,
  ExecutionStatus,
  ResultStatus,
} from "@/lib/rpc/generated/dokime/v1/execution_pb";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { CycleHistorySummary } from "@/lib/types";
import { listNativeCycles, listNativeExecutions } from "./native-planning";
import { durationSeconds, mapBounded, safeNumber, timestampIso } from "./native-values";

/* === Public API === */

/**
 * Read the native cycle index and project its authored summary fields into the
 * view model consumed by the original cycle selector/sidebar.
 *
 * CycleSummary intentionally does not contain a per-execution status
 * histogram. Active cycles therefore read execution summaries as a bounded
 * second pass so running counts stay truthful without loading execution
 * details, steps, or evidence.
 */
export async function fetchNativeCycleHistory(
  session: SessionIdentity,
): Promise<CycleHistorySummary[]> {
  const cycles = await listNativeCycles(session);
  const activeCycles = cycles.filter((cycle) =>
    [CycleStatus.RUNNING, CycleStatus.PAUSED, CycleStatus.STOPPING].includes(cycle.status),
  );
  const activeExecutions = await mapBounded(activeCycles, (cycle) =>
    listNativeExecutions(session, cycle.cycleId),
  );
  const activeByCycleId = new Map(
    activeCycles.map((cycle, index) => [cycle.cycleId, activeExecutions[index]]),
  );

  return cycles
    .map((cycle) => mapCycleHistorySummary(cycle, activeByCycleId.get(cycle.cycleId)))
    .sort(compareCycleHistorySummary);
}

/* === Mapping === */

function mapCycleHistorySummary(
  cycle: CycleSummary,
  executions: Awaited<ReturnType<typeof listNativeExecutions>> | undefined,
): CycleHistorySummary {
  const caseCount = safeNumber(cycle.caseCount, "Execution.ListCycles case count");
  const executionCount = safeNumber(cycle.executionCount, "Execution.ListCycles execution count");
  const result = cycle.result;
  const terminal = [
    CycleStatus.COMPLETED,
    CycleStatus.FAILED,
    CycleStatus.BLOCKED,
    CycleStatus.STOPPED,
    CycleStatus.CLEARED,
  ].includes(cycle.status);
  const startedAt = timestampIso(cycle.startedAt);
  const updatedAt = timestampIso(cycle.updatedAt) ?? timestampIso(cycle.createdAt) ?? "";
  const finishedAt = terminal ? updatedAt || null : null;
  const activeRunning = executions?.filter(
    (item) => item.status === ExecutionStatus.RUNNING,
  ).length;

  return {
    test_cycle_id: cycle.cycleId,
    session_id: null,
    title: cycle.metadata?.label || cycle.cycleId,
    // The current execution index carries no suite identity. The selector
    // falls back to the cycle id until the history checkpoint descriptor is
    // available, preserving the native value instead of guessing a suite.
    suite_id: null,
    suite_name: null,
    lifecycle: cycleLifecycle(cycle.status),
    result_status: resultStatus(result?.status),
    case_count: caseCount,
    execution_count: executionCount,
    queued_count: Math.max(caseCount - executionCount, 0),
    running_count: activeRunning ?? 0,
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
  switch (status) {
    case CycleStatus.PREVIEWED:
      return "previewed";
    case CycleStatus.READY:
      return "ready";
    case CycleStatus.RUNNING:
      return "running";
    case CycleStatus.PAUSED:
      return "paused";
    case CycleStatus.STOPPING:
      return "stopping";
    case CycleStatus.COMPLETED:
      return "completed";
    case CycleStatus.FAILED:
      return "failed";
    case CycleStatus.BLOCKED:
      return "blocked";
    case CycleStatus.STOPPED:
      return "stopped";
    case CycleStatus.CLEARED:
      return "cleared";
    default:
      return "unspecified";
  }
}

function resultStatus(status?: ResultStatus): string | null {
  switch (status) {
    case ResultStatus.PENDING:
      return "pending";
    case ResultStatus.PASSED:
      return "passed";
    case ResultStatus.FAILED:
      return "failed";
    case ResultStatus.BLOCKED:
      return "blocked";
    case ResultStatus.CANCELLED:
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
