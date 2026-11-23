<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createMutation } from "@tanstack/svelte-query";
  import type { EvidenceScope } from "@generated/host/model";
  import type { ConnectedSession } from "../../app/native-session/session";
  import type { NoteSubmission } from "../../features/cycle/operator-notes-model";
  import { workflowQueryKeys } from "../../features/cycle/workflow-query-keys";
  import { createNativeNote, editNativeNote } from "../../lib/api/native-annotations";
  import type { Wire } from "../../lib/rpc/canonical";
  import type { OperatorNote } from "../../lib/types";
  import OperatorNotes from "./OperatorNotes.svelte";

  let {
    notes,
    scope,
    session,
  }: {
    notes: OperatorNote[];
    scope: Wire<EvidenceScope>;
    session: ConnectedSession;
  } = $props();

  const client = () => session.queryClient;
  const addNote = createMutation(
    () => ({
      mutationFn: (text: string) => createNativeNote(session.identity, text, scope),
      onSuccess: async () => {
        await Promise.all([
          session.queryClient.invalidateQueries({ queryKey: workflowQueryKeys.notes() }),
          session.queryClient.invalidateQueries({ queryKey: workflowQueryKeys.runtimeShell() }),
        ]);
      },
    }),
    client,
  );
  const editNote = createMutation(
    () => ({
      mutationFn: ({ noteId, text }: { noteId: string; text: string }) => editNativeNote(session.identity, noteId, text),
      onSuccess: async () => {
        await session.queryClient.invalidateQueries({ queryKey: workflowQueryKeys.notes() });
      },
    }),
    client,
  );

  function onReset() {
    addNote.reset();
    editNote.reset();
  }

  function onSubmit(submission: NoteSubmission, saved: () => void) {
    const callbacks = { onSuccess: saved };
    if (submission.kind === "create") {
      addNote.mutate(submission.markdown, callbacks);
    } else {
      editNote.mutate({ noteId: submission.noteId, text: submission.markdown }, callbacks);
    }
  }
</script>

<OperatorNotes {notes} pending={addNote.isPending || editNote.isPending} failure={addNote.error ?? editNote.error} {onReset} {onSubmit} />
