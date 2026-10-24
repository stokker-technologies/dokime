/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Prompt lifecycle markers for time-series plots: when an execution asked the
 * operator something, and when the operator answered, read from its records.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import type { HubRecord } from "@/lib/types";

/* === Re-Exports === */

/* === Constants === */

/** The request records of every prompt kind; each one asks the operator. */
const ASKED_RECORD_TYPES: ReadonlySet<string> = new Set([
  "prompt.request.confirmation",
  "prompt.request.form",
  "prompt.request.choice",
  "prompt.request.status",
]);

const ANSWERED_RECORD_TYPE = "prompt.response.submitted";

/* === Types === */

/** The two generic lifecycle moments a plot marks, whatever the prompt's kind. */
export type PromptMarkerKind = "asked" | "answered";

/** One prompt lifecycle moment at its record's time, in epoch seconds. */
export type PromptMarker = {
  kind: PromptMarkerKind;
  x: number;
  title: string;
};

/* === Classes === */

/* === Public Functions === */

/**
 * The prompt markers of `records`, in time order. A response carries no title
 * of its own, so it borrows its request's through `prompt_id`; a record
 * without a parseable timestamp marks nothing.
 */
export function promptMarkers(records: readonly HubRecord[]): PromptMarker[] {
  const titles = new Map<string, string>();
  for (const record of records) {
    if (!ASKED_RECORD_TYPES.has(record.record_type)) continue;
    const id = textDetail(record, "prompt_id");
    if (id) titles.set(id, promptTitle(record));
  }

  const markers: PromptMarker[] = [];
  for (const record of records) {
    const asked = ASKED_RECORD_TYPES.has(record.record_type);
    if (!asked && record.record_type !== ANSWERED_RECORD_TYPE) continue;
    const at = Date.parse(record.timestamp) / 1000;
    if (!Number.isFinite(at)) continue;
    const id = textDetail(record, "prompt_id");
    const title = asked ? promptTitle(record) : ((id ? titles.get(id) : undefined) ?? "Prompt");
    markers.push({ kind: asked ? "asked" : "answered", x: at, title });
  }
  return markers.sort((left, right) => left.x - right.x);
}

/** What a marker's hover reads: the moment, then the prompt's title. */
export function promptMarkerLabel(marker: PromptMarker): string {
  return `${marker.kind === "asked" ? "Prompt asked" : "Prompt answered"}: ${marker.title}`;
}

/* === Private Functions === */

function promptTitle(record: HubRecord): string {
  return textDetail(record, "title") ?? record.message ?? "Prompt";
}

function textDetail(record: HubRecord, key: string): string | null {
  const value = record.details?.[key];
  return typeof value === "string" && value.trim() ? value : null;
}

/* === Entry Point === */
