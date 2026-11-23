<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { DocumentViewPlot, HubRecord, JsonObject, TelemetryStream } from "../../lib/types";
  import type { PlotMeasurements } from "../../plots/plot-interaction";
  import type { PlotDisplaySettings, ReportDefaults } from "../../plots/plot-settings";
  import type { LivePlotStates } from "../../plots/live-frames";
  import type { PlotViewportMode } from "../../plots/types";

  export type PlotSettingsChange = (plotId: string, settings: PlotDisplaySettings) => Promise<void>;

  export type PlotGroupGridProps = {
    documentPlots: DocumentViewPlot[];
    liveStates?: LivePlotStates;
    onPlotSettingsChange?: PlotSettingsChange;
    parameters?: JsonObject | null;
    records?: readonly HubRecord[];
    telemetry?: Record<string, TelemetryStream> | null;
    resetKey?: string;
    showGroupHeaders?: boolean;
    title: string;
    viewportMode?: PlotViewportMode;
    zoom?: boolean;
    measurements?: PlotMeasurements | null;
    plotPalette?: readonly string[];
    reportDefaults?: ReportDefaults;
  };

  const emptyRecords: readonly HubRecord[] = [];
</script>

<script lang="ts">
  import { resolvePlotThemeColors } from "../../plots/colors";
  import { documentPlotGroupDescription, groupRepeatsPlotTitle } from "../../plots/grouping";
  import { emptyLivePlotState } from "../../plots/live-frames";
  import { providePlotInteraction } from "../../plots/plot-interaction";
  import { type PromptMarker, promptMarkers } from "../../plots/prompt-markers";
  import PlotPanel from "./PlotPanel.svelte";

  let {
    documentPlots,
    liveStates,
    onPlotSettingsChange,
    parameters,
    records = emptyRecords,
    telemetry,
    resetKey,
    showGroupHeaders = true,
    title,
    viewportMode = "live",
    zoom = false,
    measurements,
    plotPalette,
    reportDefaults,
  }: PlotGroupGridProps = $props();

  const syncKey = $props.id();
  const emptyState = emptyLivePlotState();
  const palette = $derived(plotPalette ?? resolvePlotThemeColors().palette);
  const titleRepeated = $derived(groupRepeatsPlotTitle(title, documentPlots));
  const description = $derived(documentPlots.find((plot) => plot.description)?.description ?? documentPlotGroupDescription(title));
  let previousPrompts: PromptMarker[] = [];
  const prompts = $derived.by(() => {
    const next = promptMarkers(records);
    const same =
      next.length === previousPrompts.length &&
      next.every((marker, index) => {
        const held = previousPrompts[index];
        return marker.kind === held.kind && marker.x === held.x && marker.title === held.title;
      });
    if (!same) {
      previousPrompts = next;
    }
    return previousPrompts;
  });

  providePlotInteraction({
    syncKey,
    get zoom() {
      return zoom;
    },
    get measurements() {
      return measurements;
    },
  });
</script>

{#snippet cards()}
  {#each documentPlots as descriptor (descriptor.plot_id)}
    <PlotPanel
      {descriptor}
      {onPlotSettingsChange}
      {parameters}
      plotPalette={palette}
      {prompts}
      {resetKey}
      {titleRepeated}
      liveState={liveStates?.get(descriptor.plot_id) ?? emptyState}
      {telemetry}
      {viewportMode}
      {reportDefaults}
    />
  {/each}
{/snippet}

{#if !documentPlots.length}
  <div class="plot-empty col-span-full grid min-h-44 place-content-center gap-2 rounded-lg bg-muted/20 text-center text-muted-foreground">
    <strong>Document Plots Are Loading</strong>
    <span>The selected case document has not provided plot metadata yet.</span>
  </div>
{:else if showGroupHeaders}
  <section class="grid gap-2">
    <div>
      <h3 class="document-section-title">{title}</h3>
      <p class="text-sm text-muted-foreground">{description}</p>
    </div>
    <div class="grid gap-2">{@render cards()}</div>
  </section>
{:else}
  {@render cards()}
{/if}
