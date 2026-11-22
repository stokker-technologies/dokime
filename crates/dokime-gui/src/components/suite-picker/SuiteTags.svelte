<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Tooltip } from "bits-ui";
  import { buttonVariants } from "../ui/button-variants";
  import { occurrenceKeys } from "../../lib/utils";
  import { formatTagLabel } from "./model";
  import TagChip from "./TagChip.svelte";

  const TAG_COLUMN_BUDGET = 44;
  const OVERFLOW_BUDGET = 5;
  let { tags }: { tags: string[] } = $props();

  const { visible, hidden } = $derived.by(() => {
    const visible: string[] = [];
    let used = 0;
    for (const [index, tag] of tags.entries()) {
      const budget = index < tags.length - 1 ? TAG_COLUMN_BUDGET - OVERFLOW_BUDGET : TAG_COLUMN_BUDGET;
      const label = formatTagLabel(tag);
      const cost = label.length + 3;
      if (used + cost > budget) {
        return { hidden: tags.slice(index).map(formatTagLabel), visible };
      }
      used += cost;
      visible.push(label);
    }
    return { hidden: [], visible };
  });
  const hiddenLabels = $derived(hidden.join(", "));
</script>

<div class="flex min-w-0 flex-nowrap items-center gap-1 overflow-hidden">
  {#each occurrenceKeys(visible, (label) => label) as { key, value } (key)}
    <TagChip label={value} />
  {/each}
  {#if hidden.length}
    <Tooltip.Root>
      <Tooltip.Trigger
        class={buttonVariants({ variant: "plain", size: "inline" })}
        aria-label={`${hidden.length} More Tags: ${hiddenLabels}`}
        ><TagChip label={`+${hidden.length}`} /></Tooltip.Trigger
      >
      <Tooltip.Portal>
        <Tooltip.Content class="z-50 rounded-lg bg-popover px-3 py-2 text-xs text-popover-foreground ring-1 ring-border">
          {hiddenLabels}
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  {/if}
</div>
