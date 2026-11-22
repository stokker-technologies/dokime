/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { EvidenceScope } from "@generated/host/model";
import type { Wire } from "@/lib/rpc/canonical";
import { notesNewestFirst } from "@/features/cycle/operator-notes-model";
/* === Local Imports === */
import { useAddNoteMutation, useEditNoteMutation } from "@/features/cycle/server-state";
import type { OperatorNote } from "@/lib/types";
import { EditableOperatorNotes, FrozenOperatorNotes } from "@/components/document/OperatorNotesView";

/* === Types === */
type OperatorNotesContentProps = {
  notes: OperatorNote[];
  scope?: Wire<EvidenceScope> | null;
};

/* === Components === */
export function OperatorNotesContent({ notes, scope = null }: OperatorNotesContentProps) {
  return scope ? <ConnectedOperatorNotes notes={notes} scope={scope} /> : <FrozenOperatorNotes notes={notes} />;
}

/* === Private Components === */
function ConnectedOperatorNotes({ notes, scope }: { notes: OperatorNote[]; scope: Wire<EvidenceScope> }) {
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
