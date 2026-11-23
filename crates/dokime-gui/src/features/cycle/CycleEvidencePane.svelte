<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HubLiveState, PromptRequest, TestCyclePlan } from "../../lib/types";
  import PageChrome from "../../shell/PageChrome.svelte";
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
    hasExecutions,
    lead,
    pageActions,
    plan,
    prompts,
    snapshot,
  }: {
    context: LiveCycleContext;
    cycleId: string;
    /** A cycle without executions has no document yet: nothing to read, nothing failed. */
    hasExecutions: boolean;
    lead: Snippet;
    pageActions: Snippet;
    plan: TestCyclePlan;
    prompts: PromptRequest[];
    snapshot: HubLiveState | null;
  } = $props();
  const session = () => context.session;
  const plotWindow = createPlotWindow();
  const query = createScopedDocumentQuery(session, () => ({ cycleId, executionId: null, enabled: hasExecutions }));
  const document = $derived(query.data ?? null);
  const surface = createLiveDocumentSurface(session, {
    get document() {
      return document;
    },
    execution: null,
    live: true,
    get snapshot() {
      return snapshot;
    },
    get scope() {
      return { cycleId };
    },
    parameters: { case: undefined, execution: undefined },
    get noteElapsedStart() {
      return plan.test_cycle.started_at;
    },
    plotWindow,
    traceabilityExportName: null,
  });
  const pane = $derived(
    evidencePaneState({
      hasDocument: document !== null,
      error: query.error,
      failureReason: query.failureReason,
      pending: hasExecutions && query.isPending,
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

<CycleEvidenceView live {surface} {pane} {onRetry} {lead}>
  {#snippet promptQueue()}
    <PromptQueue {prompts} {records} session={context.session} />
  {/snippet}
  {#snippet documentView()}
    <LiveCycleDocument {context} {surface} windowControl cardFilter={isCycleEvidenceCard} {onRetry} />
  {/snippet}
</CycleEvidenceView>
