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
 * document that leaves one out keeps it out. The module imports types only, so
 * a node test can load it directly.
 */

/* === Local Imports === */

import type { DocumentViewCard, JsonObject, ParameterDefinition, PromptRequest } from "@/lib/types";

/* === Types === */

/** The document-backed sections, in page order; Control and Parameters are the page's own. */
export type ExecutionDocumentSection =
  | "steps-criteria"
  | "notes"
  | "media"
  | "artifacts"
  | "report"
  | "records"
  | "traceability";

/** One materialized parameter, as the Parameters grid prints it. */
export type ExecutionParameterRow = {
  /** The parameter's name in the case, which is also its stable key. */
  key: string;
  /** The declared label, or the name when the template declares none. */
  label: string;
  value: unknown;
  /** The declared unit; absent when the template declares none. */
  unit: string | null;
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
export function executionDocumentSections(
  cards: readonly Pick<DocumentViewCard, "kind">[],
): ExecutionDocumentSection[] {
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

  const declaredRows = declared.map((definition) => ({
    key: definition.name,
    label: definition.label?.trim() || definition.name,
    value: values[definition.name],
    unit: definition.unit?.trim() || null,
  }));
  const undeclaredRows = Object.keys(values)
    .filter((name) => !declaredNames.has(name))
    .map((name) => ({ key: name, label: name, value: values[name], unit: null }));
  return [...declaredRows, ...undeclaredRows];
}

/** The prompts the execution page answers: its own, and the cycle's that no execution owns. */
export function executionPagePrompts<T extends Pick<PromptRequest, "scope">>(
  prompts: readonly T[],
  executionId: string | null,
): T[] {
  return prompts.filter((prompt) => {
    const owner = prompt.scope?.test_execution_id;
    return !owner || owner === executionId;
  });
}
