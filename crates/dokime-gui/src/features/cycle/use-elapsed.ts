/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The durations the Steps card and the execution header print, and the live
 * clock that ticks a running one.
 *
 * A pending step has no duration; a running one counts up from its own start
 * on a monotonic client clock, without polling the runtime; an ended one
 * prints what the runtime measured. A frozen report never ticks: it prints
 * measured durations only and leaves running and pending ones empty.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useEffect, useState } from "react";

// Local Imports
import { formatDuration } from "@/lib/format";
import { parseTimestampMs } from "@/lib/time-format";

/* === Re-Exports === */

/* === Constants === */

const TICK_MS = 1_000;

/* === Types === */

/** What a duration cell is decided from. */
export type DurationSubject = {
  status: string;
  started_at?: string | null;
  duration_s?: number | null;
};

/* === Classes === */

/* === Public Functions === */

/**
 * The text of a duration cell: empty while pending, the ticking elapsed time
 * while running live (empty in a file), the measured duration once ended.
 */
export function durationText(subject: DurationSubject, live: boolean, elapsedS: number | null): string {
  if (subject.status === "pending" || subject.status === "planned" || subject.status === "created") return "";
  if (subject.status === "running") return live && elapsedS !== null ? (formatDuration(Math.floor(elapsedS)) ?? "") : "";
  return formatDuration(subject.duration_s) ?? "";
}

/**
 * Seconds since `startedAt`, ticking once a second while `running`.
 *
 * The wall clock is read once, when the clock starts, to place the start; the
 * count then advances on `performance.now()`, which never jumps with a clock
 * change. `null` while there is nothing to count.
 */
export function useElapsedSeconds(startedAt: string | null | undefined, running: boolean): number | null {
  const [elapsed, setElapsed] = useState<number | null>(null);
  useEffect(() => {
    const started = parseTimestampMs(startedAt);
    if (!running || started === null) {
      setElapsed(null);
      return;
    }
    const base = Math.max(0, Date.now() - started);
    const origin = performance.now();
    const tick = () => setElapsed((base + performance.now() - origin) / 1_000);
    tick();
    const handle = window.setInterval(tick, TICK_MS);
    return () => window.clearInterval(handle);
  }, [running, startedAt]);
  return elapsed;
}

/* === Private Functions === */

/* === Entry Point === */
