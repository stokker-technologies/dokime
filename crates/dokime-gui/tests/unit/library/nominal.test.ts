/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { test } from "vitest";

// Local Imports
import type { EditorParameterRow } from "@/components/parameter-editor/model";
import { caseAppend, summaryNeeded } from "@/features/library/template-dialog-model";
import type { TestCycleSelection } from "@/lib/types";

/* === Re-Exports === */

/* === Constants === */

const ROWS = [
  {
    definition: { label: "Current", name: "current", unit: "A", value_type: "int", values: [16] },
    name: "current",
  },
] as unknown as EditorParameterRow[];

/* === Types === */

type Preview = Parameters<typeof summaryNeeded>[0];

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function preview(...cases: Record<string, unknown>[]): Preview {
  return {
    cases: cases.map((parameters, index) => ({ case_id: `case-${index}`, parameters })),
  } as unknown as Preview;
}

/* === Entry Point === */

test("Summary Skip", () => {
  // One Case At Its Registered Defaults Has Nothing To Review
  assert.equal(summaryNeeded(preview({ current: 16 }), ROWS), false);

  // An Edited Value Keeps The Summary
  assert.equal(summaryNeeded(preview({ current: 32 }), ROWS), true);

  // An Expansion Into Several Cases Keeps The Summary
  assert.equal(summaryNeeded(preview({ current: 16 }, { current: 16 }), ROWS), true);
});

test("Append Requires Cycle", () => {
  const selections: TestCycleSelection[] = [{ parameters: { current: [16] }, suite_id: "suite.charger", template_id: "charge" }];

  // Without An Active Cycle Nothing Is Added
  assert.equal(caseAppend(undefined, selections), null);
  assert.equal(caseAppend("  ", selections), null);

  // With One, The Cases Join That Cycle And No Other
  assert.deepEqual(caseAppend("cycle-1", selections), { cycleId: "cycle-1", selections });
});
