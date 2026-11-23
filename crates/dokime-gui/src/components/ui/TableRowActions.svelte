<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { Ellipsis } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import Button from "./Button.svelte";
  import { buttonVariants } from "./button-variants";

  let {
    primary,
    label,
    onSelect,
    icon,
    disabled,
    children,
  }: {
    primary: string;
    label: string;
    onSelect: () => void;
    icon: Snippet;
    disabled?: boolean;
    children?: Snippet;
  } = $props();
</script>

<div class="flex flex-wrap items-center justify-end gap-1">
  <Button variant="ghost" size="icon-sm" aria-label={primary} title={primary} {disabled} onclick={onSelect}> {@render icon()} </Button>
  {#if children}
    <DropdownMenu.Root>
      <DropdownMenu.Trigger
        class={buttonVariants({ variant: "ghost", size: "icon-sm" })}
        aria-label={`Actions for ${label}`}
        data-testid="table-row-actions"
      >
        <Ellipsis />
      </DropdownMenu.Trigger>
      <DropdownMenu.Portal>
        <DropdownMenu.Content align="end" class="z-50 min-w-32 rounded-lg bg-popover p-1 text-popover-foreground ring-1 ring-border">
          {@render children()}
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  {/if}
</div>
