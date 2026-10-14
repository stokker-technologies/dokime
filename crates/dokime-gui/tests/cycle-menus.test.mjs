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
  archiveRefusal,
  nodeMenuGroups,
  reportDownloadState,
  reportFilename,
} from "../src/features/cycle/cycle-workspace-model.ts";

/* === Tests === */

test("the cycle menu opens its report and archives, with no bundle, folder or clearing", () => {
  const items = nodeMenuGroups("cycle", false).flat();
  assert.deepEqual(nodeMenuGroups("cycle", false), [
    ["Copy Cycle", "Add Cases", "Runtime Policy", "Follow Active"],
    ["Open Report"],
    ["Archive"],
  ]);
  for (const removed of ["Download Bundle", "Write Report", "Open Folder", "Clear Cycle"]) {
    assert.ok(!items.includes(removed), removed);
  }
});

test("the execution menu offers Re-Run, Open Report and Archive", () => {
  assert.deepEqual(nodeMenuGroups("execution", false), [["Re-Run"], ["Open Report"], ["Archive"]]);
});

test("an archived node offers Unarchive in place of Archive", () => {
  assert.deepEqual(nodeMenuGroups("cycle", true).at(-1), ["Unarchive"]);
  assert.deepEqual(nodeMenuGroups("execution", true).at(-1), ["Unarchive"]);
});

test("archiving waits for a terminal run and says so", () => {
  assert.equal(archiveRefusal(true), undefined);
  assert.match(archiveRefusal(false), /once the run has ended/);
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

test("the Archived view lists every archived cycle in one group", async () => {
  const { archivedCycleGroups } = await import("../src/features/cycles-home/cycles-model.ts");
  const rows = [{ cycleId: "one" }, { cycleId: "two" }];
  const [group, ...rest] = archivedCycleGroups(rows);
  assert.equal(rest.length, 0);
  assert.equal(group.title, "Archived");
  assert.deepEqual(group.rows, rows);
  assert.equal(archivedCycleGroups([], true)[0].emptyText, "No Archived Cycles Match This Filter");
});
