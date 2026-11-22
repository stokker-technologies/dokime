<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import { isDocumentSurfaceCard } from "../../components/document/document-surface-policy";
  import type { DocumentViewCard } from "../../lib/types";

  /** The Cycle Closes With One Shared Records Section. */
  export function isCycleEvidenceCard(card: DocumentViewCard): boolean {
    return isDocumentSurfaceCard(card) && card.kind !== "records";
  }
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import FrozenConsole from "../../shell/FrozenConsole.svelte";
  import { pageContentClass } from "../../shell/page-layout";
  import { BUSY_NOTICE, type evidencePaneState } from "./document-query-model";
  import type { DocumentSurface } from "./document-view-model";
  import DocumentViewPresentation from "./DocumentViewPresentation.svelte";
  import type { PlotWindowState } from "./plot-window";
  import PromptHistory from "./PromptHistory.svelte";
  import RecordsSection from "./RecordsSection.svelte";

  type Props = {
    live: boolean;
    surface: DocumentSurface;
    pane: ReturnType<typeof evidencePaneState>;
    plotWindow?: PlotWindowState;
    onRetry?: (cardId?: string) => void;
    documentView?: Snippet;
    lead?: Snippet;
    promptQueue?: Snippet;
  };

  let { live, surface, pane, plotWindow, onRetry, documentView, lead, promptQueue }: Props = $props();
  const cycleRecords = $derived(surface.view.records.filter((record) => !record.scope?.executionId));
</script>

<div class="grid min-h-0">
  <div class={[pageContentClass, "min-h-0 overflow-y-auto"]}>
    {@render lead?.()}
    {#if promptQueue}
      {@render promptQueue()}
    {:else}
      <PromptHistory records={cycleRecords} />
    {/if}
    {#if pane.kind === "failed"}
      <ErrorCard failure={pane.failure} {onRetry} />
    {:else if pane.kind === "busy" || pane.kind === "loading"}
      <div class="grid justify-items-stretch gap-2 py-8">
        <p class="font-sans text-sm font-semibold">Loading</p>
        <p class="max-w-prose text-sm text-muted-foreground">
          {pane.kind === "busy" ? BUSY_NOTICE : "Reading the document for this selection."}
        </p>
      </div>
    {:else}
      <div class="grid gap-6">
        {#if pane.failure}
          <ErrorCard failure={pane.failure} {onRetry} />
        {/if}
        {#if documentView}
          {@render documentView()}
        {:else}
          <DocumentViewPresentation cardFilter={isCycleEvidenceCard} {plotWindow} {surface} {onRetry} />
        {/if}
        <RecordsSection {live} records={surface.view.records} />
        {#if !surface.options.live}
          <FrozenConsole records={surface.view.records} />
        {/if}
      </div>
    {/if}
  </div>
</div>
