/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Run Focus Selection And Clock Behavior.
 */

import { expect, test } from "vitest";
import type { TestCyclePlan, TestExecution } from "@/lib/types";
import { activeRunFocus, elapsedMinutes, runDestination } from "@/shell/run-focus";

test("execution lifecycle", () => {
  const plan = { test_cycle: { test_cycle_id: "cycle", metadata: {} }, cases: [], templates: [] } as unknown as TestCyclePlan;
  const execution = {
    test_cycle_id: "cycle",
    test_execution_id: "older",
    metadata: {},
    steps: [],
    status: "running",
    started_at: "2026-10-20T00:00:00Z",
  } as unknown as TestExecution;
  const details = [{ plan, executions: [execution, { ...execution, test_execution_id: "newer", started_at: "2026-10-21T00:00:00Z" }] }];
  const focus = activeRunFocus(details);
  expect(focus.map((item) => item.execution.test_execution_id)).toEqual(["newer", "older"]);
  expect(focus[0]?.percent).toBeNull();
  expect(runDestination(focus)).toEqual({ to: "/cycles", hash: "active" });
  const second = details[0]?.executions[1];
  if (!second) throw new Error("Missing execution fixture");
  second.status = "passed";
  const remaining = activeRunFocus(details);
  expect(runDestination(remaining)).toEqual({ to: "/cycles/$cycleId", params: { cycleId: "cycle" }, search: { executionId: "older" } });
  execution.status = "passed";
  expect(runDestination(activeRunFocus(details))).toBeNull();
});
test("known step progress", () => {
  const execution = {
    test_execution_id: "run",
    template_id: "template",
    metadata: {},
    status: "running",
    steps: [
      { record_type: "step", step_id: "one", status: "passed" },
      { record_type: "step", step_id: "two", status: "running" },
    ],
  } as unknown as TestExecution;
  const plan = {
    test_cycle: { metadata: {} },
    cases: [],
    templates: [{ template_id: "template", steps: [{ step_id: "one" }, { step_id: "two" }] }],
  } as unknown as TestCyclePlan;
  expect(activeRunFocus([{ plan, executions: [execution] }])[0]?.percent).toBe(50);
  execution.steps = [];
  expect(activeRunFocus([{ plan, executions: [execution] }])[0]?.percent).toBeNull();
});
test("multi day elapsed", () => {
  const start = "2026-10-20T00:00:00Z";
  const now = Date.parse(start) + (3 * 1440 + 125) * 60000;
  expect(elapsedMinutes(start, now)).toBe(4445);
  expect(elapsedMinutes(start, now + 600000)).toBe(4455);
  expect(elapsedMinutes(start, Date.parse(start) - 1000)).toBe(0);
  expect(elapsedMinutes(undefined, now)).toBeNull();
});
