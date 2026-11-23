<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { testSuitesQueryOptions } from "../../components/suite-picker/server-state";
  import MediaCaptureControls from "../../components/MediaCaptureControls.svelte";
  import MediaGalleryView from "../../components/MediaGalleryView.svelte";
  import NativeArtifactTree from "../../components/NativeArtifactTree.svelte";
  import { nativeMediaTransfers } from "../../components/native-media-transfers";
  import OperatorNotes from "../../components/document/OperatorNotes.svelte";
  import OperatorNotesContent from "../../components/document/OperatorNotesContent.svelte";
  import { createExecutionProgress } from "../../app/live/execution-progress.svelte";
  import type { LiveExecutionProgress } from "../../app/live/execution-progress";
  import type { HubLiveState, PromptRequest, TestCyclePlan } from "../../lib/types";
  import type { FocusMode } from "../../shell/focus-mode";
  import PageChrome from "../../shell/PageChrome.svelte";
  import { type createCycleCommands, executionDetailQueryOptions } from "./cycle-queries";
  import { type CycleExecutionRow, isUnstartedExecution } from "./cycle-workspace-model";
  import DocumentPagination from "./DocumentPagination.svelte";
  import { createScopedDocumentQuery } from "./document-query.svelte";
  import { evidencePaneState } from "./document-query-model";
  import { executionActions } from "./execution-actions";
  import ExecutionActionButton from "./ExecutionActionButton.svelte";
  import ExecutionControl from "./ExecutionControl.svelte";
  import { executionPagePrompts } from "./execution-sections-model";
  import ExecutionSectionsView, { executionReportCardChrome, isExecutionReportCard } from "./ExecutionSectionsView.svelte";
  import LiveCycleDocument, { type LiveCycleContext } from "./LiveCycleDocument.svelte";
  import { createLiveDocumentSurface } from "./live-document.svelte";
  import NodeActionCluster from "./NodeActionCluster.svelte";
  import type { ExecutionMenu } from "./node-menu";
  import { createPlotWindow } from "./plot-window-state";
  import PromptQueue from "./PromptQueue.svelte";

  let {
    context,
    commands,
    cycleId,
    focus,
    plan,
    row,
    snapshot,
    titlePrefix,
    label,
    onRename,
    menu,
    prompts,
    onSelectExecution,
    observedExecutionId,
    progress,
  }: {
    context: LiveCycleContext;
    commands: ReturnType<typeof createCycleCommands>;
    cycleId: string;
    focus: FocusMode;
    plan: TestCyclePlan;
    row: CycleExecutionRow;
    snapshot: HubLiveState | null;
    titlePrefix: string;
    label: string;
    onRename: ((label: string) => Promise<unknown>) | null;
    menu: ExecutionMenu;
    prompts: PromptRequest[];
    onSelectExecution: (executionId: string) => void;
    observedExecutionId: string | null;
    progress: LiveExecutionProgress | null;
  } = $props();

  const session = () => context.session;
  const client = () => session().queryClient;
  const plotWindow = createPlotWindow();
  const suites = createQuery(() => testSuitesQueryOptions(session().identity), client);
  const pluginAvailable = $derived(
    suites.data?.some((suite) => suite.members.some((template) => template.template_id === row.case.templateId)),
  );
  const started = $derived(row.execution && !isUnstartedExecution(row.execution.status) ? row.execution : null);
  const startedId = $derived(started?.test_execution_id ?? null);
  const scope = $derived(startedId ? { cycleId, executionId: startedId } : { cycleId, caseId: row.case.caseId });
  const detail = createQuery(() => executionDetailQueryOptions(session().identity, startedId), client);
  const execution = $derived(detail.data ?? started);
  const documentQuery = createScopedDocumentQuery(session, () => ({ cycleId, executionId: startedId, enabled: Boolean(startedId) }));
  const document = $derived(started ? (documentQuery.data ?? null) : null);
  const surface = createLiveDocumentSurface(session, {
    get document() {
      return document;
    },
    get execution() {
      return execution;
    },
    live: true,
    get snapshot() {
      return snapshot;
    },
    get scope() {
      return scope;
    },
    get parameters() {
      return { case: row.case.testCase?.parameters, execution: execution?.parameters };
    },
    get noteElapsedStart() {
      return started?.started_at ?? plan.test_cycle.started_at;
    },
    plotWindow,
    get traceabilityExportName() {
      return { cycleLabel: plan.test_cycle.title, executionOrdinal: row.ordinal };
    },
  });
  const pane = $derived(
    evidencePaneState({
      hasDocument: document !== null,
      error: documentQuery.error,
      failureReason: documentQuery.failureReason,
      pending: documentQuery.isPending,
    }),
  );
  const executionId = $derived(row.execution?.test_execution_id ?? null);
  const pagePrompts = $derived(executionPagePrompts(prompts, executionId));
  const commanded = $derived(execution ?? row.execution);
  const selectedProgress = createExecutionProgress(session, () => (startedId === observedExecutionId ? null : startedId));
  const actions = $derived(
    executionActions(
      {
        cycleActive: plan.test_cycle.phase === "active",
        execution: commanded,
        onSelectExecution,
        prompts: pagePrompts,
        pluginAvailable,
        row,
      },
      commands,
      startedId === observedExecutionId ? progress : selectedProgress(),
    ),
  );
  const primaryAction = $derived(actions?.state.actions[0]);
  const reportSubject = $derived(executionId ? { executionId, label: titlePrefix + label, sealed: commanded?.sealed === true } : null);
  const transfers = $derived(nativeMediaTransfers(session()));
  const onRetry = (cardId?: string) => void documentQuery.retryCard(cardId);
</script>

<PageChrome title={`Cycle › ${titlePrefix}${label}`} actions={headerActions} />

{#snippet primary()}
  {#if actions && primaryAction && primaryAction !== "Stop" && primaryAction !== "Cancel"}
    <ExecutionActionButton action={primaryAction} {actions} />
  {/if}
{/snippet}

{#snippet headerActions()}
  {#if execution?.report}
    {@const report = execution.report}
    <span role="status" class="text-xs text-muted-foreground" title={report.failure ?? undefined}>
      Report {report.state === "pending" ? "Pending" : report.state === "failed" ? "Failed" : "Ready"}{report.failure
        ? `: ${report.failure}`
        : ""}
    </span>
  {/if}
  <NodeActionCluster
    {commands}
    {focus}
    menu={pluginAvailable === false ? { ...menu, onRerun: null, rerunRefusal: "Plugin Unavailable" } : menu}
    {primary}
    subject={reportSubject}
    reportSealed={execution?.sealed === true}
  />
  <DocumentPagination navigation={documentQuery.navigation} />
{/snippet}

<ExecutionSectionsView
  {plan}
  {row}
  live
  {titlePrefix}
  {label}
  {onRename}
  {started}
  {surface}
  {pane}
  {plotWindow}
  {onRetry}
  traceabilitySave={context.traceabilitySave}
>
  {#snippet documentView()}
    <LiveCycleDocument {context} {surface} cardChrome={executionReportCardChrome} cardFilter={isExecutionReportCard} {onRetry} />
  {/snippet}
  {#snippet control()}
    {#if actions}
      <ExecutionControl {actions} ordinal={row.ordinal} />
    {/if}
  {/snippet}
  {#snippet promptQueue()}
    <PromptQueue prompts={pagePrompts} records={surface.view.records} steps={surface.view.stepRecords} session={session()} />
  {/snippet}
  {#snippet notesContent()}
    {#if startedId}
      <OperatorNotesContent notes={surface.view.notes} {scope} session={session()} />
    {:else}
      <OperatorNotes notes={surface.view.notes} />
    {/if}
  {/snippet}
  {#snippet mediaContent()}
    {#if startedId}
      <MediaCaptureControls />
    {/if}
    {#if surface.shared.mediaArtifacts.length}
      <MediaGalleryView artifacts={surface.shared.mediaArtifacts} emptyMessage="" {transfers} />
    {/if}
  {/snippet}
  {#snippet artifactsContent()}
    <NativeArtifactTree artifacts={surface.shared.artifacts} emptyMessage="" session={session()} />
  {/snippet}
</ExecutionSectionsView>
