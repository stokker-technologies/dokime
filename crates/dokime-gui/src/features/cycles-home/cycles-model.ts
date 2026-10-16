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

import type { Wire } from "@/lib/rpc/canonical";
import type { Row2 as GeneratedCycleSummary } from "../../../.generated/dokime/v1/ExecutionMessages/GetCycle/messages_pb";

type CycleSummary = Wire<GeneratedCycleSummary>;

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
  status: NonNullable<NonNullable<CycleSummary["status"]>["HarmosVariant"]["case"]>;
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

/* === Constants === */

const ACTIVE_STATUSES = ["Running", "Paused", "Stopping"];

/**
 * The lifecycles after which a skipped attempt is a real skip.
 *
 * A cycle nobody has run yet already holds one attempt per planned case, each
 * reported `SKIPPED` because the runtime may still select it. Counting those as
 * finished made every fresh cycle read as complete on this page, so they only
 * count once the cycle is sealed.
 */
const SEALED_STATUSES = ["Completed", "Failed", "Stopped"];

const STATUS_LABELS: Record<CycleRow["status"], string> = {
  Unspecified: "Unknown",
  Ready: "Ready",
  Running: "Running",
  Paused: "Paused",
  Stopping: "Stopping",
  Completed: "Completed",
  Failed: "Failed",
  Blocked: "Blocked",
  Stopped: "Stopped",
};

/* === Public Functions === */

export function cycleRows(
  cycles: CycleSummary[],
  attentionByCycleId: ReadonlyMap<string, number>,
): CycleRow[] {
  return cycles.map((cycle) => ({
    attentionCount: attentionByCycleId.get(cycle.cycleId) ?? 0,
    caseCount: Number(cycle.caseCount),
    createdAtIso: isoOrNull(createdMs(cycle)),
    cycleId: cycle.cycleId,
    finishedCount: finishedCases(cycle),
    executionCount: Number(cycle.executionCount),
    label: cycle.metadata?.label || cycle.cycleId,
    status: cycle.status?.HarmosVariant.case ?? "Unspecified",
    statusLabel: STATUS_LABELS[cycle.status?.HarmosVariant.case ?? "Unspecified"] ?? "Unknown",
    statusTone: statusTone(cycle.status),
    updatedAtIso: isoOrNull(timestampMs(cycle)),
    updatedAtMs: timestampMs(cycle),
  }));
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
  const attention = rows.filter((row) => row.attentionCount > 0 || row.status === "Blocked");
  const attentionIds = new Set(attention.map((row) => row.cycleId));
  const active = rows.filter(
    (row) => !attentionIds.has(row.cycleId) && ACTIVE_STATUSES.includes(row.status),
  );
  const activeIds = new Set(active.map((row) => row.cycleId));
  const recent = rows.filter(
    (row) => !attentionIds.has(row.cycleId) && !activeIds.has(row.cycleId),
  );

  return [
    {
      emptyText: "Nothing Is Waiting On You",
      id: "attention",
      rows: attention,
      title: "Needs Attention",
    },
    { emptyText: "No Cycle Is Running", id: "active", rows: active, title: "Active" },
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

function statusTone(status: CycleSummary["status"]): CycleRow["statusTone"] {
  switch (status?.HarmosVariant.case) {
    case "Running":
    case "Completed":
      return "good";
    case "Paused":
    case "Stopping":
    case "Blocked":
      return "warn";
    case "Failed":
      return "bad";
    default:
      return "neutral";
  }
}

/**
 * The cases this cycle has finished, as its own result summary states them.
 *
 * The summary carries no result until the cycle is sealed, and an unsealed
 * cycle's `skipped` cases are planned work rather than outcomes, so both cases
 * agree: nothing is finished until the runtime says something finished.
 */
function finishedCases(cycle: CycleSummary): number {
  const result = cycle.result;
  if (!result) return 0;
  const ended = result.passed + result.failed + result.blocked;
  return SEALED_STATUSES.includes(cycle.status?.HarmosVariant.case ?? "")
    ? ended + result.skipped
    : ended;
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
