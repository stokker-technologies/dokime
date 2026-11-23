<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HubLiveState, PromptRequest, TestCyclePlan } from "../../lib/types";
  import PageChrome from "../../shell/PageChrome.svelte";
  import { storeState } from "../../stores/store-state";
  import CycleEvidenceView, { isCycleEvidenceCard } from "./CycleEvidenceView.svelte";
  import DocumentPagination from "./DocumentPagination.svelte";
  import { createScopedDocumentQuery } from "./document-query.svelte";
  import { evidencePaneState } from "./document-query-model";
  import LiveCycleDocument, { type LiveCycleContext } from "./LiveCycleDocument.svelte";
  import { createLiveDocumentSurface } from "./live-document.svelte";
  import { createPlotWindow } from "./plot-window-state";
  import PromptQueue from "./PromptQueue.svelte";

  let {
    context,
    cycleId,
    lead,
    pageActions,
    plan,
    prompts,
    snapshot,
  }: {
    context: LiveCycleContext;
    cycleId: string;
    lead: Snippet;
    pageActions: Snippet;
    plan: TestCyclePlan;
    prompts: PromptRequest[];
    snapshot: HubLiveState | null;
  } = $props();
  const session = () => context.session;
  const windowState = storeState(createPlotWindow(), (state) => state);
  const plotWindow = $derived(windowState());
  const query = createScopedDocumentQuery(session, () => ({ cycleId, executionId: null, enabled: true }));
  const document = $derived(query.data ?? null);
  const readSurface = createLiveDocumentSurface(session, () => ({
    document,
    execution: null,
    live: true,
    snapshot,
    scope: { cycleId },
    parameters: { case: undefined, execution: undefined },
    noteElapsedStart: plan.test_cycle.started_at,
    plotWindow: plotWindow.option,
    plotRefresh: plotWindow.refresh,
    traceabilityExportName: null,
  }));
  const surface = $derived(readSurface());
  const pane = $derived(
    evidencePaneState({
      hasDocument: document !== null,
      error: query.error,
      failureReason: query.failureReason,
      pending: query.isPending,
    }),
  );
  const records = $derived(surface.view.records.filter((record) => !record.scope?.executionId));
  const onRetry = (cardId?: string) => void query.retryCard(cardId);
</script>

<PageChrome title={`Cycles › ${plan.test_cycle.title}`} actions={headerActions} />

{#snippet headerActions()}
  {@render pageActions()}
  <DocumentPagination navigation={query.navigation} />
{/snippet}

<CycleEvidenceView live {surface} {pane} {plotWindow} {onRetry} {lead}>
  {#snippet promptQueue()}
    <PromptQueue {prompts} {records} session={context.session} />
  {/snippet}
  {#snippet documentView()}
    <LiveCycleDocument {context} {surface} {plotWindow} cardFilter={isCycleEvidenceCard} {onRetry} />
  {/snippet}
</CycleEvidenceView>
