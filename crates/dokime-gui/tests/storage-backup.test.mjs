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
const { BACKUP_RESET_LABEL, backupFailure, backupResetAvailable, backupSteps, formatBytes } =
  await import(pathToFileURL(resolve(gui, "src/app/native-session/storage-backup-model.ts")).href);

/* === Fixture === */

function context(overrides = {}) {
  return { localBackendAvailable: true, failedStart: true, busy: false, ...overrides };
}

function states(phase, archived = null) {
  return backupSteps(phase, archived).map((step) => [step.label, step.state, step.detail]);
}

/* === Tests === */

test("a failed owned start offers the backup reset under its label", () => {
  assert.equal(BACKUP_RESET_LABEL, "Back Up And Reset Local Storage");
  assert.equal(backupResetAvailable(context()), true);
});

test("the backup reset is offered on the failure screen only", () => {
  assert.equal(backupResetAvailable(context({ failedStart: false })), false);
  assert.equal(backupResetAvailable(context({ localBackendAvailable: false })), false);
  assert.equal(backupResetAvailable(context({ busy: true })), false);
});

test("the steps start at stopping before the shell reports a phase", () => {
  assert.deepEqual(states(null), [
    ["Stopping Backend", "active", null],
    ["Archiving Local Storage", "waiting", null],
    ["Resetting Local Storage", "waiting", null],
    ["Restarting Backend", "waiting", null],
  ]);
});

test("archiving shows the bytes written", () => {
  assert.deepEqual(states("archiving", 3 * 1024 * 1024), [
    ["Stopping Backend", "done", null],
    ["Archiving Local Storage", "active", "3.0 MiB written"],
    ["Resetting Local Storage", "waiting", null],
    ["Restarting Backend", "waiting", null],
  ]);
});

test("resetting follows the finished archive", () => {
  assert.deepEqual(
    states("deleting").map(([, state]) => state),
    ["done", "done", "active", "waiting"],
  );
});

test("every backend startup phase belongs to restarting", () => {
  for (const phase of ["starting", "storage", "plugins", "requests", "unknown"]) {
    assert.deepEqual(
      states(phase).map(([, state]) => state),
      ["done", "done", "done", "active"],
      phase,
    );
  }
});

test("byte counts use the largest binary unit", () => {
  assert.equal(formatBytes(0), "0 B");
  assert.equal(formatBytes(1023), "1023 B");
  assert.equal(formatBytes(1536), "1.5 KiB");
  assert.equal(formatBytes(5 * 1024 ** 3), "5.0 GiB");
});

test("a failure after archiving names the kept archive", () => {
  const inner = { kind: "storage_reset", reason: "failed", message: "denied" };
  assert.deepEqual(
    backupFailure({ kind: "archived", archive: "/data-backup-20261015T120000Z.zip", error: inner }),
    { archive: "/data-backup-20261015T120000Z.zip", error: inner },
  );
});

test("a failure before archiving names no archive", () => {
  for (const error of [
    null,
    "failed",
    { kind: "storage_reset", reason: "backend_running", message: "held" },
    { kind: "startup", message: "exited", diagnostics: null },
    { kind: "archived" },
  ]) {
    assert.equal(backupFailure(error), null, JSON.stringify(error));
  }
});
