<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import Button from "@/components/ui/Button.svelte";
  import { checkpointFileKind, checkpointFileState, checkpointKey, NATIVE_CHECKPOINT_WINDOW_BYTES } from "@/lib/api/native-checkpoints";
  import { readCheckpointOutput } from "../explore-data-source";
  import type { CheckpointProps } from "./checkpoint-view";
  import ErrorCard from "@/components/ErrorCard.svelte";
  import { failureOf } from "@/lib/notification-details";

  const WINDOW_BYTES = BigInt(NATIVE_CHECKPOINT_WINDOW_BYTES);
  let { checkpoint, connection, path }: CheckpointProps & { path: string } = $props();
  let offset = $state(0n);
  const query = createQuery(
    () => {
      const { session } = connection;
      const at = checkpoint;
      const filename = path;
      const start = offset;
      return {
        queryKey: ["checkpoint-output", session.generation, checkpointKey(at), filename, start.toString()],
        queryFn: ({ signal }: { signal: AbortSignal }) => readCheckpointOutput(session, at, filename, start, signal),
        staleTime: Infinity,
        gcTime: 0,
        retry: false,
      };
    },
    () => connection.queryClient,
  );
</script>

{#if query.isPending}
  <p role="status" class="text-sm text-muted-foreground">Loading file bytes…</p>
{:else if query.isError}
  <ErrorCard failure={failureOf(query.error, "Evidence Unavailable")} onRetry={() => void query.refetch()} />
{:else}
  {@const file = query.data}
  {@const end = file.offset + BigInt(file.data.byteLength)}
  {@const kind = checkpointFileKind(file.kind)}
  <section class="grid min-w-0 gap-2" aria-label="File Preview">
    <p class="break-all text-sm">
      {path}
      · {kind} · {checkpointFileState(file.state)} · bytes {file.offset.toString()}–{end.toString()}
      of
      {file.observedSizeBytes.toString()}
    </p>
    <pre class="max-h-80 overflow-auto whitespace-pre-wrap break-all rounded bg-muted p-2 text-xs">{kind === "text"
          ? new TextDecoder().decode(file.data)
          : Array.from(file.data, (byte) => byte.toString(16).padStart(2, "0")).join(" ")}</pre>
    <div class="flex gap-2">
      <Button
        size="sm"
        variant="outline"
        disabled={offset === 0n}
        onclick={() => (offset = file.offset > WINDOW_BYTES ? file.offset - WINDOW_BYTES : 0n)}
      >
        Previous Bytes
      </Button>
      <Button size="sm" variant="outline" disabled={end >= file.observedSizeBytes || !file.data.byteLength} onclick={() => (offset = end)}>
        Next Bytes
      </Button>
    </div>
  </section>
{/if}
