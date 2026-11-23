<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { occurrenceKeys } from "../lib/utils";

  let { items }: { items: readonly string[] } = $props();
</script>

{#if items.length}
  <details class="text-sm [overflow-wrap:anywhere]">
    <summary class="cursor-pointer" onclick={(event) => event.stopPropagation()} onkeydown={(event) => event.stopPropagation()}>
      {items.length} {items.length === 1 ? "operation" : "operations"} unavailable because startup failed
    </summary>
    <ul class="list-disc space-y-1 pl-5">
      {#each occurrenceKeys([...items], (item) => item) as { key, value } (key)}
        <li>{value}</li>
      {/each}
    </ul>
  </details>
{/if}
