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
import {
  BACKUP_RESET_LABEL,
  type BackupResetContext,
  backupFailure,
  backupResetAvailable,
  backupSteps,
  formatBytes,
} from "@/app/native-session/storage-backup-model";
import {
  STORAGE_RESET_CONFIRMATION,
  type StorageResetContext,
  storageResetAvailable,
  storageResetConfirmed,
} from "@/features/settings/storage-reset-model";

/* === Fixture === */

function resetContext(overrides: Partial<StorageResetContext> = {}): StorageResetContext {
  return {
    development: true,
    desktop: true,
    ownership: "owned_local",
    directory: "/home/operator/.local/share/com.stokker.dokime/data",
    ...overrides,
  } as StorageResetContext;
}

function backupContext(overrides: Partial<BackupResetContext> = {}): BackupResetContext {
  return { localBackendAvailable: true, failedStart: true, busy: false, ...overrides };
}

function states(phase: string | null, archived: number | null = null) {
  return backupSteps(phase, archived).map((step) => [step.label, step.state, step.detail]);
}

/* === Tests === */

test("Storage Reset", () => {
  // A Development Desktop That Owns Its Backend Offers The Reset
  assert.equal(storageResetAvailable(resetContext()), true);

  // An Installer Build Never Offers The Reset
  assert.equal(storageResetAvailable(resetContext({ development: false })), false);

  // A Browser Build Never Offers The Reset
  assert.equal(storageResetAvailable(resetContext({ desktop: false })), false);

  // A Backend This Desktop Did Not Start Is Never Offered The Reset
  for (const ownership of ["attached", "none", null, undefined]) {
    const context = resetContext({ ownership: ownership as StorageResetContext["ownership"] });
    assert.equal(storageResetAvailable(context), false, String(ownership));
  }

  // The Reset Waits Until The Backend Names Its Data Directory
  for (const directory of ["", null, undefined]) {
    assert.equal(storageResetAvailable(resetContext({ directory })), false, String(directory));
  }

  // Only The Exact Confirmation Word Arms The Reset
  assert.equal(STORAGE_RESET_CONFIRMATION, "RESET");
  assert.equal(storageResetConfirmed("RESET"), true);
  for (const typed of ["", "reset", "Reset", " RESET", "RESET ", "RESETS", "RES"]) {
    assert.equal(storageResetConfirmed(typed), false, JSON.stringify(typed));
  }
});

test("Backup Progress", () => {
  // A Failed Owned Start Offers The Backup Reset Under Its Label
  assert.equal(BACKUP_RESET_LABEL, "Back Up And Reset Local Storage");
  assert.equal(backupResetAvailable(backupContext()), true);

  // The Backup Reset Is Offered On The Failure Screen Only
  assert.equal(backupResetAvailable(backupContext({ failedStart: false })), false);
  assert.equal(backupResetAvailable(backupContext({ localBackendAvailable: false })), false);
  assert.equal(backupResetAvailable(backupContext({ busy: true })), false);

  // The Steps Start At Stopping Before The Shell Reports A Phase
  assert.deepEqual(states(null), [
    ["Stopping Backend", "active", null],
    ["Archiving Local Storage", "waiting", null],
    ["Resetting Local Storage", "waiting", null],
    ["Restarting Backend", "waiting", null],
  ]);

  // Archiving Shows The Bytes Written
  assert.deepEqual(states("archiving", 3 * 1024 * 1024), [
    ["Stopping Backend", "done", null],
    ["Archiving Local Storage", "active", "3.0 MiB written"],
    ["Resetting Local Storage", "waiting", null],
    ["Restarting Backend", "waiting", null],
  ]);

  // Resetting Follows The Finished Archive
  assert.deepEqual(
    states("deleting").map(([, state]) => state),
    ["done", "done", "active", "waiting"],
  );

  // Every Backend Startup Phase Belongs To Restarting
  for (const phase of ["starting", "storage", "plugins", "requests", "unknown"]) {
    assert.deepEqual(
      states(phase).map(([, state]) => state),
      ["done", "done", "done", "active"],
      phase,
    );
  }

  // Byte Counts Use The Largest Binary Unit
  assert.equal(formatBytes(0), "0 B");
  assert.equal(formatBytes(1023), "1023 B");
  assert.equal(formatBytes(1536), "1.5 KiB");
  assert.equal(formatBytes(5 * 1024 ** 3), "5.0 GiB");
});

test("Backup Failure", () => {
  // A Failure After Archiving Names The Kept Archive
  const inner = { kind: "storage_reset", reason: "failed", message: "denied" };
  assert.deepEqual(backupFailure({ kind: "archived", archive: "/data-backup-20261015T120000Z.zip", error: inner }), {
    archive: "/data-backup-20261015T120000Z.zip",
    error: inner,
  });

  // A Failure Before Archiving Names No Archive
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
