<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { DocumentCardContentProps } from "./DocumentCardContent.svelte";
  import type { DocumentWorkspaceView } from "./DocumentWorkspacePresentation.svelte";

  type Props = DocumentWorkspaceView & {
    hasCardOverride: boolean;
    onRetry?: (cardId?: string) => void;
    traceabilitySave?: ((json: string, filename: string) => Promise<void>) | null;
    attachPortal: (element: HTMLElement, content: DocumentCardContentProps) => () => void;
  };
</script>

<script lang="ts">
  import DocumentWorkspacePresentation from "./DocumentWorkspacePresentation.svelte";

  let { attachPortal, hasCardOverride, ...workspace }: Props = $props();
</script>

{#snippet cardOverride(
  content: DocumentCardContentProps,
)}
  <div class="contents" {@attach (element) => attachPortal(element, content)}></div>
{/snippet}

<DocumentWorkspacePresentation {...workspace} cardOverride={hasCardOverride ? cardOverride : undefined} />
