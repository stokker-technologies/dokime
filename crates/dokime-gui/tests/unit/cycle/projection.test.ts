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
import { mapNativeExecutionSummary, projectExecutionStatus } from "@/lib/api/native-planning";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Unstarted Attempt Status", () => {
  const skipped = { HarmosVariant: { case: "Skipped" as const, value: {} } };
  assert.equal(projectExecutionStatus(skipped, null, false), "planned");
  assert.equal(projectExecutionStatus(skipped, null, true), "skipped");
  for (const outcome of ["Completed", "Failed", "Stopped", "Interrupted"]) {
    const cycle = { cycleId: "cycle", status: { HarmosVariant: { case: outcome, value: {} } } };
    const attempt = { executionId: "unstarted", caseId: "case", ordinal: 1n, status: skipped, sealed: false, archived: false };
    const projected = mapNativeExecutionSummary(attempt, [], cycle as never);
    assert.equal(projected.status, "planned");
    assert.equal(projected.sealed, false);
  }
});
