/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Verify Declared Numeric Parameter Editing
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { create } from "@bufbuild/protobuf";
import { ParameterSchema, ValueType } from "@host/model";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test } from "vitest";

// Local Imports
import { editorRow } from "@/components/parameter-editor/model";
import { ParameterValueCell } from "@/components/parameter-editor/parameter-cells";
import { registeredParameterRaw } from "@/components/start-cycle-form-model";
import { mapParameter } from "@dokime/document/lib/host/suites";
import { jsonToNativeValue } from "@dokime/plots/lib/host/values";
import { parseParameterInput } from "@/lib/parameter-parser";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test.each([
  [ValueType.UNSIGNED_INTEGER, { case: "unsignedInteger" as const, value: 4096n }],
  [ValueType.UNSIGNED_INTEGER, { case: "unsignedInteger" as const, value: 18446744073709551615n }],
  [ValueType.INTEGER, { case: "integer" as const, value: -9223372036854775808n }],
  [ValueType.INTEGER, { case: "integer" as const, value: 9223372036854775807n }],
  [ValueType.NUMBER, { case: "number" as const, value: 4096 }],
  [ValueType.NUMBER, { case: "number" as const, value: 0.001 }],
] as const)("Numeric Default Retains Its Declared Wire Type: Case %# (%s)", (type, kind) => {
  const definition = mapParameter(
    create(ParameterSchema, {
      metadata: { identifier: "sample" },
      type,
      unit: "s",
      expression: { kind: { case: "literal", value: { kind } } },
    }),
  );
  const row = editorRow(definition, registeredParameterRaw(definition));
  assert.equal(row.kind, "number");
  assert.ok(row.parsed.ok);
  assert.deepEqual(row.parsed.values.map(jsonToNativeValue), [{ kind }]);
  const markup = renderToStaticMarkup(createElement(ParameterValueCell, { row, onChange: () => {} }));
  assert.match(markup, /type="number"/);
  assert.match(markup, /text-right/);
  assert.match(markup, /aria-hidden="true"[^>]*>s</);
});

test.each([
  [ValueType.UNSIGNED_INTEGER, "-1"],
  [ValueType.UNSIGNED_INTEGER, "18446744073709551616"],
  [ValueType.UNSIGNED_INTEGER, "1.5"],
  [ValueType.INTEGER, "9223372036854775808"],
  [ValueType.INTEGER, "-9223372036854775809"],
  [ValueType.INTEGER, "1.5"],
  [ValueType.NUMBER, "Infinity"],
  [ValueType.NUMBER, "text"],
] as const)("Numeric Input Refuses Invalid Values: %s %s", (type, raw) => {
  const definition = mapParameter(create(ParameterSchema, { metadata: { identifier: "sample" }, type }));
  assert.equal(parseParameterInput(raw, definition).ok, false);
});

test("Unsigned Inputs Enforce Bounds and Preserve Expanded and List Element Types", () => {
  const definition = mapParameter(
    create(ParameterSchema, {
      metadata: { identifier: "iterations" },
      type: ValueType.UNSIGNED_INTEGER,
      min: 1,
      max: 65536,
      expansion: { kind: { case: "cartesian", value: true } },
    }),
  );
  for (const raw of ["0", "65537", "-1..2", "linspace(1, 2, 3)"]) {
    assert.equal(parseParameterInput(raw, definition).ok, false);
  }
  const parsed = parseParameterInput("1..3", definition);
  assert.ok(parsed.ok);
  assert.deepEqual(
    parsed.values.map(jsonToNativeValue),
    [1n, 2n, 3n].map((value) => ({ kind: { case: "unsignedInteger", value } })),
  );
  const list = mapParameter(
    create(ParameterSchema, {
      metadata: { identifier: "counts" },
      type: ValueType.LIST,
      elementType: ValueType.UNSIGNED_INTEGER,
    }),
  );
  const items = parseParameterInput('["18446744073709551615", 1]', list);
  assert.ok(items.ok);
  assert.deepEqual(items.values.map(jsonToNativeValue), [
    {
      kind: {
        case: "list",
        value: {
          values: [{ kind: { case: "unsignedInteger", value: 18446744073709551615n } }, { kind: { case: "unsignedInteger", value: 1n } }],
        },
      },
    },
  ]);
  assert.equal(parseParameterInput("[-1]", list).ok, false);
});
