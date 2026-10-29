/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { MetricState } from "@/lib/types";

export type OverviewMetric = {
  label: string;
  value: string;
  rawValue?: unknown;
  detail?: string;
  presentation?: "value" | "progress" | "badge";
  state?: MetricState | null;
};

export type OverviewMetricGroup = {
  groupId: string;
  title: string;
  metrics: OverviewMetric[];
};

export const METRIC_FLOW_STORAGE_PREFIX = "dokime-test-hub.metric-flow.v1";

export const METRIC_GROUP_CARD_WIDTH_REM = 9.5;

export const METRIC_GROUP_GAP_REM = 0.5;

export const METRIC_GROUP_MAX_COLUMNS = 4;

export function metricFlowStorageKey({
  blockId,
  documentId,
  fallbackId,
}: {
  blockId?: string | null;
  documentId?: string | null;
  fallbackId?: string | null;
}): string {
  return [
    METRIC_FLOW_STORAGE_PREFIX,
    sanitizeStorageKeySegment(documentId || fallbackId || "overview"),
    sanitizeStorageKeySegment(blockId || "default"),
  ].join(".");
}

export function readMetricFlowOrder(storageKey: string): string[] {
  if (typeof window === "undefined") {
    return [];
  }
  try {
    const parsed = JSON.parse(window.localStorage.getItem(storageKey) ?? "[]");
    return Array.isArray(parsed) ? parsed.filter((value): value is string => typeof value === "string") : [];
  } catch {
    return [];
  }
}

export function writeMetricFlowOrder(storageKey: string, groupIds: string[]) {
  if (typeof window === "undefined") {
    return;
  }
  try {
    window.localStorage.setItem(storageKey, JSON.stringify(groupIds));
  } catch {
    // Local report files and locked-down browser contexts can reject storage.
    // The drag interaction should remain usable for the current view.
  }
}

export function removeMetricFlowOrder(storageKey: string) {
  if (typeof window === "undefined") {
    return;
  }
  try {
    window.localStorage.removeItem(storageKey);
  } catch {
    // Ignore storage cleanup failures for read-only browser contexts.
  }
}

export function orderMetricGroups(groups: OverviewMetricGroup[], savedOrder: string[]): OverviewMetricGroup[] {
  if (!savedOrder.length) {
    return groups;
  }

  const byId = new Map(groups.map((group) => [group.groupId, group]));
  const ordered = savedOrder.map((groupId) => byId.get(groupId)).filter((group): group is OverviewMetricGroup => Boolean(group));
  const knownIds = new Set(ordered.map((group) => group.groupId));
  return [...ordered, ...groups.filter((group) => !knownIds.has(group.groupId))];
}

export function moveMetricGroup(groupIds: string[], sourceGroupId: string, targetGroupId: string, insertAfter: boolean): string[] {
  const withoutSource = groupIds.filter((groupId) => groupId !== sourceGroupId);
  const targetIndex = withoutSource.indexOf(targetGroupId);
  if (targetIndex === -1) {
    return groupIds;
  }
  const insertIndex = targetIndex + (insertAfter ? 1 : 0);
  return [...withoutSource.slice(0, insertIndex), sourceGroupId, ...withoutSource.slice(insertIndex)];
}

export function metricGroupWidthRem(metricCount: number): number {
  const columns = Math.min(Math.max(metricCount, 1), METRIC_GROUP_MAX_COLUMNS);
  return columns * METRIC_GROUP_CARD_WIDTH_REM + Math.max(0, columns - 1) * METRIC_GROUP_GAP_REM;
}

export function sanitizeStorageKeySegment(value: string): string {
  return (
    value
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9_.-]+/g, "_") || "overview"
  );
}
