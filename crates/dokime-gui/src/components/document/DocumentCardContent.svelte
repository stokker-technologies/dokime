<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { EvidenceScope } from "@generated/host/model";
  import type { ArtifactTreeInput } from "@/lib/artifact-tree";
  import type { Wire } from "@/lib/rpc/canonical";
  import type {
    DocumentViewCard,
    DocumentViewPlot,
    HealthItem,
    HubRecord,
    OperatorNote,
    PlotPayload,
    StepExecutionRecord,
    TelemetryStream,
    TraceabilitySnapshot,
  } from "@/lib/types";
  import { emptyLivePlotState, type LivePlotStateMap } from "@/plots/live-frames";
  import type { PlotMeasurements } from "@/plots/plot-interaction";
  import type { ReportDefaults } from "@/plots/plot-settings";
  import type { MediaGalleryArtifact } from "../media-gallery-model";
  import type { PlotSettingsChange } from "./PlotGroupGrid.svelte";

  type SharedProps = {
    artifacts: ArtifactTreeInput[];
    card: DocumentViewCard;
    streams: TelemetryStream[];
    criterionRecords: StepExecutionRecord[];
    documentId?: string;
    records: HubRecord[];
    health: HealthItem[];
    mediaArtifacts: MediaGalleryArtifact[];
    notes: OperatorNote[];
    plotParameterContext?: Record<string, unknown> | null;
    plotPalette?: readonly string[];
    runDir?: string | null;
    sectionTitle: string;
    stepRecords: StepExecutionRecord[];
    traceability: TraceabilitySnapshot | null | undefined;
    onRetry?: (cardId: string) => void;
  };

  type LiveProps = SharedProps & {
    captureScope?: Wire<EvidenceScope> | null;
    noteScope?: Wire<EvidenceScope> | null;
    livePlotStates: LivePlotStateMap;
    onPlotSettingsChange?: PlotSettingsChange;
    plotContextTelemetry: Record<string, TelemetryStream> | null;
    plotResetKey: string;
    measurements?: PlotMeasurements | null;
    reportDefaults?: ReportDefaults;
    renderMode: "live";
  };

  type ReportProps = SharedProps & {
    renderMode: "report";
    reportPlots: Record<string, PlotPayload>;
    reportTelemetry: Record<string, TelemetryStream> | null;
  };

  export type DocumentCardContentProps = LiveProps | ReportProps;

  function reportPlotStates(plots: DocumentViewPlot[], payloads: Record<string, PlotPayload>): LivePlotStateMap {
    return Object.fromEntries(
      plots.flatMap((plot) => {
        const payload = payloads[plot.plot_id];
        return payload ? [[plot.plot_id, { ...emptyLivePlotState(), payload, status: "live" as const }]] : [];
      }),
    );
  }
</script>

<script lang="ts">
  import ExecutionStepsCriteria from "../../features/cycle/ExecutionStepsCriteria.svelte";
  import ArtifactTreeView from "../ArtifactTreeView.svelte";
  import ErrorCard from "../ErrorCard.svelte";
  import MediaGalleryView from "../MediaGalleryView.svelte";
  import DocumentMetricFlow from "./overview/DocumentMetricFlow.svelte";
  import { metricFlowStorageKey } from "./overview/metric-model";
  import DocumentStatus from "./DocumentStatus.svelte";
  import DocumentTable from "./DocumentTable.svelte";
  import DocumentTraceability from "./DocumentTraceability.svelte";
  import OperatorNotes from "./OperatorNotes.svelte";
  import PlotGroupGrid from "./PlotGroupGrid.svelte";
  import RecordTimeline from "./RecordTimeline.svelte";

  let props: DocumentCardContentProps = $props();
  const card = $derived(props.card);
  const report = $derived(props.renderMode === "report");
  const groups = $derived(
    card.kind === "overview"
      ? card.groups.map((group) => ({
          groupId: group.group_id,
          title: group.title,
          metrics: group.metrics.map((metric) => ({
            metricId: metric.metric_id,
            detail: metric.detail ?? undefined,
            label: metric.label,
            rawValue: metric.value,
            presentation: metric.presentation,
            state: metric.state,
            value: metric.formatted,
            exactValue: metric.exact ?? undefined,
          })),
        }))
      : [],
  );
</script>

{#if card.failure}
  <ErrorCard failure={card.failure} onRetry={report || !props.onRetry ? undefined : () => props.onRetry?.(card.card_id)} />
{:else if card.loading}
  <p role="status" class="text-sm text-muted-foreground">Loading…</p>
{:else if card.kind === "overview"}
  <DocumentMetricFlow {groups} storageKey={metricFlowStorageKey({ blockId: card.card_id, documentId: props.documentId })} />
{:else if card.kind === "table"}
  <DocumentTable rows={card.rows} />
{:else if card.kind === "plots"}
  <div class="plot-grid grid gap-3">
    <PlotGroupGrid
      documentPlots={card.plots}
      records={props.records}
      liveStates={props.renderMode === "report" ? reportPlotStates(card.plots, props.reportPlots) : props.livePlotStates}
      parameters={props.plotParameterContext}
      plotPalette={props.plotPalette}
      resetKey={props.renderMode === "report" ? "All" : props.plotResetKey}
      showGroupHeaders={false}
      telemetry={props.renderMode === "report" ? props.reportTelemetry : props.plotContextTelemetry}
      title={props.sectionTitle}
      viewportMode={report ? "fit" : "live"}
      zoom={report}
      measurements={props.renderMode === "live" ? props.measurements : undefined}
      reportDefaults={props.renderMode === "live" ? props.reportDefaults : undefined}
      onPlotSettingsChange={props.renderMode === "live" ? props.onPlotSettingsChange : undefined}
    />
  </div>
{:else if card.kind === "media"}
  <div class="grid gap-3">
    <MediaGalleryView artifacts={props.mediaArtifacts} emptyMessage="No media recorded." runDir={props.runDir} />
  </div>
{:else if card.kind === "artifacts"}
  <ArtifactTreeView artifacts={props.artifacts} emptyMessage="No artifacts recorded." runDir={props.runDir} />
{:else if card.kind === "traceability"}
  <DocumentTraceability traceability={props.traceability} />
{:else if card.kind === "telemetry_context"}
  <DocumentStatus streams={props.streams} />
{:else if card.kind === "notes"}
  <OperatorNotes notes={props.notes} />
{:else if card.kind === "steps"}
  <ExecutionStepsCriteria
    layout="document"
    records={props.stepRecords}
    emptyMessage={report ? "No step records for the selected case run." : "No step records for the selected execution yet."}
  />
{:else if card.kind === "criteria"}
  <ExecutionStepsCriteria
    layout="document"
    records={props.criterionRecords}
    emptyMessage={report ? "No criteria records for the selected case run." : "No criteria records for the selected execution yet."}
  />
{:else if card.kind === "health"}
  <DocumentStatus health={props.health} />
{:else if card.kind === "records"}
  <RecordTimeline live={!report} records={props.records} />
{/if}
