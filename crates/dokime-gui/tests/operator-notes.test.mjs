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
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

/* === Constants === */

const gui = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const {
  CLOSED_NOTE_DRAFT,
  noteSubmission,
  notesNewestFirst,
  openNewNote,
  openNoteEdit,
  selectNoteTab,
  updateNoteMarkdown,
} = await import(pathToFileURL(resolve(gui, "src/features/cycle/operator-notes-model.ts")).href);

/* === Fixture === */

function note(id, timestamp, text = `note ${id}`) {
  return { note_id: id, text, timestamp };
}

/* === Tests === */

test("adding a note opens an empty draft on the Code tab", () => {
  const draft = openNewNote();

  assert.equal(draft.mode, "adding");
  assert.equal(draft.tab, "code");
  assert.equal(draft.markdown, "");
  assert.equal(noteSubmission(draft), null, "a blank draft files nothing");
});

test("the tabs switch between Code and Preview and keep the Markdown source", () => {
  const written = updateNoteMarkdown(openNewNote(), "Fan **audible** at start.");
  const preview = selectNoteTab(written, "preview");

  assert.equal(preview.tab, "preview");
  assert.equal(preview.markdown, "Fan **audible** at start.");
  assert.equal(selectNoteTab(preview, "code").tab, "code");
  assert.equal(selectNoteTab(CLOSED_NOTE_DRAFT, "preview"), CLOSED_NOTE_DRAFT);
});

test("adding files the trimmed Markdown as a new note", () => {
  const draft = updateNoteMarkdown(openNewNote(), "  - checked fuses\n- reseated probe  \n");

  assert.deepEqual(noteSubmission(draft), {
    kind: "create",
    markdown: "- checked fuses\n- reseated probe",
  });
  assert.deepEqual(noteSubmission(selectNoteTab(draft, "preview")), noteSubmission(draft));
});

test("editing files only a changed message against the same note", () => {
  const draft = openNoteEdit(note("n1", "2026-10-14T08:00:00Z", "first look"));

  assert.equal(draft.mode, "editing");
  assert.equal(draft.markdown, "first look");
  assert.equal(noteSubmission(draft), null, "an unchanged edit files nothing");
  assert.equal(noteSubmission(updateNoteMarkdown(draft, "first look  ")), null);
  assert.deepEqual(noteSubmission(updateNoteMarkdown(draft, "second look")), {
    kind: "edit",
    noteId: "n1",
    markdown: "second look",
  });
});

test("filed notes list newest first", () => {
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
