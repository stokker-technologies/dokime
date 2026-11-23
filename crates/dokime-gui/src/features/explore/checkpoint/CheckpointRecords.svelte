<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { checkpointKey, nativeRecordText } from "@/lib/api/native-checkpoints";
  import { fetchCheckpointRecords } from "../explore-data-source";
  import { formatStructured } from "./checkpoint-view";
  import CheckpointPager from "./CheckpointPager.svelte";
  import ErrorCard from "@/components/ErrorCard.svelte";
  import { failureOf } from "@/lib/notification-details";

  import type { CheckpointProps } from "./checkpoint-view";
  let { checkpoint, connection }: CheckpointProps = $props();
  const client = () => connection.queryClient;

  const GUI_ERROR_PREVIEW_CHARS = 500;
  let cursors = $state.raw<(string | null)[]>([null]);
  const after = $derived(cursors.at(-1) ?? null);
  const query = createQuery(() => {
    const { session } = connection;
    const at = checkpoint;
    const cursor = after;
    return {
      queryKey: ["checkpoint-records", session.generation, checkpointKey(at), cursor],
      queryFn: ({ signal }: { signal: AbortSignal }) => fetchCheckpointRecords(session, at, cursor, signal),
      staleTime: Infinity,
      gcTime: 0,
      retry: false,
    };
  }, client);

  function identified(records: Awaited<ReturnType<typeof fetchCheckpointRecords>>["records"]) {
    return records.map((row) => {
      if (!row.record || !row.epoch) {
        throw new Error("Open a checkpoint with recorded source identity.");
      }
      return { row, record: row.record };
    });
  }
</script>

{#if query.isPending}
  <p role="status" class="text-sm text-muted-foreground">Loading records…</p>
{:else if query.isError}
  <ErrorCard failure={failureOf(query.error, "Evidence Unavailable")} onRetry={() => void query.refetch()} />
{:else}
  <section class="grid min-w-0 gap-3" aria-label="Captured Records">
    {#if !query.data.records.length}
      <p>No execution records at this checkpoint.</p>
    {/if}
    {#each identified(query.data.records) as { row, record } (`${row.epoch}:${record.sequence.toString()}:${record.recordId}`)}
      {@const text = nativeRecordText(record, row.decoded)}
      {@const message = record.message ?? record.error?.message ?? ""}
      <details class="min-w-0 rounded border p-2 text-sm">
        <summary class="cursor-pointer break-words">
          <span class="text-muted-foreground"> {text.timestamp} · {text.severity} · {row.epoch} </span>
          <span class="block font-medium">{record.recordType || "Record"}</span>
          {message.slice(0, GUI_ERROR_PREVIEW_CHARS)}
        </summary>
        <pre class="max-h-72 overflow-auto whitespace-pre-wrap break-all pt-2 text-xs">{message}
              {Object.keys(text.details).length ? `\n${formatStructured(text.details)}` : ""}</pre>
      </details>
    {/each}
    <CheckpointPager next={query.data.nextAfter ?? null} bind:cursors />
  </section>
{/if}
