/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { ElapsedUnit, ValueFormat } from "@host/model";
import type { Wire } from "@/lib/rpc/canonical";

/* === Types === */

export type NumberFormatOptions = {
  /** The axis tick increment; sets the fraction digits when no format declares its own. */
  increment?: number;
};

export type TimeFormatOrigin = {
  runStart?: number | null;
  cycleStart?: number | null;
};

/* === Constants === */

/** The longest value a summary tile shows whole; longer ones are abbreviated. */
export const COMPACT_DECIMAL_CHARACTERS = 12;

/** The significant digits an abbreviated tile value keeps. */
const COMPACT_SIGNIFICANT_DIGITS = 6;

/** Presentation for each serialized elapsed unit; generated messages stay outside the report bundle. */
const ELAPSED_FORMATTERS: Record<ElapsedUnit, (seconds: number) => string> = {
  0: (seconds) => `${seconds.toFixed(1)}s`,
  1: (seconds) => `${seconds.toFixed(1)}s`,
  2: (seconds) => `${Math.round(seconds * 1000)}ms`,
  3: (seconds) => `${(seconds / 60).toFixed(1)}m`,
};

const TITLE_SMALL_WORDS = new Set([
  "a",
  "an",
  "the",
  "and",
  "or",
  "but",
  "nor",
  "of",
  "in",
  "on",
  "at",
  "to",
  "for",
  "by",
  "with",
  "from",
  "as",
  "into",
  "per",
  "via",
  "vs",
  "over",
  "up",
  "off",
]);

/** The units below a minute, each used while the rounded value stays under its limit (seconds). */
const SUBSECOND_SCALES: ReadonlyArray<{ divisor: number; limit: number; unit: string }> = [
  { divisor: 1e-6, limit: 1e-3, unit: "µs" },
  { divisor: 1e-3, limit: 1, unit: "ms" },
  { divisor: 1, limit: 59.5, unit: "s" },
];

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

/** Decimal display uses a fixed decimal point and never groups digits. */
export function formatDecimal(value: number, maximumFractionDigits = 3): string {
  return value.toLocaleString("en-US", { useGrouping: false, maximumFractionDigits });
}

/** Format one generated title word while retaining caller-owned acronyms. */
export function formatTitleWord(word: string, index: number, words: readonly string[]): string {
  const lower = word.toLowerCase();
  if (
    index > 0 &&
    index < words.length - 1 &&
    TITLE_SMALL_WORDS.has(lower) &&
    !(
      ["up", "off", "on", "in", "over"].includes(lower) &&
      /^(back|turn|log|sign|set|power|switch|clean|shut|hand|take)$/i.test(words[index - 1])
    )
  ) {
    return lower;
  }
  return word.charAt(0).toUpperCase() + word.slice(1);
}

/** A label and its optional unit share one line in every presentation. */
export function formatUnitLabel(label: string, unit?: string | null): string {
  const symbol = unit?.trim();
  return symbol ? `${label} [${symbol}]` : label;
}

/** A duration split into the figure and the unit it is printed in. */
export type DurationParts = { figure: string; unit: string };

/**
 * Formats a duration **given in seconds** — the unit every stored duration in
 * the contracts uses, so a caller holding milliseconds divides at the call
 * site rather than reaching for a second function. Absent or non-finite input
 * returns `null`, leaving the placeholder to the surface that renders it.
 * Every surface that prints a duration — parameters, step timings, elapsed
 * and age labels, the report — prints it through this one helper.
 */
export function formatDuration(seconds: number | null | undefined): string | null {
  if (seconds === null || seconds === undefined || !Number.isFinite(seconds)) {
    return null;
  }
  const parts = durationParts(seconds);
  return `${parts.figure} ${parts.unit}`;
}

/**
 * A duration in seconds with the precision its magnitude needs: microseconds
 * and milliseconds below a second, seconds to three significant digits below
 * a minute, then minutes and seconds, then hours and minutes. "0.05" is
 * "50 ms", "0.001" is "1 ms", "2.5" is "2.5 s" and "80" is "1 min 20 s"; a
 * value that rounds up to the next unit is printed in that unit.
 */
export function durationParts(seconds: number): DurationParts {
  const sign = seconds < 0 ? "-" : "";
  const magnitude = Math.abs(seconds);
  if (magnitude === 0) {
    return { figure: "0", unit: "s" };
  }
  const signed = (parts: DurationParts): DurationParts => ({ ...parts, figure: `${sign}${parts.figure}` });
  for (const scale of SUBSECOND_SCALES) {
    const rounded = significant(magnitude / scale.divisor);
    if (magnitude < scale.limit && rounded * scale.divisor < scale.limit) {
      return signed({ figure: `${rounded}`, unit: scale.unit });
    }
  }
  const whole = Math.round(magnitude);
  if (whole < 3600) {
    const minutes = Math.floor(whole / 60);
    const remainder = whole % 60;
    return signed(remainder > 0 ? { figure: `${minutes} min ${remainder}`, unit: "s" } : { figure: `${minutes}`, unit: "min" });
  }
  const hours = Math.floor(whole / 3600);
  const minutes = Math.floor((whole % 3600) / 60);
  return signed(minutes > 0 ? { figure: `${hours} h ${minutes}`, unit: "min" } : { figure: `${hours}`, unit: "h" });
}

/**
 * Formats one numeric value for an axis tick, a legend readout, or a table
 * cell, per a declared `ValueFormat`. A `null` format — no declaration at any
 * level of the precedence chain — falls back to {@link defaultPlotNumber}. A
 * time-kind format reaching here (a mismatched or stale caller) falls back the
 * same way, so a renderer never throws over a format it cannot use.
 */
export function formatNumber(value: number, format: Wire<ValueFormat> | null, options?: NumberFormatOptions): string {
  if (!Number.isFinite(value)) {
    return "";
  }
  const kind = format?.kind?.kind;
  switch (kind?.case) {
    case undefined:
      return defaultPlotNumber(value, options?.increment);
    case "normal":
      return kind.value.maxFractionDigits == null
        ? defaultPlotNumber(value, options?.increment)
        : formatDecimal(value, kind.value.maxFractionDigits);
    case "fixed":
      return value.toFixed(kind.value.decimals);
    case "scientific":
      return value.toExponential(kind.value.significantDigits - 1);
    case "engineering":
      return engineeringText(value, kind.value.significantDigits);
    case "siPrefix": {
      const { mantissaText, prefix } = siPrefixParts(value, kind.value.significantDigits);
      return prefix ? `${mantissaText}${prefix}` : mantissaText;
    }
    default:
      // A time-kind format: not this function's kind. Render the number plainly.
      return defaultPlotNumber(value, options?.increment);
  }
}

/**
 * Formats one epoch-seconds timestamp per a declared time `ValueFormat`. A
 * `null` format, or a number-kind format reaching here, falls back to a
 * locale wall-clock string. An elapsed format whose origin is absent renders
 * as zero elapsed — the caller is responsible for filling `origin` with the
 * documented fallback (the payload's `run_started_at`/`cycle_started_at`, or
 * else the first point of the first series) before calling this.
 */
export function formatTime(epochSeconds: number, format: Wire<ValueFormat> | null, origin: TimeFormatOrigin): string {
  if (!Number.isFinite(epochSeconds)) {
    return "";
  }
  const kind = format?.kind?.kind;
  if (kind?.case === "absoluteTime") {
    return applyTimePattern(new Date(epochSeconds * 1000), kind.value.pattern);
  }
  if (kind?.case === "elapsedSinceRun" || kind?.case === "elapsedSinceCycle") {
    const originSeconds = kind.case === "elapsedSinceRun" ? origin.runStart : origin.cycleStart;
    const elapsedSeconds = Number.isFinite(originSeconds) ? epochSeconds - (originSeconds as number) : 0;
    return elapsedText(elapsedSeconds, kind.value.unit);
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
 * two significant digits; the rest use ungrouped decimal formatting, with the fraction
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
  return formatDecimal(value, maximumFractionDigits);
}

/**
 * Abbreviates an exact decimal for a narrow summary tile. Integers, terminating
 * decimals and rationals (`1/3`) of up to {@link COMPACT_DECIMAL_CHARACTERS}
 * characters stay whole. Longer values keep six significant digits: fixed
 * notation from 1e-4 up to 1e6, exponential notation as plot axes print it
 * otherwise. The digits come from the decimal text through integer arithmetic,
 * never a floating-point conversion; text that is not a decimal is unchanged.
 */
export function formatCompactDecimal(decimal: string): string {
  if (decimal.length <= COMPACT_DECIMAL_CHARACTERS) {
    return decimal;
  }
  const parsed = parseDecimal(decimal);
  if (!parsed) {
    return decimal;
  }
  const { negative, numerator, denominator } = parsed;
  if (numerator === 0n) {
    return "0";
  }
  // The digit counts place the magnitude within one decade of 10^exponent.
  let exponent = numerator.toString().length - denominator.toString().length;
  if (!atLeastPower(numerator, denominator, exponent)) {
    exponent -= 1;
  }
  const shift = COMPACT_SIGNIFICANT_DIGITS - 1 - exponent;
  const scaledNumerator = shift >= 0 ? numerator * 10n ** BigInt(shift) : numerator;
  const scaledDenominator = shift >= 0 ? denominator : denominator * 10n ** BigInt(-shift);
  let mantissa = (2n * scaledNumerator + scaledDenominator) / (2n * scaledDenominator);
  if (mantissa === 10n ** BigInt(COMPACT_SIGNIFICANT_DIGITS)) {
    mantissa /= 10n;
    exponent += 1;
  }
  const digits = mantissa.toString();
  const sign = negative ? "-" : "";
  if (exponent >= -4 && exponent < 6) {
    const whole = exponent >= 0 ? digits.slice(0, exponent + 1) : "0";
    const fraction = (exponent >= 0 ? digits.slice(exponent + 1) : "0".repeat(-exponent - 1) + digits).replace(/0+$/, "");
    return `${sign}${whole}${fraction ? `.${fraction}` : ""}`;
  }
  const fraction = digits.slice(1).replace(/0+$/, "");
  return `${sign}${digits[0]}${fraction ? `.${fraction}` : ""}e${exponent < 0 ? "-" : "+"}${Math.abs(exponent)}`;
}

/* === Private Functions === */

/** An integer, decimal, exponential or rational text as one exact fraction. */
function parseDecimal(text: string): { negative: boolean; numerator: bigint; denominator: bigint } | null {
  const rational = /^(-?)(\d+)\/(\d+)$/.exec(text);
  if (rational) {
    const denominator = BigInt(rational[3]);
    return denominator === 0n ? null : { negative: rational[1] === "-", numerator: BigInt(rational[2]), denominator };
  }
  const match = /^(-?)(\d+)(?:\.(\d+))?(?:e([+-]?\d+))?$/i.exec(text);
  if (!match) {
    return null;
  }
  const fraction = match[3] ?? "";
  const power = Number(match[4] ?? 0) - fraction.length;
  const digits = BigInt(`${match[2]}${fraction}`);
  return {
    negative: match[1] === "-",
    numerator: power >= 0 ? digits * 10n ** BigInt(power) : digits,
    denominator: power >= 0 ? 1n : 10n ** BigInt(-power),
  };
}

/** Is numerator / denominator at least 10^exponent? */
function atLeastPower(numerator: bigint, denominator: bigint, exponent: number): boolean {
  return exponent >= 0 ? numerator >= denominator * 10n ** BigInt(exponent) : numerator * 10n ** BigInt(-exponent) >= denominator;
}

/** Three significant digits, without trailing zeros. */
function significant(value: number): number {
  return Number(value.toPrecision(3));
}

/** Elapsed seconds since the format's origin, rendered with the declared unit. */
function elapsedText(elapsedSeconds: number, unit: ElapsedUnit): string {
  const format = ELAPSED_FORMATTERS[unit] ?? ELAPSED_FORMATTERS[0];
  return format(elapsedSeconds);
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
