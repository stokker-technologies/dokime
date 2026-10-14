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
import {
  nodeMenuGroups,
  reportDownloadState,
  reportFilename,
} from "../src/features/cycle/cycle-workspace-model.ts";

/* === Tests === */

test("the cycle menu opens its report and offers no bundle, folder or written report", () => {
  const items = nodeMenuGroups("cycle").flat();
  assert.deepEqual(nodeMenuGroups("cycle"), [
    ["Copy Cycle", "Add Cases", "Runtime Policy", "Follow Active"],
    ["Open Report"],
    ["Clear Cycle"],
  ]);
  for (const removed of ["Download Bundle", "Write Report", "Open Folder"]) {
    assert.ok(!items.includes(removed), removed);
  }
});

test("the execution menu offers Re-Run and Open Report", () => {
  assert.deepEqual(nodeMenuGroups("execution"), [["Re-Run"], ["Open Report"]]);
});

test("Download Report waits only for a save in progress", () => {
  assert.deepEqual(reportDownloadState(false), {
    disabled: false,
    label: "Download Report",
    title: "Download Report",
  });
  const saving = reportDownloadState(true);
  assert.equal(saving.disabled, true);
  assert.equal(saving.label, "Download Report");
  assert.notEqual(saving.title, "Download Report");
});

test("a saved report is named after its cycle or execution", () => {
  assert.equal(reportFilename("Charger Soak"), "Charger Soak.html");
  assert.equal(
    reportFilename("Execution 1 · HCL / Reconnection"),
    "Execution 1 · HCL - Reconnection.html",
  );
  assert.equal(reportFilename(' a:b*c? "d" '), "a-b-c- -d.html");
  assert.equal(reportFilename("..."), "report.html");
});
