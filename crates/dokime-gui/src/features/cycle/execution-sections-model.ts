/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* How the execution page divides what it shows into its fixed sections.
 *
 * The page reads in one order whatever the document authored: Control,
 * Parameters, Steps & Criteria, Operator Notes, Media, Artifacts, Report,
 * Records, Traceability. The built-in card kinds each own a section; every
 * other kind is authored content and lands in Report, in the order the
 * document gave it. A section exists when the document holds its kind, so a
 * document that leaves one out keeps it out. Parameter rows carry the figure
 * the Parameters grid prints and the column widths both halves share.
 */

/* === Local Imports === */

import { formatValue } from "@/lib/dashboard";
import { durationParts } from "@/lib/format";
import type { DocumentViewCard, JsonObject, ParameterDefinition, PromptRequest } from "@/lib/types";

/* === Types === */

/** The document-backed sections, in page order; Control and Parameters are the page's own. */
export type ExecutionDocumentSection = "steps-criteria" | "notes" | "media" | "artifacts" | "report" | "records" | "traceability";

/** One materialized parameter, as the Parameters grid prints it. */
export type ExecutionParameterRow = {
  /** The parameter's name in the case, which is also its stable key. */
  key: string;
  /** The declared label, or the name when the template declares none. */
  label: string;
  value: unknown;
  /** The value as the figure column prints it, without its unit. */
  text: string;
  /** The declared unit, or the time unit a duration name implies; `null` for none. */
  unit: string | null;
};

/** Widths, in characters of the figure font, both halves of the grid share. */
export type ExecutionParameterColumns = {
  value: number;
  unit: number;
};

/* === Constants === */

export const EXECUTION_DOCUMENT_SECTIONS: readonly ExecutionDocumentSection[] = [
  "steps-criteria",
  "notes",
  "media",
  "artifacts",
  "report",
  "records",
  "traceability",
];

/** A figure longer than this wraps inside its column instead of widening it. */
const MAX_FIGURE_CHARACTERS = 28;

/** Seconds per time unit a parameter may declare. */
const TIME_UNITS: Record<string, number> = { us: 1e-6, µs: 1e-6, ms: 1e-3, s: 1, min: 60, h: 3600 };

const BUILT_IN_SECTION: Partial<Record<DocumentViewCard["kind"], ExecutionDocumentSection>> = {
  steps: "steps-criteria",
  criteria: "steps-criteria",
  notes: "notes",
  media: "media",
  artifacts: "artifacts",
  records: "records",
  traceability: "traceability",
};

/* === Public Functions === */

/** The section a card of this kind belongs to; authored kinds belong to Report. */
export function executionSectionOf(kind: DocumentViewCard["kind"]): ExecutionDocumentSection {
  return BUILT_IN_SECTION[kind] ?? "report";
}

/** The sections the given cards fill, in page order. */
export function executionDocumentSections(cards: readonly Pick<DocumentViewCard, "kind">[]): ExecutionDocumentSection[] {
  const present = new Set(cards.map((card) => executionSectionOf(card.kind)));
  return EXECUTION_DOCUMENT_SECTIONS.filter((section) => present.has(section));
}

/**
 * The materialized parameters in declaration order, each with its declared
 * label and unit; names the template does not declare follow in case order.
 */
export function executionParameterRows(
  parameters: JsonObject | null | undefined,
  definitions: readonly Pick<ParameterDefinition, "name" | "label" | "unit">[],
): ExecutionParameterRow[] {
  const values = parameters ?? {};
  const declared = definitions.filter((definition) => Object.hasOwn(values, definition.name));
  const declaredNames = new Set(declared.map((definition) => definition.name));

  const declaredRows = declared.map((definition) =>
    parameterRow(definition.name, definition.label?.trim() || definition.name, values[definition.name], definition.unit?.trim() || null),
  );
  const undeclaredRows = Object.keys(values)
    .filter((name) => !declaredNames.has(name))
    .map((name) => parameterRow(name, name, values[name], null));
  return [...declaredRows, ...undeclaredRows];
}

/** The figure and unit columns' widths: the longest entry of each, the figure capped. */
export function executionParameterColumns(rows: readonly Pick<ExecutionParameterRow, "text" | "unit">[]): ExecutionParameterColumns {
  return {
    value: Math.min(MAX_FIGURE_CHARACTERS, Math.max(1, ...rows.map((row) => row.text.length))),
    unit: Math.max(0, ...rows.map((row) => row.unit?.length ?? 0)),
  };
}

/**
 * The prompts a page answers. A prompt has one home, the node that asked it:
 * an execution's page answers its own prompts, and the cycle's page — `null`
 * — the prompts no execution owns.
 */
export function executionPagePrompts<T extends Pick<PromptRequest, "scope">>(prompts: readonly T[], executionId: string | null): T[] {
  return prompts.filter((prompt) => (prompt.scope?.test_execution_id ?? null) === executionId);
}

/* === Private Functions === */

/**
 * One row, printed the same way wherever it appears. A duration — declared in
 * a time unit, or named `_s`, `_ms` or `duration` — goes through the one
 * duration formatter with the precision its value needs, so 0.05 s reads
 * "50 ms" rather than "0s"; a list of durations shares one unit when its
 * elements agree on one.
 */
function parameterRow(key: string, label: string, value: unknown, declaredUnit: string | null): ExecutionParameterRow {
  const scale = durationScale(key, declaredUnit);
  const duration = scale === null ? null : durationFigure(value, scale);
  if (duration) return { key, label, value, ...duration };
  return { key, label, value, text: formatValue(value, null), unit: declaredUnit };
}

/** Seconds per declared or implied time unit, or `null` for a value that is no duration. */
function durationScale(key: string, declaredUnit: string | null): number | null {
  if (declaredUnit !== null) return TIME_UNITS[declaredUnit] ?? null;
  const name = key.toLowerCase();
  if (name.endsWith("_ms")) return TIME_UNITS.ms;
  if (name.endsWith("_s") || name.includes("duration")) return TIME_UNITS.s;
  return null;
}

function durationFigure(value: unknown, scale: number): Pick<ExecutionParameterRow, "text" | "unit"> | null {
  const values = Array.isArray(value) ? value : [value];
  if (!values.length || !values.every((item): item is number => typeof item === "number" && Number.isFinite(item))) return null;
  const parts = values.map((item) => durationParts(item * scale));
  const unit = parts[0].unit;
  return parts.every((part) => part.unit === unit)
    ? { text: parts.map((part) => part.figure).join(", "), unit }
    : { text: parts.map((part) => `${part.figure} ${part.unit}`).join(", "), unit: null };
}
