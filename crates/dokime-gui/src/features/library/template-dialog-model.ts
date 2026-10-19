/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Where the materialization dialog's cases go, and what its summary says.
 *
 * Cases join the active cycle and nothing else: without one the dialog refuses
 * to add and offers New Cycle instead. The summary earns its step only when it
 * tells the operator something the parameters step did not: an expansion into
 * several cases, or values that differ from what the template registered. One
 * case at its defaults goes straight to Finish And Add. */

/* === External Imports === */

/* === Local Imports === */

import { type EditorParameterRow, parameterValueType } from "@/components/parameter-editor/model";
import { formatPreview, titleCaseIdentifier } from "@/components/start-cycle-form-model";
import type { ParameterDefinition, PreviewTestCycleResponse, TestCycleSelection } from "@/lib/types";

/* === Types === */

export type CaseAppend = { cycleId: string; selections: TestCycleSelection[] };

/* === Constants === */

export const NO_ACTIVE_CYCLE = "Cases join the active cycle. Open a cycle or create one first.";

/* === Public Functions === */

/** The append the dialog performs: into the active cycle, or `null` without one. */
export function caseAppend(activeCycleId: string | null | undefined, selections: TestCycleSelection[]): CaseAppend | null {
  const cycleId = activeCycleId?.trim();
  return cycleId ? { cycleId, selections } : null;
}

/** Whether the preview has anything to review before the cases are added. */
export function summaryNeeded(preview: Pick<PreviewTestCycleResponse, "cases">, rows: EditorParameterRow[]): boolean {
  if (preview.cases.length !== 1) return true;
  return preview.cases.some((item) => caseDeltas(item.parameters, rows).length > 0);
}

/**
 * The values of one planned case that differ from what the template registered.
 *
 * The comparison is against the definition, not against the operator's edit
 * list: an expanded parameter gives every case a different value, and a case
 * whose value happens to match the default must not claim a change. A
 * parameter the preview names but the template does not define is dropped
 * rather than printed as a raw key.
 */
export function caseDeltas(parameters: Record<string, unknown>, rows: EditorParameterRow[]): string[] {
  const deltas: string[] = [];
  for (const row of rows) {
    if (!(row.name in parameters)) continue;
    const valueType = parameterValueType(row.definition);
    const actual = valueText(parameters[row.name], valueType);
    if (actual === registeredValueText(row.definition, valueType)) continue;
    const unit = row.definition.unit?.trim();
    const label = row.definition.label ?? titleCaseIdentifier(row.name);
    deltas.push(`${label}: ${actual}${unit ? ` ${unit}` : ""}`);
  }
  return deltas;
}

/* === Private Functions === */

function valueText(value: unknown, valueType: string): string {
  return formatPreview(Array.isArray(value) ? value : [value], valueType);
}

/** The registered value, formatted the same way a case's value is. */
function registeredValueText(definition: ParameterDefinition, valueType: string): string {
  const single = (definition.expansion?.kind ?? "single") === "single";
  return single && definition.list ? valueText(definition.values[0], valueType) : valueText(definition.values, valueType);
}
