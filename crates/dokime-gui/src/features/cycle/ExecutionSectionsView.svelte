<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { DocumentBlockChrome } from "../../components/document/DocumentViewRenderer.svelte";
  import { isDocumentSurfaceCard } from "../../components/document/document-surface-policy";
  import type { DocumentViewCard } from "../../lib/types";
  import { executionSectionOf } from "./execution-sections-model";

  /** Report Holds The Authored Main Cards With No Dedicated Section. */
  export function isExecutionReportCard(card: DocumentViewCard): boolean {
    return isDocumentSurfaceCard(card) && executionSectionOf(card.kind) === "report";
  }

  export function executionReportCardChrome(card: DocumentViewCard): DocumentBlockChrome {
    return card.kind === "plots" ? "flat" : "embedded";
  }
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import ArtifactTreeView from "../../components/ArtifactTreeView.svelte";
  import DocumentTraceability from "../../components/document/DocumentTraceability.svelte";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import MediaGalleryView from "../../components/MediaGalleryView.svelte";
  import OperatorNotes from "../../components/document/OperatorNotes.svelte";
  import TraceabilityJsonDialog from "../../components/traceability/TraceabilityJsonDialog.svelte";
  import type { TestCyclePlan, TestExecution } from "../../lib/types";
  import FrozenConsole from "../../shell/FrozenConsole.svelte";
  import { pageContentClass } from "../../shell/page-layout";
  import { cycleRowStatus, type CycleExecutionRow } from "./cycle-workspace-model";
  import { BUSY_NOTICE, type evidencePaneState } from "./document-query-model";
  import type { DocumentSurface } from "./document-view-model";
  import { documentHasWindowedCards, documentWindowElapsedUnit } from "./document-workspace";
  import DocumentViewPresentation from "./DocumentViewPresentation.svelte";
  import ExecutionParameters from "./ExecutionParameters.svelte";
  import ExecutionSectionCard from "./ExecutionSectionCard.svelte";
  import ExecutionStepsCriteria from "./ExecutionStepsCriteria.svelte";
  import { executionDocumentSections, executionParameterRows } from "./execution-sections-model";
  import NodeSummary from "./NodeSummary.svelte";
  import type { PlotWindowState } from "./plot-window";
  import PlotWindowControl from "./PlotWindowControl.svelte";
  import PromptHistory from "./PromptHistory.svelte";
  import RecordsSection from "./RecordsSection.svelte";

  type Props = {
    plan: TestCyclePlan | null;
    row: CycleExecutionRow;
    live: boolean;
    titlePrefix: string;
    label: string;
    onRename: ((label: string) => Promise<unknown>) | null;
    started: TestExecution | null;
    surface: DocumentSurface;
    pane: ReturnType<typeof evidencePaneState>;
    plotWindow?: PlotWindowState;
    onRetry?: (cardId?: string) => void;
    control?: Snippet;
    promptQueue?: Snippet;
    notesContent?: Snippet;
    mediaContent?: Snippet;
    artifactsContent?: Snippet;
    documentView?: Snippet;
    traceabilitySave?: ((json: string, filename: string) => Promise<void>) | null;
  };

  let {
    plan,
    row,
    live,
    titlePrefix,
    label,
    onRename,
    started,
    surface,
    pane,
    plotWindow,
    onRetry,
    control,
    promptQueue,
    notesContent,
    mediaContent,
    artifactsContent,
    documentView,
    traceabilitySave,
  }: Props = $props();

  const { shared, view } = $derived(surface);
  const present = $derived.by(() => {
    const cards = view.sections.flatMap((section) => section.cards).filter(isDocumentSurfaceCard);
    const sections = new Set(executionDocumentSections(cards));
    if (view.sections.some((section) => section.loading || section.unavailable)) {
      sections.add("report");
    }
    return sections;
  });
  const reportSections = $derived(view.sections.map((section) => ({ ...section, cards: section.cards.filter(isExecutionReportCard) })));
  const template = $derived(plan?.templates.find((item) => item.template_id === row.case.templateId));
  const parameterRows = $derived(executionParameterRows(row.case.testCase?.parameters, template?.parameters ?? []));
  const hasSteps = $derived(view.stepRecords.length > 0 || view.criterionRecords.length > 0);
  const hasTraceability = $derived(Boolean(view.traceability?.groups.length || view.traceability?.warnings.length));

  const unstartedNotice = "This case has not run yet. These are the parameters the next attempt will use.";
</script>

{#snippet plotAction()}
  {#if plotWindow}
    <PlotWindowControl elapsedUnit={documentWindowElapsedUnit(reportSections)} {live} window={plotWindow} />
  {/if}
{/snippet}

<div class="grid min-h-0">
  <div class={[pageContentClass, "min-h-0 overflow-y-auto"]}>
    <NodeSummary
      description={template?.description}
      markdown
      {live}
      {onRename}
      prefix={titlePrefix}
      executionProgress={{ execution: started, steps: view.stepRecords }}
      status={cycleRowStatus({ ...row, execution: started ?? row.execution })}
      subject="Execution"
      title={label}
    />

    {@render control?.()}
    {#if promptQueue}
      {@render promptQueue()}
    {:else}
      <PromptHistory records={view.records} />
    {/if}

    <ExecutionSectionCard
      description={started ? null : unstartedNotice}
      empty={parameterRows.length ? null : "This case has no parameters."}
      title="Parameters"
    >
      {#if parameterRows.length}
        <ExecutionParameters rows={parameterRows} />
      {/if}
    </ExecutionSectionCard>

    {#if started}
      {#if pane.kind === "failed"}
        <ErrorCard failure={pane.failure} {onRetry} />
      {:else if pane.kind === "busy" || pane.kind === "loading"}
        <ExecutionSectionCard empty={pane.kind === "busy" ? BUSY_NOTICE : "Reading the document for this execution."} title="Loading" />
      {:else}
        {#if pane.failure}
          <ErrorCard failure={pane.failure} {onRetry} />
        {/if}

        {#if present.has("steps-criteria")}
          <ExecutionSectionCard empty={hasSteps ? null : "No steps or criteria recorded yet."} title="Steps & Criteria">
            {#if hasSteps}
              <ExecutionStepsCriteria criteria={view.criterionRecords} {live} steps={view.stepRecords} />
            {/if}
          </ExecutionSectionCard>
        {/if}

        {#if present.has("notes")}
          <ExecutionSectionCard empty={notesContent || view.notes.length ? null : "No operator notes recorded."} title="Operator Notes">
            {#if notesContent}
              {@render notesContent()}
            {:else if view.notes.length}
              <OperatorNotes notes={view.notes} />
            {/if}
          </ExecutionSectionCard>
        {/if}

        {#if present.has("media") && (live || shared.mediaArtifacts.length)}
          <ExecutionSectionCard empty={shared.mediaArtifacts.length ? null : "No media recorded."} title="Media">
            {#if mediaContent}
              {@render mediaContent()}
            {:else if shared.mediaArtifacts.length}
              <MediaGalleryView artifacts={shared.mediaArtifacts} emptyMessage="" />
            {/if}
          </ExecutionSectionCard>
        {/if}

        {#if present.has("artifacts") && (live || shared.artifacts.length)}
          <ExecutionSectionCard empty={shared.artifacts.length ? null : "No artifacts recorded."} title="Artifacts">
            {#if shared.artifacts.length}
              {#if artifactsContent}
                {@render artifactsContent()}
              {:else}
                <ArtifactTreeView artifacts={shared.artifacts} emptyMessage="" />
              {/if}
            {/if}
          </ExecutionSectionCard>
        {/if}

        {#if present.has("report") || live}
          <ExecutionSectionCard
            action={documentHasWindowedCards(reportSections) ? plotAction : undefined}
            empty={present.has("report") ? null : "This execution's document authors no report content."}
            title="Report"
          >
            {#if present.has("report")}
              {#if documentView}
                {@render documentView()}
              {:else}
                <DocumentViewPresentation cardChrome={executionReportCardChrome} cardFilter={isExecutionReportCard} {surface} {onRetry} />
              {/if}
            {/if}
          </ExecutionSectionCard>
        {/if}

        <RecordsSection {live} records={view.records} />

        {#if present.has("traceability")}
          <ExecutionSectionCard empty={hasTraceability ? null : "No traceability information available."} title="Traceability">
            {#snippet action()}
              <TraceabilityJsonDialog exportName={shared.traceabilityExportName} traceability={view.traceability} save={traceabilitySave} />
            {/snippet}
            {#if hasTraceability}
              <DocumentTraceability traceability={view.traceability} />
            {/if}
          </ExecutionSectionCard>
        {/if}

        {#if !surface.options.live}
          <FrozenConsole records={view.records} />
        {/if}
      {/if}
    {/if}
  </div>
</div>
