/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * The Operator Notes editor as plain state.
 *
 * One draft is open at a time: a new note or an edit of a filed one. The draft
 * keeps its Markdown source and which tab shows it, Code for the source and
 * Preview for the rendered text. Submitting turns the draft into the one call
 * that files or edits a note, or into nothing while the text is blank or
 * unchanged. The module imports types only, so a node test can load it directly.
 */

/* === Local Imports === */

import type { OperatorNote } from "@/lib/types";

/* === Types === */

export type NoteEditorTab = "code" | "preview";

export type NoteDraft =
  | { mode: "closed" }
  | { mode: "adding"; markdown: string; tab: NoteEditorTab }
  | { mode: "editing"; noteId: string; original: string; markdown: string; tab: NoteEditorTab };

export type NoteSubmission = { kind: "create"; markdown: string } | { kind: "edit"; noteId: string; markdown: string };

/* === Constants === */

export const CLOSED_NOTE_DRAFT: NoteDraft = { mode: "closed" };

/* === Public Functions === */

export function openNewNote(): NoteDraft {
  return { mode: "adding", markdown: "", tab: "code" };
}

export function openNoteEdit(note: Pick<OperatorNote, "note_id" | "text">): NoteDraft {
  return {
    mode: "editing",
    noteId: note.note_id,
    original: note.text,
    markdown: note.text,
    tab: "code",
  };
}

export function selectNoteTab(draft: NoteDraft, tab: NoteEditorTab): NoteDraft {
  return draft.mode === "closed" ? draft : { ...draft, tab };
}

export function updateNoteMarkdown(draft: NoteDraft, markdown: string): NoteDraft {
  return draft.mode === "closed" ? draft : { ...draft, markdown };
}

/** The call a draft files, or `null` while it has nothing new to say. */
export function noteSubmission(draft: NoteDraft): NoteSubmission | null {
  if (draft.mode === "closed") return null;
  const markdown = draft.markdown.trim();
  if (!markdown) return null;
  if (draft.mode === "adding") return { kind: "create", markdown };
  if (markdown === draft.original.trim()) return null;
  return { kind: "edit", noteId: draft.noteId, markdown };
}

/** Newest first; an unreadable timestamp sorts last, and a tie puts the later-listed note first. */
export function notesNewestFirst<T extends Pick<OperatorNote, "timestamp">>(notes: T[]): T[] {
  return notes
    .map((note, index) => ({ note, index, at: Date.parse(note.timestamp) }))
    .sort((left, right) => {
      const leftAt = Number.isFinite(left.at) ? left.at : Number.NEGATIVE_INFINITY;
      const rightAt = Number.isFinite(right.at) ? right.at : Number.NEGATIVE_INFINITY;
      return rightAt - leftAt || right.index - left.index;
    })
    .map(({ note }) => note);
}
