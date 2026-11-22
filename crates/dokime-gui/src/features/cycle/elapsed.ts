// Dokime — SPDX-License-Identifier: Apache-2.0
import { formatDuration } from "../../lib/format";
import { parseTimestampMs } from "../../lib/time-format";

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
