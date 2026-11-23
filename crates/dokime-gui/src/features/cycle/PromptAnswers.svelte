<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { absoluteTimestampLabel } from "../../lib/time-format";
  import type { answeredPrompts } from "./prompt-banner-model";

  let { answered }: { answered: ReturnType<typeof answeredPrompts> } = $props();
</script>

{#if answered.length}
  <ol aria-label="Answered Prompts" class="grid min-w-0 grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] gap-x-4 border-t pt-2 text-xs">
    {#each answered as row (row.promptId)}
      <li class="col-span-3 grid grid-cols-subgrid items-baseline py-1" data-answered-prompt={row.promptId}>
        <span class="min-w-0 font-medium [overflow-wrap:anywhere]">{row.question}</span>
        <span class="min-w-0 text-muted-foreground [overflow-wrap:anywhere]">{row.answer}</span>
        <time class="text-muted-foreground tabular-nums" datetime={row.at}>{absoluteTimestampLabel(row.at)}</time>
      </li>
    {/each}
  </ol>
{/if}
