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
import { backendExitMeaning } from "@/app/native-session/errors";
import type { RuntimeInstalledPluginRecord } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { statusPresentation } from "@/features/settings/plugins/PluginList";
import { programSource } from "@/features/settings/plugins/PluginPrograms";
import {
  type OperationFrame,
  operationPercent,
  operationPhaseLabel,
  operationsCountLabel,
  operationsFromFrame,
  summarizeOperations,
} from "@/shell/status-bar-operations";

/* === Fixture === */

type FrameOptions = { fraction?: number; phase?: string; seconds?: number };

function frame(
  id: string,
  status: string,
  { fraction, phase = "Materializing Evidence", seconds = 100 }: FrameOptions = {},
): OperationFrame {
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

test("Operation Progress", () => {
  // One Running Operation Shows Its Label, Phase And Determinate Progress
  const summary = summarizeOperations(operationsFromFrame([frame("a", "Running", { fraction: 0.426 })]));
  assert.ok(summary.kind === "single");
  assert.equal(summary.operation.label, "Operation a");
  assert.equal(operationPhaseLabel(summary.operation), "Materializing Evidence");
  assert.equal(operationPercent(summary.operation), 43);
  assert.equal(summary.operation.startedAtMs, 100_000);

  // An Operation Without A Fraction Is Indeterminate
  const [indeterminate] = operationsFromFrame([frame("a", "Running", { phase: "" })]);
  assert.equal(operationPercent(indeterminate), null);
  assert.equal(operationPhaseLabel(indeterminate), "Starting");

  // Fractions Outside Zero To One Are Clamped
  const [over, under] = operationsFromFrame([frame("a", "Running", { fraction: 1.7 }), frame("b", "Running", { fraction: -0.2 })]);
  assert.equal(operationPercent(over), 100);
  assert.equal(operationPercent(under), 0);
});

test("Operation Summary", () => {
  // An Idle Backend Shows Nothing
  assert.deepEqual(summarizeOperations(operationsFromFrame([])), { kind: "idle" });
  assert.deepEqual(summarizeOperations(operationsFromFrame(undefined)), { kind: "idle" });

  // Several Running Operations Collapse To A Count Named After The Longest Running
  const many = summarizeOperations(
    operationsFromFrame([
      frame("late", "Running", { seconds: 300 }),
      frame("early", "Running", { seconds: 100 }),
      frame("done", "Finished", { seconds: 50 }),
    ]),
  );
  assert.ok(many.kind === "many");
  assert.equal(many.count, 2);
  assert.equal(operationsCountLabel(many.count), "2 Operations");
  assert.equal(many.longest.id, "early");
  assert.deepEqual(
    many.operations.map((operation) => operation.id),
    ["late", "early"],
  );

  // With Nothing Running, The Last Settled Operation Shows Until It Retires
  const finished = summarizeOperations(operationsFromFrame([frame("a", "Finished")]));
  assert.ok(finished.kind === "single");
  assert.equal(operationPhaseLabel(finished.operation), "Done");
  const failed = summarizeOperations(operationsFromFrame([{ ...frame("b", "Failed"), failure: "report does not fit" }]));
  assert.ok(failed.kind === "single");
  assert.equal(failed.operation.status, "failed");
  assert.equal(failed.operation.failure, "report does not fit");
  assert.equal(operationPhaseLabel(failed.operation), "Failed");

  // A Running Operation Outranks One That Settled
  const outranked = summarizeOperations(operationsFromFrame([frame("done", "Finished"), frame("busy", "Running")]));
  assert.ok(outranked.kind === "single");
  assert.equal(outranked.operation.id, "busy");
});

test("Backend Exit Meaning", () => {
  // Each Code The Server Defines Names Its Cause
  assert.match(backendExitMeaning(2) ?? "", /^Invalid input/);
  assert.match(backendExitMeaning(3) ?? "", /^Runtime failure/);
  assert.match(backendExitMeaning(4) ?? "", /^Incompatible storage/);
  assert.equal(backendExitMeaning(0), "Clean stop");

  // An Undefined Code Or A Signal Claims No Meaning
  assert.equal(backendExitMeaning(1), null);
  assert.equal(backendExitMeaning(null), null);
});

test("Brand Label", () => {
  // Host Programs Are Named After The Running Brand
  const program = {
    owner: { kind: "host" },
    version: "1.2.0",
  } as unknown as RuntimeInstalledPluginRecord;
  assert.equal(programSource(program, "Acme"), "Acme Host · v1.2.0");

  // A Pending Removal Waits For The Brand To Restart
  const entry = {
    active: true,
    consented_at: 1,
    error: null,
    state: "removal_pending",
  } as unknown as Parameters<typeof statusPresentation>[0];
  assert.equal(statusPresentation(entry, "Acme").reason, "Active until Acme restarts.");
});
