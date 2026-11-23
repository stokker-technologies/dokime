<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "../../lib/utils";

  type Props = {
    title: string;
    action?: Snippet;
    children?: Snippet;
    class?: string;
    description?: string | null;
    empty?: string | null;
  };

  let { title, action, children, class: style, description, empty }: Props = $props();
  const titleId = $props.id();
</script>

<section aria-labelledby={titleId} class={cn("grid min-w-0 gap-3 border-t bg-transparent p-4", style)} data-execution-section={title}>
  <header class="grid min-h-8 min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-x-3 gap-y-1">
    <h2 class="document-section-title truncate" id={titleId}>{title}</h2>
    {#if action}
      <div class="grid auto-cols-max grid-flow-col items-center">{@render action()}</div>
    {:else}
      <span></span>
    {/if}
    {#if description}
      <div class="col-span-2 text-sm text-muted-foreground [overflow-wrap:anywhere]">{description}</div>
    {/if}
  </header>
  {@render children?.()}
  {#if empty}
    <p class="rounded-lg border border-dashed px-3 py-4 text-center text-sm text-muted-foreground">{empty}</p>
  {/if}
</section>
