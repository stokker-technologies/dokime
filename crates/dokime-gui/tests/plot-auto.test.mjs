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
import { test } from "node:test";
import { reduceIndices } from "../src/plots/reduce.ts";

/* === Tests === */

test("Auto switches to an envelope only above four times the display budget", () => {
  const x = Array.from({ length: 41 }, (_, index) => index);
  const y = x.map((value) => Math.sin(value));
  const budget = 10;
  const moderateX = x.slice(0, 40);
  const moderateY = y.slice(0, 40);

  assert.deepEqual(
    reduceIndices(moderateX, moderateY, { budget, technique: "auto" }),
    reduceIndices(moderateX, moderateY, { budget, technique: "lttb" }),
  );
  assert.deepEqual(
    reduceIndices(x, y, { budget, technique: "auto" }),
    reduceIndices(x, y, { budget, technique: "min_max" }),
  );
});

test("Auto preserves raw points for an unlimited display", () => {
  const x = [0, 1, 2];
  assert.equal(reduceIndices(x, x, { budget: null, technique: "auto" }), null);
});
