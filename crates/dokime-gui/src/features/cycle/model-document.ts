/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

import type {
  BoxSummarySectionSpec,
  BoxSummaryTopicGroup,
  HistogramSectionSpec,
  HistogramTopicGroup,
} from "@/components/document/DocumentSurface";
import { isCycleDocumentSurfaceCard } from "@/components/document/document-surface-policy";
import { documentPlotGroupDescription } from "@/lib/document-plots";
import type { DocumentBoxSummaryPayload } from "@/lib/report-types";
import type {
  CycleHistorySummary,
  DocumentCardLayout,
  DocumentViewBoxSummary,
  DocumentViewCard,
  DocumentViewHistogram,
  ExecutionQueueItem,
  HubLiveState,
  TelemetryStream,
  TestCyclePlan,
  TestExecution,
} from "@/lib/types";
import { isNumber, numericValue } from "@/lib/utils";
import { histogramAxisTitle } from "@/plots/histogram-axes";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Statistics / document-builder helpers ===
   Pure stats, document section/group builders, plot-context derivation,
   and cycle-history merge/sort. */

export function isCycleLiveDocumentCard(card: DocumentViewCard): boolean {
  return isCycleDocumentSurfaceCard(card);
}

export function mergeParameterContexts(
  ...contexts: Array<Record<string, unknown> | null | undefined>
): Record<string, unknown> | null {
  const merged: Record<string, unknown> = {};
  for (const context of contexts) {
    if (!context) {
      continue;
    }
    Object.assign(merged, context);
  }
  return Object.keys(merged).length ? merged : null;
}

export function buildDocumentHistogramSections({
  cards,
  snapshot,
  windowS,
}: {
  cards: DocumentViewCard[];
  snapshot: HubLiveState | null;
  windowS?: number | null;
}): HistogramSectionSpec[] {
  if (!snapshot) {
    return [];
  }
  return cards.map((card) => ({
    key: card.card_id,
    title: card.title,
    groups: buildDocumentHistogramGroups({
      histograms: card.histograms,
      snapshot,
      windowS,
    }),
  }));
}

export function buildDocumentBoxSummarySections({
  cards,
  snapshot,
  windowS,
}: {
  cards: DocumentViewCard[];
  snapshot: HubLiveState | null;
  windowS?: number | null;
}): BoxSummarySectionSpec[] {
  if (!snapshot) {
    return [];
  }
  return cards.map((card) => ({
    key: card.card_id,
    title: card.title,
    groups: buildDocumentBoxSummaryGroups({
      boxSummaries: card.box_summaries,
      snapshot,
      windowS,
    }),
  }));
}

export function documentLayout({
  base = 12,
  md = 12,
  order = 0,
  rowSpan = 1,
  xl = md,
}: {
  base?: number;
  md?: number;
  order?: number;
  rowSpan?: number;
  xl?: number;
}): DocumentCardLayout {
  return {
    col_span: { base, sm: null, md, lg: null, xl },
    row_span: rowSpan,
    order,
    density: "normal",
    rail: "main",
  };
}

export function mergeCycleHistorySummaries(
  summaries: CycleHistorySummary[],
  plan: TestCyclePlan | null,
  executions: TestExecution[],
): CycleHistorySummary[] {
  if (!plan) {
    return summaries;
  }

  const merged = new Map(summaries.map((summary) => [summary.test_cycle_id, summary] as const));
  merged.set(plan.test_cycle.test_cycle_id, liveCycleHistorySummary(plan, executions));

  return [...merged.values()].sort(compareCycleHistorySummary);
}

export function liveCycleHistorySummary(
  plan: TestCyclePlan,
  executions: TestExecution[],
): CycleHistorySummary {
  const { queue, test_cycle: cycle } = plan;
  const countByStatus = (status: ExecutionQueueItem["status"]) =>
    queue.filter((item) => item.status === status).length;
  const updatedAt = cycle.finished_at ?? cycle.started_at ?? cycle.generated_at;

  return {
    test_cycle_id: cycle.test_cycle_id,
    session_id: null,
    title: cycle.title,
    suite_id: cycle.suite_id ?? null,
    suite_name: cycle.suite_name ?? null,
    lifecycle: cycle.status,
    result_status:
      cycle.status === "completed"
        ? "passed"
        : cycle.status === "failed"
          ? "failed"
          : cycle.status === "stopped" || cycle.status === "interrupted"
            ? "cancelled"
            : null,
    case_count: plan.cases.length,
    execution_count: executions.length,
    queued_count: countByStatus("queued"),
    running_count:
      countByStatus("running") + countByStatus("waiting_for_prompt") + countByStatus("finalizing"),
    passed_count: countByStatus("passed"),
    failed_count: countByStatus("failed"),
    blocked_count: countByStatus("blocked"),
    created_at: cycle.generated_at,
    started_at: cycle.started_at ?? null,
    finished_at: cycle.finished_at ?? null,
    updated_at: updatedAt,
    duration_s: cycle.duration_s ?? null,
  };
}

export function compareCycleHistorySummary(
  left: CycleHistorySummary,
  right: CycleHistorySummary,
): number {
  return (
    summarySortAnchor(right).localeCompare(summarySortAnchor(left)) ||
    right.updated_at.localeCompare(left.updated_at) ||
    right.created_at.localeCompare(left.created_at)
  );
}

export function summarySortAnchor(summary: CycleHistorySummary): string {
  return summary.started_at ?? summary.updated_at;
}

export function plotContextTelemetrySnapshot(
  snapshot: HubLiveState | null,
  documentPlots: DocumentViewCard["plots"],
): Record<string, TelemetryStream> | null {
  if (!snapshot) {
    return null;
  }
  const entries = plotContextStreams(documentPlots)
    .map((stream) => [stream, snapshot.telemetry[stream]] as const)
    .filter((entry): entry is readonly [string, TelemetryStream] => Boolean(entry[1]));
  return entries.length ? Object.fromEntries(entries) : null;
}

export function plotContextStreams(documentPlots: DocumentViewCard["plots"]): string[] {
  const streams = new Set<string>();
  documentPlots.forEach((plot) => {
    const stream = plot.stream;
    if (!stream) {
      return;
    }
    streams.add(stream);
    if (stream === "host.wifi.channel_occupancy") {
      streams.add("host.wifi");
    }
  });
  return [...streams].sort();
}

export function uniqueArtifacts<T extends { artifact_id: string }>(artifacts: T[]): T[] {
  const seen = new Set<string>();
  return artifacts.filter((artifact) => {
    if (seen.has(artifact.artifact_id)) {
      return false;
    }
    seen.add(artifact.artifact_id);
    return true;
  });
}

export function buildDocumentHistogramGroups({
  histograms,
  snapshot,
  windowS,
}: {
  histograms: DocumentViewHistogram[];
  snapshot: HubLiveState;
  windowS?: number | null;
}): HistogramTopicGroup[] {
  const groups = new Map<string, HistogramTopicGroup>();
  const groupOrder: string[] = [];

  histograms.forEach((histogram) => {
    const stream = snapshot.telemetry[histogram.stream];
    const samples = stream?.samples ?? [];
    const windowedSamples = windowTelemetrySamples(
      samples,
      windowS === undefined ? histogram.live_window_s : windowS,
    );
    const values = windowedSamples
      .map((sample) => histogramDisplayNumber(sample[histogram.field], histogram))
      .filter(isNumber);
    const title = histogram.group || "Other";
    let group = groups.get(title);
    if (!group) {
      group = {
        title,
        description: histogram.description ?? documentPlotGroupDescription(title),
        histograms: [],
      };
      groups.set(title, group);
      groupOrder.push(title);
    } else if (!group.description && histogram.description) {
      group.description = histogram.description;
    }

    group.histograms.push({
      key: histogram.histogram_id,
      title: histogram.title,
      values,
      unit: histogramDisplayUnit(histogram),
      xTitle: histogramAxisTitle({
        label:
          stream?.spec?.columns.find((column) => column.name === histogram.field)?.label ??
          histogram.field,
        unit: histogram.unit,
      }),
      bins: Math.max(2, histogram.bins || 12),
    });
  });

  return groupOrder
    .map((title) => groups.get(title))
    .filter((group): group is HistogramTopicGroup => Boolean(group));
}

export function buildDocumentBoxSummaryGroups({
  boxSummaries,
  snapshot,
  windowS,
}: {
  boxSummaries: DocumentViewBoxSummary[];
  snapshot: HubLiveState;
  windowS?: number | null;
}): BoxSummaryTopicGroup[] {
  const groups = new Map<string, BoxSummaryTopicGroup>();
  const groupOrder: string[] = [];

  boxSummaries.forEach((boxSummary) => {
    const stream = snapshot.telemetry[boxSummary.stream];
    const samples = stream?.samples ?? [];
    const windowedSamples = windowTelemetrySamples(
      samples,
      windowS === undefined ? boxSummary.live_window_s : windowS,
    );
    const values = windowedSamples
      .map((sample) => boxSummaryDisplayNumber(sample[boxSummary.field], boxSummary))
      .filter(isNumber);
    const title = boxSummary.group || "Other";
    let group = groups.get(title);
    if (!group) {
      group = {
        title,
        description: boxSummary.description ?? documentPlotGroupDescription(title),
        summaries: [],
      };
      groups.set(title, group);
      groupOrder.push(title);
    } else if (!group.description && boxSummary.description) {
      group.description = boxSummary.description;
    }

    group.summaries.push({
      box_summary_id: boxSummary.box_summary_id,
      title: boxSummary.title,
      stream: boxSummary.stream,
      field: boxSummary.field,
      group: title,
      unit: boxSummary.unit,
      display_unit: boxSummaryDisplayUnit(boxSummary),
      sample_count: values.length,
      ...boxSummaryStats(values),
    });
  });

  return groupOrder
    .map((title) => groups.get(title))
    .filter((group): group is BoxSummaryTopicGroup => Boolean(group));
}

export function windowTelemetrySamples(
  samples: Array<Record<string, unknown>>,
  windowS?: number | null,
): Array<Record<string, unknown>> {
  if (windowS === undefined || windowS === null || windowS <= 0 || samples.length < 2) {
    return samples;
  }
  const latest = telemetrySampleX(samples[samples.length - 1], samples.length - 1);
  const minX = latest - windowS;
  const windowedSamples = samples.filter(
    (sample, index) => telemetrySampleX(sample, index) >= minX,
  );
  return windowedSamples.length > 0 ? windowedSamples : samples.slice(-500);
}

export function telemetrySampleX(
  sample: Record<string, unknown> | undefined,
  index: number,
): number {
  const value = sample?.timestamp;
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }
  if (typeof value === "string") {
    const parsedDate = Date.parse(value);
    if (Number.isFinite(parsedDate)) {
      return parsedDate / 1000;
    }
    const parsedNumber = Number(value);
    if (Number.isFinite(parsedNumber)) {
      return parsedNumber;
    }
  }
  return index;
}

export function histogramDisplayNumber(
  value: unknown,
  _histogram: DocumentViewHistogram,
): number | undefined {
  const number = numberValue(value);
  if (number === undefined) {
    return undefined;
  }
  return number;
}

export function histogramDisplayUnit(histogram: DocumentViewHistogram): string {
  return histogram.unit ?? "value";
}

export function boxSummaryDisplayNumber(
  value: unknown,
  _boxSummary: DocumentViewBoxSummary,
): number | undefined {
  const number = numberValue(value);
  if (number === undefined) {
    return undefined;
  }
  return number;
}

export function boxSummaryDisplayUnit(boxSummary: DocumentViewBoxSummary): string {
  return boxSummary.unit ?? "value";
}

export function boxSummaryStats(
  values: number[],
): Pick<DocumentBoxSummaryPayload, "minimum" | "q1" | "median" | "q3" | "maximum" | "outliers"> {
  if (!values.length) {
    return {
      minimum: null,
      q1: null,
      median: null,
      q3: null,
      maximum: null,
      outliers: [],
    };
  }
  const ordered = [...values].sort((left, right) => left - right);
  const q1 = percentile(ordered, 0.25);
  const median = percentile(ordered, 0.5);
  const q3 = percentile(ordered, 0.75);
  const iqr = q3 - q1;
  const lowerFence = q1 - 1.5 * iqr;
  const upperFence = q3 + 1.5 * iqr;
  const inliers = ordered.filter((value) => value >= lowerFence && value <= upperFence);
  return {
    minimum: inliers[0] ?? ordered[0] ?? null,
    q1,
    median,
    q3,
    maximum: inliers[inliers.length - 1] ?? ordered[ordered.length - 1] ?? null,
    outliers: ordered.filter((value) => value < lowerFence || value > upperFence),
  };
}

export function percentile(values: number[], fraction: number): number {
  if (values.length === 1) {
    return values[0] ?? 0;
  }
  const position = (values.length - 1) * fraction;
  const lower = Math.floor(position);
  const upper = Math.ceil(position);
  if (lower === upper) {
    return values[lower] ?? 0;
  }
  const weight = position - lower;
  return (values[lower] ?? 0) * (1 - weight) + (values[upper] ?? 0) * weight;
}

export function numberValue(value: unknown): number | undefined {
  value = numericValue(value) ?? value;
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }
  if (typeof value === "string" && value.trim()) {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) {
      return parsed;
    }
  }
  return undefined;
}

/* === Private Functions === */

/* === Entry Point === */
