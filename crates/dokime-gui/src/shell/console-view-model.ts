// Dokime — SPDX-License-Identifier: Apache-2.0
import { recordLevelLabel } from "../lib/record-display";
import type { HubRecord } from "../lib/types";

const consoleLevels = ["trace", "debug", "info", "warning", "error"] as const;
export type ConsoleLevel = (typeof consoleLevels)[number];
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
