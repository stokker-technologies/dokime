<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { DocumentViewPlot, JsonObject, TelemetryStream } from "../../lib/types";
  import { documentPlotSpec } from "../../plots/grouping";
  import { type LivePlotState, plotWaitingLabel } from "../../plots/live-frames";
  import { STREAM_ERROR } from "../../plots/plot-placeholder";
  import type { PlotCardSpec, PlotViewportMode } from "../../plots/types";

  const visibilityListeners = new WeakMap<Element, (visible: boolean) => void>();
  let visibilityObserver: IntersectionObserver | undefined;

  function plotCardClassName(plot: PlotCardSpec): string {
    const baseClassName = "min-w-0 max-w-full overflow-hidden";
    if (plot.plotType === "heat_strip" || plot.plotType === "grid_chart" || plot.plotType === "bar" || plot.plotType === "pie") {
      return `${baseClassName} min-h-0`;
    }
    return plotHasNumericSamples(plot) || isChartLikePlot(plot) ? `${baseClassName} min-h-72` : `${baseClassName} min-h-44`;
  }

  function livePlotSpec({
    descriptor,
    parameters,
    plotPalette,
    state,
    telemetry,
  }: {
    descriptor: DocumentViewPlot;
    parameters?: JsonObject | null;
    plotPalette: readonly string[];
    state: LivePlotState;
    telemetry?: Record<string, TelemetryStream> | null;
  }): PlotCardSpec {
    const spec = documentPlotSpec({
      parameters,
      payload: state.payload,
      plot: descriptor,
      plotPalette,
      telemetry,
    });

    const label =
      state.status === "waiting"
        ? plotWaitingLabel(state)
        : state.status === "loading"
          ? "Loading Plot"
          : state.status === "stale"
            ? "Stream Stale"
            : state.status === "reconnecting"
              ? "Reconnecting"
              : state.status === "error"
                ? STREAM_ERROR
                : spec.stateLabel;
    // A State Text Shows Only When Nothing Is Drawn, So It Never Claims Visible Samples.
    const detail =
      state.error ??
      (state.status === "waiting"
        ? "The plot is subscribed; its first frame is still being read."
        : state.status === "loading"
          ? "Waiting for the first plot payload."
          : state.status === "stale"
            ? "The plot stream ended before samples arrived."
            : spec.stateDetail);
    // Frame sizes and compaction counts are transport diagnostics, not plot
    // context; they stay out of what an operator reads under a title.
    return {
      ...spec,
      failure: state.failure,
      stateLabel: label,
      stateDetail: detail,
      // A window or zoom change reopens the stream: until its first frame the
      // shown statistics cover the old range. A run that ends keeps its final
      // numbers undimmed.
      summaryPending: state.summaryInFlight,
    };
  }

  function plotHasNumericSamples(plot: PlotCardSpec): boolean {
    return plot.series.some((series) =>
      series.points.some(
        (point) => Number.isFinite(point.y) || Number.isFinite(point.z) || Number.isFinite(point.low) || Number.isFinite(point.high),
      ),
    );
  }

  function isChartLikePlot(plot: PlotCardSpec): boolean {
    return (
      plot.plotType === "time_series" ||
      plot.plotType === "xy" ||
      plot.plotType === "trajectory" ||
      plot.plotType === "frequency_series" ||
      plot.plotType === "box_plot" ||
      plot.plotType === "contour" ||
      plot.plotType === "grid_chart"
    );
  }
</script>

<script lang="ts">
  import { untrack } from "svelte";
  import { getPlotInteraction } from "../../plots/plot-interaction";
  import PlotDispatcher from "../../plots/PlotDispatcher.svelte";
  import PlotSettingsDialog from "../../plots/PlotSettingsDialog.svelte";
  import type { PlotDisplaySettings, ReportDefaults } from "../../plots/plot-settings";
  import type { PromptMarker } from "../../plots/prompt-markers";
  import type { PlotSettingsChange } from "./PlotGroupGrid.svelte";

  let {
    descriptor,
    onPlotSettingsChange,
    parameters,
    plotPalette,
    prompts,
    resetKey,
    titleRepeated,
    liveState,
    telemetry,
    viewportMode,
    reportDefaults,
  }: {
    descriptor: DocumentViewPlot;
    onPlotSettingsChange?: PlotSettingsChange;
    parameters?: JsonObject | null;
    plotPalette: readonly string[];
    prompts: PromptMarker[];
    resetKey?: string;
    titleRepeated: boolean;
    liveState: LivePlotState;
    telemetry?: Record<string, TelemetryStream> | null;
    viewportMode: PlotViewportMode;
    reportDefaults?: ReportDefaults;
  } = $props();

  const interaction = getPlotInteraction();
  const eager = $derived(viewportMode !== "live" || typeof IntersectionObserver === "undefined");
  const plotId = $derived(descriptor.plot_id);
  const reportVisibility = $derived(interaction.measurements?.setVisible);
  const reportWidth = $derived(interaction.zoom ? undefined : interaction.measurements?.setPanelWidth);
  let visible = $state(false);
  let mounted = $state(false);
  let root: HTMLDivElement | undefined = $state();
  let panel: HTMLDivElement | undefined = $state();

  // Keep The Last Drawn Frame And Viewport While The Card Is Off Screen.
  let cachedPlot: PlotCardSpec | undefined;
  const plot = $derived.by(() => {
    if (!eager && !visible) {
      return cachedPlot;
    }
    const spec = livePlotSpec({ descriptor, parameters, plotPalette, state: liveState, telemetry });
    // Prompts Use Wall-Clock Coordinates.
    const marked = spec.timeAxis && !spec.xElapsedUnit ? { ...spec, promptMarkers: prompts } : spec;
    cachedPlot = titleRepeated ? { ...marked, titleRepeated } : marked;
    return cachedPlot;
  });

  $effect(() => {
    if (eager || !root) {
      return;
    }
    const element = root;
    const id = plotId;
    const report = reportVisibility;
    visibilityObserver ??= new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          visibilityListeners.get(entry.target)?.(entry.isIntersecting);
        }
      },
      { rootMargin: "200px" },
    );
    visibilityListeners.set(element, (shown) => {
      visible = shown;
      if (shown) {
        mounted = true;
      }
      report?.(id, shown);
    });
    visibilityObserver.observe(element);
    return () => {
      visibilityObserver?.unobserve(element);
      visibilityListeners.delete(element);
      report?.(id, null);
    };
  });

  $effect(() => {
    const report = reportWidth;
    if (!panel || !report) {
      return;
    }
    const element = panel;
    const id = plotId;
    const measure = () => {
      const width = Math.round(element.getBoundingClientRect().width);
      if (width > 0) {
        report(id, width);
      }
    };
    untrack(measure);
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => {
      observer.disconnect();
      report(id, null);
    };
  });

  async function submit(settings: PlotDisplaySettings) {
    await onPlotSettingsChange?.(descriptor.plot_id, settings);
  }
</script>

<div bind:this={root} class="min-w-0" data-plot-viewport={descriptor.plot_id}>
  {#if plot && (mounted || eager)}
    {@const shown = plot}
    <div bind:this={panel} class={plotCardClassName(plot)} data-live-mounted="true" data-plot-id={descriptor.plot_id}>
      {#if onPlotSettingsChange}
        <PlotSettingsDialog plot={descriptor} defaults={reportDefaults} onSubmit={submit}>
          {#snippet children(
            actions,
          )}
            <PlotDispatcher plot={shown} {resetKey} {viewportMode} {actions} />
          {/snippet}
        </PlotSettingsDialog>
      {:else}
        <PlotDispatcher {plot} {resetKey} {viewportMode} />
      {/if}
    </div>
  {:else}
    <div class="min-h-72" aria-hidden="true"></div>
  {/if}
</div>
