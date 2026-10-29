/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The parameter editor's row model (concept 11a).
 *
 * A row is the registered default in raw editor form, the raw text the
 * operator currently holds, and the parse of that text. "Edited" is one string
 * comparison against the default, which is also what the overrides store
 * persists, so the reset affordance, the gilt rule and the summary's delta
 * list all read the same fact. */

import {
  type FileRowState,
  nonEmptyString,
  type ParameterChoice,
  parameterChoices,
  registeredParameterRaw,
} from "@/components/start-cycle-form-model";
import type { ParameterOverrides } from "@/lib/desktop/settings";
import { type ParseResult, parseParameterInput } from "@/lib/parameter-parser";
import type { FileDefinition, ParameterDefinition, SuiteTemplateEntry } from "@/lib/types";

/** Above this a choice parameter gets a Select instead of a segmented control. */
export const SEGMENTED_CHOICE_LIMIT = 4;

const BOOLEAN_TYPES = new Set(["bool", "boolean"]);
const NUMERIC_TYPES = new Set(["int", "integer", "float", "number"]);

/**
 * How a value cell is drawn. One kind per parameter, decided once so the cell
 * component stays a switch rather than a chain of conditions.
 */
export type ParameterCellKind = "boolean" | "list" | "number" | "segmented" | "select" | "text" | "wifi";

export type EditorParameterRow = {
  definition: ParameterDefinition;
  /** The registered default in raw editor form. */
  defaultRaw: string;
  kind: ParameterCellKind;
  name: string;
  parsed: ParseResult;
  raw: string;
};

export type EditorGroup = {
  label: string;
  rows: EditorParameterRow[];
};

export function parameterValueType(definition: ParameterDefinition): string {
  return (definition.value_type ?? "str").toLowerCase();
}

export function isExpandedParameter(definition: ParameterDefinition): boolean {
  return (definition.expansion?.kind ?? "single") !== "single";
}

export function parameterCellKind(definition: ParameterDefinition): ParameterCellKind {
  const valueType = parameterValueType(definition);
  if (definition.list || definition.expansion?.kind === "zip") {
    return "list";
  }
  if (definition.metadata.editor === "wifi_channel_selector" && isExpandedParameter(definition)) {
    return "wifi";
  }
  const choices = parameterChoices(definition);
  if (choices.length) {
    // An expanded choice takes several values at once, so it always uses the
    // multi-select segmented control; a Select cannot express that.
    return isExpandedParameter(definition) || choices.length <= SEGMENTED_CHOICE_LIMIT ? "segmented" : "select";
  }
  if (BOOLEAN_TYPES.has(valueType)) {
    return "boolean";
  }
  return NUMERIC_TYPES.has(valueType) ? "number" : "text";
}

export function parameterChoiceList(definition: ParameterDefinition): ParameterChoice[] {
  const choices = parameterChoices(definition);
  if (choices.length) {
    return choices;
  }
  return BOOLEAN_TYPES.has(parameterValueType(definition))
    ? [
        { label: "True", value: "true" },
        { label: "False", value: "false" },
      ]
    : [];
}

/** The eyebrow a parameter belongs under; templates without groups get one. */
export function parameterGroupLabel(definition: ParameterDefinition): string {
  return (
    nonEmptyString(definition.metadata.group_label) ??
    nonEmptyString(definition.metadata.group) ??
    nonEmptyString(definition.metadata.group_id) ??
    "Parameters"
  );
}

export function editorRow(definition: ParameterDefinition, raw: string): EditorParameterRow {
  return {
    definition,
    defaultRaw: registeredParameterRaw(definition),
    kind: parameterCellKind(definition),
    name: definition.name,
    parsed: parseParameterInput(raw, definition),
    raw,
  };
}

/**
 * Seed every row of a template: the persisted override when the operator has
 * one, otherwise the registered default.
 */
export function seedEditorRows(template: SuiteTemplateEntry, overrides: ParameterOverrides[string] | undefined): EditorParameterRow[] {
  return template.parameter_definitions.map((definition) => {
    const override = overrides?.[definition.parameter_id];
    const row = editorRow(definition, override?.raw ?? registeredParameterRaw(definition));
    if (override !== undefined && (override.valueType !== parameterValueType(definition) || override.unit !== (definition.unit ?? null))) {
      row.parsed = {
        ok: false,
        error: "Review this remembered value: its type or unit changed. Edit it or reset to the default.",
      };
    }
    return row;
  });
}

export function groupEditorRows(rows: EditorParameterRow[]): EditorGroup[] {
  const groups: EditorGroup[] = [];
  for (const row of rows) {
    const label = parameterGroupLabel(row.definition);
    const existing = groups.find((group) => group.label === label);
    if (existing) {
      existing.rows.push(row);
    } else {
      groups.push({ label, rows: [row] });
    }
  }
  return groups;
}

/**
 * How wide the unit column inside every numeric field has to be.
 *
 * One measure for the whole step, taken from the longest unit the template
 * declares, so the values share one right edge whether their unit is "V" or
 * "kOhm". Three characters is the floor: a narrower column reads as a typo.
 */
export function unitColumnCharacters(rows: EditorParameterRow[]): number {
  return rows.reduce((widest, row) => Math.max(widest, (row.definition.unit?.trim() ?? "").length), 3);
}

export function isEditedRow(row: EditorParameterRow): boolean {
  return row.raw !== row.defaultRaw;
}

/**
 * Whether the row's value cell takes an expression, so the reference is worth
 * offering beside it.
 *
 * Chips always do. A plain number does not: an expression there resolves to
 * several values and a single-valued parameter refuses them, so the help icon
 * would be an invitation to an error message.
 */
export function acceptsExpressions(row: EditorParameterRow): boolean {
  return row.kind === "list" || (row.kind === "number" && isExpandedParameter(row.definition));
}

/** The problems the operator must clear before Next is offered. */
export function editorProblemCount(rows: EditorParameterRow[], fileRows: FileRowState[], fileDefinitions: FileDefinition[]): number {
  return (
    rows.filter((row) => !row.parsed.ok).length +
    fileDefinitions.filter((definition) => {
      const row = fileRows.find((candidate) => candidate.name === definition.name);
      return Boolean(row?.error) || Boolean(row?.uploading) || (definition.required && !(row?.path.trim() || definition.default?.trim()));
    }).length
  );
}

/**
 * How a row expands, as the declared strategy and one plain sentence.
 *
 * The strategy is the word the template declared — "Cartesian" or "Zip" — and
 * the sentence says what that word does to the values beside it. The badge
 * states; it never invites a click, so it carries no control styling.
 */
export type ExpansionStatement = { badge: string | null; sentence: string };

export function expansionStatement(definition: ParameterDefinition): ExpansionStatement | null {
  if (definition.list && !isExpandedParameter(definition)) {
    return { badge: null, sentence: "One list, one case." };
  }
  switch (definition.expansion?.kind) {
    case "zip":
      return {
        badge: "Zip",
        sentence: "Values are paired by position with the other zipped lists.",
      };
    case "cartesian":
      return {
        badge: "Cartesian",
        sentence: "Each value is combined with the other lists' values.",
      };
    default:
      return definition.list ? { badge: null, sentence: "Each value becomes a case." } : null;
  }
}
