/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Grouping, ordering and labelling for the Cycles home. Pure so the ordering
   rules stay testable away from the query layer. */

/* === Local Imports === */

import { CyclePhase, CycleStatus } from "@host/dokime/v1/client_types_pb.js";
import { type CycleState, cycleState } from "@/features/cycle/cycle-workspace-model";
import { mapCycleStatus } from "@/lib/api/native-planning";
import type { CycleSummary } from "@/lib/api/native-planning";
import type { PromptRequest } from "@/lib/types";

/* === Constants === */

const STATE_TONES: Record<CycleState["variant"], CycleRow["statusTone"]> = {
  default: "good",
  destructive: "bad",
  outline: "neutral",
  secondary: "neutral",
  warning: "warn",
};

/* === Types === */

export type CycleGroupId = "attention" | "active" | "recent" | "archived";

export type CycleRow = {
  attentionCount: number;
  caseCount: number;
  /** The cases this cycle has actually finished, out of `caseCount`. */
  finishedCount: number;
  /** When the cycle was created, as the row's fallback subtitle. */
  createdAtIso: string | null;
  cycleId: string;
  executionCount: number;
  label: string;
  phase: "Active" | "Sealing" | "SealFailed" | "Sealed" | "Unknown";
  status: string;
  statusLabel: string;
  statusTone: "neutral" | "good" | "warn" | "bad";
  updatedAtIso: string | null;
  updatedAtMs: number | null;
};

export type CycleGroup = {
  id: CycleGroupId;
  title: string;
  /** The smallest useful statement when the group has no rows. */
  emptyText: string;
  rows: CycleRow[];
};

/* === Public Functions === */

export function cycleRows(cycles: CycleSummary[], attentionByCycleId: ReadonlyMap<string, number>): CycleRow[] {
  return cycles.map((cycle) => {
    const attentionCount = attentionByCycleId.get(cycle.cycleId) ?? 0;
    const state = cycleState(statePhase(cycle), mapCycleStatus(cycle.status), attentionCount > 0);
    return {
      attentionCount,
      caseCount: Number(cycle.caseCount),
      createdAtIso: isoOrNull(createdMs(cycle)),
      cycleId: cycle.cycleId,
      finishedCount: finishedCases(cycle),
      executionCount: Number(cycle.executionCount),
      label: cycle.metadata?.label || cycle.cycleId,
      phase:
        cycle.phase === CyclePhase.ACTIVE
          ? "Active"
          : cycle.phase === CyclePhase.SEALING
            ? "Sealing"
            : cycle.phase === CyclePhase.SEAL_FAILED
              ? "SealFailed"
              : cycle.phase === CyclePhase.SEALED
                ? "Sealed"
                : "Unknown",
      status:
        cycle.status === CycleStatus.BLOCKED
          ? "Blocked"
          : mapCycleStatus(cycle.status) === "planned"
            ? "Ready"
            : mapCycleStatus(cycle.status).replace(/^./, (letter) => letter.toUpperCase()),
      statusLabel: state.label,
      statusTone: STATE_TONES[state.variant],
      updatedAtIso: isoOrNull(timestampMs(cycle)),
      updatedAtMs: timestampMs(cycle),
    };
  });
}

/** How many open prompts wait in each cycle, keyed by cycle identifier. */
export function promptCountsByCycle(prompts: readonly PromptRequest[]): Map<string, number> {
  const counts = new Map<string, number>();
  for (const prompt of prompts) {
    const cycleId = prompt.scope?.test_cycle_id;
    if (cycleId) counts.set(cycleId, (counts.get(cycleId) ?? 0) + 1);
  }
  return counts;
}

/**
 * The three fixed groups, over the rows the server selected.
 *
 * Selection, the archive view and newest-first order are the server's:
 * this read asks for `filter.text`, `filter.archive` and `order UPDATED_AT
 * descending`, so the page cannot disagree with the pages after it and the
 * grouping does no filtering or sorting of its own. `filtered` is the one
 * thing the grouping cannot see from the rows: an empty Recent means "nothing
 * has been loaded" or "nothing matched", and those are different sentences.
 */
export function groupCycleRows(rows: CycleRow[], filtered = false): CycleGroup[] {
  const attention = rows.filter((row) => row.attentionCount > 0 || row.status === "Blocked" || row.phase === "SealFailed");
  const attentionIds = new Set(attention.map((row) => row.cycleId));
  const active = rows.filter((row) => !attentionIds.has(row.cycleId) && row.phase !== "Sealed");
  const activeIds = new Set(active.map((row) => row.cycleId));
  const recent = rows.filter((row) => !attentionIds.has(row.cycleId) && !activeIds.has(row.cycleId));

  return [
    {
      emptyText: "Nothing Is Waiting on You",
      id: "attention",
      rows: attention,
      title: "Needs Attention",
    },
    { emptyText: "No Active Cycles", id: "active", rows: active, title: "Active" },
    {
      emptyText: filtered ? "No Cycles Match This Filter" : "No Cycles Loaded Yet",
      id: "recent",
      rows: recent,
      title: "Recent",
    },
  ];
}

/** The Archived view: one group over the archived rows the server selected. */
export function archivedCycleGroups(rows: CycleRow[], filtered = false): CycleGroup[] {
  return [
    {
      emptyText: filtered ? "No Archived Cycles Match This Filter" : "No Archived Cycles",
      id: "archived",
      rows,
      title: "Archived",
    },
  ];
}

/** Reorders the rows to a frozen id sequence, appending ids that arrived since. */
export function applyFrozenOrder(rows: CycleRow[], frozenIds: readonly string[]): CycleRow[] {
  if (!frozenIds.length) {
    return rows;
  }
  const byId = new Map(rows.map((row) => [row.cycleId, row]));
  const ordered = frozenIds.flatMap((id) => {
    const row = byId.get(id);
    if (!row) {
      return [];
    }
    byId.delete(id);
    return [row];
  });
  return [...ordered, ...byId.values()];
}

/* === Private Functions === */

function statePhase(cycle: CycleSummary): Parameters<typeof cycleState>[0] {
  switch (cycle.phase) {
    case CyclePhase.SEALING:
      return "sealing";
    case CyclePhase.SEAL_FAILED:
      return "seal_failed";
    case CyclePhase.SEALED:
      return "sealed";
    default:
      return "active";
  }
}

/**
 * The cases this cycle has finished, as its own result summary states them.
 *
 * Execution Results Arrive Independently Of The Container Seal. Planned
 * Members Remain Unfinished Until The Cycle Seal Settles Them.
 */
function finishedCases(cycle: CycleSummary): number {
  const result = cycle.result;
  if (!result) return 0;
  const ended = result.passed + result.failed + result.blocked;
  return statePhase(cycle) === "active" ? ended : ended + result.skipped;
}

function isoOrNull(milliseconds: number | null): string | null {
  return milliseconds === null ? null : new Date(milliseconds).toISOString();
}

function createdMs(cycle: CycleSummary): number | null {
  const timestamp = cycle.createdAt;
  return timestamp ? Number(timestamp.seconds) * 1000 + Math.floor(timestamp.nanos / 1e6) : null;
}

function timestampMs(cycle: CycleSummary): number | null {
  const timestamp = cycle.updatedAt ?? cycle.startedAt ?? cycle.createdAt;
  return timestamp ? Number(timestamp.seconds) * 1000 + Math.floor(timestamp.nanos / 1e6) : null;
}
