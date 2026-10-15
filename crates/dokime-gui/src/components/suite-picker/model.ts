/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Suite selection model. */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

import {
  createPlanSelection,
  type TestCyclePlanSelectionDraft,
  type TestcaseFormDraft,
} from "@/components/start-cycle-form-model";

import { reviveNativeValues } from "@/lib/api/native-values";

import type { SuiteTemplateEntry, TestCycleSelection, TestSuiteEntry } from "@/lib/types";

import { isRecord } from "@/lib/utils";

/* === Re-Exports === */

/* === Constants === */

export const SUITE_DRAFT_STORAGE_KEY = "dokime-test-hub.suite-drafts.v1";

/* === Types === */

/* SuitePicker model: draft-store, search/filter, sanitize, guards */

export type Selection = {
  suiteId: string;
  suiteName: string;
  template: SuiteTemplateEntry;
};

export type SuiteDraftStore = {
  schema_version: 1;
  formDrafts: Record<string, TestcaseFormDraft>;
  planDrafts: Record<string, TestCyclePlanSelectionDraft>;
  planSelections: TestCyclePlanSelectionDraft[];
};

export type TestSuiteRow = {
  members: SuiteTemplateEntry[];
  suite: TestSuiteEntry;
};

/** Additional searchable text for a suite, such as its registering plugin's label. */
export type SuiteSearchExtra = (suite: TestSuiteEntry) => unknown[];

export type DiagramStep = {
  description?: string | null;
  key: string;
  label: string;
  order: number;
};

/* === Classes === */

/* === Public Functions === */

export function selectionDraftKey(suiteId: string, templateId: string): string {
  return `${suiteId}::${templateId}`;
}

export function templateSelectionId(template: SuiteTemplateEntry): string {
  const identifier = template.metadata.identifier;
  return typeof identifier === "string" && identifier.length > 0
    ? identifier
    : template.template_id;
}

export function matchesTemplateSelection(
  template: SuiteTemplateEntry,
  selectionId: string,
): boolean {
  return template.template_id === selectionId || templateSelectionId(template) === selectionId;
}

export function apiSelection(item: TestCyclePlanSelectionDraft): TestCycleSelection {
  return {
    suite_id: item.suite_id,
    template_id: item.template_id,
    parameters: item.parameters,
    files: item.files,
    label: item.label,
    suite_label: item.suite_label,
    template_label: item.template_label,
  };
}

export function suiteDiagramSteps(template: SuiteTemplateEntry): DiagramStep[] {
  return template.steps.map((step, index) => ({
    description: step.description,
    key: step.step_id,
    label: step.label ?? step.method_name,
    // A running 1..n counter. The projection's `order` is a zero-based
    // position, so reading it here printed 1, 1, 2 for the first three steps.
    order: index + 1,
  }));
}

export function filterTestSuites(
  suites: TestSuiteEntry[],
  query: string,
  extra?: SuiteSearchExtra,
): TestSuiteRow[] {
  const tokens = searchTokens(query);
  if (!tokens.length) {
    return suites.map((suite) => ({ suite, members: suite.members }));
  }
  return suites
    .map((suite) => {
      const suiteMatches = matchesTokens(suiteSearchText(suite, extra), tokens);
      const members = suiteMatches
        ? suite.members
        : suite.members.filter((template) =>
            matchesTokens(templateSearchText(template, suite, extra), tokens),
          );
      return { suite, members };
    })
    .filter((row) => row.members.length > 0);
}

export function searchableText(parts: unknown[]): string {
  return parts.flat().filter(Boolean).join(" ").toLowerCase();
}

export function suitePathLabel(path: string[]): string {
  return path.join(" / ");
}

export function formatTagLabel(tag: string): string {
  return tag.toLowerCase() === "plt400" ? "PLT400" : tag;
}

export function readSuiteDraftStore(suites: TestSuiteEntry[]): SuiteDraftStore {
  const empty: SuiteDraftStore = {
    schema_version: 1,
    formDrafts: {},
    planDrafts: {},
    planSelections: [],
  };
  if (typeof window === "undefined") {
    return empty;
  }
  try {
    const parsed = reviveNativeValues(
      JSON.parse(window.localStorage.getItem(SUITE_DRAFT_STORAGE_KEY) ?? "null"),
    );
    if (!isSuiteDraftStore(parsed)) {
      return empty;
    }
    const validKeys = validSelectionKeys(suites);
    const formDrafts = Object.fromEntries(
      Object.entries(parsed.formDrafts)
        .filter(([key]) => validKeys.has(key))
        .map(([key, draft]) => [key, sanitizeFormDraft(suites, key, draft)]),
    );
    const planDrafts = Object.fromEntries(
      Object.entries(parsed.planDrafts)
        .filter(([key, draft]) => validKeys.has(key) && isSelectionDraft(draft))
        .map(([key, draft]) => [
          key,
          sanitizePlanSelection(suites, draft as TestCyclePlanSelectionDraft),
        ]),
    );
    const planSelections = parsed.planSelections
      .filter(
        (draft) =>
          isSelectionDraft(draft) &&
          validKeys.has(selectionDraftKey(draft.suite_id, draft.template_id)),
      )
      .map((draft) => sanitizePlanSelection(suites, draft));
    return {
      schema_version: 1,
      formDrafts,
      planDrafts,
      planSelections,
    };
  } catch {
    return empty;
  }
}

export function sanitizeFormDraft(
  suites: TestSuiteEntry[],
  key: string,
  value: unknown,
): TestcaseFormDraft {
  if (!isRecord(value)) {
    return { files: {}, parameters: {} };
  }
  const template = templateForSelectionKey(suites, key);
  if (!template) {
    return { files: {}, parameters: {} };
  }
  const parameters = isRecord(value.parameters)
    ? filterRecord(
        value.parameters,
        template.parameter_definitions.map((definition) => definition.name),
      )
    : {};
  const files = isRecord(value.files)
    ? filterRecord(
        value.files,
        template.file_definitions.map((definition) => definition.name),
      )
    : {};
  return {
    parameters: stringifyRecord(parameters),
    files: stringifyRecord(files),
  };
}

export function sanitizePlanSelection(
  suites: TestSuiteEntry[],
  draft: TestCyclePlanSelectionDraft,
): TestCyclePlanSelectionDraft {
  const match = templateForSelection(suites, draft.suite_id, draft.template_id);
  if (!match) {
    return draft;
  }
  const parameters = filterRecord(
    draft.parameters,
    match.template.parameter_definitions.map((definition) => definition.name),
  );
  const files = filterRecord(
    draft.files ?? {},
    match.template.file_definitions.map((definition) => definition.name),
  );
  return createPlanSelection({
    suiteId: draft.suite_id,
    suiteName: match.suite.name,
    template: match.template,
    values: normalizeParameterArrays(parameters),
    files: normalizeFiles(files),
  });
}

export function templateForSelectionKey(
  suites: TestSuiteEntry[],
  key: string,
): SuiteTemplateEntry | null {
  for (const suite of suites) {
    for (const template of suite.members) {
      if (
        key === selectionDraftKey(suite.suite_id, template.template_id) ||
        key === selectionDraftKey(suite.suite_id, templateSelectionId(template))
      ) {
        return template;
      }
    }
  }
  return null;
}

export function templateForSelection(
  suites: TestSuiteEntry[],
  suiteId: string,
  templateId: string,
): { suite: TestSuiteEntry; template: SuiteTemplateEntry } | null {
  const suite = suites.find((candidate) => candidate.suite_id === suiteId);
  const template = suite?.members.find((candidate) =>
    matchesTemplateSelection(candidate, templateId),
  );
  return suite && template ? { suite, template } : null;
}

export function normalizeParameterArrays(
  record: Record<string, unknown>,
): Record<string, unknown[]> {
  return Object.fromEntries(
    Object.entries(record).map(([key, value]) => [key, Array.isArray(value) ? value : [value]]),
  );
}

export function normalizeFiles(record: Record<string, unknown>): Record<string, string | null> {
  return Object.fromEntries(
    Object.entries(record).map(([key, value]) => [
      key,
      typeof value === "string" && value.trim() ? value : null,
    ]),
  );
}

export function writeSuiteDraftStore(store: SuiteDraftStore): void {
  if (typeof window === "undefined") {
    return;
  }
  try {
    if (
      !Object.keys(store.formDrafts).length &&
      !Object.keys(store.planDrafts).length &&
      !store.planSelections.length
    ) {
      window.localStorage.removeItem(SUITE_DRAFT_STORAGE_KEY);
      return;
    }
    window.localStorage.setItem(SUITE_DRAFT_STORAGE_KEY, JSON.stringify(store));
  } catch {
    // Browser storage can be unavailable in restricted contexts.
  }
}

export function clearSuiteDraftStore(): void {
  if (typeof window === "undefined") {
    return;
  }
  try {
    window.localStorage.removeItem(SUITE_DRAFT_STORAGE_KEY);
  } catch {
    // Browser storage can be unavailable in restricted contexts.
  }
}

export function validSelectionKeys(suites: TestSuiteEntry[]): Set<string> {
  return new Set(
    suites.flatMap((suite) =>
      suite.members.flatMap((template) => [
        selectionDraftKey(suite.suite_id, template.template_id),
        selectionDraftKey(suite.suite_id, templateSelectionId(template)),
      ]),
    ),
  );
}

export function isSuiteDraftStore(value: unknown): value is SuiteDraftStore {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return false;
  }
  const record = value as Record<string, unknown>;
  return (
    record.schema_version === 1 &&
    isRecord(record.formDrafts) &&
    isRecord(record.planDrafts) &&
    Array.isArray(record.planSelections)
  );
}

export function isSelectionDraft(value: unknown): value is TestCyclePlanSelectionDraft {
  if (!isRecord(value)) {
    return false;
  }
  return (
    typeof value.suite_id === "string" &&
    typeof value.template_id === "string" &&
    typeof value.key === "string" &&
    typeof value.label === "string" &&
    typeof value.suite_label === "string" &&
    typeof value.template_label === "string" &&
    typeof value.summary === "string" &&
    isRecord(value.parameters) &&
    (value.files === undefined || isRecord(value.files))
  );
}

export function stableJson(value: unknown): string {
  return JSON.stringify(sortJson(value));
}

/* === Private Functions === */

function searchTokens(query: string): string[] {
  return query.trim().toLowerCase().split(/\s+/).filter(Boolean);
}

function matchesTokens(text: string, tokens: string[]): boolean {
  return tokens.every((token) => text.includes(token));
}

function suiteSearchText(suite: TestSuiteEntry, extra: SuiteSearchExtra | undefined): string {
  return searchableText([
    suite.suite_id,
    suite.source,
    extra?.(suite),
    suite.name,
    suite.description,
    suite.path,
    suite.tags,
  ]);
}

function templateSearchText(
  template: SuiteTemplateEntry,
  suite: TestSuiteEntry,
  extra: SuiteSearchExtra | undefined,
): string {
  return searchableText([
    suiteSearchText(suite, extra),
    template.identifier,
    templateSelectionId(template),
    template.name,
    template.description,
    template.path,
    template.tags,
  ]);
}

function filterRecord(record: Record<string, unknown>, keys: string[]): Record<string, unknown> {
  const allowed = new Set(keys);
  return Object.fromEntries(Object.entries(record).filter(([key]) => allowed.has(key)));
}

function stringifyRecord(record: Record<string, unknown>): Record<string, string> {
  return Object.fromEntries(
    Object.entries(record)
      .filter(([, value]) => ["boolean", "number", "string"].includes(typeof value))
      .map(([key, value]) => [key, String(value)]),
  );
}

function sortJson(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(sortJson);
  }
  if (!isRecord(value)) {
    return value;
  }
  return Object.fromEntries(
    Object.entries(value)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, item]) => [key, sortJson(item)]),
  );
}

/* === Entry Point === */
