// Dokime — SPDX-License-Identifier: Apache-2.0
import { recordLevelLabel } from "../lib/record-display";
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
    return !needle || `${record.source} ${record.message ?? ""}`.toLowerCase().includes(needle);
  });
}

export function resolvedConsoleScope(scope: ConsoleScope, cycleId: string | null, executionId: string | null): ConsoleScope {
  if (scope === "execution" && !executionId) {
    return cycleId ? "cycle" : "instance";
  }
  if (scope === "cycle" && !cycleId) {
    return "instance";
  }
  return scope;
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
