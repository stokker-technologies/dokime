/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { Pencil, Plus } from "lucide-react";
import { type ReactNode, useState } from "react";

/* === Local Imports === */

import {
  DocumentEmbeddedBlock,
  DocumentEmbeddedEmptyState,
} from "@/components/document/DocumentEmbeddedContent";
import { MarkdownText } from "@/components/MarkdownText";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import {
  CLOSED_NOTE_DRAFT,
  type NoteDraft,
  type NoteEditorTab,
  noteSubmission,
  notesNewestFirst,
  openNewNote,
  openNoteEdit,
  selectNoteTab,
  updateNoteMarkdown,
} from "@/features/cycle/operator-notes-model";
import { useAddNoteMutation, useEditNoteMutation } from "@/features/cycle/server-state";
import { timeOfDayLabel } from "@/lib/time-format";
import type { OperatorNote, RecordScope } from "@/lib/types";
import { cn } from "@/lib/utils";

/* === Types === */

type OperatorNotesContentProps = {
  notes: OperatorNote[];
  /** The unsealed cycle or execution a new note is filed against; absent is read-only. */
  scope?: RecordScope | null;
};

/* === Constants === */

const TABS: { id: NoteEditorTab; label: string }[] = [
  { id: "code", label: "Code" },
  { id: "preview", label: "Preview" },
];

/* === Components === */

export function OperatorNotesContent({ notes, scope = null }: OperatorNotesContentProps) {
  const ordered = notesNewestFirst(notes);
  if (!scope) {
    return ordered.length ? (
      <NoteList notes={ordered} />
    ) : (
      <DocumentEmbeddedEmptyState>No operator notes recorded.</DocumentEmbeddedEmptyState>
    );
  }
  return <LiveOperatorNotes notes={ordered} scope={scope} />;
}

function LiveOperatorNotes({ notes, scope }: { notes: OperatorNote[]; scope: RecordScope }) {
  const [draft, setDraft] = useState<NoteDraft>(CLOSED_NOTE_DRAFT);
  const addNote = useAddNoteMutation();
  const editNote = useEditNoteMutation();
  const pending = addNote.isPending || editNote.isPending;
  const failure = addNote.error ?? editNote.error;

  const close = () => {
    addNote.reset();
    editNote.reset();
    setDraft(CLOSED_NOTE_DRAFT);
  };
  const submit = () => {
    const submission = noteSubmission(draft);
    if (!submission) return;
    const settled = { onSuccess: () => setDraft(CLOSED_NOTE_DRAFT) };
    if (submission.kind === "create") {
      addNote.mutate({ scope, text: submission.markdown }, settled);
    } else {
      editNote.mutate({ noteId: submission.noteId, text: submission.markdown }, settled);
    }
  };
  const editor = (
    <NoteEditor
      draft={draft}
      failure={failure}
      onCancel={close}
      onChange={setDraft}
      onSubmit={submit}
      pending={pending}
    />
  );

  return (
    <div className="grid gap-3">
      {draft.mode === "adding" ? (
        editor
      ) : (
        <button
          className="flex items-center gap-2 rounded-lg border border-dashed border-border/70 px-3 py-2 text-left text-sm text-muted-foreground transition-colors hover:border-border hover:text-foreground disabled:pointer-events-none disabled:opacity-50"
          disabled={draft.mode !== "closed"}
          onClick={() => setDraft(openNewNote())}
          type="button"
        >
          <Plus size={16} />
          Add a note
        </button>
      )}
      {notes.length ? (
        <NoteList
          editing={draft.mode === "editing" ? draft.noteId : null}
          editor={editor}
          notes={notes}
          onEdit={draft.mode === "closed" ? (note) => setDraft(openNoteEdit(note)) : undefined}
        />
      ) : null}
    </div>
  );
}

function NoteEditor({
  draft,
  failure,
  onCancel,
  onChange,
  onSubmit,
  pending,
}: {
  draft: NoteDraft;
  failure: Error | null;
  onCancel: () => void;
  onChange: (draft: NoteDraft) => void;
  onSubmit: () => void;
  pending: boolean;
}) {
  if (draft.mode === "closed") return null;
  const ready = noteSubmission(draft) !== null;
  return (
    <div className="grid gap-2 rounded-lg border border-border/70 p-3">
      <div aria-label="Note editor" className="flex gap-1" role="tablist">
        {TABS.map((tab) => (
          <button
            aria-selected={draft.tab === tab.id}
            className={cn(
              "rounded-md px-2.5 py-1 text-xs font-semibold uppercase text-muted-foreground transition-colors hover:text-foreground",
              draft.tab === tab.id && "bg-muted text-foreground",
            )}
            key={tab.id}
            onClick={() => onChange(selectNoteTab(draft, tab.id))}
            role="tab"
            type="button"
          >
            {tab.label}
          </button>
        ))}
      </div>
      {draft.tab === "code" ? (
        <Textarea
          aria-label="Note Markdown"
          className="min-h-28 font-mono text-sm"
          disabled={pending}
          onChange={(event) => onChange(updateNoteMarkdown(draft, event.target.value))}
          placeholder="Write the note in Markdown."
          value={draft.markdown}
        />
      ) : (
        <div className="min-h-28 rounded-lg border border-border/40 px-2.5 py-2">
          <MarkdownText text={draft.markdown} />
        </div>
      )}
      {failure ? (
        <p className="text-sm text-destructive" role="alert">
          {failure.message}
        </p>
      ) : null}
      <div className="flex justify-end gap-2">
        <Button disabled={pending} onClick={onCancel} size="sm" type="button" variant="ghost">
          Cancel
        </Button>
        <Button disabled={!ready || pending} onClick={onSubmit} size="sm" type="button">
          {draft.mode === "adding" ? "Add" : "Save"}
        </Button>
      </div>
    </div>
  );
}

function NoteList({
  editing = null,
  editor = null,
  notes,
  onEdit,
}: {
  editing?: string | null;
  editor?: ReactNode;
  notes: OperatorNote[];
  onEdit?: (note: OperatorNote) => void;
}) {
  return (
    <div className="grid gap-2">
      {notes.map((note) =>
        note.note_id === editing ? (
          <div key={note.note_id}>{editor}</div>
        ) : (
          <DocumentEmbeddedBlock as="article" key={note.note_id}>
            <div className="mb-1 flex flex-wrap items-center gap-2 text-xs font-semibold uppercase text-muted-foreground">
              <span className="normal-case text-foreground">{note.author || "Operator"}</span>
              <time dateTime={note.timestamp}>{timeOfDayLabel(note.timestamp)}</time>
              {note.elapsed_since_start_label ? (
                <span title="Elapsed since test start">T+{note.elapsed_since_start_label}</span>
              ) : null}
              {onEdit ? (
                <Button
                  aria-label="Edit note"
                  className="ml-auto"
                  onClick={() => onEdit(note)}
                  size="icon-xs"
                  type="button"
                  variant="ghost"
                >
                  <Pencil />
                </Button>
              ) : null}
            </div>
            <MarkdownText text={note.text} />
          </DocumentEmbeddedBlock>
        ),
      )}
    </div>
  );
}
