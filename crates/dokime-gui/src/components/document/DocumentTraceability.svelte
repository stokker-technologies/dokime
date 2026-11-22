<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { formatValue } from "../../lib/dashboard";
  import { isStatedGroup } from "../../lib/traceability";
  import type { TraceabilitySnapshot } from "../../lib/types";
  import CopyableTraceabilityValue from "../traceability/CopyableTraceabilityValue.svelte";

  let { traceability }: { traceability?: TraceabilitySnapshot | null } = $props();
  const groups = $derived(traceability?.groups ?? []);
</script>

{#if groups.length}
  <div class="min-w-0 columns-1 gap-4 md:columns-2 xl:columns-3">
    {#each groups as group (group.group_id)}
      <section class="mb-4 grid min-w-0 break-inside-avoid gap-2 border-l-2 border-border/70 pl-3">
        <div class="min-w-0">
          <strong class="block truncate" title={group.title}>{group.title}</strong>
          {#if group.summary}
            <p class="text-xs text-muted-foreground [overflow-wrap:anywhere]">{group.summary}</p>
          {/if}
        </div>
        <dl class="grid min-w-0 gap-2">
          {#each isStatedGroup(group) ? group.items : group.items.slice(0, 8) as item}
            <div class="grid min-w-0 grid-cols-2 gap-2">
              <dt class="truncate text-xs text-muted-foreground" title={item.label}>{item.label}</dt>
              <dd class="min-w-0 text-right">
                <CopyableTraceabilityValue value={formatValue(item.value, item.unit)} />
              </dd>
            </div>
          {/each}
        </dl>
      </section>
    {/each}
    {#if traceability?.warnings.length}
      <ul
        class="mb-4 grid min-w-0 break-inside-avoid gap-1 border-l-2 border-warning/40 bg-warning/10 px-3 py-2 text-sm text-muted-foreground"
      >
        {#each traceability.warnings.slice(0, 6) as warning}
          <li>{warning}</li>
        {/each}
      </ul>
    {/if}
  </div>
{:else}
  <p class="text-sm text-muted-foreground">No traceability information available.</p>
{/if}
