<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Severity } from "@generated/host/model";
  import { createQuery } from "@tanstack/svelte-query";
  import { checkpointKey } from "@/lib/api/native-checkpoints";
  import { nativeValuesToJson } from "@/lib/host/values";
  import { required } from "@/lib/rpc/canonical";
  import { fetchCheckpointContext } from "../explore-data-source";
  import { formatStructured } from "./checkpoint-view";
  import ErrorCard from "@/components/ErrorCard.svelte";
  import { failureOf } from "@/lib/notification-details";

  import type { CheckpointProps } from "./checkpoint-view";
  let { checkpoint, connection }: CheckpointProps = $props();
  const client = () => connection.queryClient;

  const query = createQuery(() => {
    const { session } = connection;
    const at = checkpoint;
    return {
      queryKey: ["checkpoint-context", session.generation, checkpointKey(at)],
      queryFn: ({ signal }: { signal: AbortSignal }) => fetchCheckpointContext(session, at, signal),
      staleTime: Infinity,
      gcTime: 0,
      retry: false,
    };
  }, client);

  function severityLabel(value: Severity | undefined) {
    return value === undefined ? "unspecified" : (Severity[value]?.toLowerCase() ?? "unspecified");
  }
</script>

{#if query.isPending}
  <p role="status" class="text-sm text-muted-foreground">Loading captured context…</p>
{:else if query.isError}
  <ErrorCard failure={failureOf(query.error, "Evidence Unavailable")} onRetry={() => void query.refetch()} />
{:else}
  {@const context = query.data}
  <div class="grid min-w-0 gap-3">
    <h3 class="font-medium">{context.cycle?.metadata?.label ?? "Execution Context"}</h3>
    <h4 class="text-sm font-medium">Notes at Capture</h4>
    {#if !context.notes.length}
      <p>No notes captured.</p>
    {/if}
    {#each context.notes as note (note.noteId)}
      <div class="break-words text-sm">
        {#if note.author}
          <p class="text-muted-foreground">{note.author}</p>
        {/if}
        <p class="whitespace-pre-wrap">{note.message}</p>
      </div>
    {/each}
    <h4 class="text-sm font-medium">Health at Capture</h4>
    {#if !context.health.length}
      <p>No health items captured.</p>
    {/if}
    {#each context.health as item (item.healthItemId)}
      <p class="break-words text-sm">{severityLabel(item.severity)} · {item.message}</p>
    {/each}
    <h4 class="text-sm font-medium">Prompts at Capture</h4>
    <p class="text-xs text-muted-foreground">Historical prompts are read-only.</p>
    {#if !context.prompts.length}
      <p>No prompts captured.</p>
    {/if}
    {#each context.prompts as view (view.prompt?.promptId)}
      {@const prompt = required(view.prompt, "Open a checkpoint with captured prompt values")}
      <div class="rounded border p-2">
        <p>{prompt.definition?.metadata?.label || prompt.promptId}</p>
        <pre class="overflow-auto whitespace-pre-wrap break-all text-xs">{formatStructured(nativeValuesToJson(prompt.values?.values))}</pre>
      </div>
    {/each}
    <h4 class="text-sm font-medium">Captured Documents</h4>
    <p>{context.document.header.metadata?.label || context.document.header.documentId || "Captured Document"}</p>
  </div>
{/if}
