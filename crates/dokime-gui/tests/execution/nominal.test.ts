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
  executionDocumentSections,
  executionPagePrompts,
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
import {
  advanceFollowCursor,
  connectingFollowCursor,
  EXECUTION_FILE_WINDOW_BYTES,
  followTailOffset,
  pageExecutionFile,
} from "@/lib/execution-file-paging";
import type {
  DocumentViewCard,
  ExecutionFileIdentity,
  ExecutionFileLifecycle,
  ExecutionFileReadWindow,
  ParameterDefinition,
  PromptRequest,
} from "@/lib/types";

/* === Fixture === */

type CardKind = DocumentViewCard["kind"];
type PromptLevel = RankedPrompt["level"];

const IDENTITY: ExecutionFileIdentity = {
  store: "store",
  epoch: "epoch",
  fileId: "file",
  executionId: "execution",
  relativePath: "mini.charger.steady/stream.log",
} as ExecutionFileIdentity;

function cards(...kinds: string[]): Pick<DocumentViewCard, "kind">[] {
  return kinds.map((kind) => ({ kind: kind as CardKind }));
}

function note(id: string, timestamp: string, text = `note ${id}`) {
  return { note_id: id, text, timestamp };
}

function prompt(id: string, level: string, createdAt: string): RankedPrompt {
  return { correlation_id: id, created_at: createdAt, level: level as PromptLevel };
}

/** A server that refuses any window over the contract bound, as the backend does. */
function boundedServer(
  content: Uint8Array,
  { state = "finalized", shrinkTo = Number.POSITIVE_INFINITY } = {},
) {
  const requests: { offset: number; maxBytes: number }[] = [];
  const read = async (offset: number, maxBytes: number): Promise<ExecutionFileReadWindow> => {
    requests.push({ offset, maxBytes });
    if (maxBytes < 1 || maxBytes > EXECUTION_FILE_WINDOW_BYTES) {
      throw new Error("max_bytes must be between 0 and 65536");
    }
    const start = Math.min(offset, content.byteLength);
    const end = Math.min(content.byteLength, start + Math.min(maxBytes, shrinkTo));
    return {
      offset: start,
      observedSizeBytes: content.byteLength,
      data: content.subarray(start, end),
      state: state as ExecutionFileLifecycle,
    } as ExecutionFileReadWindow;
  };
  return { read, requests };
}

function bytes(length: number) {
  return Uint8Array.from({ length }, (_, index) => index % 251);
}

/* === Tests === */

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
  const sections = executionDocumentSections(
    cards("traceability", "plots", "records", "criteria", "media", "notes", "artifacts", "steps"),
  );
  assert.deepEqual(sections, [
    "steps-criteria",
    "notes",
    "media",
    "artifacts",
    "report",
    "records",
    "traceability",
  ]);

  // A Section The Document Leaves Out Stays Out
  assert.deepEqual(executionDocumentSections(cards("steps", "records")), [
    "steps-criteria",
    "records",
  ]);

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
    { key: "voltage", label: "Supply Voltage", value: 230, unit: "V" },
    { key: "current", label: "current", value: 16, unit: null },
    { key: "extra", label: "extra", value: true, unit: null },
  ]);

  // A Case Without Parameters Has No Rows
  assert.deepEqual(executionParameterRows(null, []), []);
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

test("File Paging", async () => {
  // A Small File Loads In One Request
  const small = new TextEncoder().encode("x".repeat(314));
  const streaming = boundedServer(small, { state: "streaming" });
  const smallPage = await pageExecutionFile(streaming.read, { budgetBytes: 256 * 1024 });
  assert.equal(streaming.requests.length, 1);
  assert.equal(smallPage.stop, "complete");
  assert.equal(smallPage.nextOffset, 314);
  assert.equal(smallPage.state, "streaming");
  assert.deepEqual(smallPage.data, small);

  // A File Larger Than The Bound Is Read In Bounded Pages With Offset Continuation
  const large = bytes(3 * EXECUTION_FILE_WINDOW_BYTES + 17);
  const bounded = boundedServer(large);
  const largePage = await pageExecutionFile(bounded.read, { budgetBytes: 256 * 1024 });
  assert.equal(largePage.stop, "complete");
  assert.equal(bounded.requests.length, 4);
  assert.ok(bounded.requests.every(({ maxBytes }) => maxBytes <= EXECUTION_FILE_WINDOW_BYTES));
  assert.deepEqual(
    bounded.requests.map(({ offset }) => offset),
    [0, 1, 2, 3].map((index) => index * EXECUTION_FILE_WINDOW_BYTES),
  );
  assert.deepEqual(largePage.data, large);

  // Continuation Follows The Bytes The Server Actually Returned
  const shrinking = bytes(10_000);
  const short = boundedServer(shrinking, { shrinkTo: 4_000 });
  const shortPage = await pageExecutionFile(short.read, { budgetBytes: 256 * 1024 });
  assert.deepEqual(
    short.requests.map(({ offset }) => offset),
    [0, 4_000, 8_000],
  );
  assert.deepEqual(shortPage.data, shrinking);

  // The Budget Caps Retained Bytes And The Last Request
  const budgeted = boundedServer(bytes(5 * EXECUTION_FILE_WINDOW_BYTES));
  const budgetPage = await pageExecutionFile(budgeted.read, { budgetBytes: 100_000 });
  assert.equal(budgetPage.stop, "budget");
  assert.equal(budgetPage.data.byteLength, 100_000);
  assert.equal(budgeted.requests.at(-1)?.maxBytes, 100_000 - EXECUTION_FILE_WINDOW_BYTES);

  // An Empty Window Before The Observed End Stops Instead Of Spinning
  let calls = 0;
  const stalled = await pageExecutionFile(
    async (offset) => {
      calls += 1;
      return {
        offset,
        observedSizeBytes: 314,
        data: new Uint8Array(),
        state: "streaming",
      } as ExecutionFileReadWindow;
    },
    { budgetBytes: 256 * 1024 },
  );
  assert.equal(calls, 1);
  assert.equal(stalled.stop, "stalled");

  // An Empty File Is Complete After One Request
  const empty = boundedServer(new Uint8Array());
  const emptyPage = await pageExecutionFile(empty.read, { budgetBytes: 256 * 1024 });
  assert.equal(empty.requests.length, 1);
  assert.equal(emptyPage.stop, "complete");
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
  assert.throws(() =>
    advanceFollowCursor(connectingFollowCursor(), { kind: "chunk", offset: 0, data: bytes(1) }),
  );
  assert.throws(() => advanceFollowCursor(started, { kind: "chunk", offset: 5, data: bytes(1) }));
  assert.throws(() => advanceFollowCursor(started, { kind: "chunk", offset: 0, data: bytes(0) }));
  assert.throws(() =>
    advanceFollowCursor(started, { kind: "terminal", phase: "abandoned", length: 3, sha256: null }),
  );
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
  const unreadable = orderPendingPrompts([
    prompt("broken", "warning", "not a time"),
    prompt("dated", "warning", "2026-10-13T08:00:00Z"),
  ]);
  assert.deepEqual(
    unreadable.map((item) => item.correlation_id),
    ["dated", "broken"],
  );

  // Ordering Leaves The Pending List It Was Given Untouched
  const pending = [
    prompt("info", "info", "2026-10-13T08:00:00Z"),
    prompt("error", "error", "2026-10-13T08:00:00Z"),
  ];
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
    highestPromptLevel([
      prompt("info", "info", "2026-10-13T08:00:00Z"),
      prompt("warning", "warning", "2026-10-13T08:00:00Z"),
    ]),
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
  const pending: Owned[] = [
    scoped("running", "one"),
    scoped("interrupted", "two"),
    { correlation_id: "cycle" },
  ];
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
