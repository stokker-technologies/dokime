<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Clipboard } from "lucide-svelte";
  import Button from "../../components/ui/Button.svelte";
  import { failureOf } from "../../lib/notification-details";
  import { occurrenceKeys } from "../../lib/utils";
  import { toast } from "../../lib/notifications";

  let { value }: { value: string } = $props();
  const parts = $derived(occurrenceKeys(value.split("."), (part) => part));

  async function copyIdentifier() {
    try {
      await navigator.clipboard.writeText(value);
      toast.success("Identifier Copied");
    } catch (error: unknown) {
      toast.error(failureOf(error, "Could Not Copy Identifier"));
    }
  }
</script>

<span class="flex min-w-0 items-center gap-1 font-mono text-xs leading-4 text-muted-foreground">
  <span class="min-w-0 whitespace-normal">
    {#each parts as { key, value: part }, index (key)}
      {part}
      {#if index < parts.length - 1}
        .<wbr>
      {/if}
    {/each}
  </span>
  <Button
    variant="plain"
    size="inline"
    aria-label={`Copy Template Identifier ${value}`}
    class="grid size-4 shrink-0 place-items-center rounded-lg hover:bg-muted hover:text-foreground"
    onclick={copyIdentifier}
    title="Copy Identifier"
    ><Clipboard aria-hidden="true" class="size-3" /></Button
  >
</span>
