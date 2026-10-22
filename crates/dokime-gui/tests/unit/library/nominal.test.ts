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
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test } from "vitest";

// Local Imports
import type { EditorParameterRow } from "@/components/parameter-editor/model";
import type { CycleRow } from "@/features/cycles-home/cycles-model";
import { ActiveCycleInline, ActiveCyclePickerList, ActiveCycleSummary } from "@/features/library/ActiveCycle";
import { openCycles, resolveActiveCycle } from "@/features/library/active-cycle-model";
import { caseAppend, NO_ACTIVE_CYCLE, NO_CHOSEN_CYCLE, summaryNeeded } from "@/features/library/template-dialog-model";
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

function row(cycleId: string, label: string, phase: CycleRow["phase"], statusLabel = "Active · Idle"): CycleRow {
  return {
    attentionCount: 0,
    caseCount: 3,
    createdAtIso: "2026-10-20T20:03:00.000Z",
    cycleId,
    executionCount: 3,
    finishedCount: 0,
    label,
    phase,
    status: "Ready",
    statusLabel,
    statusTone: "neutral",
    updatedAtIso: null,
    updatedAtMs: null,
  };
}

function text(markup: string): string {
  return markup
    .replace(/<[^>]+>/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

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

test("Active Cycle", () => {
  const cycles = openCycles([
    row("0f3a91c2-aaaa-bbbb-cccc-000000000001", "Bench Cycle", "Active", "Active · Running"),
    row("77d1e0b4-aaaa-bbbb-cccc-000000000002", "Sealed Cycle", "Sealed", "Sealed"),
    row("c40e8a57-aaaa-bbbb-cccc-000000000003", "Night Cycle", "Active"),
  ]);

  // Only Open Cycles Can Be Active, Each With A Short Identifier
  assert.deepEqual(
    cycles.map((cycle) => [cycle.label, cycle.shortId]),
    [
      ["Bench Cycle", "0f3a91c2"],
      ["Night Cycle", "c40e8a57"],
    ],
  );

  // A Stored Choice Stands While Its Cycle Is Open, And Falls Back Once It Sealed Or Vanished
  assert.equal(resolveActiveCycle("c40e8a57-aaaa-bbbb-cccc-000000000003", cycles)?.label, "Night Cycle");
  assert.equal(resolveActiveCycle("77d1e0b4-aaaa-bbbb-cccc-000000000002", cycles), null);
  assert.equal(resolveActiveCycle("missing", cycles), null);
  assert.equal(resolveActiveCycle(null, cycles), null);

  // The Shell Element States Name, Short Identifier And State
  const element = text(renderToStaticMarkup(createElement(ActiveCycleSummary, { cycle: cycles[0] })));
  assert.match(element, /^Bench Cycle 0f3a91c2 .*Active · Running$/);

  // The Picker Lists Every Open Cycle On One Grid, Marks The Active One, And Searches
  const list = (search: string) =>
    renderToStaticMarkup(
      createElement(ActiveCyclePickerList, {
        activeId: cycles[1].cycleId,
        cycles,
        loading: false,
        onChoose: () => {},
        onNewCycle: () => {},
        onSearch: () => {},
        search,
      }),
    );
  const all = list("");
  assert.match(all, /grid-cols-subgrid/);
  for (const heading of ["Cycle", "State", "Created", "Identifier", "Members"]) assert.ok(text(all).includes(heading), heading);
  assert.ok(text(all).includes("Bench Cycle") && text(all).includes("Night Cycle") && !text(all).includes("Sealed Cycle"));
  assert.ok(text(all).includes("3 Cases"));
  assert.match(all, /aria-pressed="true"[^>]*>(?:(?!<\/button>).)*Night Cycle/);
  assert.ok(text(all).includes("New Cycle"));
  assert.ok(!text(list("c40e")).includes("Bench Cycle"), "search matches the identifier");
  assert.ok(text(list("nothing")).includes("No Open Cycle Matches This Search"));

  // The Template Dialog Names The Active Cycle With Change
  const inline = (cycle: (typeof cycles)[number] | null, hasOpenCycles: boolean) =>
    text(renderToStaticMarkup(createElement(ActiveCycleInline, { cycle, hasOpenCycles, onChange: () => {}, onNewCycle: () => {} })));
  assert.match(inline(cycles[1], true), /^Active Cycle Night Cycle c40e8a57 .*Change$/);

  // Open Cycles Without A Choice Ask For One; Only No Open Cycle Blocks
  assert.equal(inline(null, true), `Active Cycle ${NO_CHOSEN_CYCLE} Choose`);
  assert.equal(inline(null, false), `Active Cycle ${NO_ACTIVE_CYCLE} New Cycle`);
});
