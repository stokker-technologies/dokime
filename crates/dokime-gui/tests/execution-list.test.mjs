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
import { test } from "node:test";
import {
  buildCycleExecutionRows,
  cycleRowSelection,
  selectedCycleRow,
} from "../src/features/cycle/cycle-workspace-model.ts";

/* === Tests === */

test("a rerun appends a distinct row while the old selection still names its evidence", () => {
  const cases = [{ caseId: "first" }, { caseId: "second" }];
  const first = { case_id: "first", test_execution_id: "one", attempt_index: 0 };
  const second = { case_id: "second", test_execution_id: "two", attempt_index: 1 };
  const rerun = { case_id: "first", test_execution_id: "three", attempt_index: 2 };
  const rows = buildCycleExecutionRows(cases, [rerun, second, first]);

  assert.deepEqual(
    rows.map((row) => row.key),
    ["one", "two", "three"],
  );
  assert.equal(rows[2].rerunOf, 1);
  assert.equal(rows[0].newest, false);
  assert.equal(rows[2].newest, true);
  assert.equal(selectedCycleRow(rows, { executionId: "one" }).execution, first);
  assert.deepEqual(cycleRowSelection(rows[2]), { executionId: "three" });
  assert.equal(selectedCycleRow(rows, { caseId: "first" }), null);
});

test("a case without an execution has a placeholder and stale executions select no row", () => {
  const cases = [{ caseId: "first" }, { caseId: "second" }];
  const execution = { case_id: "first", test_execution_id: "one", attempt_index: 0 };
  const rows = buildCycleExecutionRows(cases, [execution]);

  assert.equal(rows.length, 2);
  assert.deepEqual(cycleRowSelection(rows[1]), { caseId: "second" });
  assert.equal(selectedCycleRow(rows, { caseId: "second" }), rows[1]);
  assert.equal(selectedCycleRow(rows, { executionId: "removed" }), null);
  assert.equal(selectedCycleRow(rows, {}), null);
});
