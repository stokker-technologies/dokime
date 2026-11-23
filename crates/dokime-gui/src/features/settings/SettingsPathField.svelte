<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Check, Copy, FolderOpen } from "lucide-svelte";
  import { onDestroy } from "svelte";
  import Button from "../../components/ui/Button.svelte";
  import { openLocalPath } from "../../lib/desktop-open";
  import { failureOf } from "../../lib/notification-details";
  import { toast } from "../../lib/notifications";

  const GUI_COPIED_INDICATOR_MS = 1_500;
  let {
    id,
    openable = false,
    placeholder = "Not Set",
    value,
  }: {
    id?: string;
    openable?: boolean;
    placeholder?: string;
    value: string | null;
  } = $props();
  let copied = $state(false);
  let indicator: number | undefined;

  onDestroy(() => window.clearTimeout(indicator));

  async function copyPath() {
    if (!value) {
      return;
    }
    try {
      await navigator.clipboard.writeText(value);
      copied = true;
      window.clearTimeout(indicator);
      indicator = window.setTimeout(() => (copied = false), GUI_COPIED_INDICATOR_MS);
    } catch (error: unknown) {
      toast.error(failureOf(error, "Could Not Copy Path"));
    }
  }

  function openPath() {
    if (value) {
      void openLocalPath(value).catch(() => undefined);
    }
  }
</script>

<div class="flex h-[var(--control-height)] w-full min-w-0 items-center gap-1 rounded-lg border bg-background pr-1 pl-2">
  <span class="min-w-0 flex-1 truncate text-left font-mono text-xs [direction:rtl]" {id} title={value ?? undefined}>
    {#if value}
      <bdi>{value}</bdi>
    {:else}
      <span class="text-muted-foreground">{placeholder}</span>
    {/if}
  </span>
  <Button aria-label="Copy Path" disabled={!value} onclick={copyPath} size="icon-sm" title="Copy Path" variant="ghost">
    {#if copied}
      <Check />
    {:else}
      <Copy />
    {/if}
  </Button>
  {#if openable}
    <Button aria-label="Open Path" disabled={!value} onclick={openPath} size="icon-sm" title="Open Path" variant="ghost">
      <FolderOpen />
    </Button>
  {/if}
</div>
