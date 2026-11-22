<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import ErrorCard from "../components/ErrorCard.svelte";
  import { failureOf } from "../lib/notification-details";
  import type { PlotPlaceholderText } from "./plot-placeholder";

  type Props = PlotPlaceholderText & {
    title?: string;
    description?: string | null;
    actions?: Snippet;
  };

  let { actions, description, detail, label, title, tone }: Props = $props();
</script>

{#if tone === "error"}
  <article class="plot-card grid min-w-0 max-w-full overflow-hidden gap-2">
    {#if title || actions}
      <div class="flex items-center justify-between gap-3">
        {#if title}
          <strong class="min-w-0 truncate" {title}>{title}</strong>
        {/if}
        {@render actions?.()}
      </div>
    {/if}
    <ErrorCard failure={failureOf(detail, title ?? label, [title, label, detail].join(":"))} />
  </article>
{:else}
  <div
    class="plot-empty relative grid min-h-44 place-content-center justify-items-center gap-2 rounded-lg bg-muted/20 p-4 text-center text-muted-foreground"
    data-testid="plot-placeholder"
    data-tone={tone}
  >
    {#if actions}
      <div class="absolute top-2 right-2">{@render actions()}</div>
    {/if}
    {#if title}
      <strong>{title}</strong>
    {/if}
    {#if description}
      <span class="text-xs">{description}</span>
    {/if}
    <span>{label}</span>
    <span class="max-w-prose text-xs [overflow-wrap:anywhere]">{detail}</span>
  </div>
{/if}
