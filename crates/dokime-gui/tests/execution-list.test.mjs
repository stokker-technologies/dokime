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
  followSelection,
  isCycleTerminal,
  isFollowing,
  isUnstartedExecution,
  rerunnableCaseIds,
  rowOffersRunAgain,
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

test("run again is offered on the newest ended attempt and the ended cycle", () => {
  const cases = [{ caseId: "first" }, { caseId: "second" }, { caseId: "third" }];
  const rows = buildCycleExecutionRows(cases, [
    { case_id: "first", test_execution_id: "one", attempt_index: 0, status: "interrupted" },
    { case_id: "second", test_execution_id: "two", attempt_index: 1, status: "running" },
    { case_id: "first", test_execution_id: "three", attempt_index: 2, status: "passed" },
  ]);

  assert.deepEqual(
    rows.map((row) => [row.key, rowOffersRunAgain(row)]),
    [
      ["one", false],
      ["two", false],
      ["three", true],
      ["case:third", false],
    ],
  );
  assert.deepEqual(
    ["completed", "failed", "stopped", "interrupted", "running", "stopping", "planned"].map(
      isCycleTerminal,
    ),
    [true, true, true, true, false, false, false],
  );
  assert.deepEqual(
    rerunnableCaseIds([
      { caseId: "allowed", actions: { schedule_rerun: true } },
      { caseId: "refused", actions: { schedule_rerun: false } },
      { caseId: "unstated", actions: null },
    ]),
    ["allowed", "unstated"],
  );
});

test("an explicit selection beats Follow Active until the operator returns to it", () => {
  const follow = (selection, extra = {}) =>
    followSelection({
      active: "ten",
      previousActive: "ten",
      selectedEnded: false,
      selection,
      ...extra,
    });

  assert.equal(follow({}), "ten", "landing follows the running execution");
  assert.equal(follow({ executionId: "twelve" }), null, "a click on a queued row stays there");
  assert.equal(follow({ view: "cycle" }), null, "the cycle is a selection of its own");
  assert.equal(follow({ caseId: "unrun" }), null);
  assert.equal(follow({ executionId: "ten" }), null, "already on the active one");
  assert.equal(
    follow({ executionId: "ten" }, { active: "eleven" }),
    "eleven",
    "a following selection moves on with the runtime",
  );
  assert.equal(
    follow({ executionId: "three" }, { active: "eleven", selectedEnded: true }),
    "eleven",
    "the selected execution ending resumes following",
  );
  assert.equal(
    followSelection({ active: null, previousActive: null, selectedEnded: false, selection: {} }),
    null,
  );

  assert.equal(isFollowing({}, "ten"), true);
  assert.equal(isFollowing({ executionId: "ten" }, "ten"), true);
  assert.equal(isFollowing({ executionId: "twelve" }, "ten"), false);
  assert.equal(isFollowing({ view: "cycle" }, "ten"), false);
  assert.deepEqual(["planned", "created", "running", "passed"].map(isUnstartedExecution), [
    true,
    true,
    false,
    false,
  ]);
});
