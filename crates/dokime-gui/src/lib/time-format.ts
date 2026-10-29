/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { formatDuration } from "./format";

/* === Constants === */

/* === Public Functions === */

/**
 * Parse a wire timestamp into epoch milliseconds. Missing and unparseable
 * values render as "no timestamp"; valid historical dates remain dates.
 */
export function parseTimestampMs(timestamp: string | null | undefined): number | null {
  if (!timestamp) {
    return null;
  }
  const parsed = Date.parse(timestamp);
  if (!Number.isFinite(parsed)) {
    return null;
  }
  return parsed;
}

export function elapsedLabel(startedAt: string | null | undefined, now: number, finishedAt?: string | null): string {
  const started = parseTimestampMs(startedAt);
  if (started === null) {
    return "-";
  }
  return formatDuration(Math.max(0, Math.floor((elapsedEndTime(finishedAt, now) - started) / 1000))) ?? "-";
}

/** Full local date-time label, or "-" for missing/invalid/epoch-zero timestamps. */
export function absoluteTimestampLabel(timestamp: string | null | undefined): string {
  const parsed = parseTimestampMs(timestamp);
  if (parsed === null) {
    return "-";
  }
  return new Date(parsed).toLocaleString();
}

/** Local time-of-day label (hh:mm), or "-" for missing/invalid/epoch-zero timestamps. */
export function timeOfDayLabel(timestamp: string | null | undefined): string {
  const parsed = parseTimestampMs(timestamp);
  if (parsed === null) {
    return "-";
  }
  return new Date(parsed).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

/**
 * The title the New Cycle form opens with: the moment it opened, written the
 * way the user's locale writes dates and times, so the list reads as a
 * timeline.
 */
export function defaultCycleLabel(now: Date = new Date()): string {
  const formatted = new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "numeric",
    minute: "2-digit",
  }).format(now);
  return `Test Cycle - ${formatted}`;
}

/* === Private Functions === */

function elapsedEndTime(finishedAt: string | null | undefined, now: number): number {
  return parseTimestampMs(finishedAt) ?? now;
}
