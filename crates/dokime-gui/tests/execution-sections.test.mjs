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
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

/* === Constants === */

const gui = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const {
  executionDocumentSections,
  executionPagePrompts,
  executionParameterRows,
  executionSectionOf,
} = await import(
  pathToFileURL(resolve(gui, "src/features/cycle/execution-sections-model.ts")).href
);

/* === Tests === */

test("built-in card kinds own their sections and every other kind is Report", () => {
  assert.equal(executionSectionOf("steps"), "steps-criteria");
  assert.equal(executionSectionOf("criteria"), "steps-criteria");
  assert.equal(executionSectionOf("notes"), "notes");
  assert.equal(executionSectionOf("traceability"), "traceability");
  for (const kind of ["plots", "histograms", "box_summaries", "overview", "table", "health"]) {
    assert.equal(executionSectionOf(kind), "report", kind);
  }
});

test("the sections follow page order whatever order the document authored", () => {
  const sections = executionDocumentSections([
    { kind: "traceability" },
    { kind: "plots" },
    { kind: "records" },
    { kind: "criteria" },
    { kind: "media" },
    { kind: "notes" },
    { kind: "artifacts" },
    { kind: "steps" },
  ]);

  assert.deepEqual(sections, [
    "steps-criteria",
    "notes",
    "media",
    "artifacts",
    "report",
    "records",
    "traceability",
  ]);
});

test("a section the document leaves out stays out", () => {
  assert.deepEqual(executionDocumentSections([{ kind: "steps" }, { kind: "records" }]), [
    "steps-criteria",
    "records",
  ]);
});

test("parameters read in declaration order with their declared label and unit", () => {
  const rows = executionParameterRows({ extra: true, current: 16, voltage: 230 }, [
    { name: "voltage", label: "Supply Voltage", unit: "V" },
    { name: "current", label: null, unit: " " },
    { name: "absent", label: "Absent", unit: "s" },
  ]);

  assert.deepEqual(rows, [
    { key: "voltage", label: "Supply Voltage", value: 230, unit: "V" },
    { key: "current", label: "current", value: 16, unit: null },
    { key: "extra", label: "extra", value: true, unit: null },
  ]);
});

test("a case without parameters has no rows", () => {
  assert.deepEqual(executionParameterRows(null, []), []);
});

test("the execution page answers its own prompts and the cycle's, not another execution's", () => {
  const prompts = [
    { correlation_id: "own", scope: { test_execution_id: "e1" } },
    { correlation_id: "other", scope: { test_execution_id: "e2" } },
    { correlation_id: "cycle", scope: { test_execution_id: null } },
    { correlation_id: "unscoped" },
  ];

  assert.deepEqual(
    executionPagePrompts(prompts, "e1").map((prompt) => prompt.correlation_id),
    ["own", "cycle", "unscoped"],
  );
});
