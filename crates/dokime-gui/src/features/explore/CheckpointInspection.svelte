<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import type { ConnectedSession } from "@/app/native-session/session";
  import ErrorCard from "@/components/ErrorCard.svelte";
  import Badge from "@/components/ui/Badge.svelte";
  import { checkpointKey, checkpointStepStatus, executionStatusLabel } from "@/lib/api/native-checkpoints";
  import { timestampIso } from "@/lib/api/native-values";
  import { failureOf } from "@/lib/notification-details";
  import type { ExecutionBrowseEntry } from "@/lib/types/browse";
  import CheckpointEvidence from "./CheckpointEvidence.svelte";
  import { fetchCheckpoint } from "./explore-data-source";

  let { session, entry }: { session: ConnectedSession; entry: ExecutionBrowseEntry } = $props();
  const checkpoint = $derived(entry.checkpoint);
  const query = createQuery(
    () => {
      const at = checkpoint;
      const { identity } = session;
      return {
        queryKey: ["execution-checkpoint", at ? checkpointKey(at) : null],
        queryFn: ({ signal }: { signal: AbortSignal }) => {
          if (!at) {
            throw new Error("Choose an execution with a retained checkpoint.");
          }
          return fetchCheckpoint(identity, at, signal);
        },
        enabled: Boolean(at),
        staleTime: Number.POSITIVE_INFINITY,
      };
    },
    () => session.queryClient,
  );
  const resolvedCheckpoint = $derived(query.data?.checkpoint ?? checkpoint);
  const execution = $derived(query.data?.execution);
</script>

{#if query.isPending}
  <p role="status" class="text-sm text-muted-foreground">Opening checkpoint…</p>
{:else if query.isError}
  <ErrorCard failure={failureOf(query.error, "Data Unavailable")} />
{:else if !resolvedCheckpoint}
  <ErrorCard failure={failureOf("The checkpoint identity is missing.", "Checkpoint Unavailable", "missing-checkpoint-identity")} />
{:else if !execution}
  <ErrorCard failure={failureOf("The execution document is missing.", "Checkpoint Unavailable", "missing-checkpoint-document")} />
{:else}
  <section class="grid gap-3 rounded-lg border bg-card p-4" aria-label="Captured Execution">
    <h2 class="font-semibold">{query.data.cycle?.metadata?.label ?? entry.metadata.title}</h2>
    <p class="break-all text-xs text-muted-foreground">
      Checkpoint {(query.data.checkpoint?.position?.sequence ?? checkpoint?.position.sequence)?.toString()}
    </p>
    <dl class="grid gap-2 text-sm sm:grid-cols-2">
      <div>
        <dt class="text-muted-foreground">Recorded Status</dt>
        <dd>{executionStatusLabel(execution.summary?.state, execution.outcome, execution.startedAt !== undefined)}</dd>
      </div>
      <div>
        <dt class="text-muted-foreground">Acquisition</dt>
        <dd>{query.data.acquisitionFinished ? "Sealed" : "Incomplete"}</dd>
      </div>
      <div>
        <dt class="text-muted-foreground">Started</dt>
        <dd>{timestampIso(execution.startedAt) ?? "Not Recorded"}</dd>
      </div>
      <div>
        <dt class="text-muted-foreground">Sealed</dt>
        <dd>{timestampIso(execution.finishedAt) ?? "Not Recorded"}</dd>
      </div>
    </dl>
    <h3 class="text-sm font-semibold">Recorded Steps</h3>
    {#if query.data.steps.length}
      {#each query.data.steps as step (step.stepId)}
        <div class="flex justify-between gap-3 border-t py-2 text-sm">
          <span>{step.metadata?.label ?? step.stepId}</span>
          <Badge variant="outline">{checkpointStepStatus(step.status)}</Badge>
        </div>
      {/each}
    {:else}
      <p class="text-sm text-muted-foreground">No steps recorded at this checkpoint.</p>
    {/if}
    <CheckpointEvidence checkpoint={resolvedCheckpoint} connected={session} />
  </section>
{/if}
