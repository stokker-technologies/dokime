/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Plot Colour Selection
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { test } from "vitest";

// Local Imports
import { markerColor, sequentialColorScale } from "@dokime/plots/colors";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

test("Marker Kinds Take the Reference Palette by Index", () => {
  const references = ["#a1a1a1", "#b2b2b2", "#c3c3c3"];
  const expected = ["#a1a1a1", "#b2b2b2", "#c3c3c3", "#a1a1a1", "#b2b2b2", "#c3c3c3"];
  for (const [index, color] of expected.entries()) {
    assert.equal(markerColor(references, index), color);
  }
});

/* === Entry Point === */

test("Sequential Colour Domains Ignore Missing Values and Share Constant and Empty Behaviour", () => {
  const scale = sequentialColorScale([null, -3, undefined, Number.NaN, 9, Number.POSITIVE_INFINITY]);
  assert.deepEqual([scale.min, scale.max, scale.present], [-3, 9, true]);
  assert.deepEqual(scale.color.domain(), [-3, 9]);
  const constant = sequentialColorScale([4, null, 4]);
  assert.equal(constant.present, true);
  assert.equal(constant.color(4), sequentialColorScale([4]).color(4));
  assert.notEqual(constant.color(4), undefined);
  const empty = sequentialColorScale([null, Number.NaN]);
  assert.equal(empty.present, false);
  assert.ok(empty.color.domain().every(Number.isFinite));
});
