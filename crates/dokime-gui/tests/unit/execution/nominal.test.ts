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
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test } from "vitest";
import { ExecutionParameters } from "@/features/cycle/ExecutionParameters";
// Local Imports
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
  highestPromptLevel,
  orderPendingPrompts,
  pendingPromptCountLabel,
  promptLevelTone,
  type RankedPrompt,
  type ScopedPrompt,
} from "@/features/cycle/prompt-banner-model";
import { advanceFollowCursor, connectingFollowCursor, followTailOffset } from "@/lib/execution-file-paging";
import { durationParts, formatDuration } from "@/lib/format";
import type { DocumentViewCard, ExecutionFileIdentity, ParameterDefinition, PromptRequest } from "@/lib/types";

/* === Re-Exports === */

/* === Constants === */

const IDENTITY: ExecutionFileIdentity = {
  store: "store",
  epoch: "epoch",
  fileId: "file",
  executionId: "execution",
  relativePath: "obc.charger.steady/stream.log",
} as ExecutionFileIdentity;

/* === Types === */

type CardKind = DocumentViewCard["kind"];

type PromptLevel = RankedPrompt["level"];

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

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

/* === Entry Point === */

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

  // The Execution Page Answers Its Own Prompts And The Cycle's, Not Another Execution's
  const prompts = [
    { correlation_id: "own", scope: { test_execution_id: "e1" } },
    { correlation_id: "other", scope: { test_execution_id: "e2" } },
    { correlation_id: "cycle", scope: { test_execution_id: null } },
    { correlation_id: "unscoped" },
  ] as unknown as PromptRequest[];
  assert.deepEqual(
    executionPagePrompts(prompts, "e1").map((item) => item.correlation_id),
    ["own", "cycle", "unscoped"],
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
