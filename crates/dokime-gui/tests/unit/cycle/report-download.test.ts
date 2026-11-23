// Dokime — SPDX-License-Identifier: Apache-2.0

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { test } from "vitest";

// Local Imports
import { cycleReportSealed, reportDownloadState } from "@/features/cycle/cycle-workspace-model";

/* === Entry Point === */

test("Reports Wait for the Seal and Any Save in Progress", () => {
  assert.equal(reportDownloadState(false, false).disabled, true);
  assert.match(reportDownloadState(false, false).title, /once the run is sealed/);
  assert.deepEqual(reportDownloadState(false, true), { disabled: false, label: "Download Report", title: "Download Report" });
  assert.equal(reportDownloadState(true, true).disabled, true);
  assert.equal(reportDownloadState(true, false).disabled, true);
});

test("Cycle Reports Wait for Every Execution to Seal", () => {
  const executions = [{ sealed: true }, { sealed: false }];
  assert.equal(cycleReportSealed("completed", executions), false);
  executions[1].sealed = true;
  assert.equal(cycleReportSealed("completed", executions), true);
  for (const status of ["running", "paused", "stopping"] as const) {
    assert.equal(cycleReportSealed(status, executions), false);
  }
});
