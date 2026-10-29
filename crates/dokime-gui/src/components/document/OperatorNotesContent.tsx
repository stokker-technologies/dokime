/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { notesNewestFirst } from "@/features/cycle/operator-notes-model";

import { useAddNoteMutation, useEditNoteMutation } from "@/features/cycle/server-state";
import type { OperatorNote, RecordScope } from "@/lib/types";
import { EditableOperatorNotes, FrozenOperatorNotes } from "./OperatorNotesView";

type OperatorNotesContentProps = {
  notes: OperatorNote[];
  scope?: RecordScope | null;
};

export function OperatorNotesContent({ notes, scope = null }: OperatorNotesContentProps) {
  return scope ? <ConnectedOperatorNotes notes={notes} scope={scope} /> : <FrozenOperatorNotes notes={notes} />;
}

function ConnectedOperatorNotes({ notes, scope }: { notes: OperatorNote[]; scope: RecordScope }) {
  const addNote = useAddNoteMutation();
  const editNote = useEditNoteMutation();
  return (
    <EditableOperatorNotes
      notes={notesNewestFirst(notes)}
      pending={addNote.isPending || editNote.isPending}
      failure={addNote.error ?? editNote.error}
      onReset={() => {
        addNote.reset();
        editNote.reset();
      }}
      onSubmit={(submission, saved) => {
        const settled = { onSuccess: saved };
        if (submission.kind === "create") {
          addNote.mutate({ scope, text: submission.markdown }, settled);
        } else {
          editNote.mutate({ noteId: submission.noteId, text: submission.markdown }, settled);
        }
      }}
    />
  );
}
