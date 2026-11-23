<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { checkpointKey } from "@/lib/api/native-checkpoints";
  import { timestampIso } from "@/lib/api/native-values";
  import { nativeValuesToJson } from "@/lib/host/values";
  import { fetchCheckpointTelemetry } from "../explore-data-source";
  import { type CheckpointProps, formatStructured } from "./checkpoint-view";
  import CheckpointPager from "./CheckpointPager.svelte";
  import CheckpointValues from "./CheckpointValues.svelte";
  import ErrorCard from "@/components/ErrorCard.svelte";
  import { failureOf } from "@/lib/notification-details";

  let { checkpoint, connection, table }: CheckpointProps & { table: string } = $props();
  let cursors = $state.raw<(string | null)[]>([null]);
  const after = $derived(cursors.at(-1) ?? null);
  const query = createQuery(
    () => {
      const { session } = connection;
      const at = checkpoint;
      const cursor = after;
      const stream = table;
      return {
        queryKey: ["checkpoint-telemetry", session.generation, checkpointKey(at), stream, cursor],
        queryFn: ({ signal }: { signal: AbortSignal }) => fetchCheckpointTelemetry(session, at, stream, cursor, signal),
        staleTime: Infinity,
        gcTime: 0,
        retry: false,
      };
    },
    () => connection.queryClient,
  );

  function samples(rows: Awaited<ReturnType<typeof fetchCheckpointTelemetry>>["samples"]) {
    return rows.map((row) => {
      if (row.kind.case !== "raw") {
        throw new Error("Open a checkpoint with raw telemetry evidence.");
      }
      return row.kind.value;
    });
  }
</script>

{#if query.isPending}
  <p role="status" class="text-sm text-muted-foreground">Loading samples…</p>
{:else if query.isError}
  <ErrorCard failure={failureOf(query.error, "Evidence Unavailable")} onRetry={() => void query.refetch()} />
{:else}
  <section class="grid min-w-0 gap-3" aria-label="Captured Telemetry">
    {#if !query.data.samples.length}
      <p>No samples at this checkpoint.</p>
    {/if}
    {#each samples(query.data.samples) as sample (sample.sampleId)}
      <div class="min-w-0 border-b pb-2 text-sm">
        <p class="text-muted-foreground">{timestampIso(sample.timestamp) ?? "Unknown Time"}</p>
        <CheckpointValues value={sample.values?.values} />
        {#if sample.summary}
          <details class="mt-2">
            <summary class="cursor-pointer text-xs text-muted-foreground">Summary</summary>
            <pre
              class="overflow-auto whitespace-pre-wrap break-all text-xs"
            >{formatStructured(nativeValuesToJson(sample.summary.values))}</pre>
          </details>
        {/if}
      </div>
    {/each}
    <CheckpointPager next={query.data.nextAfter ?? null} bind:cursors />
  </section>
{/if}
