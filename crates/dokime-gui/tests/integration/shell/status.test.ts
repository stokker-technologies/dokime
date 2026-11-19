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
import { test } from "vitest";

// Local Imports
import { startupDiagnostics } from "@/app/native-session/errors";
import type { RuntimeInstalledPluginRecord } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { statusPresentation } from "@/features/settings/plugins/PluginList";
import { programRows, programSource } from "@/features/settings/plugins/PluginPrograms";
import { announceStopping, settleUnlessStopping } from "@/lib/desktop/stopping";
import type { SettingsFrame } from "@host/model";
import type { Wire } from "@/lib/rpc/canonical";
import { type BackendOperation, operationPhaseLabel, operationsCountLabel, summarizeOperations } from "@/shell/status-bar-operations";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type OperationOptions = { fraction?: number; phase?: string; seconds?: number };

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function frame(
  id: string,
  status: string,
  { fraction, phase = "Materializing Evidence", seconds = 100 }: OperationOptions = {},
): BackendOperation {
  return {
    fraction: fraction ?? null,
    label: `Operation ${id}`,
    id,
    phase,
    startedAtMs: seconds * 1000,
    failure: null,
    status: status === "Finished" ? "finished" : status === "Failed" ? "failed" : "running",
  };
}

/* === Entry Point === */

test("Operation Summary", () => {
  // An Idle Backend Shows Nothing
  assert.deepEqual(summarizeOperations([]), { kind: "idle" });
  assert.deepEqual(summarizeOperations([]), { kind: "idle" });

  // Several Running Operations Collapse to a Count Named After the Longest Running
  const many = summarizeOperations([
    frame("late", "Running", { seconds: 300 }),
    frame("early", "Running", { seconds: 100 }),
    frame("done", "Finished", { seconds: 50 }),
  ]);
  assert.ok(many.kind === "many");
  assert.equal(many.count, 2);
  assert.equal(operationsCountLabel(many.count), "2 Operations");
  assert.equal(many.longest.id, "early");
  assert.deepEqual(
    many.operations.map((operation) => operation.id),
    ["late", "early"],
  );

  // With Nothing Running, the Last Settled Operation Shows Until It Retires
  const finished = summarizeOperations([frame("a", "Finished")]);
  assert.ok(finished.kind === "single");
  assert.equal(operationPhaseLabel(finished.operation), "Done");
  const failed = summarizeOperations([{ ...frame("b", "Failed"), failure: "report does not fit" }]);
  assert.ok(failed.kind === "single");
  assert.equal(failed.operation.status, "failed");
  assert.equal(failed.operation.failure, "report does not fit");
  assert.equal(operationPhaseLabel(failed.operation), "Failed");

  // A Running Operation Outranks One That Settled
  const outranked = summarizeOperations([frame("done", "Finished"), frame("busy", "Running")]);
  assert.ok(outranked.kind === "single");
  assert.equal(outranked.operation.id, "busy");
});

test("Embedded Startup Diagnostics", () => {
  const diagnostics = { cause: "listener unavailable", log: "/application/log" };
  assert.equal(startupDiagnostics({ kind: "startup", diagnostics }), diagnostics);
  assert.equal(startupDiagnostics({ kind: "connection" }), null);
});

test("Brand Label", () => {
  // Host Programs Are Named After the Running Brand
  const program = {
    owner: { kind: "host" },
    version: "1.2.0",
  } as unknown as RuntimeInstalledPluginRecord;
  assert.equal(programSource(program, "Fixture"), "Fixture Built-in · v1.2.0");

  // The Library Reports Current Activation and Enablement Without Pending Installation States.
  const entry = { value: { enabled: true }, failure: { message: "activation-refused" } } as unknown as Parameters<
    typeof statusPresentation
  >[0];
  assert.equal(statusPresentation(entry).reason, entry.failure?.message);
  assert.equal(statusPresentation({ ...entry, failure: undefined, value: { enabled: false } }).reason, null);
});

test("Plugin Rows Open Their Own Settings Page", () => {
  const programs = [
    { owner: { kind: "host" }, plugin_id: "fixture.host.core", version: "1" },
    { owner: { kind: "host" }, plugin_id: "fixture.host.settings", version: "1" },
    { owner: { kind: "bundle", package: "fixture-bundle" }, plugin_id: "fixture.bundle", version: "1" },
    { owner: { kind: "embedded", generation: "g" }, plugin_id: "fixture.embedded", version: "1" },
  ] as unknown as RuntimeInstalledPluginRecord[];
  const frame = {
    cards: [],
    sections: [
      { owner: { kind: { case: "installedPluginId", value: "fixture.host.settings" } }, sectionId: "fixture-host-section" },
      { owner: { kind: { case: "installedPluginId", value: "fixture.bundle" } }, sectionId: "fixture-bundle-section" },
    ],
  } as unknown as Wire<SettingsFrame>;

  // The Host Row Opens the Section Any of Its Programs Contributes
  assert.deepEqual(
    programRows(programs, [], frame).map((row) => [row.key, row.settingsSection]),
    [
      ["host", "fixture-host-section"],
      ["bundle:fixture-bundle", "fixture-bundle-section"],
      [JSON.stringify(["embedded", null, null, "g", "fixture.embedded"]), null],
    ],
  );
});

test("a failure during the desktop's stop never settles, and the stop shows only once it lasts", async () => {
  const pending = Symbol("pending");
  const outcome = async (promise: Promise<unknown>) => {
    let result: unknown = pending;
    void promise.then(
      () => {
        result = "resolved";
      },
      () => {
        result = "rejected";
      },
    );
    await Promise.resolve();
    return result;
  };

  // Running: A Failure Is The Caller's
  assert.equal(await outcome(settleUnlessStopping(new Error("refused"))), "rejected");

  // The Shell's Own Refusal Starts the Stop Even Before Its Announcement Arrives
  assert.equal(await outcome(settleUnlessStopping({ kind: "session", detail: { kind: "shutting_down" } })), pending);

  // Once Stopping, Failures Remain Owned by the Stop.
  assert.equal(await outcome(settleUnlessStopping({ kind: "session", detail: { kind: "superseded" } })), pending);

  // A Kept-Open Window Fails Normally Again
  announceStopping({ stopping: false });
  assert.equal(await outcome(settleUnlessStopping(new Error("refused"))), "rejected");
});
