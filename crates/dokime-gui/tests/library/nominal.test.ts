/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import assert from "node:assert/strict";
import { test } from "vitest";
import type { EditorParameterRow } from "@/components/parameter-editor/model";
import { summaryNeeded } from "@/features/library/template-dialog-model";

/* === Fixture === */

type Preview = Parameters<typeof summaryNeeded>[0];

const ROWS = [
  {
    definition: { label: "Current", name: "current", unit: "A", value_type: "int", values: [16] },
    name: "current",
  },
] as unknown as EditorParameterRow[];

function preview(...cases: Record<string, unknown>[]): Preview {
  return {
    cases: cases.map((parameters, index) => ({ case_id: `case-${index}`, parameters })),
  } as unknown as Preview;
}

/* === Tests === */

test("Summary Skip", () => {
  // One Case At Its Registered Defaults Has Nothing To Review
  assert.equal(summaryNeeded(preview({ current: 16 }), ROWS), false);

  // An Edited Value Keeps The Summary
  assert.equal(summaryNeeded(preview({ current: 32 }), ROWS), true);

  // An Expansion Into Several Cases Keeps The Summary
  assert.equal(summaryNeeded(preview({ current: 16 }, { current: 16 }), ROWS), true);
});
