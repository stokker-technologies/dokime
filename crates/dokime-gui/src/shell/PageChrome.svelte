<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { Ellipsis } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import { buttonVariants } from "../components/ui/button-variants";
  import { moveToPageActions, pageActionsReady, publishPageTitle } from "./page-chrome.svelte";

  let { title, actions, overflow }: { title: string; actions?: Snippet; overflow?: Snippet } = $props();
  publishPageTitle(() => title);
</script>

{#if pageActionsReady() && (actions || overflow)}
  <div class="contents" {@attach moveToPageActions}>
    {@render actions?.()}
    {#if overflow}
      <DropdownMenu.Root>
        <DropdownMenu.Trigger aria-label="Page Actions" class={buttonVariants({ size: "icon", variant: "ghost" })}>
          <Ellipsis />
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content align="end" class="z-50 min-w-32 rounded-lg bg-popover p-1 text-popover-foreground ring-1 ring-border">
            {@render overflow()}
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>
    {/if}
  </div>
{/if}
