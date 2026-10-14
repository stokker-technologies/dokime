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
import { executionsWithSteps, reportOmissions } from "../src/report/report-steps.ts";

/* === Tests === */

test("each report execution gets back its own step and criterion rows in order", () => {
  const step = (execution, id, record_type = "step") => ({
    record_type,
    step_id: id,
    test_execution_id: execution,
  });
  const executions = executionsWithSteps({
    test_executions: [{ test_execution_id: "one" }, { test_execution_id: "two" }],
    test_steps: [
      step("one", "connect"),
      step("one", "connect", "criterion"),
      step("two", "connect"),
      step("one", "measure"),
    ],
  });

  assert.deepEqual(
    executions[0].steps.map((row) => `${row.record_type}:${row.step_id}`),
    ["step:connect", "criterion:connect", "step:measure"],
  );
  assert.deepEqual(
    executions[1].steps.map((row) => row.step_id),
    ["connect"],
  );
});

test("an execution with no recorded steps reads an empty list", () => {
  const [execution] = executionsWithSteps({
    test_executions: [{ test_execution_id: "pending" }],
    test_steps: [],
  });

  assert.deepEqual(execution.steps, []);
});

test("the report lists only what it left out to fit its budget", () => {
  assert.deepEqual(
    reportOmissions([
      "report.no_result_summary: No result summary has been produced for this cycle.",
      "report.omitted: Plot Spectrum omitted: the report did not fit its budget.",
    ]),
    ["Plot Spectrum omitted: the report did not fit its budget."],
  );
});
