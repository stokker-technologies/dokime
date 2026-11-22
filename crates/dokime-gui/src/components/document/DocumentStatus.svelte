<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { HealthItem, TelemetryStream } from "../../lib/types";

  const content: { streams: TelemetryStream[]; health?: never } | { health: HealthItem[]; streams?: never } = $props();
</script>

{#if content.streams}
  {#if content.streams.length}
    <div class="grid gap-2 md:grid-cols-2 xl:grid-cols-3">
      {#each content.streams as stream (stream.stream)}
        {@const label = stream.spec?.label ?? stream.stream}
        <section class="grid min-w-0 gap-1 border-l-2 border-border/70 pl-3">
          <div class="min-w-0">
            <strong class="block truncate" title={label}>{label}</strong>
            <span class="text-xs text-muted-foreground">{stream.stream}</span>
          </div>
          <p class="text-xs text-muted-foreground">
            {stream.total_samples}
            samples{stream.dropped_samples ? `, ${stream.dropped_samples} dropped` : ""}
          </p>
        </section>
      {/each}
    </div>
  {:else}
    <p class="text-sm text-muted-foreground">No telemetry streams recorded.</p>
  {/if}
{:else if content.health.length}
  <div class="grid gap-2">
    {#each content.health as item (item.health_id)}
      <article class="grid min-w-0 gap-1 border-l-2 border-border/70 pl-3">
        <div class="flex items-center justify-between gap-3">
          <strong class="[overflow-wrap:anywhere]">{item.title}</strong>
          <span class="rounded-lg border bg-muted/40 px-2 py-1 text-xs capitalize text-muted-foreground">{item.level}</span>
        </div>
        {#if item.message}
          <p class="text-sm text-muted-foreground [overflow-wrap:anywhere]">{item.message}</p>
        {/if}
      </article>
    {/each}
  </div>
{:else}
  <p class="text-sm text-muted-foreground">No health records for this scope.</p>
{/if}
