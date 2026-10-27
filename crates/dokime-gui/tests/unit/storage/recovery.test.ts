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
import {
  STORAGE_RESET_CONFIRMATION,
  type StorageResetContext,
  storageResetAvailable,
  storageResetConfirmed,
} from "@/features/settings/storage-reset-model";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function resetContext(overrides: Partial<StorageResetContext> = {}): StorageResetContext {
  return {
    development: true,
    desktop: true,
    ownership: "owned_local",
    directory: "/home/operator/.local/share/com.stokker.dokime/data",
    ...overrides,
  } as StorageResetContext;
}

/* === Entry Point === */

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
