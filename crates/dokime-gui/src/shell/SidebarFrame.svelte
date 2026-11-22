<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { PanelLeftClose, PanelLeftOpen } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import Button from "../components/ui/Button.svelte";
  import type { Brand } from "../lib/brand";
  import BrandMark from "./BrandMark.svelte";
  import type { NavigationMode } from "./navigation-model";

  type Props = {
    brand?: Brand;
    children?: Snippet;
    collapsed: boolean;
    width: string;
    mode: NavigationMode;
    toggle: () => void;
    resize: (width: number) => void;
  };

  let { brand, children, collapsed, width, mode, toggle, resize }: Props = $props();
</script>

{#if mode !== "phone"}
  <nav
    aria-label="Navigation"
    class="relative grid min-h-0 shrink-0 grid-rows-[auto_minmax(0,1fr)] bg-background"
    data-collapsed={collapsed || undefined}
    style:width
  >
    <div class="flex min-w-0 items-center">
      <div class="min-w-0 flex-1">
        <div class={collapsed ? "min-w-0 px-2 py-3" : "min-w-0 px-4 py-3"} data-slot="sidebar-brand">
          <BrandMark {brand} class="sidebar-brand" logoOnly={collapsed} />
        </div>
      </div>
      {#if mode === "desktop"}
        <Button aria-label={collapsed ? "Expand Sidebar" : "Collapse Sidebar"} size="icon" variant="ghost" onclick={toggle}>
          {#if collapsed}
            <PanelLeftOpen />
          {:else}
            <PanelLeftClose />
          {/if}
        </Button>
      {/if}
    </div>
    {#if mode === "desktop" && !collapsed}
      <Button
        variant="plain"
        size="inline"
        type="button"
        aria-label="Resize Sidebar"
        class="absolute inset-y-0 right-0 z-10 w-1 cursor-col-resize touch-none hover:bg-primary"
        onkeydown={(event) => {
          if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") {
            return;
          }
          event.preventDefault();
          const current = event.currentTarget.parentElement?.getBoundingClientRect().width;
          if (current !== undefined) {
            resize(current + (event.key === "ArrowRight" ? 4 : -4));
          }
        }}
        onpointerdown={(event) => event.currentTarget.setPointerCapture(event.pointerId)}
        onpointermove={(event) => {
          if (event.currentTarget.hasPointerCapture(event.pointerId)) {
            const left = event.currentTarget.parentElement?.getBoundingClientRect().left ?? 0;
            resize(event.clientX - left);
          }
        }}
        onpointerup={(event) => event.currentTarget.releasePointerCapture(event.pointerId)}
      />
    {/if}
    {@render children?.()}
  </nav>
{/if}
