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
const { FOLLOW_SLACK_PX, consoleTailPinned } = await import(
  pathToFileURL(resolve(gui, "src/shell/console-follow.ts")).href
);

/* === Fixture === */

/** A 200 px viewport over 100 rows of 22 px, scrolled to `scrollTop`. */
function at(scrollTop) {
  return { clientHeight: 200, scrollHeight: 2_200, scrollTop };
}

const BOTTOM = 2_000;

/* === Tests === */

test("the list is pinned at the bottom and within one row of it", () => {
  assert.equal(consoleTailPinned(at(BOTTOM)), true);
  assert.equal(consoleTailPinned(at(BOTTOM - FOLLOW_SLACK_PX)), true);
  assert.equal(consoleTailPinned(at(BOTTOM - FOLLOW_SLACK_PX - 1)), false);
});

test("scrolling up releases the tail", () => {
  assert.equal(consoleTailPinned(at(BOTTOM - 300)), false);
});

test("a list too short to scroll is always on its tail", () => {
  assert.equal(consoleTailPinned({ clientHeight: 200, scrollHeight: 120, scrollTop: 0 }), true);
});
