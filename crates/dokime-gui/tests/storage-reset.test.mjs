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
const { STORAGE_RESET_CONFIRMATION, storageResetAvailable, storageResetConfirmed } = await import(
  pathToFileURL(resolve(gui, "src/features/settings/storage-reset-model.ts")).href
);

/* === Fixture === */

function context(overrides = {}) {
  return {
    development: true,
    desktop: true,
    ownership: "owned_local",
    directory: "/home/operator/.local/share/com.stokker.dokime/data",
    ...overrides,
  };
}

/* === Tests === */

test("a development desktop that owns its backend offers the reset", () => {
  assert.equal(storageResetAvailable(context()), true);
});

test("an installer build never offers the reset", () => {
  assert.equal(storageResetAvailable(context({ development: false })), false);
});

test("a browser build never offers the reset", () => {
  assert.equal(storageResetAvailable(context({ desktop: false })), false);
});

test("a backend this desktop did not start is never offered the reset", () => {
  for (const ownership of ["attached", "none", null, undefined]) {
    assert.equal(storageResetAvailable(context({ ownership })), false, String(ownership));
  }
});

test("the reset waits until the backend names its data directory", () => {
  for (const directory of ["", null, undefined]) {
    assert.equal(storageResetAvailable(context({ directory })), false, String(directory));
  }
});

test("only the exact confirmation word arms the reset", () => {
  assert.equal(STORAGE_RESET_CONFIRMATION, "RESET");
  assert.equal(storageResetConfirmed("RESET"), true);
  for (const typed of ["", "reset", "Reset", " RESET", "RESET ", "RESETS", "RES"]) {
    assert.equal(storageResetConfirmed(typed), false, JSON.stringify(typed));
  }
});
