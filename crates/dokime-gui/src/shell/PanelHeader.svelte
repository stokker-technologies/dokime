<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { ChevronLeft } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import Button from "../components/ui/Button.svelte";

  let { title, actions, onBack }: { title: string; actions?: Snippet; onBack?: () => void } = $props();

  function back(event: KeyboardEvent) {
    const target = event.target;
    const menuOpen = target instanceof Element && target.closest(
      "[role=dialog], [role=menu], [role=listbox], [data-state=open][data-slot=popover-content]",
    );
    if (!onBack || event.defaultPrevented || menuOpen) {
      return;
    }
    if (event.key === "Escape" || (event.altKey && event.key === "ArrowLeft")) {
      event.preventDefault();
      onBack();
    }
  }
</script>

<svelte:window onkeydown={back} />
<header class="flex h-[var(--page-bar-height)] shrink-0 items-center justify-between gap-[var(--bar-gap)] border-b px-[var(--shell-inset)] py-[var(--bar-pad-y)]">
  <div class="flex min-w-0 items-center gap-[var(--shell-gap)]">
    {#if onBack}
      <Button aria-label="Back" size="icon" variant="ghost" onclick={onBack}>
        <ChevronLeft />
      </Button>
    {/if}
    <h1 class="min-w-0 truncate font-heading font-semibold tracking-wide">{title}</h1>
  </div>
  <div class="flex shrink-0 items-center gap-[var(--bar-gap)]">
    {@render actions?.()}
  </div>
</header>
