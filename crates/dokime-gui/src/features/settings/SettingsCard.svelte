<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "../../lib/utils";

  let {
    title,
    description,
    actions,
    children,
    footer,
    class: style,
  }: {
    title?: string;
    description?: string;
    actions?: Snippet;
    children: Snippet;
    footer?: Snippet;
    class?: string;
  } = $props();
</script>

<section class={cn("flex min-w-0 flex-col overflow-hidden border-t bg-transparent", style)}>
  {#if title || description}
    <header class="flex items-center justify-between gap-4 border-b px-4 py-3">
      <div>
        {#if title}
          <h2 class="text-sm font-semibold">{title}</h2>
        {/if}
        {#if description}
          <p class="mt-1 text-xs text-muted-foreground">{description}</p>
        {/if}
      </div>
      {#if actions}
        <div class="flex shrink-0 flex-wrap items-center gap-2">{@render actions()}</div>
      {/if}
    </header>
  {/if}
  <div class="flex-1 divide-y">{@render children()}</div>
  {#if footer}
    <footer class="border-t px-4 py-3">{@render footer()}</footer>
  {/if}
</section>
