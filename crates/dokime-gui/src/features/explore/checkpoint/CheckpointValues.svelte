<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { nativeValuesToJson } from "@/lib/host/values";
  import { formatStructured } from "./checkpoint-view";
  let { value }: { value?: Parameters<typeof nativeValuesToJson>[0] } = $props();
  const object = $derived(nativeValuesToJson(value));
</script>

{#if Object.keys(object).length}
  <dl class="grid gap-1">
    {#each Object.entries(object) as [name, item] (name)}
      <div class="flex min-w-0 flex-wrap gap-2">
        <dt class="font-medium">{name}</dt>
        <dd class="break-all">{typeof item === "string" ? item : formatStructured(item)}</dd>
      </div>
    {/each}
  </dl>
{:else}
  <p class="text-muted-foreground">No values.</p>
{/if}
