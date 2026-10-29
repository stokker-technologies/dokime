/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { formatNumber, formatTime } from "@/lib/format";
import type { RecordDisplay, RecordTone, ValueFormat } from "@/lib/types";
import { cn, isRecord, numericValue } from "@/lib/utils";

/** A datetime value on the wire: the one-key tagged map `{"$datetime": "<rfc3339>"}`. */
type DatetimeValue = { $datetime: string };

export function recordToneShellClass(tone: RecordTone, className?: string): string {
  return cn(
    "border",
    tone === "danger" && "border-destructive/30 bg-destructive/10 text-destructive",
    tone === "warning" && "border-warning/30 bg-warning/10 text-warning",
    tone === "success" && "border-success/30 bg-success/10 text-success",
    tone === "info" && "border-info/30 bg-info/10 text-info",
    tone === "accent" && "border-accent bg-accent text-accent-foreground",
    tone === "muted" && "border-border bg-muted/60 text-muted-foreground",
    tone === "neutral" && "border-border bg-muted/30 text-muted-foreground",
    className,
  );
}

export function fallbackRecordDisplay(severity: string): RecordDisplay {
  return {
    icon: "clock",
    tone: recordTone(null, severity),
  };
}

/**
 * How a record's level reads on a dense row: the word, the pill's variant, and
 * the clock.
 *
 * The console drawer and the document's records card are the same rows in two
 * places, so the level presentation is declared once here rather than in
 * either of them.
 */
export function recordLevelLabel(level: string): string {
  return level ? level.charAt(0).toUpperCase() + level.slice(1) : "Info";
}

export function recordLevelVariant(level: string): "outline" | "destructive" | "warning" {
  if (level === "error") return "destructive";
  if (level === "warning" || level === "warn") return "warning";
  return "outline";
}

/** The text colour a dense console row gives a level; no pill, colour only. */
export function recordLevelTextClass(level: string): string {
  if (level === "error") return "text-destructive";
  if (level === "warning" || level === "warn") return "text-warning";
  if (level === "info") return "text-foreground";
  return "text-muted-foreground";
}

/** The time of day a dense record row prints, or an em dash for no time. */
export function recordClockLabel(timestamp: string | null | undefined): string {
  const at = timestamp ? new Date(timestamp) : null;
  return !at || Number.isNaN(at.getTime()) ? "\u2014" : at.toLocaleTimeString();
}

export function recordToneRowClass(tone: RecordTone, className?: string): string {
  return cn(
    tone === "danger" && "border-l-destructive",
    tone === "warning" && "border-l-warning",
    tone === "success" && "border-l-success",
    tone === "info" && "border-l-info",
    tone === "accent" && "border-l-accent-foreground/60",
    tone === "muted" && "border-l-muted-foreground/40",
    tone === "neutral" && "border-l-border",
    className,
  );
}

/** Whether `value` is the wire's tagged datetime map, `{"$datetime": "<rfc3339>"}`. */
export function isDatetimeValue(value: unknown): value is DatetimeValue {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return false;
  }
  const keys = Object.keys(value as Record<string, unknown>);
  return keys.length === 1 && keys[0] === "$datetime" && typeof (value as Record<string, unknown>).$datetime === "string";
}

/**
 * Renders one record or telemetry field value for display, honouring a
 * declared column `format` where one is available. A tagged datetime value
 * renders as its wall clock instead of falling through to raw JSON; a plain
 * number honours a numeric format; anything else renders as plain text.
 */
export function formatRecordValue(value: unknown, format?: ValueFormat | null): string {
  value = numericValue(value) ?? value;
  if (isDatetimeValue(value)) {
    const epochSeconds = Date.parse(value.$datetime) / 1000;
    return Number.isFinite(epochSeconds) ? formatTime(epochSeconds, format ?? null, {}) : value.$datetime;
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return formatNumber(value, format ?? null);
  }
  if (isRecord(value) && value.__dokime_native_integer__ === true && typeof value.decimal === "string") return value.decimal;
  if (value === null || value === undefined) {
    return "-";
  }
  if (typeof value === "string") {
    return value;
  }
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  return JSON.stringify(value);
}

function recordTone(value: string | null | undefined, severity: string): RecordTone {
  if (
    value === "accent" ||
    value === "danger" ||
    value === "info" ||
    value === "muted" ||
    value === "neutral" ||
    value === "success" ||
    value === "warning"
  ) {
    return value;
  }
  if (severity === "error") return "danger";
  if (severity === "warning" || severity === "warn") return "warning";
  if (severity === "debug") return "muted";
  return "neutral";
}
