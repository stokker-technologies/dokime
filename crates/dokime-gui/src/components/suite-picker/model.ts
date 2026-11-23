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

import type { TestCyclePlanSelectionDraft } from "@/components/start-cycle-form-model";

import type { SuiteTemplateEntry, TestCycleSelection } from "@/lib/types";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Public Functions === */

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

export function formatTagLabel(tag: string): string {
  return tag.toLowerCase() === "plt400" ? "PLT400" : tag;
}
