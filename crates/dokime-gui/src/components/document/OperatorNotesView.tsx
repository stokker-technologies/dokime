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

// Third Party Imports

/* === External Imports === */

import { Pencil, Plus } from "lucide-react";
import { type ReactNode, useState } from "react";
import { NotificationMessage } from "@/components/NotificationMessage";

/* === Local Imports === */

import { DocumentEmbeddedBlock, DocumentEmbeddedEmptyState } from "@/components/document/DocumentEmbeddedContent";
import { MarkdownText } from "@/components/MarkdownText";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
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
import { timeOfDayLabel } from "@/lib/time-format";
import type { OperatorNote } from "@/lib/types";

/* === Types === */

export type NoteSubmission = NonNullable<ReturnType<typeof noteSubmission>>;

type EditableOperatorNotesProps = {
  notes: OperatorNote[];
  pending: boolean;
  failure: Error | null;
  onReset: () => void;
  onSubmit: (submission: NoteSubmission, saved: () => void) => void;
};

/* === Constants === */

const TABS: { id: NoteEditorTab; label: string }[] = [
  { id: "code", label: "Code" },
  { id: "preview", label: "Preview" },
];

/* === Components === */

export function FrozenOperatorNotes({ notes }: { notes: OperatorNote[] }) {
  const ordered = notesNewestFirst(notes);
  return ordered.length ? (
    <NoteList notes={ordered} />
  ) : (
    <DocumentEmbeddedEmptyState>No operator notes recorded.</DocumentEmbeddedEmptyState>
  );
}

export function EditableOperatorNotes({ notes, pending, failure, onReset, onSubmit }: EditableOperatorNotesProps) {
  const [draft, setDraft] = useState<NoteDraft>(CLOSED_NOTE_DRAFT);
  const close = () => {
    onReset();
    setDraft(CLOSED_NOTE_DRAFT);
  };
  const submit = () => {
    const submission = noteSubmission(draft);
    if (!submission) return;
    onSubmit(submission, () => setDraft(CLOSED_NOTE_DRAFT));
  };
  const editor = <NoteEditor draft={draft} failure={failure} onCancel={close} onChange={setDraft} onSubmit={submit} pending={pending} />;

  return (
    <div className="grid gap-3">
      {draft.mode === "adding" ? (
        editor
      ) : (
        <Button
          variant="outline"
          className="justify-start border-dashed text-muted-foreground"
          disabled={draft.mode !== "closed"}
          onClick={() => setDraft(openNewNote())}
          type="button"
        >
          <Plus size={16} />
          Add a Note
        </Button>
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
    <Tabs
      className="grid gap-2 border border-border/70 p-3"
      value={draft.tab}
      onValueChange={(value) => onChange(selectNoteTab(draft, value as NoteEditorTab))}
    >
      <TabsList aria-label="Note Editor">
        {TABS.map((tab) => (
          <TabsTrigger key={tab.id} value={tab.id}>
            {tab.label}
          </TabsTrigger>
        ))}
      </TabsList>
      <TabsContent value="code">
        <Textarea
          aria-label="Note Markdown"
          className="min-h-28 font-mono text-sm"
          disabled={pending}
          onChange={(event) => onChange(updateNoteMarkdown(draft, event.target.value))}
          placeholder="Write the note in Markdown."
          value={draft.markdown}
        />
      </TabsContent>
      <TabsContent value="preview" className="min-h-28 border border-border/40 px-3 py-2">
        <MarkdownText text={draft.markdown} />
      </TabsContent>
      {failure ? <NotificationMessage error={failure}>{failure.message}</NotificationMessage> : null}
      <div className="flex justify-end gap-2">
        <Button disabled={pending} onClick={onCancel} size="sm" type="button" variant="ghost">
          Cancel
        </Button>
        <Button disabled={!ready || pending} onClick={onSubmit} size="sm" type="button">
          {draft.mode === "adding" ? "Add" : "Save"}
        </Button>
      </div>
    </Tabs>
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
              {note.elapsed_since_start_label ? <span title="Elapsed Since Test Start">T+{note.elapsed_since_start_label}</span> : null}
              {onEdit ? (
                <Button
                  aria-label="Edit Note"
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

/* === Re-Exports === */

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
