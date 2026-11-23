<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { ArtifactState } from "@generated/host/model";
  import { createQuery } from "@tanstack/svelte-query";
  import Button from "@/components/ui/Button.svelte";
  import SelectField from "@/components/ui/SelectField.svelte";
  import { artifactStateLabel, checkpointFileLabel, checkpointKey } from "@/lib/api/native-checkpoints";
  import { fetchCheckpointEvidence } from "../explore-data-source";
  import type { CheckpointProps } from "./checkpoint-view";
  import CheckpointDownload from "./CheckpointDownload.svelte";
  import CheckpointFilePreview from "./CheckpointFilePreview.svelte";
  import CheckpointTelemetryRows from "./CheckpointTelemetryRows.svelte";
  import ErrorCard from "@/components/ErrorCard.svelte";
  import { failureOf } from "@/lib/notification-details";

  let { checkpoint, connection, panel }: CheckpointProps & { panel: "Telemetry" | "Files" } = $props();
  const client = () => connection.queryClient;
  let selectedFile = $state<string | null>(null);
  let selectedTable = $state<string | null>(null);
  const streamId = $props.id();

  const query = createQuery(() => {
    const { session } = connection;
    const at = checkpoint;
    return {
      queryKey: ["checkpoint-evidence", session.generation, checkpointKey(at)],
      queryFn: ({ signal }: { signal: AbortSignal }) => fetchCheckpointEvidence(session, at, signal),
      staleTime: Infinity,
      gcTime: 0,
      retry: false,
    };
  }, client);

  const streams = $derived(
    query.data?.telemetryTables.map((item) => ({
      value: item.table,
      label: item.schema?.metadata?.label || item.schema?.stream || item.table,
    })) ?? [],
  );
  const table = $derived(selectedTable ?? streams[0]?.value);
</script>

{#if query.isPending}
  <p role="status" class="text-sm text-muted-foreground">{panel === "Telemetry" ? "Loading streams…" : "Loading files…"}</p>
{:else if query.isError}
  <ErrorCard failure={failureOf(query.error, "Evidence Unavailable")} onRetry={() => void query.refetch()} />
{:else}
  {#if panel === "Telemetry"}
    <div class="grid min-w-0 gap-3">
      {#if table}
        <label for={streamId} class="grid gap-1 text-sm">Stream</label>
        <SelectField
          value={table}
          options={streams}
          id={streamId}
          label="Stream"
          class="max-w-full"
          onChange={(next) => (selectedTable = next)}
        />
        {#key table}
          <CheckpointTelemetryRows {checkpoint} {connection} {table} />
        {/key}
      {:else}
        <p>No telemetry streams at this checkpoint.</p>
      {/if}
    </div>
  {:else}
    <div class="grid min-w-0 gap-3">
      <h3 class="font-medium">Output Files</h3>
      {#if !query.data.outputs.length}
        <p>No output files.</p>
      {/if}
      {#each query.data.outputs as file (file.relativePath)}
        <Button
          class="h-auto justify-start whitespace-normal break-all"
          variant="outline"
          onclick={() => (selectedFile = file.relativePath)}
        >
          {checkpointFileLabel(file)}
        </Button>
      {/each}
      {#if selectedFile}
        {#key selectedFile}
          <CheckpointFilePreview {checkpoint} {connection} path={selectedFile} />
        {/key}
      {/if}
      <h3 class="font-medium">Attachments</h3>
      {#if !query.data.artifacts.length}
        <p>No attachments.</p>
      {/if}
      {#each query.data.artifacts as artifact (artifact.artifactId)}
        {#if artifact.state === ArtifactState.COMPLETE}
          <CheckpointDownload {checkpoint} {connection} id={artifact.artifactId} name={artifact.filename} />
        {:else}
          <p>{artifact.filename} · {artifactStateLabel(artifact.state)}</p>
        {/if}
      {/each}
    </div>
  {/if}
{/if}
