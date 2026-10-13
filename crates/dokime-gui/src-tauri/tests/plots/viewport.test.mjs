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

const gui = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const { PLOT_DRAG_OPTIONS, heldViewport } = await import(
  pathToFileURL(resolve(gui, "src/plots/viewport.ts")).href
);

/* A 400 x 200 plot area over x in [0, 100] and y in [0, 50], y growing upward. */
const AREA = { width: 400, height: 200 };
const posToVal = (position, axis) =>
  axis === "x" ? (position / AREA.width) * 100 : 50 - (position / AREA.height) * 50;

/* === Tests === */

test("drag options zoom either axis alone or both together", () => {
  assert.equal(PLOT_DRAG_OPTIONS.x, true);
  assert.equal(PLOT_DRAG_OPTIONS.y, true);
  assert.ok(PLOT_DRAG_OPTIONS.uni > 0, "a uni threshold makes one-axis drags possible");
  assert.ok(PLOT_DRAG_OPTIONS.dist > 0, "a click is not a zoom");
  assert.equal(PLOT_DRAG_OPTIONS.setScale, true);
});

test("a diagonal box holds both axes in value space", () => {
  const held = heldViewport({ left: 100, top: 50, width: 100, height: 50 }, AREA, posToVal);
  assert.deepEqual(held, { x: { min: 25, max: 50 }, y: { min: 25, max: 37.5 } });
});

test("a horizontal drag spans the full height and holds x alone", () => {
  const held = heldViewport({ left: 40, top: 0, width: 80, height: 200 }, AREA, posToVal);
  assert.deepEqual(held, { x: { min: 10, max: 30 }, y: null });
});

test("a vertical drag spans the full width and holds y alone", () => {
  const held = heldViewport({ left: 0, top: 20, width: 400, height: 40 }, AREA, posToVal);
  assert.deepEqual(held, { x: null, y: { min: 35, max: 45 } });
});

test("an empty or full-area box holds nothing", () => {
  assert.equal(heldViewport({ left: 0, top: 0, width: 0, height: 0 }, AREA, posToVal), null);
  assert.equal(heldViewport({ left: 10, top: 10, width: 20, height: 0 }, AREA, posToVal), null);
  assert.equal(heldViewport({ left: 0, top: 0, width: 400, height: 200 }, AREA, posToVal), null);
});

test("ranges are ordered whichever way the axis runs", () => {
  const flipped = (position, axis) => (axis === "x" ? 100 - position : position);
  const held = heldViewport({ left: 10, top: 10, width: 10, height: 10 }, AREA, flipped);
  assert.deepEqual(held, { x: { min: 80, max: 90 }, y: { min: 10, max: 20 } });
});
