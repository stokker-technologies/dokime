// Dokime — SPDX-License-Identifier: Apache-2.0
import type { EvidenceScope } from "@generated/host/model";
import { recordMessage } from "../lib/dashboard";
import { recordLevelLabel } from "../lib/record-display";
import type { Wire } from "../lib/rpc/canonical";
import type { HubRecord } from "../lib/types";

const consoleLevels = ["trace", "debug", "info", "warning", "error"] as const;
export type ConsoleLevel = (typeof consoleLevels)[number];
export type ConsoleScope = "instance" | "cycle" | "execution";
export const consoleLevelOptions = consoleLevels.map((value) => ({ value, label: recordLevelLabel(value) }));

export function visibleConsoleRecords(records: HubRecord[], level: ConsoleLevel, filter: string): HubRecord[] {
  const threshold = consoleLevels.indexOf(level);
  const needle = filter.trim().toLowerCase();
  return records.filter((record) => {
    if (consoleLevels.indexOf(record.level) < threshold) {
      return false;
    }
    return !needle || `${record.source} ${recordMessage(record)}`.toLowerCase().includes(needle);
  });
}

/**
 * The evidence the chosen scope reads, or null for the instance, which reads
 * the backend's trace. A scope the route cannot fill widens to the next one.
 */
export function consoleRecordScope(scope: ConsoleScope, cycleId: string | null, executionId: string | null): Wire<EvidenceScope> | null {
  if (scope === "execution" && executionId) {
    return cycleId ? { cycleId, executionId } : { executionId };
  }
  return scope !== "instance" && cycleId ? { cycleId } : null;
}

/** The records after the last one cleared; all of them once it left the tail. */
export function recordsAfter(records: HubRecord[], cleared: string | undefined): HubRecord[] {
  const index = cleared === undefined ? -1 : records.findIndex((record) => record.record_id === cleared);
  return records.slice(index + 1);
}

/** Warnings And Errors That Arrived After The Drawer Was Last Opened. */
export function consoleAttentionCount(records: HubRecord[], since: number): number {
  return records.filter((record) => {
    if (record.level !== "warning" && record.level !== "error") {
      return false;
    }
    const at = Date.parse(record.timestamp);
    return !Number.isFinite(at) || at > since;
  }).length;
}
