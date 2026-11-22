/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Native Response Projection
 */

/* === Imports === */

import type * as hostModel from "@generated/host/model";

import { defaultBrand } from "../brand";

import type { Wire } from "@/lib/rpc/canonical";

import type { EvidenceScope } from "@generated/host/model";

import { CycleStatus, ResultStatus } from "@generated/host/model";

import type { ExecutionResultSummary, HubLiveState, RunStatus } from "@/lib/types";

import { nowIso, safeNumber, timestampIso } from "@/lib/api/native-values";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type RuntimeShellSnapshot = Wire<NonNullable<hostModel.OverviewGetResponse["overview"]>>;

/* === Classes === */

/* === Public Functions === */

export function mapRuntimeShell(snapshots: readonly RuntimeShellSnapshot[], cycleId: string): HubLiveState {
  const first = snapshots[0];
  if (!first) throw new Error("Native runtime shell returned no snapshot.");
  const cycles = snapshots.flatMap((snapshot) => snapshot.cycleSummaries);
  // The caller names the run this shell describes. There is no fallback
  // choice: a shell that cannot say which cycle it belongs to belongs to none.
  const cycle = cycles.find((candidate) => candidate.cycleId === cycleId) ?? null;
  const title = cycle?.metadata?.label || first.hubMetadata?.label || defaultBrand.label;
  const updatedAt = timestampIso(cycle?.updatedAt) ?? timestampIso(cycle?.createdAt) ?? nowIso();
  const revision = safeNumber(first.revision, "Backend.Overview.Get revision");
  return {
    run: {
      session_id: first.sessionId,
      title,
      template_id: null,
      template_slug: "",
      label: title,
      metadata: {},
      status: mapRunStatus(cycle?.status),
      started_at: timestampIso(cycle?.startedAt),
      finished_at: cycle && isTerminalCycle(cycle.status) ? updatedAt : null,
      updated_at: updatedAt,
      run_dir: null,
      local_url: null,
      lan_url: null,
      rerun_command: null,
    },
    session_id: first.sessionId,
    title,
    revision,
    records: [],
    telemetry: {},
    pending_prompts: [],
    notes: [],
    health: [],
    result_summary: cycle?.result ? mapResult(cycle.result, { cycleId: cycle.cycleId }) : null,
    traceability: null,
    artifacts: [],
    installed_plugins: [],
    updated_at: updatedAt,
    revisions: revisions(revision),
  };
}

/* === Private Functions === */

function mapRunStatus(status?: CycleStatus): RunStatus {
  switch (status) {
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
    case CycleStatus.STOPPED:
      return "stopped";
    default:
      return "idle";
  }
}

function isTerminalCycle(status: CycleStatus): boolean {
  return [CycleStatus.COMPLETED, CycleStatus.FAILED, CycleStatus.BLOCKED, CycleStatus.STOPPED].includes(status);
}

function mapResult(
  result: NonNullable<RuntimeShellSnapshot["cycleSummaries"][number]["result"]>,
  scope: Wire<EvidenceScope>,
): ExecutionResultSummary {
  return {
    outcome:
      result.status === ResultStatus.PASSED
        ? "completed"
        : result.status === ResultStatus.FAILED
          ? "failed"
          : result.status === ResultStatus.BLOCKED
            ? "unknown"
            : "unknown",
    headline: result.status === ResultStatus.PASSED ? "passed" : "failed",
    warnings: [],
    evidence: [
      {
        passed: result.passed,
        failed: result.failed,
        blocked: result.blocked,
        skipped: result.skipped,
      },
    ],
    updated_at: nowIso(),
    scope,
  };
}

function revisions(revision: number): HubLiveState["revisions"] {
  return {
    run: revision,
    prompts: revision,
    cycles: revision,
    document: revision,
    records: revision,
    artifacts: revision,
    executionFiles: revision,
    notes: revision,
    health: revision,
    result_summary: revision,
    traceability: revision,
    contracts: revision,
    telemetry: { __all__: revision },
  };
}

/* === Entry Point === */
