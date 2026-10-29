/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Suite selection model. */

import type { TestCyclePlanSelectionDraft } from "@/components/start-cycle-form-model";

import type { SuiteTemplateEntry, TestCycleSelection, TestSuiteEntry } from "@/lib/types";

/* SuitePicker model: draft-store, search/filter, sanitize, guards */

export type TestSuiteRow = {
  members: SuiteTemplateEntry[];
  suite: TestSuiteEntry;
};

/** Additional searchable text for a suite, such as its registering plugin's label. */
export type SuiteSearchExtra = (suite: TestSuiteEntry) => unknown[];

export function selectionDraftKey(suiteId: string, templateId: string): string {
  return `${suiteId}::${templateId}`;
}

export function templateSelectionId(template: SuiteTemplateEntry): string {
  const identifier = template.metadata.identifier;
  return typeof identifier === "string" && identifier.length > 0 ? identifier : template.template_id;
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

export function filterTestSuites(suites: TestSuiteEntry[], query: string, extra?: SuiteSearchExtra): TestSuiteRow[] {
  const tokens = searchTokens(query);
  if (!tokens.length) {
    return suites.map((suite) => ({ suite, members: suite.members }));
  }
  return suites
    .map((suite) => {
      const suiteMatches = matchesTokens(suiteSearchText(suite, extra), tokens);
      const members = suiteMatches
        ? suite.members
        : suite.members.filter((template) => matchesTokens(templateSearchText(template, suite, extra), tokens));
      return { suite, members };
    })
    .filter((row) => row.members.length > 0);
}

export function searchableText(parts: unknown[]): string {
  return parts.flat().filter(Boolean).join(" ").toLowerCase();
}

export function formatTagLabel(tag: string): string {
  return tag.toLowerCase() === "plt400" ? "PLT400" : tag;
}

function searchTokens(query: string): string[] {
  return query.trim().toLowerCase().split(/\s+/).filter(Boolean);
}

function matchesTokens(text: string, tokens: string[]): boolean {
  return tokens.every((token) => text.includes(token));
}

function suiteSearchText(suite: TestSuiteEntry, extra: SuiteSearchExtra | undefined): string {
  return searchableText([suite.suite_id, suite.source, extra?.(suite), suite.name, suite.description, suite.path, suite.tags]);
}

function templateSearchText(template: SuiteTemplateEntry, suite: TestSuiteEntry, extra: SuiteSearchExtra | undefined): string {
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
