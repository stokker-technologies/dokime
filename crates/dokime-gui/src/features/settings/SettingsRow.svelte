<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import { failureOf } from "../../lib/notification-details";
  import { cn } from "../../lib/utils";

  let {
    label,
    description,
    for: controlId,
    class: style,
    error,
    failure,
    notificationContext,
    children,
    status,
  }: {
    label: string;
    description?: string;
    for?: string;
    class?: string;
    error?: string;
    failure?: unknown;
    notificationContext?: "action";
    children?: Snippet;
    status?: Snippet;
  } = $props();
  const id = $props.id();
</script>

<div
  class={cn(
    "grid min-h-[var(--settings-row-height)] grid-cols-1 xl:grid-cols-(--layout-settings-row) has-[[role=alert]]:grid-cols-1 xl:has-[[role=alert]]:grid-cols-1 items-center gap-x-6 gap-y-3 px-4 py-3",
    style,
  )}
>
  <div class="min-w-0">
    {#if controlId}
      <label class="block text-sm leading-snug font-medium" for={controlId}>{label}</label>
    {:else}
      <div class="text-sm leading-snug font-medium">{label}</div>
    {/if}
    {#if description}
      <p class="text-xs leading-snug text-pretty text-muted-foreground">{description}</p>
    {/if}
  </div>
  <div class="grid min-w-0 gap-2">
    <div class="flex min-w-0 items-center justify-end gap-3">
      <div class="flex min-w-0 flex-1 flex-wrap items-center justify-end gap-2">{@render children?.()}</div>
    </div>
    {#if status}
      <div class="text-right">{@render status()}</div>
    {/if}
  </div>
  {#if error}
    <ErrorCard {notificationContext} failure={failureOf(failure ?? error, "Operation Failed", failure ? undefined : `${id}:${error}`)} />
  {/if}
</div>
