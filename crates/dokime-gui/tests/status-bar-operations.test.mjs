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
  operationPercent,
  operationPhaseLabel,
  operationsCountLabel,
  operationsFromFrame,
  summarizeOperations,
} = await import(pathToFileURL(resolve(gui, "src/shell/status-bar-operations.ts")).href);

/* === Fixture === */

function frame(id, status, { fraction, phase = "Materializing Evidence", seconds = 100 } = {}) {
  return {
    fraction,
    label: `Operation ${id}`,
    operationId: id,
    phase,
    startedAt: { nanos: 0, seconds: BigInt(seconds) },
    status: { HarmosVariant: { case: status } },
  };
}

/* === Tests === */

test("an idle backend shows nothing", () => {
  assert.deepEqual(summarizeOperations(operationsFromFrame([])), { kind: "idle" });
  assert.deepEqual(summarizeOperations(operationsFromFrame(undefined)), { kind: "idle" });
});

test("one running operation shows its label, phase and determinate progress", () => {
  const summary = summarizeOperations(
    operationsFromFrame([frame("a", "Running", { fraction: 0.426 })]),
  );

  assert.equal(summary.kind, "single");
  assert.equal(summary.operation.label, "Operation a");
  assert.equal(operationPhaseLabel(summary.operation), "Materializing Evidence");
  assert.equal(operationPercent(summary.operation), 43);
  assert.equal(summary.operation.startedAtMs, 100_000);
});

test("an operation without a fraction is indeterminate", () => {
  const [operation] = operationsFromFrame([frame("a", "Running", { phase: "" })]);

  assert.equal(operationPercent(operation), null);
  assert.equal(operationPhaseLabel(operation), "Starting");
});

test("fractions outside zero to one are clamped", () => {
  const [over, under] = operationsFromFrame([
    frame("a", "Running", { fraction: 1.7 }),
    frame("b", "Running", { fraction: -0.2 }),
  ]);

  assert.equal(operationPercent(over), 100);
  assert.equal(operationPercent(under), 0);
});

test("several running operations collapse to a count named after the longest running", () => {
  const summary = summarizeOperations(
    operationsFromFrame([
      frame("late", "Running", { seconds: 300 }),
      frame("early", "Running", { seconds: 100 }),
      frame("done", "Finished", { seconds: 50 }),
    ]),
  );

  assert.equal(summary.kind, "many");
  assert.equal(summary.count, 2);
  assert.equal(operationsCountLabel(summary.count), "2 Operations");
  assert.equal(summary.longest.id, "early");
  assert.deepEqual(
    summary.operations.map((operation) => operation.id),
    ["late", "early"],
  );
});

test("with nothing running, the last settled operation shows until it retires", () => {
  const finished = summarizeOperations(operationsFromFrame([frame("a", "Finished")]));
  assert.equal(finished.kind, "single");
  assert.equal(operationPhaseLabel(finished.operation), "Done");

  const failed = summarizeOperations(
    operationsFromFrame([{ ...frame("b", "Failed"), failure: "report does not fit" }]),
  );
  assert.equal(failed.kind, "single");
  assert.equal(failed.operation.status, "failed");
  assert.equal(failed.operation.failure, "report does not fit");
  assert.equal(operationPhaseLabel(failed.operation), "Failed");
});

test("a running operation outranks one that settled", () => {
  const summary = summarizeOperations(
    operationsFromFrame([frame("done", "Finished"), frame("busy", "Running")]),
  );

  assert.equal(summary.kind, "single");
  assert.equal(summary.operation.id, "busy");
});
