/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The active cycle: shell state the Library adds cases to.
 */

import type { CycleRow } from "@/features/cycles-home/cycles-model";

/** The characters of a cycle identifier the shell shows beside its name. */
const SHORT_ID_LENGTH = 8;

/** An open cycle as the shell and the picker show it. */
export type OpenCycle = CycleRow & {
  createdLabel: string;
  shortId: string;
};

/** The cycles that still admit cases, newest first as the server ordered them. */
export function openCycles(rows: readonly CycleRow[]): OpenCycle[] {
  return rows
    .filter((row) => row.phase === "Active")
    .map((row) => ({ ...row, createdLabel: createdLabel(row.createdAtIso), shortId: row.cycleId.slice(0, SHORT_ID_LENGTH) }));
}

/** The stored active cycle while it is still open; a sealed or missing one is no active cycle at all. */
export function resolveActiveCycle(storedId: string | null, cycles: readonly OpenCycle[]): OpenCycle | null {
  return storedId ? (cycles.find((cycle) => cycle.cycleId === storedId) ?? null) : null;
}

/** The picker's search: the cycle's name or any part of its identifier. */
export function filterOpenCycles(cycles: readonly OpenCycle[], text: string): OpenCycle[] {
  const needle = text.trim().toLowerCase();
  if (!needle) return [...cycles];
  return cycles.filter((cycle) => cycle.label.toLowerCase().includes(needle) || cycle.cycleId.toLowerCase().includes(needle));
}

/** When the cycle was created, in the operator's locale, or a dash when the server did not say. */
export function createdLabel(createdAtIso: string | null): string {
  if (!createdAtIso) return "-";
  return new Date(createdAtIso).toLocaleString(undefined, { dateStyle: "short", timeStyle: "short" });
}

/** "N Cases", the picker's member count. */
export function caseCountLabel(count: number): string {
  return count === 1 ? "1 Case" : `${count} Cases`;
}
