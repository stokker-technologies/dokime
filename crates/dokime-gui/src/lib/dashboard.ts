/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { numericValue } from "@/lib/utils";

import { formatDuration, formatNumber } from "./format";
import { formatRecordValue, isDatetimeValue } from "./record-display";
import { parseTimestampMs } from "./time-format";
import type { HubRecord } from "./types";

/* === Types === */

/* === Constants === */

const ARTIFACT_LABEL_WORDS: Record<string, string> = {
  api: "API",
  csv: "CSV",
  html: "HTML",
  id: "ID",
  jpeg: "JPEG",
  jpg: "JPG",
  json: "JSON",
  pdf: "PDF",
  png: "PNG",
  svg: "SVG",
  txt: "TXT",
  ui: "UI",
  url: "URL",
  wifi: "WiFi",
  xml: "XML",
  yaml: "YAML",
  yml: "YAML",
  zip: "ZIP",
};

/* === Public Functions === */

export function formatValue(value: unknown, unit?: string | null, key = ""): string {
  // A datetime value serializes as `{"$datetime": "<rfc3339>"}`; without this it
  // falls through to the object branch below and prints as "-".
  if (isDatetimeValue(value)) {
    return formatRecordValue(value, null);
  }
  // A list parameter's value is one array; the parameters table reads it as a
  // list of its elements, with the unit stated once at the end.
  if (Array.isArray(value)) {
    if (value.length === 0) {
      return "(empty)";
    }
    const items = value.map((item) => formatValue(item, null, key)).join(", ");
    return unit ? `${items} ${unit}` : items;
  }
  const parsed = numeric(value);
  if (parsed === null) {
    if (typeof value === "string" && value.length > 0) {
      return value;
    }
    if (typeof value === "boolean") {
      return value ? "True" : "False";
    }
    return "-";
  }
  const normalizedKey = key.toLowerCase();
  if (unit) {
    return `${dashboardNumber(parsed)} ${unit}`;
  }
  if (unit === "m" || normalizedKey.includes("distance")) {
    return `${dashboardNumber(parsed)} m`;
  }
  if (unit === "ms" || normalizedKey.includes("latency")) {
    return `${dashboardNumber(parsed)} ms`;
  }
  if (normalizedKey.includes("uptime")) {
    return `${formatDuration(parsed)} (${dashboardNumber(parsed)} s)`;
  }
  if (unit === "s" || normalizedKey.endsWith("_s") || normalizedKey.includes("duration")) {
    return formatDuration(parsed) ?? "-";
  }
  return Number.isInteger(parsed) ? `${parsed}` : dashboardNumber(parsed);
}

export function recordMessage(record: HubRecord): string {
  return record.message || record.details.message?.toString() || record.record_type;
}

export function recordTitle(record: HubRecord): string {
  const message = recordMessage(record);
  if (record.record_type.startsWith("test.execution.")) {
    return compactLifecycleMessage(message, record, "Execution");
  }
  if (record.record_type.startsWith("test.cycle.")) {
    return compactLifecycleMessage(message, record, "Test Cycle");
  }
  return message;
}

export function recordTime(record: HubRecord): string {
  const parsed = parseTimestampMs(record.timestamp);
  return parsed === null ? "-" : new Date(parsed).toLocaleTimeString();
}

export function artifactLabel(label: string): string {
  const words = label
    .trim()
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .replace(/([A-Z])([A-Z][a-z])/g, "$1 $2")
    .split(/[\s_.-]+/)
    .filter(Boolean);
  if (!words.length) {
    return label.trim();
  }
  return words.map(titleCaseArtifactWord).join(" ");
}

/* === Private Functions === */

function numeric(value: unknown): number | null {
  value = numericValue(value) ?? value;
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }
  if (typeof value === "string" && value.trim()) {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : null;
  }
  return null;
}

function titleCaseArtifactWord(word: string): string {
  const mapped = ARTIFACT_LABEL_WORDS[word.toLowerCase()];
  if (mapped) {
    return mapped;
  }
  if (word.toUpperCase() === word && word.length <= 5) {
    return word;
  }
  return `${word.slice(0, 1).toUpperCase()}${word.slice(1).toLowerCase()}`;
}

/* Dashboard facts read as fixed columns: two decimals until the integer part
 * takes three digits, then none. A value too small for two decimals keeps its
 * significant digits instead: a 1 ms sample interval is not "0.00 s". */
function dashboardNumber(value: number): string {
  if (value !== 0 && Math.abs(value) < 0.005) {
    return String(Number(value.toPrecision(3)));
  }
  return formatNumber(value, { kind: "fixed", decimals: Math.abs(value) >= 100 ? 0 : 2 });
}

function compactLifecycleMessage(message: string, record: HubRecord, fallbackSubject: string): string {
  const match = /^(?<prefix>[^:]+:\s*)(?<label>.+)$/u.exec(message);
  const prefix = match?.groups?.prefix ?? `${fallbackSubject}: `;
  const label = match?.groups?.label ?? message;
  return `${prefix}${compactCaseLabel(label, record)}`;
}

function compactCaseLabel(label: string, record: HubRecord): string {
  const parenIndex = label.indexOf(" (");
  if (parenIndex > 0) {
    return label.slice(0, parenIndex);
  }
  const scopedTemplate = record.scope?.template_id ?? record.template_id;
  if (scopedTemplate && label.includes(scopedTemplate)) {
    return titleFromIdentifier(scopedTemplate);
  }
  return label;
}

function titleFromIdentifier(value: string): string {
  const tail = value.split(".").at(-1) ?? value;
  return tail
    .split(/[_-]+/u)
    .filter(Boolean)
    .map((word) => `${word.slice(0, 1).toUpperCase()}${word.slice(1)}`)
    .join(" ");
}
