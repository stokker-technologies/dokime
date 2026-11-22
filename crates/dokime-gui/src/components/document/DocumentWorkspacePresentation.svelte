<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { EvidenceScope } from "@generated/host/model";
  import type { Wire } from "@/lib/rpc/canonical";
  import type { PlotRefreshState, PlotWindowState } from "../../features/cycle/plot-window";
  import type { DocumentCardContentProps } from "./DocumentCardContent.svelte";
  import type { DocumentBlockChrome } from "./DocumentViewRenderer.svelte";
  import type { MediaGalleryArtifact } from "../media-gallery-model";
  import type { ArtifactTreeInput } from "../../lib/artifact-tree";
  import type { TraceabilityExportName } from "../../lib/traceability";
  import type {
    DocumentViewCard,
    DocumentViewPayload,
    DocumentViewSection,
    HealthItem,
    HubRecord,
    Metadata,
    OperatorNote,
    PlotPayload,
    StepExecutionRecord,
    TelemetryStream,
    TraceabilitySnapshot,
  } from "@/lib/types";
  import type { LivePlotStateMap } from "@/plots/live-frames";
  import type { PlotMeasurements } from "@/plots/plot-interaction";
  import type { ReportDefaults } from "@/plots/plot-settings";
  import type { PlotSettingsChange } from "./PlotGroupGrid.svelte";

  type DocumentWorkspaceViewBaseProps = {
    artifacts: ArtifactTreeInput[];
    cardFilter?: (card: DocumentViewCard) => boolean;
    /** The frame each card gets; absent keeps the renderer's default. */
    cardChrome?: (card: DocumentViewCard) => DocumentBlockChrome;
    streams: TelemetryStream[];
    criterionRecords: StepExecutionRecord[];
    document: DocumentViewPayload | null | undefined;
    emptyMessage?: string;
    records: HubRecord[];
    health: HealthItem[];
    mediaArtifacts: MediaGalleryArtifact[];
    notes: OperatorNote[];
    plotPalette?: readonly string[];
    plotParameterContext?: Record<string, unknown> | null;
    runDir?: string | null;
    sectionChrome?: "plain" | "card";
    sections: DocumentViewSection[];
    stepRecords: StepExecutionRecord[];
    traceability: TraceabilitySnapshot | null | undefined;
    /** Names the traceability JSON a reader downloads from the traceability card. */
    traceabilityExportName?: TraceabilityExportName | null;
  };

  type LiveDocumentWorkspaceView = DocumentWorkspaceViewBaseProps & {
    livePlotStates?: LivePlotStateMap;
    onPlotSettingsChange?: PlotSettingsChange;
    plotContextTelemetry: Record<string, TelemetryStream> | null;
    plotResetKey: string;
    plotWindowS?: number | null;
    plotRefresh?: PlotRefreshState;
    sectionWindow?: { sectionId: string; elapsedUnit: string | null; window: PlotWindowState };
    measurements?: PlotMeasurements | null;
    reportDefaults?: ReportDefaults;
    renderMode: "live";
    scope: Wire<EvidenceScope> | null;
  };

  type ReportDocumentWorkspaceView = DocumentWorkspaceViewBaseProps & {
    renderMode: "report";
    reportPlots: Record<string, PlotPayload>;
    reportTelemetry: Record<string, TelemetryStream> | null;
  };

  export type DocumentWorkspaceView = LiveDocumentWorkspaceView | ReportDocumentWorkspaceView;

  function documentCardContentProps(
    props: DocumentWorkspaceView,
    card: DocumentViewCard,
    section: Metadata | null,
    onRetry?: (cardId?: string) => void,
  ): DocumentCardContentProps {
    const shared = {
      artifacts: props.artifacts,
      card,
      sectionTitle: section?.label ?? card.title,
      streams: props.streams,
      criterionRecords: props.criterionRecords,
      documentId: props.document?.document_id,
      records: props.records,
      health: props.health,
      mediaArtifacts: props.mediaArtifacts,
      notes: props.notes,
      onRetry,
      plotPalette: props.plotPalette,
      plotParameterContext: props.plotParameterContext ?? null,
      runDir: props.runDir,
      stepRecords: props.stepRecords,
      traceability: props.traceability,
    };

    if (props.renderMode === "report") {
      return {
        ...shared,
        renderMode: "report",
        reportPlots: props.reportPlots,
        reportTelemetry: props.reportTelemetry,
      };
    }

    return {
      ...shared,
      captureScope: props.scope?.cycleId && props.scope.executionId ? props.scope : null,
      livePlotStates: props.livePlotStates ?? {},
      measurements: props.measurements,
      reportDefaults: props.reportDefaults,
      onPlotSettingsChange: props.onPlotSettingsChange,
      noteScope: props.scope?.cycleId ? props.scope : null,
      plotContextTelemetry: props.plotContextTelemetry,
      plotResetKey: props.plotResetKey,
      renderMode: "live",
    };
  }

  export function visibleDocumentPlots(sections: DocumentViewSection[], cardFilter: (card: DocumentViewCard) => boolean) {
    return sections.flatMap((section) =>
      section.cards.filter((card) => card.kind === "plots" && cardFilter(card)).flatMap((card) => card.plots),
    );
  }
</script>

<script lang="ts">
  import DocumentCardContent from "./DocumentCardContent.svelte";
  import DocumentViewRenderer from "./DocumentViewRenderer.svelte";
  import { isDocumentSurfaceCard } from "./document-surface-policy";
  import TraceabilityJsonDialog from "../traceability/TraceabilityJsonDialog.svelte";
  import PlotWindowControl from "../../features/cycle/PlotWindowControl.svelte";

  type Props = DocumentWorkspaceView & {
    cardOverride?: Snippet<[DocumentCardContentProps]>;
    onRetry?: (cardId?: string) => void;
    traceabilitySave?: ((json: string, filename: string) => Promise<void>) | null;
  };

  let props: Props = $props();

  function cardFilter(card: DocumentViewCard): boolean {
    const authoredFilter = props.cardFilter ?? isDocumentSurfaceCard;
    if (!authoredFilter(card)) {
      return false;
    }
    if (props.renderMode === "report") {
      if (card.kind === "artifacts") {
        return props.artifacts.length > 0;
      }
      if (card.kind === "media") {
        return props.mediaArtifacts.length > 0;
      }
    }
    return true;
  }
</script>

<DocumentViewRenderer
  {cardFilter}
  cardChrome={props.cardChrome}
  document={props.document}
  emptyMessage={props.emptyMessage}
  sectionChrome={props.sectionChrome}
  onRetry={props.onRetry}
  hasSectionAction={(sectionId) => props.renderMode === "live" && props.sectionWindow?.sectionId === sectionId}
  hasCardAction={(card) => card.kind === "traceability"}
>
  {#snippet renderCard(
    card,
    section,
  )}
    {@const content = documentCardContentProps(props, card, section, props.onRetry)}
    {#if props.cardOverride &&
      content.renderMode === "live" &&
      !card.failure &&
      !card.loading &&
      (card.kind === "artifacts" || card.kind === "media" || card.kind === "notes")}
      {@render props.cardOverride(content)}
    {:else}
      <DocumentCardContent {...content} />
    {/if}
  {/snippet}
  {#snippet cardAction()}
    <TraceabilityJsonDialog exportName={props.traceabilityExportName} traceability={props.traceability} save={props.traceabilitySave} />
  {/snippet}
  {#snippet sectionAction()}
    {#if props.renderMode === "live" && props.sectionWindow}
      <PlotWindowControl elapsedUnit={props.sectionWindow.elapsedUnit} live window={props.sectionWindow.window} />
    {/if}
  {/snippet}
</DocumentViewRenderer>
