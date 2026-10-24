/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { formatNumber, formatTime, type TimeFormatOrigin } from "@/lib/format";
import { isDatetimeValue } from "@/lib/record-display";
import type { ValueFormat } from "@/lib/types";
import { numericValue } from "@/lib/utils";

// The default numeric axis/legend rule now lives beside the declared-format
// formatter it falls back to; re-exported so this module — and its test file
// — stay the plot layer's home for it.
export { defaultPlotNumber } from "@/lib/format";

/* === Constants === */

/**
 * The duration units a declared x column may name, mapped to the symbol the
 * axis prints, and how many seconds one of them is.
 *
 * The GUI reads an *elapsed* axis off the column the plot declares, never off
 * the values it receives: a trace whose first sample is 0.1 is 0.1 of
 * something, and only the declaration says of what. See
 * `.stkr/agents/todos/plot-time-axis-semantics.md` for the wire facts that
 * keep the column's own `unit` out of reach here.
 */
const DURATION_UNITS: ReadonlyMap<string, string> = new Map([
  ["s", "s"],
  ["sec", "s"],
  ["secs", "s"],
  ["second", "s"],
  ["seconds", "s"],
  ["ms", "ms"],
  ["msec", "ms"],
  ["msecs", "ms"],
  ["millisecond", "ms"],
  ["milliseconds", "ms"],
  ["us", "us"],
  ["\u00b5s", "us"],
  ["usec", "us"],
  ["usecs", "us"],
  ["microsecond", "us"],
  ["microseconds", "us"],
  ["min", "min"],
  ["mins", "min"],
  ["minute", "min"],
  ["minutes", "min"],
  ["h", "h"],
  ["hr", "h"],
  ["hrs", "h"],
  ["hour", "h"],
  ["hours", "h"],
]);

/** One of each duration unit, in seconds. The window label converts with it. */
const UNIT_SECONDS: Readonly<Record<string, number>> = {
  h: 3600,
  min: 60,
  ms: 0.001,
  s: 1,
  us: 0.000_001,
};

/** Beyond this the tick label is noise rather than precision. */
const MAX_TICK_FRACTION_DIGITS = 6;

/** A wall clock resolves no finer than a millisecond. */
const MAX_CLOCK_FRACTION_DIGITS = 3;

/* === Types === */

/** How a plot's x axis prints, and what one pixel of it spans. */
export type CursorXAxis = {
  timeAxis: boolean;
  xFormat: ValueFormat | null;
  xElapsedUnit: string | null;
  origin: TimeFormatOrigin;
  /** The visible x span, in the axis's own units. */
  span: number;
  /** The plot area's width in CSS pixels. */
  widthPx: number;
};

/* === Public Functions === */

/**
 * The duration unit a plot's declared x column names, or `null` when the
 * column does not affirmatively name one.
 *
 * The match is anchored on the column's last underscore-separated segment and
 * requires a segment before it, so `elapsed_s` is seconds and `duration_ms` is
 * milliseconds while `ms_per_frame` is not, and the bare fallback names every
 * plot without an x column shares — `timestamp`, `time`, `t` — are wall time
 * by construction. An unrecognized name changes nothing: wall time is the
 * floor, and only a declared duration lifts the axis off it.
 */
export function elapsedColumnUnit(column: string | null | undefined): string | null {
  const segments = (column ?? "").trim().toLowerCase().split("_").filter(Boolean);
  if (segments.length < 2) {
    return null;
  }
  return DURATION_UNITS.get(segments[segments.length - 1] as string) ?? null;
}

/**
 * One axis tick of an elapsed axis: the duration itself, in the unit the
 * column declared, never a date.
 *
 * The values on such an axis are already the duration — there is no origin to
 * subtract, which is what separates this from `formatTime`'s
 * `elapsed_since_run`/`elapsed_since_cycle`, where the values are epoch
 * seconds. The fraction digits come from the tick increment so neighbouring
 * ticks never collapse onto one label.
 */
export function formatElapsedTick(value: number, unit: string, increment?: number): string {
  if (!Number.isFinite(value)) {
    return "";
  }
  const digits =
    increment !== undefined && Number.isFinite(increment) && increment > 0
      ? Math.min(MAX_TICK_FRACTION_DIGITS, Math.max(0, Math.ceil(-Math.log10(increment))))
      : undefined;
  const text = digits === undefined ? String(value) : value.toFixed(digits);
  // `(-0.004).toFixed(2)` is "-0.00", a tick that reads as a negative zero.
  return `${Number(text) === 0 ? (digits === undefined ? "0" : (0).toFixed(digits)) : text} ${unit}`;
}

/**
 * A plot window of `seconds` stated in an elapsed axis's own unit, so the
 * control and the axis under it are counting the same thing.
 */
export function elapsedWindowLabel(seconds: number, unit: string): string {
  const perUnit = UNIT_SECONDS[unit] ?? 1;
  const value = seconds / perUnit;
  const text = Number.isInteger(value) || Math.abs(value) >= 100 ? String(Math.round(value)) : String(Number(value.toPrecision(3)));
  return `${text} ${unit}`;
}

/**
 * The x cursor readout, printed the way the axis under it prints: an elapsed
 * axis as its duration, a declared format through `formatTime`, a plain time
 * axis as the local date and clock (the axis's own time zone), anything else
 * as a number. The precision is what one pixel of the visible window resolves,
 * so a fifteen-second zoom shows milliseconds and an hour does not.
 */
export function formatCursorX(x: number, axis: CursorXAxis): string {
  if (!Number.isFinite(x)) return "";
  const perPixel = axis.widthPx > 0 && axis.span > 0 ? axis.span / axis.widthPx : undefined;

  if (axis.xElapsedUnit) return formatElapsedTick(x, axis.xElapsedUnit, perPixel);
  if (!axis.timeAxis) return formatNumber(x, axis.xFormat, { increment: perPixel });
  if (axis.xFormat) return formatTime(x, axis.xFormat, axis.origin);

  const digits = perPixel === undefined || perPixel >= 1 ? 0 : Math.min(MAX_CLOCK_FRACTION_DIGITS, Math.ceil(-Math.log10(perPixel)));
  return clockLabel(x, digits);
}

export function xValue(value: unknown, index: number): number {
  value = isDatetimeValue(value) ? value.$datetime : (numericValue(value) ?? value);
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

/* === Private Functions === */

/** Local `M/D/YY h:mm:ss[.fff] AM`, the date-and-clock form the time axis reads in. */
function clockLabel(epochSeconds: number, fractionDigits: number): string {
  const scale = 10 ** fractionDigits;
  const rounded = Math.round(epochSeconds * scale) / scale;
  const date = new Date(Math.floor(rounded) * 1000);

  const hours = date.getHours();
  const hour12 = hours % 12 === 0 ? 12 : hours % 12;
  const two = (value: number) => String(value).padStart(2, "0");
  const fraction =
    fractionDigits > 0 ? `.${String(Math.round((rounded - Math.floor(rounded)) * scale)).padStart(fractionDigits, "0")}` : "";

  const day = `${date.getMonth() + 1}/${date.getDate()}/${two(date.getFullYear() % 100)}`;
  const clock = `${hour12}:${two(date.getMinutes())}:${two(date.getSeconds())}${fraction} ${hours < 12 ? "AM" : "PM"}`;
  return `${day} ${clock}`;
}
