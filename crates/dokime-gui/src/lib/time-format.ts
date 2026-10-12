/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Constants === */

/**
 * Wall-clock floor for plausible timestamps. The runtime emits epoch-relative
 * placeholders in transitional states (e.g. `1970-01-01T00:00:00.027Z`, a
 * few milliseconds past epoch zero); anything before this floor is a
 * placeholder, never a real test timestamp.
 */
const MIN_VALID_TIMESTAMP_MS = Date.UTC(2000, 0, 1);

/* === Public Functions === */

/**
 * Parse a wire timestamp into epoch milliseconds, rejecting missing values,
 * unparseable strings, and epoch-relative placeholders that the runtime
 * emits in transitional states. Returns `null` for anything that should
 * render as "no timestamp".
 */
export function parseTimestampMs(timestamp: string | null | undefined): number | null {
  if (!timestamp) {
    return null;
  }
  const parsed = Date.parse(timestamp);
  if (!Number.isFinite(parsed) || parsed < MIN_VALID_TIMESTAMP_MS) {
    return null;
  }
  return parsed;
}

export function compactElapsedLabel(
  startedAt: string | null | undefined,
  now: number,
  finishedAt?: string | null,
): string {
  const started = parseTimestampMs(startedAt);
  if (started === null) {
    return "-";
  }
  return compactDurationLabel(started, elapsedEndTime(finishedAt, now));
}

export function compactAgeLabel(timestamp: string | null | undefined, now: number): string {
  const then = parseTimestampMs(timestamp);
  if (then === null) {
    return "-";
  }
  return compactDurationLabel(then, now);
}

export function elapsedLabel(
  startedAt: string | null | undefined,
  now: number,
  finishedAt?: string | null,
): string {
  const started = parseTimestampMs(startedAt);
  if (started === null) {
    return "-";
  }
  return durationLabel(Math.max(0, Math.floor((elapsedEndTime(finishedAt, now) - started) / 1000)));
}

export function ageLabel(timestamp: string | null | undefined, now: number): string {
  const updated = parseTimestampMs(timestamp);
  if (updated === null) {
    return "-";
  }
  return durationLabel(Math.max(0, Math.floor((now - updated) / 1000)));
}

export function durationLabel(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}s`;
  }
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  if (minutes < 60) {
    return `${minutes}m ${remainingSeconds}s`;
  }
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  return `${hours}h ${remainingMinutes}m`;
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

/* === Private Functions === */

function compactDurationLabel(thenMs: number, now: number): string {
  const seconds = Math.max(0, Math.floor((now - thenMs) / 1000));
  if (seconds < 60) {
    return `${seconds}s`;
  }
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes}m`;
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours}h ${minutes % 60}m`;
  }
  return `${Math.floor(hours / 24)}d`;
}

function elapsedEndTime(finishedAt: string | null | undefined, now: number): number {
  return parseTimestampMs(finishedAt) ?? now;
}
