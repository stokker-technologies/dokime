// Dokime — SPDX-License-Identifier: Apache-2.0
import { formatDuration } from "../../lib/format";
import { parseTimestampMs } from "../../lib/time-format";
import type { TestExecution } from "../../lib/types";
import { isActiveExecutionStatus } from "./cycle-workspace-model";

const TICK_MS = 1_000;

export type DurationSubject = {
  status: string;
  started_at?: string | null;
  duration_s?: number | null;
};

/**
 * The text of a duration cell: empty while pending, the ticking elapsed time
 * while running live (empty in a file), the measured duration once ended.
 */
export function durationText(subject: DurationSubject, live: boolean, elapsedS: number | null): string {
  if (subject.status === "pending" || subject.status === "planned" || subject.status === "created") {
    return "";
  }
  if (subject.status === "running") {
    return live && elapsedS !== null ? (formatDuration(Math.floor(elapsedS)) ?? "") : "";
  }
  return formatDuration(subject.duration_s) ?? "";
}

/**
 * Seconds since `startedAt`, ticking once a second while `running`.
 *
 * The wall clock is read once, when the clock starts, to place the start; the
 * count then advances on `performance.now()`, which never jumps with a clock
 * change. `null` while there is nothing to count.
 */
export function observeElapsedSeconds(
  startedAt: string | null | undefined,
  running: boolean,
  update: (seconds: number | null) => void,
): (() => void) | undefined {
  const started = parseTimestampMs(startedAt);
  if (!running || started === null) {
    update(null);
    return;
  }

  const base = Math.max(0, Date.now() - started);
  const origin = performance.now();
  const tick = () => update((base + performance.now() - origin) / 1_000);
  tick();
  const handle = window.setInterval(tick, TICK_MS);
  return () => window.clearInterval(handle);
}

export function executionProgressText(
  execution: TestExecution | null,
  steps: readonly { status: string }[],
  live: boolean,
  elapsedS: number | null,
): string {
  const settled = steps.filter((step) => step.status !== "pending" && step.status !== "running").length;
  const progress = steps.length ? `${settled} of ${steps.length} steps finished` : "";
  if (!execution) {
    return progress;
  }

  const started = parseTimestampMs(execution.started_at);
  const finished = parseTimestampMs(execution.finished_at);
  const durationS = started === null || finished === null ? null : Math.max(0, (finished - started) / 1_000);
  const status = isActiveExecutionStatus(execution.status) ? "running" : execution.status;
  const duration = durationText({ duration_s: durationS, status }, live, elapsedS);
  return [progress, duration].filter(Boolean).join(" · ");
}
