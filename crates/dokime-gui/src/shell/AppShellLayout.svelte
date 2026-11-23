<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Tooltip } from "bits-ui";
  import type { Snippet } from "svelte";
  import { cn } from "../lib/utils";

  type Props = {
    children?: Snippet;
    header?: Snippet;
    sidebar?: Snippet;
    statusBar?: Snippet;
    menuBar?: Snippet;
    console?: Snippet;
  };

  let { children, header, menuBar, sidebar, statusBar, console }: Props = $props();
</script>

<Tooltip.Provider>
  <div
    class={cn(
      "fixed inset-0 grid h-dvh overflow-hidden bg-background text-foreground",
      menuBar ? "grid-rows-[var(--menu-bar-height)_minmax(0,1fr)_auto] pb-16 tablet:pb-0" : "grid-rows-[minmax(0,1fr)_auto]",
    )}
    style:font-size="var(--shell-text-base)"
  >
    {@render menuBar?.()}
    <div class="flex min-h-0 min-w-0">
      {@render sidebar?.()}
      <div class="flex min-w-0 flex-1 flex-col overflow-hidden bg-card">
        {@render header?.()}
        <!-- Keep The Viewport Still While Live Plots Redraw. -->
        <main class="min-h-0 min-w-0 flex-1 overflow-y-auto [overflow-anchor:none]">{@render children?.()}</main>
        {@render console?.()}
      </div>
    </div>
    {@render statusBar?.()}
  </div>
</Tooltip.Provider>
