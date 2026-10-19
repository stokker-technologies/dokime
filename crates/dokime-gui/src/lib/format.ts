/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { ValueFormat } from "@/lib/types";

/* === Types === */

export type NumberFormatOptions = {
  /** The axis tick increment; sets the fraction digits when no format declares its own. */
  increment?: number;
};

export type FormattedValueWithUnit = {
  text: string;
  unit: string | null;
};

export type TimeFormatOrigin = {
  runStart?: number | null;
  cycleStart?: number | null;
};

/* === Constants === */

// IEC units: artifact sizes and memory readouts are counted in binary steps.
const BYTE_UNITS = ["B", "KiB", "MiB", "GiB", "TiB"] as const;

// Exponent -> SI prefix symbol, in multiples of three; "" is the unprefixed magnitude.
const SI_PREFIXES: ReadonlyArray<readonly [number, string]> = [
  [-24, "y"],
  [-21, "z"],
  [-18, "a"],
  [-15, "f"],
  [-12, "p"],
  [-9, "n"],
  [-6, "µ"],
  [-3, "m"],
  [0, ""],
  [3, "k"],
  [6, "M"],
  [9, "G"],
  [12, "T"],
  [15, "P"],
  [18, "E"],
];

const MIN_SI_EXPONENT = SI_PREFIXES[0][0];
const MAX_SI_EXPONENT = SI_PREFIXES[SI_PREFIXES.length - 1][0];

/* === Public Functions === */

/**
 * Formats a byte count for a size or memory readout: binary steps with IEC
 * units, one fraction digit below ten and none above it. An absent or
 * non-finite count renders as a dash, so a caller can drop it into a label
 * without a guard.
 */
export function formatBytes(value: number | null | undefined): string {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return "-";
  }
  let scaled = value;
  let unit = 0;
  while (Math.abs(scaled) >= 1024 && unit < BYTE_UNITS.length - 1) {
    scaled /= 1024;
    unit += 1;
  }
  if (unit === 0) {
    return `${scaled} ${BYTE_UNITS[0]}`;
  }
  return `${scaled.toFixed(Math.abs(scaled) >= 10 ? 0 : 1)} ${BYTE_UNITS[unit]}`;
}

/**
 * Formats a duration **given in seconds** — the unit every stored duration in
 * the contracts uses, so a caller holding milliseconds divides at the call
 * site rather than reaching for a second function. Absent or non-finite input
 * returns `null`, leaving the placeholder to the surface that renders it.
 */
export function formatDuration(seconds: number | null | undefined): string | null {
  if (seconds === null || seconds === undefined || !Number.isFinite(seconds)) {
    return null;
  }
  if (seconds > 0 && seconds < 1) {
    return `${Math.round(seconds * 1000)} ms`;
  }
  if (seconds >= 3600) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return minutes > 0 ? `${hours} h ${minutes} min` : `${hours} h`;
  }
  if (seconds >= 60) {
    const minutes = Math.floor(seconds / 60);
    const remainder = Math.round(seconds % 60);
    return remainder > 0 ? `${minutes} min ${remainder} s` : `${minutes} min`;
  }
  if (seconds > 0 && seconds < 10 && seconds % 1 !== 0) {
    return `${seconds.toFixed(1)} s`;
  }
  return `${Math.round(seconds)} s`;
}

/**
 * Formats one numeric value for an axis tick, a legend readout, or a table
 * cell, per a declared `ValueFormat`. A `null` format — no declaration at any
 * level of the precedence chain — falls back to {@link defaultPlotNumber}. A
 * time-kind format reaching here (a mismatched or stale caller) falls back the
 * same way, so a renderer never throws over a format it cannot use.
 */
export function formatNumber(value: number, format: ValueFormat | null, options?: NumberFormatOptions): string {
  if (!Number.isFinite(value)) {
    return "";
  }
  switch (format?.kind) {
    case undefined:
      return defaultPlotNumber(value, options?.increment);
    case "normal":
      return format.max_fraction_digits == null
        ? defaultPlotNumber(value, options?.increment)
        : value.toLocaleString(undefined, { maximumFractionDigits: format.max_fraction_digits });
    case "fixed":
      return value.toFixed(format.decimals);
    case "scientific":
      return value.toExponential(format.significant_digits - 1);
    case "engineering":
      return engineeringText(value, format.significant_digits);
    case "si_prefix": {
      const { mantissaText, prefix } = siPrefixParts(value, format.significant_digits);
      return prefix ? `${mantissaText}${prefix}` : mantissaText;
    }
    default:
      // A time-kind format: not this function's kind. Render the number plainly.
      return defaultPlotNumber(value, options?.increment);
  }
}

/**
 * Formats one numeric value alongside its unit. `si_prefix` folds the chosen
 * prefix into the unit (`120 µm`) rather than the number text, which is why
 * the unit travels with the call instead of being appended by the caller.
 */
export function formatWithUnit(value: number, format: ValueFormat | null, unit?: string | null): FormattedValueWithUnit {
  if (!Number.isFinite(value)) {
    return { text: "", unit: unit ?? null };
  }
  if (format?.kind === "si_prefix") {
    const { mantissaText, prefix } = siPrefixParts(value, format.significant_digits);
    return { text: mantissaText, unit: unit ? `${prefix}${unit}` : prefix || null };
  }
  return { text: formatNumber(value, format), unit: unit ?? null };
}

/**
 * Formats one epoch-seconds timestamp per a declared time `ValueFormat`. A
 * `null` format, or a number-kind format reaching here, falls back to a
 * locale wall-clock string. An elapsed format whose origin is absent renders
 * as zero elapsed — the caller is responsible for filling `origin` with the
 * documented fallback (the payload's `run_started_at`/`cycle_started_at`, or
 * else the first point of the first series) before calling this.
 */
export function formatTime(epochSeconds: number, format: ValueFormat | null, origin: TimeFormatOrigin): string {
  if (!Number.isFinite(epochSeconds)) {
    return "";
  }
  if (format?.kind === "absolute_time") {
    return applyTimePattern(new Date(epochSeconds * 1000), format.pattern);
  }
  if (format?.kind === "elapsed_since_run" || format?.kind === "elapsed_since_cycle") {
    const originSeconds = format.kind === "elapsed_since_run" ? origin.runStart : origin.cycleStart;
    const elapsedSeconds = Number.isFinite(originSeconds) ? epochSeconds - (originSeconds as number) : 0;
    return elapsedText(elapsedSeconds, format.unit);
  }
  return new Date(epochSeconds * 1000).toLocaleString();
}

/**
 * Formats one numeric value for an axis tick or a legend readout when no
 * `ValueFormat` is declared.
 *
 * uPlot's default is `Intl.NumberFormat` with three fraction digits, which
 * prints every reading below a thousandth as `0` and a negative one as `-0`.
 * Values that small or beyond a million switch to exponential notation with
 * two significant digits; the rest keep locale formatting, with the fraction
 * digits taken from the tick increment when an axis supplies one so that
 * neighbouring ticks never collapse onto the same label.
 */
export function defaultPlotNumber(value: number, increment?: number): string {
  if (!Number.isFinite(value)) {
    return "";
  }
  if (value === 0) {
    return "0";
  }
  const magnitude = Math.abs(value);
  if (magnitude < 1e-3 || magnitude >= 1e6) {
    // A tick increment says how much precision the axis actually separates.
    // Without it every tick keeps one fraction digit, which turns a 0-to-1e-13
    // axis into rows that differ only in a digit nobody asked for.
    const digits =
      increment !== undefined && Number.isFinite(increment) && increment > 0
        ? Math.min(6, Math.max(0, Math.floor(Math.log10(magnitude)) - Math.floor(Math.log10(increment))))
        : 1;
    return value.toExponential(digits);
  }
  const fromIncrement =
    increment !== undefined && Number.isFinite(increment) && increment > 0 ? Math.max(0, Math.ceil(-Math.log10(increment))) : undefined;
  const maximumFractionDigits = Math.min(6, fromIncrement ?? 4);
  return value.toLocaleString(undefined, { maximumFractionDigits });
}

/* === Private Functions === */

/** Elapsed seconds since the format's origin, rendered with the declared unit. */
function elapsedText(elapsedSeconds: number, unit: "seconds" | "milliseconds" | "minutes"): string {
  if (unit === "milliseconds") {
    return `${Math.round(elapsedSeconds * 1000)}ms`;
  }
  if (unit === "minutes") {
    return `${(elapsedSeconds / 60).toFixed(1)}m`;
  }
  return `${elapsedSeconds.toFixed(1)}s`;
}

/** Substitutes `%Y %m %d %H %M %S %L %%`; every other run of text, including an unknown token, is literal. */
function applyTimePattern(date: Date, pattern: string): string {
  return pattern.replace(/%Y|%m|%d|%H|%M|%S|%L|%%/g, (token) => {
    switch (token) {
      case "%Y":
        return String(date.getFullYear());
      case "%m":
        return pad(date.getMonth() + 1, 2);
      case "%d":
        return pad(date.getDate(), 2);
      case "%H":
        return pad(date.getHours(), 2);
      case "%M":
        return pad(date.getMinutes(), 2);
      case "%S":
        return pad(date.getSeconds(), 2);
      case "%L":
        return pad(date.getMilliseconds(), 3);
      default:
        // "%%"
        return "%";
    }
  });
}

function pad(value: number, width: number): string {
  return String(value).padStart(width, "0");
}

/** `value` normalized to a mantissa in `[1, 1000)` and an exponent that is a multiple of three. */
function engineeringParts(value: number): { sign: number; mantissa: number; exponent: number } {
  if (value === 0) {
    return { sign: 1, mantissa: 0, exponent: 0 };
  }
  const sign = value < 0 ? -1 : 1;
  const magnitude = Math.abs(value);
  let exponent = Math.floor(Math.log10(magnitude));
  exponent -= ((exponent % 3) + 3) % 3;
  let mantissa = magnitude / 10 ** exponent;
  // Guard the boundary a float rounds past (e.g. 999.9999999 -> 1000).
  if (mantissa >= 1000) {
    mantissa /= 1000;
    exponent += 3;
  }
  return { sign, mantissa, exponent };
}

function engineeringText(value: number, significantDigits: number): string {
  const { sign, mantissa, exponent } = engineeringParts(value);
  if (mantissa === 0) {
    return "0";
  }
  const mantissaText = (sign * mantissa).toFixed(fractionDigitsFor(mantissa, significantDigits));
  return `${mantissaText}e${exponent >= 0 ? "+" : ""}${exponent}`;
}

function siPrefixParts(value: number, significantDigits: number): { mantissaText: string; prefix: string } {
  const { sign, mantissa, exponent } = engineeringParts(value);
  if (mantissa === 0) {
    return { mantissaText: "0", prefix: "" };
  }
  const clampedExponent = Math.min(MAX_SI_EXPONENT, Math.max(MIN_SI_EXPONENT, exponent));
  const rescaled = mantissa * 10 ** (exponent - clampedExponent);
  const mantissaText = (sign * rescaled).toFixed(fractionDigitsFor(rescaled, significantDigits));
  const prefix = SI_PREFIXES.find(([candidate]) => candidate === clampedExponent)?.[1] ?? "";
  return { mantissaText, prefix };
}

/** Fraction digits left for `significantDigits` once the mantissa's integer digits are spent. */
function fractionDigitsFor(mantissa: number, significantDigits: number): number {
  const integerDigits = Math.max(1, Math.floor(Math.log10(mantissa)) + 1);
  return Math.max(0, significantDigits - integerDigits);
}
