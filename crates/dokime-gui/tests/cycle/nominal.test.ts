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
import { test } from "vitest";
import {
  archiveRefusal,
  buildCycleExecutionRows,
  type CycleCase,
  type CycleSelection,
  cycleRowSelection,
  executionControlSet,
  followSelection,
  isCycleTerminal,
  isFollowing,
  isUnstartedExecution,
  nodeMenuGroups,
  reportDownloadState,
  reportFilename,
  rerunnableCaseIds,
  rowOffersRunAgain,
  selectedCycleRow,
} from "@/features/cycle/cycle-workspace-model";
import {
  BUSY_RETRY_MAX_MS,
  DOCUMENT_FAILED_FALLBACK,
  documentRetry,
  documentRetryDelay,
  evidencePaneState,
  isBusyRefusal,
} from "@/features/cycle/document-query-model";
import {
  CLOSED_LABEL_DRAFT,
  EMPTY_LABEL_ERROR,
  labelEditorKey,
  openLabelEdit,
  refuseLabel,
  updateLabelText,
} from "@/features/cycle/label-editor-model";
import { archivedCycleGroups, type CycleRow } from "@/features/cycles-home/cycles-model";
import {
  EMPTY_TITLE_ERROR,
  initialNewCycleForm,
  newCycleFormError,
  newCycleRequest,
} from "@/features/cycles-home/new-cycle-form";
import { defaultCycleLabel } from "@/lib/time-format";
import { reportFixturePayload } from "@/report/report-fixture";
import { focusPresentation } from "@/shell/focus-mode";

/* === Fixture === */

type Execution = Parameters<typeof buildCycleExecutionRows>[1][number];
type RunControl = Parameters<typeof executionControlSet>[1];
type FollowInput = Parameters<typeof followSelection>[0];

function cases(...ids: string[]): CycleCase[] {
  return ids.map((caseId) => ({ caseId }) as CycleCase);
}

function execution(fields: Record<string, unknown>): Execution {
  return fields as unknown as Execution;
}

/* === Tests === */

test("Node Menus", () => {
  // Cycle Menu Opens Its Report And Archives, With No Bundle, Folder Or Clearing
  const items: string[] = nodeMenuGroups("cycle", false).flat();
  assert.deepEqual(nodeMenuGroups("cycle", false), [
    ["Copy Cycle", "Add Cases", "Runtime Policy", "Follow Active"],
    ["Open Report"],
    ["Archive"],
  ]);
  for (const removed of ["Download Bundle", "Write Report", "Open Folder", "Clear Cycle"]) {
    assert.ok(!items.includes(removed), removed);
  }

  // Execution Menu Offers Re-Run, Open Report And Archive
  assert.deepEqual(nodeMenuGroups("execution", false), [["Re-Run"], ["Open Report"], ["Archive"]]);

  // An Archived Node Offers Unarchive In Place Of Archive
  assert.deepEqual(nodeMenuGroups("cycle", true).at(-1), ["Unarchive"]);
  assert.deepEqual(nodeMenuGroups("execution", true).at(-1), ["Unarchive"]);

  // Archiving Waits For A Terminal Run And Says So
  assert.equal(archiveRefusal(true), undefined);
  assert.match(archiveRefusal(false) ?? "", /once the run has ended/);

  // Download Report Waits Only For A Save In Progress
  assert.deepEqual(reportDownloadState(false), {
    disabled: false,
    label: "Download Report",
    title: "Download Report",
  });
  const saving = reportDownloadState(true);
  assert.equal(saving.disabled, true);
  assert.equal(saving.label, "Download Report");
  assert.notEqual(saving.title, "Download Report");

  // A Saved Report Is Named After Its Cycle Or Execution
  assert.equal(reportFilename("Charger Soak"), "Charger Soak.html");
  assert.equal(
    reportFilename("Execution 1 · OBC / Reconnection"),
    "Execution 1 · OBC - Reconnection.html",
  );
  assert.equal(reportFilename(' a:b*c? "d" '), "a-b-c- -d.html");
  assert.equal(reportFilename("..."), "report.html");

  // The Archived View Lists Every Archived Cycle In One Group
  const rows = [{ cycleId: "one" }, { cycleId: "two" }] as CycleRow[];
  const [group, ...rest] = archivedCycleGroups(rows);
  assert.equal(rest.length, 0);
  assert.equal(group.title, "Archived");
  assert.deepEqual(group.rows, rows);
  assert.equal(archivedCycleGroups([], true)[0].emptyText, "No Archived Cycles Match This Filter");
});

test("Execution Controls", () => {
  // An Execution That Has Not Started Offers Start And Nothing Else
  assert.deepEqual(executionControlSet("planned", null), ["Start"]);
  assert.deepEqual(executionControlSet("created", null), ["Start"]);

  // Start Leaves Once The Execution Runs Or Ends, In Start | Re-Run | Stop Order
  const running = {
    canPause: false,
    canResume: false,
    canStop: true,
    mode: null,
    paused: false,
    requestId: "r",
    stopRequested: false,
  } as unknown as NonNullable<RunControl>;
  assert.deepEqual(executionControlSet("running", running), ["Stop Execution"]);
  assert.deepEqual(executionControlSet("running", { ...running, canPause: true }), [
    "Pause Execution",
    "Stop Execution",
  ]);
  assert.deepEqual(executionControlSet("passed", null), ["Re-Run"]);
  assert.deepEqual(executionControlSet("failed", null), ["Re-Run"]);
  assert.deepEqual(executionControlSet("running", null), []);
});

test("Label Editing", () => {
  // The Pencil Opens A Draft Holding The Current Label
  assert.deepEqual(openLabelEdit("Charger #2"), {
    mode: "editing",
    original: "Charger #2",
    text: "Charger #2",
    error: null,
  });

  // Enter Saves The Trimmed Label
  const renamed = updateLabelText(openLabelEdit("Charger #2"), "  Charger Burn-In  ");
  assert.deepEqual(labelEditorKey(renamed, "Enter"), { kind: "save", label: "Charger Burn-In" });

  // Escape Cancels Whatever Was Typed
  const typed = updateLabelText(openLabelEdit("Charger #2"), "Charger Burn-In");
  assert.deepEqual(labelEditorKey(typed, "Escape"), { kind: "cancel" });

  // Enter On An Unchanged Label Closes The Draft Without Saving
  const unchanged = updateLabelText(openLabelEdit("Charger #2"), " Charger #2 ");
  assert.deepEqual(labelEditorKey(unchanged, "Enter"), { kind: "cancel" });

  // Enter On A Blank Label Is Refused In Place
  const blank = updateLabelText(openLabelEdit("Charger #2"), "   ");
  assert.deepEqual(labelEditorKey(blank, "Enter"), { kind: "refuse", error: EMPTY_LABEL_ERROR });

  // Other Keys Type, And A Closed Editor Ignores Every Key
  assert.deepEqual(labelEditorKey(openLabelEdit("Charger #2"), "a"), { kind: "none" });
  assert.deepEqual(labelEditorKey(CLOSED_LABEL_DRAFT, "Enter"), { kind: "none" });

  // A Refusal Stays Beside The Draft Until The Operator Types Again
  const refused = refuseLabel(openLabelEdit("Charger #2"), "label must not be empty");
  assert.ok(refused.mode === "editing");
  assert.equal(refused.error, "label must not be empty");
  assert.equal(refused.text, "Charger #2");
  const retyped = updateLabelText(refused, "Charger");
  assert.ok(retyped.mode === "editing");
  assert.equal(retyped.error, null);
  assert.equal(refuseLabel(CLOSED_LABEL_DRAFT, "ignored"), CLOSED_LABEL_DRAFT);
});

test("Execution Rows", () => {
  // A Rerun Appends A Distinct Row While The Old Selection Still Names Its Evidence
  const first = execution({ case_id: "first", test_execution_id: "one", attempt_index: 0 });
  const second = execution({ case_id: "second", test_execution_id: "two", attempt_index: 1 });
  const rerun = execution({ case_id: "first", test_execution_id: "three", attempt_index: 2 });
  const rows = buildCycleExecutionRows(cases("first", "second"), [rerun, second, first]);

  assert.deepEqual(
    rows.map((row) => row.key),
    ["one", "two", "three"],
  );
  assert.equal(rows[2].rerunOf, 1);
  assert.equal(rows[0].newest, false);
  assert.equal(rows[2].newest, true);
  assert.equal(selectedCycleRow(rows, { executionId: "one" })?.execution, first);
  assert.deepEqual(cycleRowSelection(rows[2]), { executionId: "three" });
  assert.equal(selectedCycleRow(rows, { caseId: "first" }), null);

  // A Case Without An Execution Has A Placeholder And Stale Executions Select No Row
  const only = execution({ case_id: "first", test_execution_id: "one", attempt_index: 0 });
  const placeholders = buildCycleExecutionRows(cases("first", "second"), [only]);

  assert.equal(placeholders.length, 2);
  assert.deepEqual(cycleRowSelection(placeholders[1]), { caseId: "second" });
  assert.equal(selectedCycleRow(placeholders, { caseId: "second" }), placeholders[1]);
  assert.equal(selectedCycleRow(placeholders, { executionId: "removed" }), null);
  assert.equal(selectedCycleRow(placeholders, {}), null);

  // Run Again Is Offered On The Newest Ended Attempt And The Ended Cycle
  const attempts = buildCycleExecutionRows(cases("first", "second", "third"), [
    execution({
      case_id: "first",
      test_execution_id: "one",
      attempt_index: 0,
      status: "interrupted",
    }),
    execution({ case_id: "second", test_execution_id: "two", attempt_index: 1, status: "running" }),
    execution({ case_id: "first", test_execution_id: "three", attempt_index: 2, status: "passed" }),
  ]);

  assert.deepEqual(
    attempts.map((row) => [row.key, rowOffersRunAgain(row)]),
    [
      ["one", false],
      ["two", false],
      ["three", true],
      ["case:third", false],
    ],
  );
  const lifecycles = [
    "completed",
    "failed",
    "stopped",
    "interrupted",
    "running",
    "stopping",
    "planned",
  ] as Parameters<typeof isCycleTerminal>[0][];
  assert.deepEqual(lifecycles.map(isCycleTerminal), [true, true, true, true, false, false, false]);
  assert.deepEqual(
    rerunnableCaseIds([
      { caseId: "allowed", actions: { schedule_rerun: true } },
      { caseId: "refused", actions: { schedule_rerun: false } },
      { caseId: "unstated", actions: null },
    ] as unknown as CycleCase[]),
    ["allowed", "unstated"],
  );
});

test("Follow Active", () => {
  // An Explicit Selection Beats Follow Active Until The Operator Returns To It
  const follow = (selection: CycleSelection, extra: Partial<FollowInput> = {}) =>
    followSelection({
      active: "ten",
      previousActive: "ten",
      selectedEnded: false,
      selection,
      ...extra,
    });

  assert.equal(follow({}), "ten", "landing follows the running execution");
  assert.equal(follow({ executionId: "twelve" }), null, "a click on a queued row stays there");
  assert.equal(follow({ view: "cycle" }), null, "the cycle is a selection of its own");
  assert.equal(follow({ caseId: "unrun" }), null);
  assert.equal(follow({ executionId: "ten" }), null, "already on the active one");
  assert.equal(
    follow({ executionId: "ten" }, { active: "eleven" }),
    "eleven",
    "a following selection moves on with the runtime",
  );
  assert.equal(
    follow({ executionId: "three" }, { active: "eleven", selectedEnded: true }),
    "eleven",
    "the selected execution ending resumes following",
  );
  assert.equal(
    followSelection({ active: null, previousActive: null, selectedEnded: false, selection: {} }),
    null,
  );

  // Following Reads The Selection Against The Active Execution
  assert.equal(isFollowing({}, "ten"), true);
  assert.equal(isFollowing({ executionId: "ten" }, "ten"), true);
  assert.equal(isFollowing({ executionId: "twelve" }, "ten"), false);
  assert.equal(isFollowing({ view: "cycle" }, "ten"), false);
  const statuses = ["planned", "created", "running", "passed"] as Parameters<
    typeof isUnstartedExecution
  >[0][];
  assert.deepEqual(statuses.map(isUnstartedExecution), [true, true, false, false]);
});

test("Report Steps", () => {
  // A Written Report Carries Its Steps Under Their Own Execution
  const payload = reportFixturePayload();

  assert.equal("test_steps" in payload, false);
  for (const item of payload.test_executions) {
    assert.deepEqual(
      item.steps.map((row) => row.record_type),
      ["step", "criterion"],
    );
    assert.ok(item.steps.every((row) => row.test_execution_id === item.test_execution_id));
  }

  // A Written Report Carries Each Artifact Complete With Its Download Target
  assert.ok(payload.snapshot);
  const [artifact] = payload.snapshot.artifacts;

  assert.equal("artifact_links" in payload, false);
  assert.equal(artifact.downloadHref, artifact.href);
  assert.equal(artifact.group_label, "Execution Artifacts");
});

test("Focus Keeps Controls", () => {
  // Focus Drops The Heading But Keeps Start, Stop And A Labelled Exit
  const focused = focusPresentation(true);
  assert.equal(focused.actions, true);
  assert.equal(focused.heading, false);
  assert.equal(focused.toggleLabel, "Leave Focus");

  // Outside Focus The Whole Bar Shows
  assert.deepEqual(focusPresentation(false), {
    actions: true,
    heading: true,
    toggleLabel: "Focus",
  });

  // Only A Frozen Report Has No Actions
  assert.equal(focusPresentation(false, true).actions, false);
});

test("Evidence Pane", () => {
  const busy = Object.assign(new Error("document resolver is busy"), {
    kind: "grpc",
    detail: { code: 8, message: "document resolver is busy", details: [] },
  });
  const failed = Object.assign(new Error("document scope is unknown"), {
    kind: "grpc",
    detail: { code: 5, message: "document scope is unknown", details: [] },
  });
  const budget = Object.assign(new Error("document artifact source budget exhausted"), {
    kind: "grpc",
    detail: { code: 8, message: "document artifact source budget exhausted", details: [] },
  });

  // Busy Refusals Are Recognized By Wording Or Bridge Kind, Not By Status Alone
  assert.ok(isBusyRefusal(busy));
  assert.ok(isBusyRefusal({ kind: "busy" }));
  assert.ok(!isBusyRefusal(failed));
  assert.ok(!isBusyRefusal(budget), "an exhausted budget is a real failure");
  assert.ok(!documentRetry(1, budget));
  assert.ok(!isBusyRefusal(null));

  // A Busy Read Retries With Capped Backoff; A Failure Retries Once
  assert.ok(documentRetry(50, busy), "busy reads keep retrying");
  assert.ok(documentRetry(0, failed));
  assert.ok(!documentRetry(1, failed));
  assert.equal(documentRetryDelay(0, busy), 1_000);
  assert.equal(documentRetryDelay(2, busy), 4_000);
  assert.equal(documentRetryDelay(20, busy), BUSY_RETRY_MAX_MS);

  // A Read Document Stays On Screen Whatever Later Reads Do
  const pane = (fields: Partial<Parameters<typeof evidencePaneState>[0]>) =>
    evidencePaneState({
      hasDocument: false,
      error: null,
      failureReason: null,
      pending: false,
      ...fields,
    });
  assert.deepEqual(pane({ hasDocument: true, failureReason: busy }), {
    kind: "document",
    staleReason: null,
  });
  assert.deepEqual(pane({ hasDocument: true, error: failed }), {
    kind: "document",
    staleReason: "document scope is unknown",
  });

  // Without A Document, Busy Is Pending And Only A Real Failure Is A Notice
  assert.deepEqual(pane({ pending: true, failureReason: busy }), { kind: "busy" });
  assert.deepEqual(pane({ error: busy }), { kind: "busy" });
  assert.deepEqual(pane({ pending: true }), { kind: "loading" });
  assert.deepEqual(pane({ error: failed }), {
    kind: "failed",
    message: "document scope is unknown",
  });
  assert.deepEqual(pane({ error: new Error(" ") }), {
    kind: "failed",
    message: DOCUMENT_FAILED_FALLBACK,
  });
});

test("New Cycle Form", () => {
  const opened = new Date(2026, 9, 16, 14, 5);
  const form = initialNewCycleForm(opened);

  // The Title Opens As The Moment The Dialog Opened, Continuing Past Failures
  assert.equal(form.title, defaultCycleLabel(opened));
  assert.notEqual(form.title, defaultCycleLabel(new Date(2026, 9, 17, 9, 30)));
  assert.equal(form.description, "");
  assert.equal(form.continueOnFailure, true);
  assert.equal(newCycleFormError(form), null);

  // A Blank Title Is Refused
  assert.equal(newCycleFormError({ ...form, title: "   " }), EMPTY_TITLE_ERROR);

  // The Request Trims Its Text And Omits A Blank Description
  assert.deepEqual(
    newCycleRequest({ continueOnFailure: false, description: "  ", title: "  Bench A  " }),
    { continueOnFailure: false, description: undefined, label: "Bench A" },
  );
  assert.deepEqual(
    newCycleRequest({ continueOnFailure: true, description: " Night run ", title: "Bench B" }),
    { continueOnFailure: true, description: "Night run", label: "Bench B" },
  );
});
