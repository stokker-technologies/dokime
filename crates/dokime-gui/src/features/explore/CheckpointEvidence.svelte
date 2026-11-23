<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { ConnectedSession } from "@/app/native-session/session";
  import Button from "@/components/ui/Button.svelte";
  import type { ExecutionCheckpoint } from "@/lib/types/browse";
  import CheckpointCatalog from "./checkpoint/CheckpointCatalog.svelte";
  import CheckpointContext from "./checkpoint/CheckpointContext.svelte";
  import CheckpointRecords from "./checkpoint/CheckpointRecords.svelte";

  let { checkpoint, connected }: { checkpoint: ExecutionCheckpoint; connected: ConnectedSession } = $props();
  const panels = ["Records", "Telemetry", "Files", "Context"] as const;
  let panel = $state<(typeof panels)[number] | null>(null);
  const connection = $derived({ session: { ...connected.identity, own: connected.own }, queryClient: connected.queryClient });
</script>

<div class="grid min-w-0 gap-3 border-t pt-3">
  <fieldset class="flex min-w-0 flex-wrap gap-2" aria-label="Evidence Sections">
    {#each panels as name (name)}
      <Button size="sm" variant={panel === name ? "default" : "outline"} aria-pressed={panel === name} onclick={() => (panel = name)}>
        {name}
      </Button>
    {/each}
  </fieldset>
  {#if panel === "Records"}
    <CheckpointRecords {checkpoint} {connection} />
  {:else if panel === "Context"}
    <CheckpointContext {checkpoint} {connection} />
  {:else if panel}
    {#key panel}
      <CheckpointCatalog {checkpoint} {connection} {panel} />
    {/key}
  {/if}
</div>
