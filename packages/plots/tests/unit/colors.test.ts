/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Plot Marker Colors
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { test } from "vitest";

// Local Imports
import { markerColor } from "@dokime/plots/colors";

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
