<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { ConnectedSession } from "@/app/native-session/session";
  import { createScopedDocumentQuery } from "@/features/cycle/document-query.svelte";
  import type { DocumentViewPayload } from "@/lib/types";

  let {
    session,
    cycleId,
    executionId,
    ready,
    onDocument,
  }: {
    session: ConnectedSession;
    cycleId: string;
    executionId: string;
    ready: (query: ReturnType<typeof createScopedDocumentQuery>) => void;
    onDocument?: (document: DocumentViewPayload | undefined) => void;
  } = $props();
  const query = createScopedDocumentQuery(
    () => session,
    () => ({ cycleId, executionId, enabled: true }),
  );
  $effect(() => ready(query));
  $effect(() => onDocument?.(query.data));
</script>
