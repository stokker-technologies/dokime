/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Cycles Listed Under The Cycles Destination On Every Route.
 *
 * The rows are the Cycles home's own read, so the sidebar and the home never
 * disagree about which cycles are still running. Pure so the selection and
 * order stay testable away from the query layer.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

import type { CycleRow } from "@/features/cycles-home/cycles-model";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type SidebarCycleSource = Pick<CycleRow, "createdAtIso" | "cycleId" | "label" | "phase">;

export type SidebarCycle = {
  cycleId: string;
  /** The cycle the route names: it expands into its cases. */
  current: boolean;
  label: string;
};

/* === Classes === */

/* === Public Functions === */

/**
 * Every cycle that has not sealed, newest first, plus the cycle the route
 * names.
 *
 * A sealed cycle leaves the list unless the operator is looking at it, and a
 * cycle created a moment ago is shown before the list catches up with it. The
 * order is creation time, not the read's update time, so a running cycle does
 * not jump to the top of the sidebar on every result.
 */
export function sidebarCycles(rows: readonly SidebarCycleSource[], currentCycleId: string | null): SidebarCycle[] {
  const listed = rows.filter((row) => row.phase !== "Sealed" || row.cycleId === currentCycleId);
  const cycles = [...listed]
    .sort(byNewest)
    .map((row) => ({ current: row.cycleId === currentCycleId, cycleId: row.cycleId, label: row.label }));
  if (currentCycleId && !cycles.some((cycle) => cycle.current)) {
    cycles.unshift({ current: true, cycleId: currentCycleId, label: currentCycleId });
  }
  return cycles;
}

/* === Private Functions === */

function byNewest(left: SidebarCycleSource, right: SidebarCycleSource): number {
  const created = (right.createdAtIso ?? "").localeCompare(left.createdAtIso ?? "");
  return created !== 0 ? created : left.cycleId.localeCompare(right.cycleId);
}

/* === Entry Point === */
