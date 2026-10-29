/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import assert from "node:assert/strict";

import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test } from "vitest";
import { executionControlState, inputNeededLabel } from "@/features/cycle/control-model";

import {
  archiveRefusal,
  buildCycleExecutionRows,
  CYCLE_STATE_LABELS,
  type CycleCase,
  type CycleSelection,
  cycleRowSelection,
  cycleState,
  followSelection,
  isCycleTerminal,
  isFollowing,
  isUnstartedExecution,
  nodeMenuGroups,
  reportDownloadState,
  reportFilename,
  rowOffersRunAgain,
  selectedCycleRow,
} from "@/features/cycle/cycle-workspace-model";
import {
  DOCUMENT_FAILED_FALLBACK,
  documentRetry,
  documentRetryDelay,
  evidencePaneState,
  isBusyRefusal,
  RECONNECT_MAX_MS,
} from "@/features/cycle/document-query-model";
import {
  CLOSED_LABEL_DRAFT,
  EMPTY_LABEL_ERROR,
  labelEditorKey,
  openLabelEdit,
  refuseLabel,
  updateLabelText,
} from "@/features/cycle/label-editor-model";
import { cycleSettingsReadOnly, SettingsFields } from "@/features/cycle/SettingsForm";
import { CycleStatusPill } from "@/features/cycles-home/CycleStatusPill";
import { archivedCycleGroups, type CycleRow, groupCycleRows } from "@/features/cycles-home/cycles-model";
import { EMPTY_TITLE_ERROR, initialNewCycleForm, newCycleFormError, newCycleRequest } from "@/features/cycles-home/new-cycle-form";
import { sameFrontiers } from "@/lib/api/native-documents";
import { appendRecordTail, RECORD_TAIL } from "@/lib/api/native-evidence";
import { defaultCycleLabel } from "@/lib/time-format";
import type { HubRecord } from "@/lib/types";
import { ROW_WINDOW_THRESHOLD, rowWindow } from "@/lib/use-row-window";
import { reportFixturePayload } from "@/report/report-fixture";
import { focusPresentation } from "@/shell/focus-mode";

type Execution = Parameters<typeof buildCycleExecutionRows>[1][number];

type ControlInput = Parameters<typeof executionControlState>[0];

type RunControl = ControlInput["controls"];

type FollowInput = Parameters<typeof followSelection>[0];

function cases(...ids: string[]): CycleCase[] {
  return ids.map((caseId) => ({ caseId }) as CycleCase);
}

function execution(fields: Record<string, unknown>): Execution {
  return fields as unknown as Execution;
}

test("Active Containers Persist", () => {
  const rows = [
    { cycleId: "finished-run", phase: "Active", status: "Completed", attentionCount: 0 },
    { cycleId: "saving-evidence", phase: "Sealing", status: "Stopped", attentionCount: 0 },
    { cycleId: "history", phase: "Sealed", status: "Completed", attentionCount: 0 },
  ] as CycleRow[];
  const groups = groupCycleRows(rows);
  assert.deepEqual(
    groups.find((group) => group.id === "active")?.rows.map((row) => row.cycleId),
    ["finished-run", "saving-evidence"],
  );
  assert.deepEqual(
    groups.find((group) => group.id === "recent")?.rows.map((row) => row.cycleId),
    ["history"],
  );
});

test("Cycle State", () => {
  // Stopping One Execution Leaves Its Active Cycle Idle
  assert.deepEqual(cycleState("active", "stopped", false), { label: "Active · Idle", variant: "outline" });
  assert.equal(cycleState("active", "completed", false).label, "Active · Idle");
  assert.equal(cycleState("active", "planned", false).label, "Active · Idle");

  // Running Covers Every Lifecycle That Still Produces Evidence
  for (const lifecycle of ["running", "paused", "stopping"] as const) {
    assert.equal(cycleState("active", lifecycle, false).label, "Active · Running");
  }

  // An Answerable Prompt Means The Cycle Waits On The Operator
  assert.equal(cycleState("active", "running", true).label, "Active · Waiting");

  // The Closing Phases Name Only Themselves
  assert.equal(cycleState("sealing", "stopping", true).label, "Sealing");
  assert.equal(cycleState("sealed", "failed", false).label, "Sealed");
});

test("Status Pill", () => {
  // The Longest State Label Renders Whole, On A Track Every Label Sizes
  const longest = CYCLE_STATE_LABELS.reduce((wide, label) => (label.length > wide.length ? label : wide));
  const markup = renderToStaticMarkup(createElement(CycleStatusPill, { label: longest, tone: "good" }));
  const badges = [...markup.matchAll(/<span ([^>]*data-slot="badge"[^>]*)>([^<]*)<\/span>/g)];
  const visible = badges.filter(([, attributes]) => !attributes.includes("aria-hidden"));
  assert.deepEqual(
    visible.map(([, , text]) => text),
    [longest],
    "the visible pill states the full label",
  );
  const sizers = [...markup.matchAll(/aria-hidden="true"[^>]*>([^<]*)</g)].map((match) => match[1]);
  assert.deepEqual(sizers, [...CYCLE_STATE_LABELS], "every label sizes the shared track");
  assert.doesNotMatch(markup, /pill-width|truncate|text-ellipsis/, "no fixed width or ellipsis can clip a label");
  assert.match(markup, /inline-grid/);
  assert.match(markup, /w-full \[grid-area:1\/1\]/, "the visible pill fills the widest label's track");
});

test("Live Data Flow", () => {
  // A Record Tail Appends From Its Stream, Keeps The Latest And Never Duplicates
  const record = (id: number) => ({ record_id: `r${id}` }) as HubRecord;
  const cursor = (position: number) => ({ epoch: "e", position: BigInt(position), stream: "s" });
  const tail = { cursor: cursor(2), records: [record(1), record(2)] };
  const appended = appendRecordTail(tail, [record(2), record(3)], cursor(3));
  assert.deepEqual(
    appended.records.map((item) => item.record_id),
    ["r1", "r2", "r3"],
  );
  assert.equal(appended.cursor?.position, 3n);
  assert.equal(appendRecordTail(tail, [record(1)], null).records, tail.records, "a page of held records leaves the tail as it was");
  const full = { cursor: cursor(1), records: Array.from({ length: RECORD_TAIL }, (_, index) => record(index)) };
  const capped = appendRecordTail(full, [record(RECORD_TAIL)], cursor(2));
  assert.equal(capped.records.length, RECORD_TAIL);
  assert.equal(capped.records.at(-1)?.record_id, `r${RECORD_TAIL}`);

  // A Capture With The Held Frontiers Is The Held Document
  const frontiers = { artifactsSha256: "a", processEpoch: "p", stateRevision: 7n } as never;
  assert.ok(sameFrontiers(frontiers, { ...(frontiers as object) } as never));
  assert.ok(!sameFrontiers(frontiers, { ...(frontiers as object), stateRevision: 8n } as never));
  assert.ok(!sameFrontiers(undefined, frontiers));

  // Short Lists Render Whole; Long Ones Render The Viewport And Its Overscan
  assert.deepEqual(rowWindow(50, 28, 0, 600), { after: 0, before: 0, first: 0, last: 50 });
  assert.equal(rowWindow(ROW_WINDOW_THRESHOLD, null, 0, 600).last, 64, "a long list renders a first slice before its pitch is known");
  const scrolled = rowWindow(1_000, 28, 28 * 500, 280);
  assert.equal(scrolled.first, 484);
  assert.equal(scrolled.last, 526);
  assert.equal(scrolled.before, 484 * 28);
  assert.equal(scrolled.after, (1_000 - 526) * 28);
  assert.equal(rowWindow(1_000, 28, 28 * 2_000, 280).last, 1_000, "the window never runs past the end");
});

test("Settings Dialogs", () => {
  const render = (props: Parameters<typeof SettingsFields>[0]) => renderToStaticMarkup(createElement(SettingsFields, props));
  const cycle = { continueAutomatically: true, description: "Bench run", title: "Test Cycle" };

  // An Active Cycle Edits Its Title And Continuation; Its Description Was Written At Creation
  const active = render({ onChange: () => undefined, readOnly: cycleSettingsReadOnly("active", false), values: cycle });
  assert.match(active, /Title/);
  assert.match(active, /Continue Automatically/);
  assert.doesNotMatch(active, /Continue after a failed case/);
  assert.equal((active.match(/ disabled=""/g) ?? []).length, 1, "only the description is read-only");
  assert.match(active, /Set when the cycle was created/);

  // A Sealed Cycle Shows Its Settings Read-Only
  const sealed = render({ onChange: () => undefined, readOnly: cycleSettingsReadOnly("sealed", false), values: cycle });
  assert.ok((sealed.match(/ disabled=""/g) ?? []).length >= 3, "title, description and continuation are all read-only");
  assert.match(sealed, /role="switch"[^>]* disabled=""/);
  assert.match(sealed, /Sealed cycles keep the settings they ran with/);

  // An Execution's Settings Are Its Title Alone, The Same Form
  const execution = render({ onChange: () => undefined, values: { title: "Charger Start-Up" } });
  assert.match(execution, /Title/);
  assert.doesNotMatch(execution, /Description|Continue Automatically/);
});

test("Node Menus", () => {
  // Cycle Menu Opens Its Report And Archives, With No Bundle, Folder Or Clearing
  const items: string[] = nodeMenuGroups("cycle", false).flat();
  assert.deepEqual(nodeMenuGroups("cycle", false), [
    ["Copy Cycle", "Add Cases", "Follow Active"],
    ["Open Report", "Settings"],
    ["Archive"],
  ]);
  for (const removed of ["Download Bundle", "Write Report", "Open Folder", "Clear Cycle"]) {
    assert.ok(!items.includes(removed), removed);
  }

  // Execution Menu Offers Re-Run, Open Report, Settings And Archive
  assert.deepEqual(nodeMenuGroups("execution", false), [["Re-Run"], ["Open Report", "Settings"], ["Archive"]]);

  // An Archived Node Offers Unarchive In Place Of Archive
  assert.deepEqual(nodeMenuGroups("cycle", true).at(-1), ["Unarchive"]);
  assert.deepEqual(nodeMenuGroups("execution", true).at(-1), ["Unarchive"]);

  // Archiving Waits For A Sealed Cycle Or An Ended Run And Says So
  assert.equal(archiveRefusal(true, "cycle"), undefined);
  assert.match(archiveRefusal(false, "cycle") ?? "", /once the cycle is sealed/);
  assert.match(archiveRefusal(false, "execution") ?? "", /once the run has ended/);

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
  assert.equal(reportFilename("Execution 1 · OBC / Reconnection"), "Execution 1 · OBC - Reconnection.html");
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
  const input = (patch: Partial<ControlInput>): ControlInput => ({
    controls: null,
    progress: null,
    promptWaiting: false,
    status: "running",
    stopping: false,
    ...patch,
  });
  const running = {
    canPause: false,
    canResume: false,
    canStop: true,
    mode: null,
    paused: false,
    requestId: "r",
    stopRequested: false,
  } as unknown as NonNullable<RunControl>;

  // An Unstarted Attempt Offers Start And Cancel, Never Stop
  for (const status of ["planned", "created"] as const) {
    const state = executionControlState(input({ status }));
    assert.deepEqual(state.actions, ["Start", "Cancel"]);
    assert.doesNotMatch(state.line, /No step in progress/);
  }
  assert.equal(executionControlState(input({ status: "planned" })).line, "Not started · waits for Start");

  // Stop Shows Only While An Attempt Runs, After Pause When It Pauses
  assert.deepEqual(executionControlState(input({ controls: running })).actions, ["Stop"]);
  assert.deepEqual(executionControlState(input({ controls: { ...running, canPause: true } })).actions, ["Pause", "Stop"]);
  assert.deepEqual(executionControlState(input({ controls: { ...running, canResume: true, paused: true } })), {
    actions: ["Resume", "Stop"],
    detail: null,
    line: "Paused",
    phase: "paused",
  });
  assert.deepEqual(executionControlState(input({})).actions, []);

  // The Running Step Reads By Its Label, Its Id Muted After It, Its Count Beside The Bar
  const step = executionControlState(
    input({ controls: running, progress: { completed: 2, id: "publish", label: "Publish Evidence", total: 5 } }),
  );
  assert.equal(step.line, "Publish Evidence");
  assert.equal(step.detail, "publish");

  // A Waiting Prompt Is Named, Not Shown, In Control
  assert.equal(executionControlState(input({ controls: running, promptWaiting: true })).line, "Waits on the prompt below");

  // Runtime Finalization Offers Nothing; A Completed Step Counter Does Not End The Run.
  const settled = executionControlState(input({ controls: { ...running, canPause: true }, status: "finalizing" }));
  assert.deepEqual(settled, { actions: [], detail: null, line: "Finalizing · saving evidence", phase: "finalizing" });
  assert.equal(
    executionControlState(input({ controls: running, progress: { completed: 5, id: "s", label: "S", total: 5 } })).phase,
    "running",
  );
  assert.equal(executionControlState(input({ controls: running, status: "finalizing" })).phase, "finalizing");
  assert.equal(executionControlState(input({ controls: running, status: "passed" })).phase, "finalizing");
  const waiting = executionControlState(
    input({ controls: running, progress: { completed: 5, id: "s", label: "S", total: 5 }, promptWaiting: true }),
  );
  assert.equal(waiting.phase, "waiting");
  assert.deepEqual(waiting.actions, ["Stop"]);

  // A Stop In Flight Offers Nothing Further
  assert.equal(executionControlState(input({ controls: running, stopping: true })).line, "Stopping · saving evidence");

  // An Ended Attempt Offers Re-Run
  assert.deepEqual(executionControlState(input({ status: "passed" })).actions, ["Re-Run"]);
  assert.equal(executionControlState(input({ status: "failed" })).line, "Ended · Failed");

  // The Cycle Names The Executions Waiting On Input
  assert.equal(inputNeededLabel([1, 3]), "Input needed on #1, #3");
  assert.equal(inputNeededLabel([]), null);
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
  const lifecycles = ["completed", "failed", "stopped", "interrupted", "running", "stopping", "planned"] as Parameters<
    typeof isCycleTerminal
  >[0][];
  assert.deepEqual(lifecycles.map(isCycleTerminal), [true, true, true, true, false, false, false]);
});

test("Reloaded Execution Selection", () => {
  const selected: FollowInput = {
    active: "running",
    previousActive: null,
    selection: { executionId: "finished" },
    selectedStatus: undefined,
    previousSelected: null,
  };
  assert.equal(followSelection(selected), null);
  assert.equal(
    followSelection({ ...selected, selectedStatus: "passed", previousSelected: { executionId: "finished", status: undefined } }),
    null,
  );
  assert.equal(
    followSelection({ ...selected, selectedStatus: "passed", previousSelected: { executionId: "finished", status: "passed" } }),
    null,
  );
  assert.equal(
    followSelection({ ...selected, selectedStatus: "passed", previousSelected: { executionId: "another", status: "running" } }),
    null,
  );
  assert.equal(
    followSelection({ ...selected, selectedStatus: "passed", previousSelected: { executionId: "finished", status: "running" } }),
    "running",
  );
});

test("Follow Active", () => {
  // An Explicit Selection Beats Follow Active Until The Operator Returns To It
  const follow = (selection: CycleSelection, extra: Partial<FollowInput> = {}) =>
    followSelection({
      active: "ten",
      previousActive: "ten",
      selectedStatus: undefined,
      previousSelected: null,
      selection,
      ...extra,
    });

  assert.equal(follow({}), "ten", "landing follows the running execution");
  assert.equal(follow({ executionId: "twelve" }), null, "a click on a queued row stays there");
  assert.equal(follow({ view: "cycle" }), null, "the cycle is a selection of its own");
  assert.equal(follow({ caseId: "unrun" }), null);
  assert.equal(follow({ executionId: "ten" }), null, "already on the active one");
  assert.equal(follow({ executionId: "ten" }, { active: "eleven" }), "eleven", "a following selection moves on with the runtime");
  assert.equal(
    follow(
      { executionId: "three" },
      { active: "eleven", selectedStatus: "passed", previousSelected: { executionId: "three", status: "running" } },
    ),
    "eleven",
    "the selected execution ending resumes following",
  );
  assert.equal(
    followSelection({ active: null, previousActive: null, selectedStatus: undefined, previousSelected: null, selection: {} }),
    null,
  );

  // Following Reads The Selection Against The Active Execution
  assert.equal(isFollowing({}, "ten"), true);
  assert.equal(isFollowing({ executionId: "ten" }, "ten"), true);
  assert.equal(isFollowing({ executionId: "twelve" }, "ten"), false);
  assert.equal(isFollowing({ view: "cycle" }, "ten"), false);
  const statuses = ["planned", "created", "running", "passed"] as Parameters<typeof isUnstartedExecution>[0][];
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
  assert.equal(documentRetryDelay(20, busy), RECONNECT_MAX_MS);

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

  // The Title Opens As The Moment The Dialog Opened
  assert.equal(form.title, defaultCycleLabel(opened));
  assert.notEqual(form.title, defaultCycleLabel(new Date(2026, 9, 17, 9, 30)));
  assert.equal(form.description, "");
  assert.equal(form.continueAutomatically, false);
  assert.equal(newCycleFormError(form), null);

  // A Blank Title Is Refused
  assert.equal(newCycleFormError({ ...form, title: "   " }), EMPTY_TITLE_ERROR);

  // The Request Trims Its Text And Omits A Blank Description
  assert.deepEqual(newCycleRequest({ continueAutomatically: false, description: "  ", title: "  Bench A  " }), {
    continueAutomatically: false,
    description: undefined,
    label: "Bench A",
  });
  assert.deepEqual(newCycleRequest({ continueAutomatically: true, description: " Night run ", title: "Bench B" }), {
    continueAutomatically: true,
    description: "Night run",
    label: "Bench B",
  });
});
