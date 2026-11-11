/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { ErrorCard } from "@/components/ErrorCard";
import { failureOf } from "@/lib/notification-details";
import { type ComponentProps, memo, type ReactNode, useEffect, useId, useMemo, useRef, useState } from "react";

/* === Local Imports === */

import { documentPlotGroupDescription, documentPlotSpec, type PlotCardSpec } from "@/lib/document-plots";
import type { DocumentViewPlot, HubRecord, JsonObject, TelemetryStream } from "@/lib/types";
import { AnalysisResult } from "@/plots/AnalysisResult";
import { PiePlot } from "@/plots/catalog/PiePlot";
import { BarPlot } from "@/plots/catalog/BarPlot";
import { BoxPlot } from "@/plots/catalog/BoxPlot";
import { FrequencySeriesPlot } from "@/plots/catalog/FrequencySeriesPlot";
import { ContourPlot } from "@/plots/catalog/ContourPlot";
import { HistogramSeriesPlot } from "@/plots/catalog/HistogramPlot";
import { TimeSeriesPlot } from "@/plots/catalog/TimeSeriesPlot";
import { TrajectoryPlot } from "@/plots/catalog/TrajectoryPlot";
import { GridChartPlot } from "@/plots/catalog/GridChartPlot";
import { XyIntervalPlot } from "@/plots/catalog/XyIntervalPlot";
import { XyPlot } from "@/plots/catalog/XyPlot";
import { groupRepeatsPlotTitle } from "@/plots/grouping";
import { PlotInteractionProvider, usePlotInteraction } from "@/plots/interaction";
import { emptyLivePlotState, type LivePlotState, type LivePlotStateMap, plotWaitingLabel } from "@/plots/live-frames";
import { PlotPlaceholder } from "@/plots/PlotPlaceholder";
import { plotPlaceholderText, plotUnavailableText, STREAM_ERROR } from "@/plots/plot-placeholder";
import { type PlotDisplaySettings, usePlotSettingsEditor } from "@/plots/plot-settings";
import { type PromptMarker, promptMarkers } from "@/plots/prompt-markers";
import type { PlotViewportMode } from "@/plots/types";
import { usePlotPalette } from "@/plots/use-plot-palette";

import { useSummaryWindowStore } from "@/stores/summary-window-store";

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
  /** The enclosing section's label: the card's group title. */
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
  const interaction = usePlotInteraction();
  const syncKey = useId();
  const selectPrompts = useMemo(() => {
    let previous: PromptMarker[] = [];
    return (source: readonly HubRecord[]) => {
      const next = promptMarkers(source);
      if (
        next.length === previous.length &&
        next.every((marker, index) => {
          const held = previous[index];
          return marker.kind === held.kind && marker.x === held.x && marker.title === held.title;
        })
      )
        return previous;
      previous = next;
      return next;
    };
  }, []);
  const prompts = useMemo(() => selectPrompts(records), [records, selectPrompts]);
  // A lone plot titled like its group or section is already named above it.
  const titleRepeated = groupRepeatsPlotTitle(title, documentPlots);
  if (!documentPlots.length) {
    return (
      <div className="plot-empty col-span-full grid min-h-44 place-content-center gap-2 rounded-lg bg-muted/20 text-center text-muted-foreground">
        <strong>Document Plots Are Loading</strong>
        <span>The selected case document has not provided plot metadata yet.</span>
      </div>
    );
  }

  const cards = documentPlots.map((descriptor) => (
    <VisiblePlotCard
      descriptor={descriptor}
      key={descriptor.plot_id}
      onPlotSettingsChange={onPlotSettingsChange}
      parameters={parameters}
      plotPalette={plotPalette}
      prompts={prompts}
      resetKey={resetKey}
      titleRepeated={titleRepeated}
      state={liveStates[descriptor.plot_id] ?? EMPTY_LIVE_PLOT_STATE}
      telemetry={telemetry}
      viewportMode={viewportMode}
    />
  ));
  if (!showGroupHeaders) {
    return (
      <PlotInteractionProvider zoom={interaction.zoom} syncKey={syncKey}>
        {cards}
      </PlotInteractionProvider>
    );
  }

  const description = documentPlots.find((plot) => plot.description)?.description ?? documentPlotGroupDescription(title);
  return (
    <section className="grid gap-2">
      <div>
        <h3 className="text-base font-semibold">{title}</h3>
        <p className="text-sm text-muted-foreground">{description}</p>
      </div>
      <PlotInteractionProvider zoom={interaction.zoom} syncKey={syncKey}>
        <div className="grid gap-2">{cards}</div>
      </PlotInteractionProvider>
    </section>
  );
});

const EMPTY_LIVE_PLOT_STATES: LivePlotStateMap = {};
const EMPTY_RECORDS: readonly HubRecord[] = [];
const EMPTY_LIVE_PLOT_STATE = emptyLivePlotState();

const visibilityListeners = new WeakMap<Element, (visible: boolean) => void>();
let visibilityObserver: IntersectionObserver | undefined;

/** Keep Mounted Plots And Their Viewports While Pausing Off-Screen Payloads. */
function VisiblePlotCard(props: ComponentProps<typeof LivePlotCard>) {
  const eager = props.viewportMode !== "live" || typeof IntersectionObserver === "undefined";
  const [visible, setVisible] = useState(eager);
  const [mounted, setMounted] = useState(eager);
  const root = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (eager || !root.current) return;
    const element = root.current;
    visibilityObserver ??= new IntersectionObserver(
      (entries) => {
        for (const entry of entries) visibilityListeners.get(entry.target)?.(entry.isIntersecting);
      },
      { rootMargin: "200px" },
    );
    visibilityListeners.set(element, (shown) => {
      setVisible(shown);
      if (shown) setMounted(true);
    });
    visibilityObserver.observe(element);
    return () => {
      visibilityObserver?.unobserve(element);
      visibilityListeners.delete(element);
    };
  }, [eager]);
  return (
    <div ref={root} className="min-w-0" data-plot-viewport={props.descriptor.plot_id}>
      {mounted || eager ? <VisiblePlotContent {...props} visible={visible || eager} /> : <div className="min-h-72" aria-hidden />}
    </div>
  );
}

const VisiblePlotContent = memo(
  function VisiblePlotContent({ visible: _visible, ...props }: ComponentProps<typeof LivePlotCard> & { visible: boolean }) {
    return <LivePlotCard {...props} />;
  },
  (previous, next) =>
    (!previous.visible && !next.visible) ||
    (Object.keys(previous).length === Object.keys(next).length &&
      (Object.keys(next) as Array<keyof typeof next>).every((key) => Object.is(previous[key], next[key]))),
);

const LivePlotCard = memo(function LivePlotCard({
  descriptor,
  onPlotSettingsChange,
  parameters,
  plotPalette,
  prompts,
  resetKey,
  titleRepeated,
  state,
  telemetry,
  viewportMode,
}: {
  descriptor: DocumentViewPlot;
  onPlotSettingsChange?: PlotSettingsChange;
  parameters?: JsonObject | null;
  plotPalette: readonly string[];
  prompts: PromptMarker[];
  resetKey?: string;
  titleRepeated: boolean;
  state: LivePlotState;
  telemetry?: Record<string, TelemetryStream> | null;
  viewportMode: PlotViewportMode;
}) {
  const panelRef = useRef<HTMLDivElement>(null);
  const { zoom } = usePlotInteraction();
  const setPanelWidth = useSummaryWindowStore((store) => store.setPanelWidth);
  useEffect(() => {
    const panel = panelRef.current;
    if (!panel || zoom) return;
    const measure = () => {
      const width = Math.round(panel.getBoundingClientRect().width);
      if (width < 1) return;
      setPanelWidth(descriptor.plot_id, width);
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(panel);
    return () => {
      observer.disconnect();
      setPanelWidth(descriptor.plot_id, null);
    };
  }, [descriptor.plot_id, setPanelWidth, zoom]);
  const plot = useMemo(() => {
    const spec = livePlotSpec({
      descriptor,
      parameters,
      plotPalette,
      state,
      telemetry,
    });
    // Prompt times are wall-clock instants: only a wall-time axis can place them.
    const marked = spec.timeAxis && !spec.xElapsedUnit ? { ...spec, promptMarkers: prompts } : spec;
    return titleRepeated ? { ...marked, titleRepeated } : marked;
  }, [descriptor, parameters, plotPalette, prompts, state, telemetry, titleRepeated]);
  const PlotSettingsButton = usePlotSettingsEditor();
  return (
    <div ref={panelRef} className={plotCardClassName(plot)} data-live-mounted="true" data-plot-id={descriptor.plot_id}>
      {onPlotSettingsChange && PlotSettingsButton ? (
        <PlotSettingsButton plot={descriptor} onSubmit={(settings) => onPlotSettingsChange(descriptor.plot_id, settings)}>
          {(actions) => plotElement(plot, resetKey, viewportMode, actions)}
        </PlotSettingsButton>
      ) : (
        plotElement(plot, resetKey, viewportMode)
      )}
    </div>
  );
});

function plotCardClassName(plot: PlotCardSpec): string {
  const baseClassName = "min-w-0 max-w-full overflow-hidden";
  if (plot.plotType === "grid_chart") {
    return `${baseClassName} min-h-0`;
  }
  if (plot.plotType === "bar" || plot.plotType === "pie") {
    return `${baseClassName} min-h-0`;
  }
  return plotHasNumericSamples(plot) || isChartLikePlot(plot) ? `${baseClassName} min-h-72` : `${baseClassName} min-h-44`;
}

function plotElement(plot: PlotCardSpec, resetKey?: string, viewportMode: PlotViewportMode = "fit", actions?: ReactNode) {
  if (plot.failure && !plot.cartesian && !plot.series.some((series) => series.points.length))
    return (
      <div className="grid gap-2">
        {actions}
        <ErrorCard failure={plot.failure} />
      </div>
    );
  if (plot.stateLabel === STREAM_ERROR) {
    return <PlotPlaceholder title={plot.title} actions={actions} {...plotPlaceholderText(plot)} />;
  }
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
    return <PlotPlaceholder title={plot.title} {...plotUnavailableText(plot.analysis.message)} />;
  }
  if (plot.plotType === "grid_chart") {
    return <GridChartPlot key={plot.key} plot={plot} />;
  }
  if (plot.plotType === "contour") {
    return <ContourPlot plot={plot} />;
  }
  if (plot.plotType === "pie") return <PiePlot plot={plot} />;
  if (plot.plotType === "bar") {
    return <BarPlot plot={plot} />;
  }
  if (plot.plotType === "xy" && !plot.xyInterval) {
    return <XyPlot actions={actions} plot={plot} resetKey={resetKey} viewportMode={viewportMode} />;
  }
  if (plot.plotType === "xy" && plot.xyInterval === "sigma") {
    return <XyIntervalPlot actions={actions} plot={plot} resetKey={resetKey} viewportMode={viewportMode} />;
  }
  if (plot.plotType === "trajectory") {
    return <TrajectoryPlot key={plot.key} actions={actions} plot={plot} resetKey={resetKey} viewportMode={viewportMode} />;
  }
  if (plot.plotType === "frequency_series") {
    return <FrequencySeriesPlot actions={actions} plot={plot} resetKey={resetKey} viewportMode={viewportMode} />;
  }
  if (plot.plotType === "histogram") {
    return <HistogramSeriesPlot plot={plot} />;
  }
  if (plot.plotType === "box_plot") {
    return <BoxPlot plot={plot} />;
  }
  if (plot.plotType === "time_series")
    return <TimeSeriesPlot actions={actions} plot={plot} resetKey={resetKey} viewportMode={viewportMode} />;
  return (
    <ErrorCard
      failure={failureOf(
        `Unsupported document plot type: ${plot.plotType}`,
        "Presentation Unavailable",
        ["unsupported-plot", plot.key, plot.plotType].join(":"),
      )}
    />
  );
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
  const detail =
    state.status === "reconnecting" && state.payload
      ? "The last samples remain visible while the plot stream reopens."
      : (state.error ??
        (state.status === "waiting"
          ? "The plot is subscribed; its first frame is still being read."
          : state.status === "loading"
            ? "Waiting for the first plot payload."
            : state.status === "stale"
              ? "The last samples remain visible while the plot stream reconnects."
              : spec.stateDetail));
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
