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
import { PromptPanel } from "@/components/PromptPanel";
import { ExecutionParameters } from "@/features/cycle/ExecutionParameters";

import {
  executionDocumentSections,
  executionPagePrompts,
  executionParameterColumns,
  executionParameterRows,
  executionSectionOf,
} from "@/features/cycle/execution-sections-model";
import {
  CLOSED_NOTE_DRAFT,
  noteSubmission,
  notesNewestFirst,
  openNewNote,
  openNoteEdit,
  selectNoteTab,
  updateNoteMarkdown,
} from "@/features/cycle/operator-notes-model";
import {
  answerablePrompts,
  answeredPrompts,
  highestPromptLevel,
  orderPendingPrompts,
  pendingPromptCountLabel,
  promptLevelTone,
  queuePrompts,
  type RankedPrompt,
  type ScopedPrompt,
} from "@/features/cycle/prompt-banner-model";
import { durationText } from "@/features/cycle/use-elapsed";
import { advanceFollowCursor, connectingFollowCursor, followTailOffset } from "@/lib/execution-file-paging";
import { durationParts, formatDuration } from "@/lib/format";
import { promptDecision } from "@/lib/prompts";
import type { DocumentViewCard, ExecutionFileIdentity, ParameterDefinition, PromptField, PromptRequest } from "@/lib/types";

const IDENTITY: ExecutionFileIdentity = {
  store: "store",
  epoch: "epoch",
  fileId: "file",
  executionId: "execution",
  relativePath: "obc.charger.steady/stream.000000.log",
} as ExecutionFileIdentity;

type CardKind = DocumentViewCard["kind"];

type PromptLevel = RankedPrompt["level"];

function cards(...kinds: string[]): Pick<DocumentViewCard, "kind">[] {
  return kinds.map((kind) => ({ kind: kind as CardKind }));
}

function note(id: string, timestamp: string, text = `note ${id}`) {
  return { note_id: id, text, timestamp };
}

function prompt(id: string, level: string, createdAt: string): RankedPrompt {
  return { correlation_id: id, created_at: createdAt, level: level as PromptLevel };
}

function bytes(length: number) {
  return Uint8Array.from({ length }, (_, index) => index % 251);
}

test("Page Sections", () => {
  // Built-In Card Kinds Own Their Sections And Every Other Kind Is Report
  assert.equal(executionSectionOf("steps"), "steps-criteria");
  assert.equal(executionSectionOf("criteria"), "steps-criteria");
  assert.equal(executionSectionOf("notes"), "notes");
  assert.equal(executionSectionOf("traceability"), "traceability");
  for (const kind of ["plots", "overview", "table", "health"]) {
    assert.equal(executionSectionOf(kind as CardKind), "report", kind);
  }

  // The Sections Follow Page Order Whatever Order The Document Authored
  const sections = executionDocumentSections(cards("traceability", "plots", "records", "criteria", "media", "notes", "artifacts", "steps"));
  assert.deepEqual(sections, ["steps-criteria", "notes", "media", "artifacts", "report", "records", "traceability"]);

  // A Section The Document Leaves Out Stays Out
  assert.deepEqual(executionDocumentSections(cards("steps", "records")), ["steps-criteria", "records"]);

  // A Prompt Has One Home: The Execution That Asked It, Or The Cycle When No Execution Owns It
  const prompts = [
    { correlation_id: "own", scope: { test_execution_id: "e1" } },
    { correlation_id: "other", scope: { test_execution_id: "e2" } },
    { correlation_id: "cycle", scope: { test_execution_id: null } },
    { correlation_id: "unscoped" },
  ] as unknown as PromptRequest[];
  assert.deepEqual(
    executionPagePrompts(prompts, "e1").map((item) => item.correlation_id),
    ["own"],
  );
  assert.deepEqual(
    executionPagePrompts(prompts, null).map((item) => item.correlation_id),
    ["cycle", "unscoped"],
  );
});

test("Parameter Rows", () => {
  // Parameters Read In Declaration Order With Their Declared Label And Unit
  const definitions = [
    { name: "voltage", label: "Supply Voltage", unit: "V" },
    { name: "current", label: null, unit: " " },
    { name: "absent", label: "Absent", unit: "s" },
  ] as Pick<ParameterDefinition, "name" | "label" | "unit">[];
  const rows = executionParameterRows({ extra: true, current: 16, voltage: 230 }, definitions);
  assert.deepEqual(rows, [
    { key: "voltage", label: "Supply Voltage", value: 230, text: "230", unit: "V" },
    { key: "current", label: "current", value: 16, text: "16", unit: null },
    { key: "extra", label: "extra", value: true, text: "True", unit: null },
  ]);

  // A Case Without Parameters Has No Rows
  assert.deepEqual(executionParameterRows(null, []), []);

  // Durations Read With The Precision Their Value Needs, In The Unit Column
  const durations = executionParameterRows({ dwell_s: 90, outage_s: [0.05, 0.1, 0.2], sample_interval_s: 0.001, soak: 600 }, [
    { name: "soak", label: "Soak Time", unit: "s" },
  ]);
  assert.deepEqual(
    durations.map((row) => [row.label, row.text, row.unit]),
    [
      ["Soak Time", "10", "min"],
      ["dwell_s", "1 min 30", "s"],
      ["outage_s", "50, 100, 200", "ms"],
      ["sample_interval_s", "1", "ms"],
    ],
  );

  // A Native Number Is A Duration Too, Not The Raw Seconds Figure
  const native = (decimal: string) => ({ __dokime_native_number__: true, decimal });
  assert.deepEqual(
    executionParameterRows({ interval: native("0.001"), observe: native("120") }, [
      { name: "interval", label: "Simulation Sample Interval", unit: "s" },
      { name: "observe", label: "Observation Duration", unit: "s" },
    ]).map((row) => [row.text, row.unit]),
    [
      ["1", "ms"],
      ["2", "min"],
    ],
  );

  // Both Halves Share The Figure And Unit Widths Of The Longest Entry
  assert.deepEqual(executionParameterColumns(durations), { value: 12, unit: 3 });
  assert.deepEqual(executionParameterColumns([]), { value: 1, unit: 0 });

  // The Grid Prints Label, Right-Aligned Figure And Dimmed Unit On One Column Template
  const html = renderToStaticMarkup(createElement(ExecutionParameters, { rows: durations }));
  assert.match(html, /--parameter-value:12ch/);
  assert.match(html, /--parameter-unit:3ch/);
  assert.match(
    html,
    /<dt[^>]*title="soak"[^>]*>Soak Time<\/dt><dd class="[^"]*text-right[^"]*">10<\/dd><dd class="text-muted-foreground">min<\/dd>/,
  );
  assert.doesNotMatch(html, /truncate/, "a label wraps, it is never clipped");
});

test("Duration Formatting", () => {
  // One Helper Prints Every Duration With The Precision Its Value Needs
  const cases: Array<[number, string]> = [
    [0, "0 s"],
    [0.000_25, "250 µs"],
    [0.001, "1 ms"],
    [0.05, "50 ms"],
    [0.2, "200 ms"],
    [0.9996, "1 s"],
    [2.5, "2.5 s"],
    [12.34, "12.3 s"],
    [59.7, "1 min"],
    [80, "1 min 20 s"],
    [3_600, "1 h"],
    [3_900, "1 h 5 min"],
    [-0.5, "-500 ms"],
  ];
  for (const [seconds, label] of cases) assert.equal(formatDuration(seconds), label, `${seconds} s`);
  assert.equal(formatDuration(null), null);
  assert.equal(formatDuration(Number.NaN), null);
  assert.deepEqual(durationParts(80), { figure: "1 min 20", unit: "s" });
});

test("Step Durations", () => {
  // A Pending Step Has No Duration, A Running One Ticks Live, An Ended One Is Measured
  assert.equal(durationText({ duration_s: 0, status: "pending" }, true, null), "");
  assert.equal(durationText({ duration_s: 0.1, status: "running" }, true, 12.7), "12 s");
  assert.equal(durationText({ duration_s: 2.5, status: "passed" }, true, null), "2.5 s");

  // A File Never Ticks: Running And Pending Stay Empty, Measured Ones Print
  assert.equal(durationText({ duration_s: 0.1, status: "running" }, false, 12.7), "");
  assert.equal(durationText({ duration_s: 80, status: "failed" }, false, null), "1 min 20 s");
});

test("Operator Notes", () => {
  // Adding A Note Opens An Empty Draft On The Code Tab
  const draft = openNewNote();
  assert.ok(draft.mode === "adding");
  assert.equal(draft.tab, "code");
  assert.equal(draft.markdown, "");
  assert.equal(noteSubmission(draft), null, "a blank draft files nothing");

  // The Tabs Switch Between Code And Preview And Keep The Markdown Source
  const written = updateNoteMarkdown(openNewNote(), "Fan **audible** at start.");
  const preview = selectNoteTab(written, "preview");
  assert.ok(preview.mode === "adding");
  assert.equal(preview.tab, "preview");
  assert.equal(preview.markdown, "Fan **audible** at start.");
  const code = selectNoteTab(preview, "code");
  assert.ok(code.mode === "adding");
  assert.equal(code.tab, "code");
  assert.equal(selectNoteTab(CLOSED_NOTE_DRAFT, "preview"), CLOSED_NOTE_DRAFT);

  // Adding Files The Trimmed Markdown As A New Note
  const list = updateNoteMarkdown(openNewNote(), "  - checked fuses\n- reseated probe  \n");
  assert.deepEqual(noteSubmission(list), {
    kind: "create",
    markdown: "- checked fuses\n- reseated probe",
  });
  assert.deepEqual(noteSubmission(selectNoteTab(list, "preview")), noteSubmission(list));

  // Editing Files Only A Changed Message Against The Same Note
  const editing = openNoteEdit(note("n1", "2026-10-14T08:00:00Z", "first look"));
  assert.ok(editing.mode === "editing");
  assert.equal(editing.markdown, "first look");
  assert.equal(noteSubmission(editing), null, "an unchanged edit files nothing");
  assert.equal(noteSubmission(updateNoteMarkdown(editing, "first look  ")), null);
  assert.deepEqual(noteSubmission(updateNoteMarkdown(editing, "second look")), {
    kind: "edit",
    noteId: "n1",
    markdown: "second look",
  });

  // Filed Notes List Newest First
  const ordered = notesNewestFirst([
    note("old", "2026-10-14T08:00:00Z"),
    note("broken", "not a time"),
    note("new", "2026-10-14T09:00:00Z"),
    note("middle", "2026-10-14T08:30:00Z"),
  ]);
  assert.deepEqual(
    ordered.map((item) => item.note_id),
    ["new", "middle", "old", "broken"],
  );
});

test("File Follow", () => {
  // A Follow Accepts Metadata, Contiguous Chunks And A Matching Terminal
  let cursor = connectingFollowCursor();
  cursor = advanceFollowCursor(cursor, {
    kind: "metadata",
    identity: IDENTITY,
    phase: "producing",
    availableLength: 0,
    offset: 0,
  });
  assert.equal(cursor.phase, "producing");
  cursor = advanceFollowCursor(cursor, { kind: "chunk", offset: 0, data: bytes(200) });
  cursor = advanceFollowCursor(cursor, { kind: "chunk", offset: 200, data: bytes(114) });
  assert.equal(cursor.offset, 314);
  cursor = advanceFollowCursor(cursor, {
    kind: "terminal",
    phase: "finished",
    length: 314,
    sha256: "0".repeat(64),
  });
  assert.equal(cursor.phase, "finished");

  // A Follow Refuses Gaps, Early Data, Short Terminals And A Replaced Identity
  const started = advanceFollowCursor(connectingFollowCursor(), {
    kind: "metadata",
    identity: IDENTITY,
    phase: "producing",
    availableLength: 10,
    offset: 0,
  });
  assert.throws(() => advanceFollowCursor(connectingFollowCursor(), { kind: "chunk", offset: 0, data: bytes(1) }));
  assert.throws(() => advanceFollowCursor(started, { kind: "chunk", offset: 5, data: bytes(1) }));
  assert.throws(() => advanceFollowCursor(started, { kind: "chunk", offset: 0, data: bytes(0) }));
  assert.throws(() => advanceFollowCursor(started, { kind: "terminal", phase: "abandoned", length: 3, sha256: null }));
  assert.throws(() =>
    advanceFollowCursor(connectingFollowCursor(IDENTITY, 10), {
      kind: "metadata",
      identity: { ...IDENTITY, fileId: "other" },
      phase: "producing",
      availableLength: 10,
      offset: 10,
    }),
  );

  // A Large Follow Starts At The Tail And A Small One At Zero
  assert.equal(followTailOffset(314, 64 * 1024), 0);
  assert.equal(followTailOffset(200_000, 64 * 1024), 200_000 - 64 * 1024);
});

test("Prompt Ordering", () => {
  // The Banner Puts The Most Urgent Level First And The Oldest First Within It
  const ordered = orderPendingPrompts([
    prompt("info-old", "info", "2026-10-13T08:00:00Z"),
    prompt("warning-new", "warning", "2026-10-13T08:05:00Z"),
    prompt("error", "error", "2026-10-13T08:10:00Z"),
    prompt("warning-old", "warning", "2026-10-13T08:01:00Z"),
    prompt("debug", "debug", "2026-10-13T07:00:00Z"),
  ]);
  assert.deepEqual(
    ordered.map((item) => item.correlation_id),
    ["error", "warning-old", "warning-new", "info-old", "debug"],
  );

  // Prompts Asked In The Same Instant Keep A Stable Order
  const at = "2026-10-13T08:00:00Z";
  const instant = orderPendingPrompts([prompt("b", "info", at), prompt("a", "info", at)]);
  assert.deepEqual(
    instant.map((item) => item.correlation_id),
    ["a", "b"],
  );

  // An Unreadable Timestamp Sorts Last Within Its Level
  const unreadable = orderPendingPrompts([prompt("broken", "warning", "not a time"), prompt("dated", "warning", "2026-10-13T08:00:00Z")]);
  assert.deepEqual(
    unreadable.map((item) => item.correlation_id),
    ["dated", "broken"],
  );

  // Ordering Leaves The Pending List It Was Given Untouched
  const pending = [prompt("info", "info", "2026-10-13T08:00:00Z"), prompt("error", "error", "2026-10-13T08:00:00Z")];
  orderPendingPrompts(pending);
  assert.deepEqual(
    pending.map((item) => item.correlation_id),
    ["info", "error"],
  );
});

test("Prompt Queue", () => {
  // The Queue Answers In The Order The Run Asked, Whatever The Level
  const queued = queuePrompts([
    prompt("error-new", "error", "2026-10-13T08:10:00Z"),
    prompt("info-old", "info", "2026-10-13T08:00:00Z"),
    prompt("b-same", "warning", "2026-10-13T08:05:00Z"),
    prompt("a-same", "warning", "2026-10-13T08:05:00Z"),
  ]);
  assert.deepEqual(
    queued.map((item) => item.correlation_id),
    ["info-old", "a-same", "b-same", "error-new"],
  );

  // Answered Prompts Read Back From Their Records: Question, Answer, Time
  const records = [
    {
      details: { fields: [{ label: "Measured Voltage", name: "voltage" }], prompt_id: "p1", title: "Read The Meter" },
      record_type: "prompt.request.form",
      timestamp: "2026-10-13T08:00:00Z",
    },
    {
      details: { action: "submit", prompt_id: "p1", values: { voltage: 230 } },
      record_type: "prompt.response.submitted",
      timestamp: "2026-10-13T08:01:00Z",
    },
    {
      details: { prompt_id: "p2", title: "Confirm Wiring" },
      record_type: "prompt.request.confirmation",
      timestamp: "2026-10-13T08:02:00Z",
    },
    {
      details: { action: "confirm", prompt_id: "p2", values: {} },
      record_type: "prompt.response.submitted",
      timestamp: "2026-10-13T08:03:00Z",
    },
    { details: { prompt_id: "p3", title: "Too Late" }, record_type: "prompt.request.form", timestamp: "2026-10-13T08:04:00Z" },
    { details: { prompt_id: "p3" }, record_type: "prompt.response.expired", timestamp: "2026-10-13T08:05:00Z" },
    { details: { prompt_id: "p4", title: "Still Open" }, record_type: "prompt.request.form", timestamp: "2026-10-13T08:06:00Z" },
  ];
  assert.deepEqual(answeredPrompts(records), [
    { answer: "Measured Voltage: 230", at: "2026-10-13T08:01:00Z", promptId: "p1", question: "Read The Meter" },
    { answer: "Confirm", at: "2026-10-13T08:03:00Z", promptId: "p2", question: "Confirm Wiring" },
    { answer: "Expired", at: "2026-10-13T08:05:00Z", promptId: "p3", question: "Too Late" },
  ]);
});

test("Prompt Decisions", () => {
  const field = (name: string, kind: PromptField["kind"], patch: Partial<PromptField> = {}) =>
    ({ kind, label: name, name, ...patch }) as PromptField;
  const action = (id: string, emphasis = "primary") => ({ emphasis, id, label: id }) as PromptRequest["actions"][number];
  const ask = (fields: PromptField[], actions = [action("cancel", "secondary"), action("submit")]) =>
    ({
      actions,
      correlation_id: "p",
      created_at: "2026-10-13T08:00:00Z",
      fields,
      kind: "form",
      level: "info",
      title: "Inspect The Charger",
    }) as unknown as PromptRequest;

  // Exactly One Boolean And One Answering Action Make The Prompt A Decision
  const inspection = ask([field("passed", "boolean", { label: "Charger Inspection Passed" }), field("note", "text", { required: true })]);
  assert.deepEqual(promptDecision(inspection), { action: "submit", field: inspection.fields[0] });
  assert.equal(promptDecision(ask([field("a", "boolean"), field("b", "boolean")])), null);
  assert.equal(promptDecision(ask([field("note", "text")])), null);
  assert.equal(promptDecision(ask([field("a", "boolean")], [action("retry"), action("submit")])), null);

  // A Decision Answers With No And Yes, Not A Checkbox, Cancel At The Far Left
  const html = renderToStaticMarkup(
    createElement(PromptPanel, { onAction: () => undefined, onValueChange: () => undefined, prompt: inspection, values: {} }),
  );
  assert.doesNotMatch(html, /role="checkbox"/);
  assert.ok(html.indexOf(">cancel<") < html.indexOf(">No<") && html.indexOf(">No<") < html.indexOf(">Yes<"));

  // An Untouched Field States No Error Before A Submit Attempt
  assert.doesNotMatch(html, /Enter a value/);

  // Several Booleans Keep Their Controls And Submit
  const several = renderToStaticMarkup(
    createElement(PromptPanel, {
      onAction: () => undefined,
      onValueChange: () => undefined,
      prompt: ask([field("a", "boolean"), field("b", "boolean")]),
      values: {},
    }),
  );
  assert.match(several, /role="checkbox"/);
  assert.match(several, />submit</);

  // The Answered Row States A Decision As Yes Or No
  assert.equal(
    answeredPrompts([
      {
        details: { fields: [{ label: "Charger Inspection Passed", name: "passed" }], prompt_id: "d", title: "Inspect" },
        record_type: "prompt.request.form",
        timestamp: "2026-10-13T08:00:00Z",
      },
      {
        details: { action: "submit", prompt_id: "d", values: { passed: false } },
        record_type: "prompt.response.submitted",
        timestamp: "2026-10-13T08:01:00Z",
      },
    ])[0].answer,
    "Charger Inspection Passed: No",
  );
});

test("Prompt Answering", () => {
  // A Count Is Coloured By Its Most Urgent Prompt And Reads As A Phrase
  assert.equal(highestPromptLevel([]), null);
  assert.equal(
    highestPromptLevel([prompt("info", "info", "2026-10-13T08:00:00Z"), prompt("warning", "warning", "2026-10-13T08:00:00Z")]),
    "warning",
  );
  assert.equal(pendingPromptCountLabel(1), "1 prompt waiting");
  assert.equal(pendingPromptCountLabel(3), "3 prompts waiting");

  // The Tone Follows The Record Severity Scale With Info Kept Visible
  const levels = ["error", "warning", "info", "debug"] as PromptLevel[];
  assert.deepEqual(levels.map(promptLevelTone), ["danger", "warning", "info", "muted"]);

  // A Prompt Is Answerable Only While Its Owner Runs
  type Owned = ScopedPrompt & { correlation_id: string };
  const scoped = (id: string, executionId: string): Owned => ({
    correlation_id: id,
    scope: { test_execution_id: executionId },
  });
  const pending: Owned[] = [scoped("running", "one"), scoped("interrupted", "two"), { correlation_id: "cycle" }];
  const owners = (cycleEnded: boolean) => ({
    cycleEnded,
    executionActive: (executionId: string) => executionId === "one",
  });
  assert.deepEqual(
    answerablePrompts(pending, owners(false)).map((item) => item.correlation_id),
    ["running", "cycle"],
  );
  assert.deepEqual(
    answerablePrompts(pending, owners(true)).map((item) => item.correlation_id),
    ["running"],
  );
});
