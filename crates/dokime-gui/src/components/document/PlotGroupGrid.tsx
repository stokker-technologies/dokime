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

import { buildDocumentPlotGroups, type PlotTopicGroup } from "@/lib/document-plots";
import type { DocumentViewPlot, JsonObject, TelemetryStream } from "@/lib/types";
import { AnalysisResult } from "@/plots/AnalysisResult";
import { elapsedColumnUnit } from "@/plots/axis";
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
import { emptyLivePlotState, type LivePlotState, type LivePlotStateMap } from "@/plots/live-data";
import { type PlotDisplaySettings, PlotSettingsButton } from "@/plots/PlotSettingsDialog";
import type { PlotViewportMode } from "@/plots/types";
import { usePlotPalette } from "@/plots/use-plot-palette";

/* === Types === */

type PlotGroupGridProps = {
  groups: PlotTopicGroup[];
  hideWhenEmpty?: boolean;
  resetKey?: string;
  showGroupHeaders?: boolean;
};

export type PlotSettingsChange = (plotId: string, settings: PlotDisplaySettings) => Promise<void>;

type LivePlotGroupGridProps = {
  documentPlots: DocumentViewPlot[];
  liveStates?: LivePlotStateMap;
  /** Present only on the live surface; the sealed report reads what it persisted. */
  onPlotSettingsChange?: PlotSettingsChange;
  parameters?: JsonObject | null;
  telemetry?: Record<string, TelemetryStream> | null;
  resetKey?: string;
  hideWhenEmpty?: boolean;
  showGroupHeaders?: boolean;
  viewportMode?: PlotViewportMode;
};

/* === Public Functions === */

export const PlotGroupGrid = memo(function PlotGroupGrid({
  groups,
  hideWhenEmpty = false,
  resetKey,
  showGroupHeaders = true,
}: PlotGroupGridProps) {
  const resolvedGroups = visibleGroups(groups);
  if (!resolvedGroups.length) {
    if (hideWhenEmpty) {
      return null;
    }
    return (
      <div className="plot-empty col-span-full grid min-h-44 place-content-center gap-2 rounded-md bg-muted/20 text-center text-muted-foreground">
        <strong>Document plots are loading</strong>
        <span>The selected case document has not provided plot metadata yet.</span>
      </div>
    );
  }

  if (!showGroupHeaders) {
    return (
      <>
        {resolvedGroups.flatMap((group) =>
          group.plots.map((plot) => <PlotCard key={plot.key} plot={plot} resetKey={resetKey} />),
        )}
      </>
    );
  }

  return (
    <>
      {resolvedGroups.map((group) => (
        <section className="grid gap-2" key={group.title}>
          <div>
            <h3 className="text-base font-semibold">{group.title}</h3>
            <p className="text-sm text-muted-foreground">{group.description}</p>
          </div>
          <div className="grid gap-2">
            {group.plots.map((plot) => (
              <PlotCard key={plot.key} plot={plot} resetKey={resetKey} />
            ))}
          </div>
        </section>
      ))}
    </>
  );
});

export const LivePlotGroupGrid = memo(function LivePlotGroupGrid({
  documentPlots,
  hideWhenEmpty = false,
  liveStates = EMPTY_LIVE_PLOT_STATES,
  onPlotSettingsChange,
  parameters,
  resetKey,
  showGroupHeaders = true,
  telemetry,
  viewportMode = "live",
}: LivePlotGroupGridProps) {
  const plotPalette = usePlotPalette();
  const groups = useMemo(
    () =>
      buildDocumentPlotGroups({
        documentPlots,
        parameters,
        plotPalette,
        plots: {},
        telemetry,
      }),
    [documentPlots, parameters, plotPalette, telemetry],
  );
  const descriptors = useMemo(
    () => new Map(documentPlots.map((plot) => [plot.plot_id, plot])),
    [documentPlots],
  );
  const resolvedGroups = visibleGroups(groups);
  if (!resolvedGroups.length) {
    if (hideWhenEmpty && documentPlots.length > 0) {
      return null;
    }
    return (
      <div className="plot-empty col-span-full grid min-h-44 place-content-center gap-2 rounded-md bg-muted/20 text-center text-muted-foreground">
        <strong>Document plots are loading</strong>
        <span>The selected case document has not provided plot metadata yet.</span>
      </div>
    );
  }

  if (!showGroupHeaders) {
    return (
      <>
        {resolvedGroups.flatMap((group) =>
          group.plots.map((plot) => {
            const descriptor = descriptors.get(plot.key);
            return descriptor ? (
              <LivePlotCard
                descriptor={descriptor}
                key={plot.key}
                onPlotSettingsChange={onPlotSettingsChange}
                resetKey={resetKey}
                state={liveStates[descriptor.plot_id] ?? EMPTY_LIVE_PLOT_STATE}
                parameters={parameters}
                plotPalette={plotPalette}
                telemetry={telemetry}
                viewportMode={viewportMode}
              />
            ) : null;
          }),
        )}
      </>
    );
  }

  return (
    <>
      {resolvedGroups.map((group) => (
        <section className="grid gap-2" key={group.title}>
          <div>
            <h3 className="text-base font-semibold">{group.title}</h3>
            <p className="text-sm text-muted-foreground">{group.description}</p>
          </div>
          <div className="grid gap-2">
            {group.plots.map((plot) => {
              const descriptor = descriptors.get(plot.key);
              return descriptor ? (
                <LivePlotCard
                  descriptor={descriptor}
                  key={plot.key}
                  onPlotSettingsChange={onPlotSettingsChange}
                  resetKey={resetKey}
                  state={liveStates[descriptor.plot_id] ?? EMPTY_LIVE_PLOT_STATE}
                  parameters={parameters}
                  plotPalette={plotPalette}
                  telemetry={telemetry}
                  viewportMode={viewportMode}
                />
              ) : null;
            })}
          </div>
        </section>
      ))}
    </>
  );
});

const PlotCard = memo(function PlotCard({
  plot,
  resetKey,
}: {
  plot: PlotTopicGroup["plots"][number];
  resetKey?: string;
}) {
  return <div className={plotCardClassName(plot)}>{plotElement(plot, resetKey)}</div>;
});

const EMPTY_LIVE_PLOT_STATES: LivePlotStateMap = {};
const EMPTY_LIVE_PLOT_STATE = emptyLivePlotState();

const LivePlotCard = memo(function LivePlotCard({
  descriptor,
  onPlotSettingsChange,
  parameters,
  plotPalette,
  resetKey,
  state,
  telemetry,
  viewportMode,
}: {
  descriptor: DocumentViewPlot;
  onPlotSettingsChange?: PlotSettingsChange;
  parameters?: JsonObject | null;
  plotPalette: readonly string[];
  resetKey?: string;
  state: LivePlotState;
  telemetry?: Record<string, TelemetryStream> | null;
  viewportMode: PlotViewportMode;
}) {
  const plot = useMemo(
    () =>
      livePlotSpec({
        descriptor,
        parameters,
        plotPalette,
        state,
        telemetry,
      }),
    [descriptor, parameters, plotPalette, state, telemetry],
  );
  const actions = onPlotSettingsChange ? (
    <PlotSettingsButton
      plot={descriptor}
      onSubmit={(settings) => onPlotSettingsChange(descriptor.plot_id, settings)}
    />
  ) : undefined;

  return (
    <div
      className={plotCardClassName(plot)}
      data-live-mounted="true"
      data-plot-id={descriptor.plot_id}
    >
      {plotElement(plot, resetKey, viewportMode, actions)}
    </div>
  );
});

function plotCardClassName(plot: PlotTopicGroup["plots"][number]): string {
  const baseClassName = "min-w-0 max-w-full overflow-hidden";
  if (plot.plotType === "wifi_channel_occupancy") {
    return `${baseClassName} min-h-0`;
  }
  if (plot.plotType === "bar") {
    return `${baseClassName} min-h-0`;
  }
  return plotHasNumericSamples(plot) || isChartLikePlot(plot)
    ? `${baseClassName} min-h-[18rem]`
    : `${baseClassName} min-h-44`;
}

function plotElement(
  plot: PlotTopicGroup["plots"][number],
  resetKey?: string,
  viewportMode: PlotViewportMode = "fit",
  actions?: ReactNode,
) {
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
    return (
      <XyScatterPlot
        actions={actions}
        plot={plot}
        resetKey={resetKey}
        viewportMode={viewportMode}
      />
    );
  }
  if (plot.plotType === "xy_line") {
    return (
      <XyLinePlot actions={actions} plot={plot} resetKey={resetKey} viewportMode={viewportMode} />
    );
  }
  if (plot.plotType === "xy_interval") {
    return (
      <XyIntervalPlot
        actions={actions}
        plot={plot}
        resetKey={resetKey}
        viewportMode={viewportMode}
      />
    );
  }
  if (plot.plotType === "trajectory") {
    return (
      <TrajectoryPlot
        actions={actions}
        plot={plot}
        resetKey={resetKey}
        viewportMode={viewportMode}
      />
    );
  }
  if (plot.plotType === "frequency_series") {
    return (
      <FrequencySeriesPlot
        actions={actions}
        plot={plot}
        resetKey={resetKey}
        viewportMode={viewportMode}
      />
    );
  }
  if (plot.plotType === "histogram") {
    return <HistogramSeriesPlot plot={plot} />;
  }
  if (plot.plotType === "box_summary") {
    return <BoxSummarySeriesPlot plot={plot} />;
  }
  if (plot.plotType === "time_series")
    return (
      <TimeSeriesPlot
        actions={actions}
        plot={plot}
        resetKey={resetKey}
        viewportMode={viewportMode}
      />
    );
  return <p role="alert">Unsupported document plot type: {plot.plotType}</p>;
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
}): PlotTopicGroup["plots"][number] {
  const plots = state.payload ? { [descriptor.plot_id]: state.payload } : {};
  const spec =
    buildDocumentPlotGroups({
      documentPlots: [descriptor],
      parameters,
      plotPalette,
      plots,
      telemetry,
    })[0]?.plots[0] ??
    buildDocumentPlotGroups({
      documentPlots: [descriptor],
      parameters,
      plotPalette,
      plots: {},
      telemetry,
    })[0]?.plots[0] ??
    placeholderPlotSpec(descriptor);

  const label =
    state.status === "loading"
      ? "Loading Plot"
      : state.status === "stale"
        ? "Stream Stale"
        : state.status === "reconnecting"
          ? "Reconnecting"
          : state.status === "error"
            ? "Stream Error"
            : spec?.stateLabel;
  const detail =
    state.status === "reconnecting"
      ? "The last samples remain visible while the plot stream reopens."
      : (state.error ??
        (state.status === "loading"
          ? "Waiting for the first plot payload."
          : state.status === "stale"
            ? "The last samples remain visible while the plot stream reconnects."
            : spec?.stateDetail));
  const diagnostics = state.diagnostics
    ? `${state.diagnostics.point_count} delta points`
    : undefined;

  return {
    ...spec,
    stateLabel: label,
    stateDetail: detail,
    contextSummary: compactPlotContext([
      spec?.contextSummary,
      state.status === "live" ? diagnostics : undefined,
      state.droppedBatches ? `${state.droppedBatches} compacted batches` : undefined,
    ]),
  };
}

function placeholderPlotSpec(descriptor: DocumentViewPlot): PlotTopicGroup["plots"][number] {
  return {
    key: descriptor.plot_id,
    stream: descriptor.stream,
    title: descriptor.title,
    description: descriptor.description,
    series: descriptor.y.map((key) => ({
      key,
      label: key,
      points: [],
      raw_points: 0,
      downsampled: false,
    })),
    cartesian: null,
    markers: [],
    plotType: descriptor.plot_type,
    style: descriptor.style ?? "line",
    xScale: descriptor.x_scale ?? "linear",
    yScale: descriptor.y_scale ?? "linear",
    fft: descriptor.fft,
    histogram: descriptor.histogram,
    timeAxis: false,
    unit: descriptor.unit,
    colors: [],
    referenceLines: descriptor.reference_lines ?? [],
    referenceBands: descriptor.reference_bands ?? [],
    stateLabel: "Loading Plot",
    stateDetail: "Waiting for the first plot payload.",
    liveWindowS: descriptor.live_window_s ?? null,
    xFormat: descriptor.x_format ?? null,
    xElapsedUnit:
      descriptor.plot_type === "time_series" && !descriptor.x_format
        ? elapsedColumnUnit(descriptor.x)
        : null,
    yFormat: descriptor.y_format ?? null,
  };
}

function compactPlotContext(parts: Array<string | undefined>): string | undefined {
  const values = parts.filter((part): part is string => Boolean(part));
  return values.length ? values.join(" - ") : undefined;
}

function visibleGroups(groups: PlotTopicGroup[]): PlotTopicGroup[] {
  return groups.filter((group) => group.plots.length > 0);
}

function plotHasNumericSamples(plot: PlotTopicGroup["plots"][number]): boolean {
  return plot.series.some((series) =>
    series.points.some(
      (point) =>
        Number.isFinite(point.y) ||
        Number.isFinite(point.z) ||
        Number.isFinite(point.low) ||
        Number.isFinite(point.high),
    ),
  );
}

function isChartLikePlot(plot: PlotTopicGroup["plots"][number]): boolean {
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
