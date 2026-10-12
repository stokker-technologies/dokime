/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { CycleHistorySummary } from "@/lib/types";
import type { ExecutionBrowseEntry } from "@/lib/types/view-contracts";

/* === Types === */

export type ExploreSourceKind = "local" | "remote" | "storage";

/** Storage is a merged source; adapters own local and remote transport. */
export type ExploreSource = {
  id: string;
  label: string;
  kind: ExploreSourceKind;
};

/** Navigation distinguishes cycle groups from standalone execution datasets. */
export type ExploreTarget = { kind: "cycle"; id: string } | { kind: "execution"; id: string };

export type ExploreEntry = Omit<CycleHistorySummary, "test_cycle_id"> & {
  target: ExploreTarget;
  source: ExploreSource;
  /** The executions that ran. Every count on this entry is derived from these. */
  executions: ExecutionBrowseEntry[];
  /** The planned cases this cycle never started, kept for the caption and for
   *  resolving a direct link to one of them. */
  unrun: ExecutionBrowseEntry[];
  owner?: string;
};

export function exploreEntryKey(target: ExploreTarget): string {
  return `${target.kind}:${target.id}`;
}

/**
 * Whether history is holding a placeholder rather than a record of work.
 *
 * A planned case carries one attempt the runtime reports `SKIPPED` with no
 * start instant: there is no acquisition behind it, no steps and no records, so
 * a history row for it states a case that was never run as if it had been
 * inspectable. History counts and lists what ran; the cases that did not are
 * one caption.
 */
export function isUnrunExecution(entry: ExecutionBrowseEntry): boolean {
  return entry.metadata.started_at === null && entry.metadata.status === "skipped";
}

/** The one line history gives the planned cases it is not listing. */
export function unrunCaption(count: number): string {
  return `${count} planned ${count === 1 ? "case" : "cases"} not run`;
}
