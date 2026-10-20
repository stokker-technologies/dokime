/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { memo, type ReactNode, useMemo } from "react";

/* === Local Imports === */

import { documentPlotGroupDescription, documentPlotSpec, type PlotCardSpec } from "@/lib/document-plots";
import type { DocumentViewPlot, HubRecord, JsonObject, TelemetryStream } from "@/lib/types";
import { AnalysisResult } from "@/plots/AnalysisResult";
import { BarPlot } from "@/plots/catalog/BarPlot";
import { BoxSummarySeriesPlot } from "@/plots/catalog/BoxSummaryPlot";
import { FrequencySeriesPlot } from "@/plots/catalog/FrequencySeriesPlot";
import { HeatmapPlot } from "@/plots/catalog/HeatmapPlot";
import { HistogramSeriesPlot } from "@/plots/catalog/HistogramPlot";
import { TimeSeriesPlot } from "@/plots/catalog/TimeSeriesPlot";
import { TrajectoryPlot } from "@/plots/catalog/TrajectoryPlot";
import { WifiChannelOccupancyPlot } from "@/plots/catalog/WifiChannelOccupancyPlot";
import { XyIntervalPlot } from "@/plots/catalog/XyIntervalPlot";
import { XyLinePlot } from "@/plots/catalog/XyLinePlot";
import { XyScatterPlot } from "@/plots/catalog/XyScatterPlot";
import { groupRepeatsPlotTitle } from "@/plots/grouping";
import { emptyLivePlotState, type LivePlotState, type LivePlotStateMap } from "@/plots/live-frames";
import { type PlotDisplaySettings, usePlotSettingsEditor } from "@/plots/plot-settings";
import { type PromptMarker, promptMarkers } from "@/plots/prompt-markers";
import type { PlotViewportMode } from "@/plots/types";
import { usePlotPalette } from "@/plots/use-plot-palette";

/* === Types === */

export type PlotSettingsChange = (plotId: string, settings: PlotDisplaySettings) => Promise<void>;

type LivePlotGroupGridProps = {
  documentPlots: DocumentViewPlot[];
  liveStates?: LivePlotStateMap;
  /** Present only on the live surface; the sealed report reads what it persisted. */
  onPlotSettingsChange?: PlotSettingsChange;
  parameters?: JsonObject | null;
  /** The shown scope's records; its prompts mark every time-series plot. */
  records?: readonly HubRecord[];
  telemetry?: Record<string, TelemetryStream> | null;
  resetKey?: string;
  showGroupHeaders?: boolean;
  /** The enclosing section's label: the card's group title and colour seed. */
  title: string;
  viewportMode?: PlotViewportMode;
};

/* === Public Functions === */

/** One plots card: its plots form one group titled by the enclosing section. */
export const LivePlotGroupGrid = memo(function LivePlotGroupGrid({
  documentPlots,
  liveStates = EMPTY_LIVE_PLOT_STATES,
  onPlotSettingsChange,
  parameters,
  records = EMPTY_RECORDS,
  resetKey,
  showGroupHeaders = true,
  telemetry,
  title,
  viewportMode = "live",
}: LivePlotGroupGridProps) {
  const plotPalette = usePlotPalette();
  const prompts = useMemo(() => promptMarkers(records), [records]);
  // A lone plot titled like its group or section is already named above it.
  const titleRepeated = groupRepeatsPlotTitle(title, documentPlots);
  if (!documentPlots.length) {
    return (
      <div className="plot-empty col-span-full grid min-h-44 place-content-center gap-2 rounded-md bg-muted/20 text-center text-muted-foreground">
        <strong>Document plots are loading</strong>
        <span>The selected case document has not provided plot metadata yet.</span>
      </div>
    );
  }

  const cards = documentPlots.map((descriptor, position) => (
    <LivePlotCard
      descriptor={descriptor}
      key={descriptor.plot_id}
      onPlotSettingsChange={onPlotSettingsChange}
      parameters={parameters}
      plotPalette={plotPalette}
      position={position}
      prompts={prompts}
      resetKey={resetKey}
      titleRepeated={titleRepeated}
      state={liveStates[descriptor.plot_id] ?? EMPTY_LIVE_PLOT_STATE}
      telemetry={telemetry}
      title={title}
      viewportMode={viewportMode}
    />
  ));
  if (!showGroupHeaders) {
    return <>{cards}</>;
  }

  const description = documentPlots.find((plot) => plot.description)?.description ?? documentPlotGroupDescription(title);
  return (
    <section className="grid gap-2">
      <div>
        <h3 className="text-base font-semibold">{title}</h3>
        <p className="text-sm text-muted-foreground">{description}</p>
      </div>
      <div className="grid gap-2">{cards}</div>
    </section>
  );
});

const EMPTY_LIVE_PLOT_STATES: LivePlotStateMap = {};
const EMPTY_RECORDS: readonly HubRecord[] = [];
const EMPTY_LIVE_PLOT_STATE = emptyLivePlotState();

const LivePlotCard = memo(function LivePlotCard({
  descriptor,
  onPlotSettingsChange,
  parameters,
  plotPalette,
  position,
  prompts,
  resetKey,
  titleRepeated,
  state,
  telemetry,
  title,
  viewportMode,
}: {
  descriptor: DocumentViewPlot;
  onPlotSettingsChange?: PlotSettingsChange;
  parameters?: JsonObject | null;
  plotPalette: readonly string[];
  position: number;
  prompts: PromptMarker[];
  resetKey?: string;
  titleRepeated: boolean;
  state: LivePlotState;
  telemetry?: Record<string, TelemetryStream> | null;
  title: string;
  viewportMode: PlotViewportMode;
}) {
  const plot = useMemo(() => {
    const spec = livePlotSpec({
      descriptor,
      parameters,
      plotPalette,
      position,
      state,
      telemetry,
      title,
    });
    // Prompt times are wall-clock instants: only a wall-time axis can place them.
    const marked = spec.timeAxis && !spec.xElapsedUnit ? { ...spec, promptMarkers: prompts } : spec;
    return titleRepeated ? { ...marked, titleRepeated } : marked;
  }, [descriptor, parameters, plotPalette, position, prompts, state, telemetry, title, titleRepeated]);
  const PlotSettingsButton = usePlotSettingsEditor();
  const actions =
    onPlotSettingsChange && PlotSettingsButton ? (
      <PlotSettingsButton plot={descriptor} onSubmit={(settings) => onPlotSettingsChange(descriptor.plot_id, settings)} />
    ) : undefined;

  return (
    <div className={plotCardClassName(plot)} data-live-mounted="true" data-plot-id={descriptor.plot_id}>
      {plotElement(plot, resetKey, viewportMode, actions)}
    </div>
  );
});

function plotCardClassName(plot: PlotCardSpec): string {
  const baseClassName = "min-w-0 max-w-full overflow-hidden";
  if (plot.plotType === "wifi_channel_occupancy") {
    return `${baseClassName} min-h-0`;
  }
  if (plot.plotType === "bar") {
    return `${baseClassName} min-h-0`;
  }
  return plotHasNumericSamples(plot) || isChartLikePlot(plot) ? `${baseClassName} min-h-[18rem]` : `${baseClassName} min-h-44`;
}

function plotElement(plot: PlotCardSpec, resetKey?: string, viewportMode: PlotViewportMode = "fit", actions?: ReactNode) {
  if (plot.computed)
    return (
      <AnalysisResult
        key={plot.computed.selection.id}
        result={plot.computed}
        title={plot.title}
        stateLabel={plot.stateLabel}
        stateDetail={plot.stateDetail}
      />
    );
  if (plot.analysis?.kind === "error") {
    return <p role="alert">{plot.analysis.message}</p>;
  }
  if (plot.plotType === "wifi_channel_occupancy") {
    return <WifiChannelOccupancyPlot plot={plot} />;
  }
  if (plot.plotType === "xy_value_heatmap" || plot.plotType === "heatmap") {
    return <HeatmapPlot plot={plot} />;
  }
  if (plot.plotType === "bar") {
    return <BarPlot plot={plot} />;
  }
  if (plot.plotType === "xy_scatter") {
    return <XyScatterPlot actions={actions} plot={plot} resetKey={resetKey} viewportMode={viewportMode} />;
  }
  if (plot.plotType === "xy_line") {
    return <XyLinePlot actions={actions} plot={plot} resetKey={resetKey} viewportMode={viewportMode} />;
  }
  if (plot.plotType === "xy_interval") {
    return <XyIntervalPlot actions={actions} plot={plot} resetKey={resetKey} viewportMode={viewportMode} />;
  }
  if (plot.plotType === "trajectory") {
    return <TrajectoryPlot actions={actions} plot={plot} resetKey={resetKey} viewportMode={viewportMode} />;
  }
  if (plot.plotType === "frequency_series") {
    return <FrequencySeriesPlot actions={actions} plot={plot} resetKey={resetKey} viewportMode={viewportMode} />;
  }
  if (plot.plotType === "histogram") {
    return <HistogramSeriesPlot plot={plot} />;
  }
  if (plot.plotType === "box_summary") {
    return <BoxSummarySeriesPlot plot={plot} />;
  }
  if (plot.plotType === "time_series")
    return <TimeSeriesPlot actions={actions} plot={plot} resetKey={resetKey} viewportMode={viewportMode} />;
  return <p role="alert">Unsupported document plot type: {plot.plotType}</p>;
}

function livePlotSpec({
  descriptor,
  parameters,
  plotPalette,
  position,
  state,
  telemetry,
  title,
}: {
  descriptor: DocumentViewPlot;
  parameters?: JsonObject | null;
  plotPalette: readonly string[];
  position: number;
  state: LivePlotState;
  telemetry?: Record<string, TelemetryStream> | null;
  title: string;
}): PlotCardSpec {
  const spec = documentPlotSpec({
    parameters,
    payload: state.payload,
    plot: descriptor,
    plotPalette,
    position,
    telemetry,
    title,
  });

  const label =
    state.status === "loading"
      ? "Loading Plot"
      : state.status === "stale"
        ? "Stream Stale"
        : state.status === "reconnecting"
          ? "Reconnecting"
          : state.status === "error"
            ? "Stream Error"
            : spec.stateLabel;
  const detail =
    state.status === "reconnecting" && state.payload
      ? "The last samples remain visible while the plot stream reopens."
      : (state.error ??
        (state.status === "loading"
          ? "Waiting for the first plot payload."
          : state.status === "stale"
            ? "The last samples remain visible while the plot stream reconnects."
            : spec.stateDetail));
  // Frame sizes and compaction counts are transport diagnostics, not plot
  // context; they stay out of what an operator reads under a title.
  return {
    ...spec,
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
    plot.plotType === "xy_scatter" ||
    plot.plotType === "xy_line" ||
    plot.plotType === "xy_interval" ||
    plot.plotType === "trajectory" ||
    plot.plotType === "frequency_series" ||
    plot.plotType === "box_summary"
  );
}
