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
import { create, fromBinary, toBinary } from "@bufbuild/protobuf";
import { SuiteSchema } from "@host/dokime/v1/catalog_pb.js";
import { SuiteIdentitySchema } from "@host/dokime/v1/client_types_pb.js";
import { ValueType } from "@host/dokime/v1/values_pb.js";

// Local Imports
import { type EditorParameterRow, editorProblems, editorRow } from "@/components/parameter-editor/model";
import { titleCaseIdentifier } from "@/components/start-cycle-form-model";
import { artifactLabel } from "@/lib/dashboard";
import type { FileDefinition, ParameterDefinition } from "@/lib/types";
import type { CycleRow } from "@/features/cycles-home/cycles-model";
import { ActiveCycleInline, ActiveCyclePickerList, ActiveCycleSummary } from "@/features/library/ActiveCycle";
import { openCycles, resolveActiveCycle } from "@/features/library/active-cycle-model";
import { caseAppend, NO_ACTIVE_CYCLE, NO_CHOSEN_CYCLE, summaryNeeded } from "@/features/library/template-dialog-model";
import type { TestCycleSelection } from "@/lib/types";
import type { SuiteTemplateEntry } from "@/lib/types";
import { TemplateProcedure } from "@/features/library/TemplateDialog";
import { mapSuite as mapHostSuite } from "@/lib/host/catalog";

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

test("Procedure Preview Uses Registered Step Order Without Mutating the Catalog", () => {
  const template = {
    steps: [
      { step_id: "second", method_name: "second", order: 2 },
      { step_id: "first", method_name: "first", order: 1 },
    ],
  } as SuiteTemplateEntry;
  const markup = renderToStaticMarkup(createElement(TemplateProcedure, { template }));
  assert.deepEqual(
    [...markup.matchAll(/data-step-id="([^"]+)"/g)].map((step) => step[1]),
    ["first", "second"],
  );
  assert.deepEqual(
    template.steps.map((step) => step.step_id),
    ["second", "first"],
  );
  assert.equal(renderToStaticMarkup(createElement(TemplateProcedure, { template: { ...template, steps: [] } })), "");
});

test("Template Problems Keep Each Field and Its Validation Cause", () => {
  const definition = {
    parameter_id: "voltage",
    name: "voltage",
    label: "Input Voltage",
    value_type: "float",
    required: true,
    metadata: {},
    values: [230],
    choices: [],
  } as ParameterDefinition;
  const invalid = editorRow(definition, "invalid-voltage");
  assert.equal(invalid.parsed.ok, false);
  const files = [
    { name: "input", label: "Input File", required: true },
    { name: "capture", label: "Capture", required: false },
    { name: "optional", required: false },
  ] as FileDefinition[];
  const fileRows = [{ name: "capture", path: "capture.bin", error: "fixture-upload-refused", uploading: false }];
  const problems = editorProblems([invalid], fileRows, files);
  assert.deepEqual(
    problems.map((problem) => problem.label),
    ["Input Voltage", "Input File", "Capture"],
  );
  assert.equal(problems[0].message, invalid.parsed.ok ? undefined : invalid.parsed.error);
  assert.equal(problems[2].message, fileRows[0].error);
  const corrected = editorRow(definition, "230");
  const completeRows = [
    { name: "input", path: "input.bin", error: null, uploading: false },
    { ...fileRows[0], error: null },
  ];
  assert.deepEqual(editorProblems([corrected], completeRows, files), []);
});

test("Summary Skip", () => {
  // One Case at Its Registered Defaults Has Nothing to Review
  assert.equal(summaryNeeded(preview({ current: 16 }), ROWS), false);

  // An Edited Value Keeps the Summary
  assert.equal(summaryNeeded(preview({ current: 32 }), ROWS), true);

  // An Expansion into Several Cases Keeps the Summary
  assert.equal(summaryNeeded(preview({ current: 16 }, { current: 16 }), ROWS), true);
});

test("Append Requires Cycle", () => {
  const selections: TestCycleSelection[] = [{ parameters: { current: [16] }, suite_id: "suite.charger", template_id: "charge" }];

  // Without an Active Cycle Nothing Is Added
  assert.equal(caseAppend(undefined, selections), null);
  assert.equal(caseAppend("  ", selections), null);

  // With One, the Cases Join That Cycle and No Other
  assert.deepEqual(caseAppend("cycle-1", selections), { cycleId: "cycle-1", selections });
});

test("Active Cycle", () => {
  const cycles = openCycles([
    row("0f3a91c2-aaaa-bbbb-cccc-000000000001", "Bench Cycle", "Active", "Active · Running"),
    row("77d1e0b4-aaaa-bbbb-cccc-000000000002", "Sealed Cycle", "Sealed", "Sealed"),
    row("c40e8a57-aaaa-bbbb-cccc-000000000003", "Night Cycle", "Active"),
  ]);

  // Only Open Cycles Can Be Active, Each with a Short Identifier
  assert.deepEqual(
    cycles.map((cycle) => [cycle.label, cycle.shortId]),
    [
      ["Bench Cycle", "0f3a91c2"],
      ["Night Cycle", "c40e8a57"],
    ],
  );

  // A Stored Choice Stands While Its Cycle Is Open, and Falls Back Once It Sealed or Vanished
  assert.equal(resolveActiveCycle("c40e8a57-aaaa-bbbb-cccc-000000000003", cycles)?.label, "Night Cycle");
  assert.equal(resolveActiveCycle("77d1e0b4-aaaa-bbbb-cccc-000000000002", cycles), null);
  assert.equal(resolveActiveCycle("missing", cycles), null);
  assert.equal(resolveActiveCycle(null, cycles), null);

  // The Shell Element States Name, Short Identifier and State
  const element = text(renderToStaticMarkup(createElement(ActiveCycleSummary, { cycle: cycles[0] })));
  assert.match(element, /^Bench Cycle 0f3a91c2 .*Active · Running$/);

  // The Picker Lists Every Open Cycle on One Grid, Marks the Active One, and Searches
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

  // The Template Dialog Names the Active Cycle with Change
  const inline = (cycle: (typeof cycles)[number] | null, hasOpenCycles: boolean) =>
    text(renderToStaticMarkup(createElement(ActiveCycleInline, { cycle, hasOpenCycles, onChange: () => {}, onNewCycle: () => {} })));
  assert.match(inline(cycles[1], true), /^Active Cycle Night Cycle c40e8a57 .*Change$/);

  // Open Cycles Without A Choice Ask For One; Only No Open Cycle Blocks
  assert.equal(inline(null, true), `Active Cycle ${NO_CHOSEN_CYCLE} Choose`);
  assert.equal(inline(null, false), `Active Cycle ${NO_ACTIVE_CYCLE} New Cycle`);
});

test("Generated Identifier and Artifact Labels Respect Word Positions", () => {
  for (const [identifier, label] of [
    ["number_of_rows", "Number of Rows"],
    ["pluginStartupAndStopBudgets", "Plugin Startup and Stop Budgets"],
    ["wait_until_chamber_is_ready", "Wait Until Chamber Is Ready"],
    ["in_the_box_and_off", "In the Box and Off"],
    ["power_on", "Power On"],
    ["log_in", "Log In"],
    ["turn_off_the_heater", "Turn Off the Heater"],
    ["api_for_usb", "API for USB"],
    ["USB_IN_THE_box", "USB in the Box"],
  ]) {
    assert.equal(titleCaseIdentifier(identifier), label);
    assert.equal(artifactLabel(identifier), label);
  }
});

test("Canonical Host Catalog Preserves Typed Parameters and Exclusive Ranges", () => {
  // Decode the Actual Host Wire Shape Before Projecting It for the Library
  const definition = create(SuiteSchema, {
    metadata: { identifier: "fixture.suite", label: "Fixture Suite" },
    templates: [
      {
        metadata: { identifier: "fixture.template", label: "Fixture Template" },
        parameters: [
          {
            metadata: { identifier: "range", label: "Range", attributes: { declared: { kind: { case: "boolean", value: true } } } },
            type: ValueType.INTEGER,
            unit: "K",
            expression: { kind: { case: "range", value: { start: 4n, stop: -2n, step: -2n } } },
            expansion: { kind: { case: "cartesian", value: true } },
            runtimeWritable: true,
          },
          {
            metadata: { identifier: "identifier" },
            type: ValueType.UNSIGNED_INTEGER,
            choices: [{ kind: { case: "unsignedInteger", value: 18446744073709551615n } }],
          },
          {
            metadata: { identifier: "names" },
            type: ValueType.LIST,
            elementType: ValueType.TEXT,
            minItems: 2,
            maxItems: 3,
            expression: {
              kind: {
                case: "literal",
                value: {
                  kind: {
                    case: "list",
                    value: { values: [{ kind: { case: "text", value: "first" } }, { kind: { case: "text", value: "second" } }] },
                  },
                },
              },
            },
          },
        ],
        files: [{ metadata: { identifier: "input" }, defaultValue: "sample.csv", required: true, mediaTypes: ["text/csv"] }],
      },
    ],
  });
  const suite = fromBinary(SuiteSchema, toBinary(SuiteSchema, definition));
  const identity = create(SuiteIdentitySchema, { suiteId: "fixture.suite", source: "fixture.plugin" });
  const projected = mapHostSuite(suite, identity);
  const [range, unsigned, names] = projected.members[0].parameter_definitions;
  assert.deepEqual(range.values, [4, 2, 0]);
  assert.equal(range.unit, "K");
  assert.equal(range.metadata.runtime_writable, true);
  assert.deepEqual(range.expansion, { kind: "cartesian" });
  assert.deepEqual(range.metadata.attributes, { declared: true });
  assert.deepEqual(
    unsigned.choices.map((value) => JSON.parse(JSON.stringify(value))),
    [{ __dokime_native_integer__: true, decimal: "18446744073709551615", unsigned: true }],
  );
  assert.deepEqual(names.values, [["first", "second"]]);
  assert.deepEqual(names.list, { element_type: "string", min_items: 2, max_items: 3 });
  assert.equal(projected.members[0].file_definitions[0].default, "sample.csv");
  assert.throws(() => mapHostSuite(suite, create(SuiteIdentitySchema, { suiteId: "another.suite" })));
});
