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
import type { NoteSubmission } from "@/features/cycle/operator-notes-model";
/* === Local Imports === */
import { useAddNoteMutation, useEditNoteMutation } from "@/features/cycle/server-state";
import type { OperatorNote } from "@/lib/types";
import OperatorNotes from "./OperatorNotes.svelte";
import { SvelteMount } from "../SvelteMount";

/* === Types === */
type OperatorNotesContentProps = {
  notes: OperatorNote[];
  scope?: Wire<EvidenceScope> | null;
};

/* === Components === */
export function OperatorNotesContent({ notes, scope = null }: OperatorNotesContentProps) {
  return scope ? <ConnectedOperatorNotes notes={notes} scope={scope} /> : <SvelteMount component={OperatorNotes} props={{ notes }} />;
}

/* === Private Components === */
function ConnectedOperatorNotes({ notes, scope }: { notes: OperatorNote[]; scope: Wire<EvidenceScope> }) {
  const addNote = useAddNoteMutation();
  const editNote = useEditNoteMutation();
  const pending = addNote.isPending || editNote.isPending;
  const failure = addNote.error ?? editNote.error;

  function onReset() {
    addNote.reset();
    editNote.reset();
  }

  function onSubmit(submission: NoteSubmission, saved: () => void) {
    const settled = { onSuccess: saved };
    if (submission.kind === "create") {
      addNote.mutate({ scope, text: submission.markdown }, settled);
    } else {
      editNote.mutate({ noteId: submission.noteId, text: submission.markdown }, settled);
    }
  }

  return <SvelteMount component={OperatorNotes} props={{ notes, pending, failure, onReset, onSubmit }} />;
}
