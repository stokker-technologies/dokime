/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * About Build Versions
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { test } from "vitest";
// Local Imports
import { buildLabel, buildText } from "@/features/settings/sections/build-version";

/* === Re-Exports === */

/* === Constants === */

const COMMIT = "766330b73c0b0e5a4c1f9d2e8a7b6c5d4e3f2a1b";

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

test("a published build shows its short commit and channel and copies the full commit", () => {
  const label = buildLabel({ productVersion: "0.1.2", sourceRevision: COMMIT, channel: "nightly" });
  assert.equal(label.commit, COMMIT);
  assert.equal(buildText(label), "v0.1.2 · 766330b7 · nightly");
});

test("a dirty checkout keeps its mark beside the short commit", () => {
  const label = buildLabel({ productVersion: "0.1.2", sourceRevision: `${COMMIT}-dirty`, channel: null });
  assert.equal(label.commit, COMMIT);
  assert.equal(buildText(label), "v0.1.2 · 766330b7-dirty");
});

test("a build without a readable commit reads unknown and offers nothing to copy", () => {
  for (const sourceRevision of [null, "", "unknown", "766330", "not-a-commit"]) {
    const label = buildLabel({ productVersion: "0.1.2", sourceRevision, channel: null });
    assert.equal(label.commit, null, String(sourceRevision));
    assert.equal(buildText(label), "v0.1.2 · unknown");
  }
});
