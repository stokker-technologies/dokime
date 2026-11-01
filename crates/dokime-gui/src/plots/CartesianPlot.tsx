/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { memo, type ReactNode, useEffect, useMemo, useRef, useState } from "react";
import type uPlot from "uplot";

/* === Local Imports === */

import { StatsSummaryTable } from "@/components/StatsSummaryTable";
import { formatNumber, formatTime, formatUnitLabel, type TimeFormatOrigin } from "@/lib/format";
import { nearestDrawnPoint, type SeriesSummary, seriesLegendColumns, seriesLegendRows, toggleTrace } from "@/lib/statistics";
import type {
  DocumentAxisScale,
  DocumentPlotStyle,
  DocumentReferenceBand,
  DocumentReferenceLine,
  PlotCartesianFrame,
  PlotMarker,
  PlotSeries,
  ValueFormat,
} from "@/lib/types";
import { isNumber } from "@/lib/utils";
import { formatBooleanState, formatCursorX, formatElapsedTick, xValue } from "@/plots/axis";
import { resolvePlotCssColor, resolvePlotThemeColors } from "@/plots/colors";
import { usePlotInteraction } from "@/plots/interaction";
import { PlotPlaceholder } from "@/plots/PlotPlaceholder";
import { plotPlaceholderText, plotUnavailableText } from "@/plots/plot-placeholder";
import { type PromptMarker, promptMarkerLabel } from "@/plots/prompt-markers";
import { windowIndices } from "@/plots/reduce";
import type { PlotViewportMode } from "@/plots/types";
import { attachPan, type HeldViewport, heldViewport, PLOT_DRAG_OPTIONS } from "@/plots/viewport";

/* === Constants === */

const GUI_POINT_MARK_THRESHOLD = 600;
const GUI_RENDERED_SERIES = 48;

const MAX_RENDERED_SERIES = GUI_RENDERED_SERIES;
const MIN_CHART_WIDTH = 160;
/* Tick marks (10px) plus the label gap (5px) plus a little breathing room. */
const Y_AXIS_LABEL_PADDING = 19;
const Y_AXIS_MIN_GUTTER = 44;
const Y_AXIS_MAX_GUTTER = 140;
const Y_AXIS_CHARACTER_WIDTH = 7;
/* Tick marks (10px) plus the label gap (5px), above the axis text itself. */
const X_AXIS_TICK_ROOM = 15;
const X_AXIS_MIN_GUTTER = 30;
const X_AXIS_MAX_GUTTER = 96;
/* uPlot draws each axis line at 1.5x the font size. */
const AXIS_LINE_GAP = 1.5;
/* uPlot's own right padding when there is no right axis: half the y gutter. */
const X_AXIS_MIN_RIGHT_PAD = 25;
/* Axis text is canvas-drawn; the touch rules in styles.css cannot reach it. */
const TOUCH_PLOT_FONT_SCALE = 1.3;
const POINT_MARK_RENDER_LIMIT = GUI_POINT_MARK_THRESHOLD;
const DENSE_POINT_MARK_SIZE = 3;
const DEFAULT_POINT_MARK_SIZE = 5;
const EMPTY_TRACES: ReadonlySet<string> = new Set();
const NO_PROMPTS: PromptMarker[] = [];
/* How near the pointer must come to a prompt marker, in CSS pixels, to read it. */
const PROMPT_HOVER_PX = 5;
/* The cap glyph drawn atop each prompt marker, in CSS pixels. */
const PROMPT_CAP_PX = 4;

/* === Types === */

type CartesianPlotProps = {
  title: string;
  /** Hide a repeated title only when it has no unit to present beside it. */
  titleRepeated?: boolean;
  cartesian?: PlotCartesianFrame | null;
  description?: string | null;
  series: PlotSeries[];
  markers?: PlotMarker[];
  /** Prompt lifecycle moments, drawn as quiet vertical lines the header can hide. */
  promptMarkers?: PromptMarker[];
  referenceLines?: DocumentReferenceLine[];
  referenceBands?: DocumentReferenceBand[];
  style?: DocumentPlotStyle;
  xScale?: DocumentAxisScale;
  yScale?: DocumentAxisScale;
  timeAxis?: boolean;
  booleanAxis?: boolean;
  colors?: string[];
  unit?: string | null;
  xFormat?: ValueFormat | null;
  xLabel?: string;
  intervals?: boolean;
  /**
   * The unit of an elapsed x axis, set when the plot's declared x column names
   * a duration. Its values are the duration itself, so the axis ticks as
   * numbers with this unit and never as a date.
   */
  xElapsedUnit?: string | null;
  yFormat?: ValueFormat | null;
  runStartedAt?: number | null;
  cycleStartedAt?: number | null;
  resetKey?: string;
  /** Per-plot controls the host places beside the follow button. */
  actions?: ReactNode;
  badge?: string;
  stateLabel?: string;
  stateDetail?: string;
  contextSummary?: string;
  liveWindowS?: number | null;
  viewportMode?: PlotViewportMode;
  /**
   * Replaces the canvas legend with the trace statistics table: one clickable
   * row per trace with the backend's statistics of the shown window.
   */
  legend?: { summaries: SeriesSummary[]; pending: boolean } | null;
  /** Reports the x range a zoom holds, or `null` once the plot follows again. */
  onViewportChange?: (window: { from: number; to: number } | null) => void;
};

type IntervalRange = {
  x: number;
  low: number;
  high: number;
};

/** Where the pointer sits on x, and what the visible window spans. */
type PlotCursor = {
  x: number;
  span: number;
  widthPx: number;
};

type ChartFrame = {
  data: uPlot.AlignedData;
  mode: PlotCartesianFrame["mode"];
  pointCount: number;
  signature: string;
};

export type PlotViewportRange = {
  min: number;
  max: number;
};

/**
 * What the canvas currently draws from the full frame: the frame itself, the
 * x range it was reduced for (`null` for the whole extent), and whether the
 * reduction kept every sample, in which case it serves any range.
 */
type DrawnWindow = {
  source: uPlot.AlignedData | null;
  aligned: boolean;
  range: PlotViewportRange | null;
  everySample: boolean;
};

type UPlotConstructor = typeof uPlot;

/* === State === */

let uPlotPromise: Promise<UPlotConstructor> | null = null;

/* === Components === */

export const CartesianPlot = memo(function CartesianPlot({
  title,
  titleRepeated = false,
  cartesian,
  description,
  series,
  markers = [],
  promptMarkers = NO_PROMPTS,
  referenceLines = [],
  referenceBands = [],
  style = "line",
  xScale = "linear",
  yScale = "linear",
  timeAxis = true,
  booleanAxis = false,
  colors = [],
  unit,
  xFormat = null,
  xLabel,
  intervals = false,
  xElapsedUnit = null,
  yFormat = null,
  runStartedAt = null,
  cycleStartedAt = null,
  resetKey,
  actions,
  badge,
  stateLabel,
  stateDetail,
  contextSummary,
  liveWindowS = null,
  viewportMode = "fit",
  legend = null,
  onViewportChange,
}: CartesianPlotProps) {
  const { zoom, syncKey } = usePlotInteraction();
  const [hiddenTraces, setHiddenTraces] = useState<ReadonlySet<string>>(EMPTY_TRACES);
  const [traceColors, setTraceColors] = useState<string[]>([]);
  const [cursor, setCursor] = useState<PlotCursor | null>(null);
  const cursorFrameRef = useRef<number | null>(null);
  const hiddenTracesRef = useRef<ReadonlySet<string>>(hiddenTraces);
  hiddenTracesRef.current = hiddenTraces;
  const onViewportChangeRef = useRef(onViewportChange);
  onViewportChangeRef.current = onViewportChange;
  const tableLegend = legend !== null;
  const [following, setFollowing] = useState(true);
  const heldRangeRef = useRef<HeldViewport | null>(null);
  const rootRef = useRef<HTMLDivElement | null>(null);
  const plotRef = useRef<uPlot | null>(null);
  const markersRef = useRef<PlotMarker[]>(markers);
  const [showPrompts, setShowPrompts] = useState(true);
  const [hoveredPrompt, setHoveredPrompt] = useState<{ marker: PromptMarker; left: number } | null>(null);
  const shownPrompts = showPrompts ? promptMarkers : NO_PROMPTS;
  const promptsRef = useRef<PromptMarker[]>(shownPrompts);
  const referenceLinesRef = useRef<DocumentReferenceLine[]>(referenceLines);
  const referenceBandsRef = useRef<DocumentReferenceBand[]>(referenceBands);
  const intervalRangesRef = useRef<IntervalRange[]>([]);
  const dataSignatureRef = useRef<string | null>(null);
  const drawnRef = useRef<DrawnWindow>({ source: null, aligned: true, range: null, everySample: true });
  const resetKeyRef = useRef<string | undefined>(resetKey);
  const viewportModeRef = useRef<PlotViewportMode>(viewportMode);
  const resizeFrameRef = useRef<number | null>(null);
  const redrawFrameRef = useRef<number | null>(null);
  const updateFrameRef = useRef<number | null>(null);
  const [uPlotConstructor, setUPlotConstructor] = useState<UPlotConstructor | null>(null);
  const [plotLoadError, setPlotLoadError] = useState<string | null>(null);
  // Whether the canvas root has a width yet. A card mounts before the grid
  // has laid it out, and uPlot built at zero width computes a negative plot
  // area, whose tick search finds no increment and pushes splits until the
  // array overflows ("Invalid array length"); the throw then unmounts every
  // sibling card in the same boundary. The plot is built only once the root
  // has been measured.
  const [laidOut, setLaidOut] = useState(false);
  const renderedSeries = useMemo(() => limitRenderedSeries(series, cartesian), [series, cartesian]);
  const backendCartesian = useMemo(() => usableCartesianFrame(cartesian, renderedSeries, series), [cartesian, renderedSeries, series]);
  const chartFrame = useMemo(() => (backendCartesian ? chartFrameFromCartesian(backendCartesian) : null), [backendCartesian]);
  const seriesKey = useMemo(
    () => renderedSeries.map((item) => `${series.indexOf(item)}:${item.key}:${item.label}:${item.downsampled ? 1 : 0}`).join("|"),
    [renderedSeries, series],
  );
  const colorsKey = colors.join("|");
  // Structural, like colorsKey/seriesKey above: a declared format changes what
  // the axes and legend print without changing the series set, so it needs its
  // own key to trigger the structural rebuild below.
  const formatsKey = JSON.stringify({
    x: xFormat ?? null,
    xLabel,
    intervals,
    booleanAxis,
    xElapsed: xElapsedUnit ?? null,
    y: yFormat ?? null,
    series: renderedSeries.map((item) => item.format ?? null),
    runStartedAt: runStartedAt ?? null,
    cycleStartedAt: cycleStartedAt ?? null,
  });
  const intervalRanges = useMemo(() => intervalRangeData(renderedSeries, { xScale, yScale }), [renderedSeries, xScale, yScale]);
  const yReferenceValues = useMemo(
    () => referenceYValues(referenceLines, referenceBands, yScale),
    [referenceBands, referenceLines, yScale],
  );
  const yReferenceKey = yReferenceValues.join("|");
  const hasData = Boolean(chartFrame && chartFrame.pointCount > 0);
  const hasTimeline =
    renderedSeries.some((item) => item.points.length > 0) || Boolean(backendCartesian?.series.some((item) => item.x.length > 0));
  const frameError =
    (cartesian != null || hasTimeline) && !backendCartesian ? "The plot data could not be rendered. Reload the plot to try again." : null;
  const displayError = plotLoadError ?? frameError;
  const chartMode = chartFrame?.mode ?? null;
  const dataSignature = chartFrame?.signature ?? null;
  const renderedPointCount = chartFrame?.pointCount ?? 0;
  // Structural, not per-sample: crossing the render limit switches the point
  // mark style, everything below it is a plain data update.
  const densePoints = renderedPointCount > POINT_MARK_RENDER_LIMIT;
  const liveXRange = useMemo(() => liveCartesianXRange(backendCartesian, liveWindowS), [backendCartesian, liveWindowS]);

  useEffect(() => {
    let cancelled = false;
    loadUPlot()
      .then((uPlotConstructor) => {
        if (cancelled) {
          return;
        }
        setUPlotConstructor(() => uPlotConstructor);
        setPlotLoadError(null);
      })
      .catch((error: unknown) => {
        if (cancelled) {
          return;
        }
        setPlotLoadError(error instanceof Error ? error.message : String(error));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const root = rootRef.current;
    if (!hasData || !root) {
      setLaidOut(false);
      return;
    }
    const measure = () => setLaidOut(root.getBoundingClientRect().width > 0);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(root);
    return () => observer.disconnect();
  }, [hasData]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: intentional structural deps — this effect creates the uPlot instance and must re-run only when the plot's STRUCTURE changes (series set, colors, scales, style, mode, references, declared formats, reset/viewport switches), reading the current render's frame for initial data. Keying on the per-sample identities (chartFrame, dataSignature) destroyed and rebuilt the plot DOM on every stream frame, which tripped the browser's scroll anchoring until it disabled itself and the viewport drifted; per-sample data flows through plot.setData in the update effect below.
  useEffect(() => {
    if (!hasData || !laidOut || !rootRef.current || !chartFrame || !uPlotConstructor) {
      return;
    }
    const root = rootRef.current;
    const themeColors = resolvePlotThemeColors(root);
    const plotFont = resolvePlotFont(root);
    const fallbackSeriesColor = themeColors.palette[0] ?? themeColors.foreground;
    const resolvedSeriesColors = renderedSeries.map((item) => {
      const index = series.indexOf(item);
      return resolvePlotCssColor(colors[index] ?? themeColors.palette[index % themeColors.palette.length], root, fallbackSeriesColor);
    });
    const faceted = chartFrame.mode === "faceted";
    const chartHeight = 230;
    const chartSize = () => ({
      width: measuredChartWidth(root),
      height: chartHeight,
    });
    const showPoints = style === "points" || (style === "line_points" && !densePoints);
    const pointSize = densePoints ? DENSE_POINT_MARK_SIZE : DEFAULT_POINT_MARK_SIZE;
    const lineWidth = style === "points" ? 0 : 2;
    // Elapsed formats need an origin; §7.4 falls back to the first point of
    // the first series (already epoch seconds, the coordinate space the axis
    // itself ticks in) when the payload names neither a run nor a cycle start.
    const firstPointSeconds = backendCartesian?.series[0]?.x[0];
    const fallbackOrigin = Number.isFinite(firstPointSeconds) ? firstPointSeconds : null;
    const timeOrigin: TimeFormatOrigin = {
      runStart: runStartedAt ?? fallbackOrigin,
      cycleStart: cycleStartedAt ?? fallbackOrigin,
    };
    const xAxisValues: uPlot.Axis.Values | undefined = xElapsedUnit
      ? (_plot, splits, _axisIdx, _foundSpace, foundIncr) =>
          splits.map((split) => (split == null ? "" : formatElapsedTick(split, xElapsedUnit, foundIncr)))
      : timeAxis
        ? xFormat
          ? (_plot, splits) => splits.map((split) => (split == null ? "" : formatTime(split, xFormat, timeOrigin)))
          : undefined
        : (_plot, splits, _axisIdx, _foundSpace, foundIncr) =>
            splits.map((split) => (split == null ? "" : formatNumber(split, xFormat, { increment: foundIncr })));
    const yAxisValues: uPlot.Axis.Values = (_plot, splits, _axisIdx, _foundSpace, foundIncr) =>
      splits.map((split) =>
        booleanAxis ? formatBooleanState(split) : split == null ? "" : formatNumber(split, yFormat, { increment: foundIncr }),
      );
    const seriesLegendValue =
      (format: ValueFormat | null): uPlot.Series.Value =>
      (_plot, rawValue) =>
        booleanAxis ? formatBooleanState(rawValue) : rawValue == null ? "--" : formatNumber(rawValue, format);
    const options: uPlot.Options = {
      ...chartSize(),
      mode: faceted ? 2 : 1,
      pxAlign: 0,
      hooks: {
        // Every zoom, pan or follow step resamples the visible window from
        // the full frame, so a zoom never stretches a reduction made for the
        // whole run.
        setScale: [
          (plot, key) => {
            if (key !== "x") return;
            const { min, max } = plot.scales.x;
            if (min == null || max == null) return;
            const range = { min, max };
            if (windowServes(drawnRef.current, range)) return;
            queueMicrotask(() => {
              if (plotRef.current !== plot) return;
              plot.setData(drawnData(drawnRef.current, range, plot.over.clientWidth, zoom), false);
              plot.setScale("x", range);
              const heldY = heldRangeRef.current?.y;
              if (heldY) plot.setScale("y", heldY);
            });
          },
        ],
        setSelect: [
          (plot) => {
            const held = heldViewport(plot.select, { width: plot.over.clientWidth, height: plot.over.clientHeight }, (position, axis) =>
              plot.posToVal(position, axis),
            );
            if (!held) return;
            heldRangeRef.current = held;
            setFollowing(false);
            // A y-only zoom returns x to its live follow, so the statistics
            // return to the window too.
            onViewportChangeRef.current?.(held.x ? { from: held.x.min, to: held.x.max } : null);
          },
        ],
        setCursor: [
          (plot) => {
            const left = plot.cursor.left;
            const prompt = left != null && left >= 0 ? nearestPrompt(plot, promptsRef.current, left) : null;
            setHoveredPrompt((current) => {
              if (!prompt) return current ? null : current;
              const offset = plot.over.getBoundingClientRect().left - root.getBoundingClientRect().left;
              const next = { marker: prompt.marker, left: offset + prompt.left };
              return current?.marker === next.marker && current.left === next.left ? current : next;
            });
            if (!tableLegend || left == null || left < 0) return;
            const next = {
              x: plot.posToVal(left, "x"),
              span: (plot.scales.x.max ?? 0) - (plot.scales.x.min ?? 0),
              widthPx: plot.over.clientWidth,
            };
            if (cursorFrameRef.current !== null) {
              window.cancelAnimationFrame(cursorFrameRef.current);
            }
            cursorFrameRef.current = window.requestAnimationFrame(() => {
              cursorFrameRef.current = null;
              setCursor(next);
            });
          },
        ],
        drawClear: [
          (plot) => {
            drawReferenceBands(plot, referenceBandsRef.current, root, themeColors.foreground);
          },
        ],
        draw: [
          (plot) => {
            drawIntervalRanges(plot, intervalRangesRef.current, resolvedSeriesColors[0] ?? fallbackSeriesColor);
            drawMarkers(plot, markersRef.current, root, themeColors.foreground);
            drawPromptMarkers(plot, promptsRef.current, themeColors.mutedForeground);
            drawReferenceLines(plot, referenceLinesRef.current, root, themeColors.foreground);
          },
        ],
      },
      // A dragged box is drawn as the selection and becomes a held viewport
      // while data continues arriving; a mostly horizontal or vertical drag
      // zooms that axis alone, a double click returns to the full view.
      cursor: {
        drag: { ...PLOT_DRAG_OPTIONS },
        focus: { prox: -1 },
        sync:
          timeAxis && syncKey
            ? {
                key: `${syncKey}:${xElapsedUnit ?? "wall"}`,
                scales: ["x", null],
                setSeries: false,
                filters: { pub: (event) => event === "mousemove", sub: (event) => event === "mousemove" },
              }
            : undefined,
      },
      select: { show: true, left: 0, top: 0, width: 0, height: 0 },
      legend: {
        show: !tableLegend,
        live: true,
      },
      scales: {
        /* `timeAxis` stays the windowing semantic — the live viewport follows
           it — while an elapsed axis only stops uPlot printing dates over
           values that were never epoch seconds. */
        x: { time: timeAxis && !xElapsedUnit, distr: xScale === "log" ? 3 : 1 },
        y: {
          auto: true,
          distr: yScale === "log" ? 3 : 1,
          range: booleanAxis
            ? () => [-0.1, 1.1]
            : yReferenceValues.length || intervals
              ? (_plot, dataMin, dataMax) =>
                  paddedYRange(
                    dataMin,
                    dataMax,
                    [...yReferenceValues, ...intervalRangesRef.current.flatMap((range) => [range.low, range.high])],
                    yScale,
                  )
              : undefined,
        },
      },
      /* The last tick label is centred on the plot's right edge, so half of it
         hangs outside the plotting area; uPlot's own 25px is not enough for a
         time label at touch density, where the axis font is scaled up. */
      padding: [null, (self) => xAxisRightPad(self, plotFont), null, null],
      axes: [
        {
          label: xLabel,
          stroke: themeColors.mutedForeground,
          font: plotFont,
          labelFont: plotFont,
          grid: { stroke: themeColors.border, width: 1 },
          size: (_plot, values) => xAxisGutter(values, plotFont),
          values: xAxisValues,
        },
        {
          stroke: themeColors.mutedForeground,
          font: plotFont,
          labelFont: plotFont,
          grid: { stroke: themeColors.border, width: 1 },
          size: (_plot, values) => yAxisGutter(values, plotFont),
          values: yAxisValues,
          splits: booleanAxis ? () => [0, 1] : undefined,
        },
      ],
      series: [
        faceted
          ? null
          : xElapsedUnit
            ? {
                value: (_plot, rawValue) => (rawValue == null ? "--" : formatElapsedTick(rawValue, xElapsedUnit)),
              }
            : {},
        ...renderedSeries.map((item, index) => ({
          facets: faceted
            ? [
                { scale: "x", auto: true },
                { scale: "y", auto: true },
              ]
            : undefined,
          label: item.label,
          // A reduced report frame reduces each trace on its own, so the
          // shared x leaves holes no sample ever had; only a full frame's
          // gaps are real.
          spanGaps: item.downsampled === true,
          value: seriesLegendValue(item.format ?? yFormat),
          pxAlign: 0,
          stroke: resolvedSeriesColors[index] ?? fallbackSeriesColor,
          width: lineWidth,
          points: faceted ? { show: false } : { show: showPoints, size: pointSize },
          paths: faceted
            ? facetedSeriesPaths(style, showPoints, pointSize, resolvedSeriesColors[index] ?? fallbackSeriesColor, themeColors.background)
            : style === "step"
              ? uPlotConstructor.paths.stepped?.({ align: 1 })
              : undefined,
        })),
      ] as uPlot.Series[],
    };
    // uPlot has no error path of its own: a scale it cannot tick (a span of
    // one float ulp, an infinite bound) throws out of its constructor or its
    // next resize, and an uncaught throw here unmounts every sibling card in
    // the boundary. This card reports the failure and stays alone in it.
    let plot: uPlot;
    try {
      drawnRef.current = { source: chartFrame.data, aligned: !faceted, range: null, everySample: true };
      plot = new uPlotConstructor(options, drawnData(drawnRef.current, null, measuredChartWidth(root), zoom), root);
    } catch (error) {
      setPlotLoadError(plotFailureMessage(error));
      return;
    }
    if (resetKeyRef.current !== resetKey || viewportModeRef.current !== viewportMode) {
      heldRangeRef.current = null;
      setFollowing(true);
      onViewportChangeRef.current?.(null);
    }
    if (heldRangeRef.current) {
      applyHeldViewport(plot, heldRangeRef.current, drawnData(drawnRef.current, null, plot.over.clientWidth, zoom), () =>
        setLiveXScale(plot, liveXRange, xScale, timeAxis),
      );
    }
    renderedSeries.forEach((item, index) => {
      if (hiddenTracesRef.current.has(item.key)) plot.setSeries(index + 1, { show: false });
    });
    setTraceColors((current) => (current.join("|") === resolvedSeriesColors.join("|") ? current : resolvedSeriesColors));
    plotRef.current = plot;
    dataSignatureRef.current = dataSignature;
    resetKeyRef.current = resetKey;
    viewportModeRef.current = viewportMode;

    const detachPan = attachPan(plot, {
      logX: xScale === "log",
      logY: yScale === "log",
      panY: !timeAxis,
      onPan: (next) => {
        const held = { x: next.x, y: next.y ?? heldRangeRef.current?.y ?? null };
        heldRangeRef.current = held;
        setFollowing(false);
        onViewportChangeRef.current?.(held.x ? { from: held.x.min, to: held.x.max } : null);
      },
    });
    const resize = () => {
      const nextSize = chartSize();
      if (plot.width === nextSize.width && plot.height === nextSize.height) {
        return;
      }
      try {
        plot.setSize(nextSize);
        if (zoom) plot.setData(drawnData(drawnRef.current, heldRangeRef.current?.x ?? null, plot.over.clientWidth, true), false);
      } catch (error) {
        setPlotLoadError(plotFailureMessage(error));
      }
    };
    const scheduleResize = () => {
      if (resizeFrameRef.current !== null) {
        return;
      }
      resizeFrameRef.current = window.requestAnimationFrame(() => {
        resizeFrameRef.current = null;
        resize();
      });
    };
    scheduleResize();
    const observer = new ResizeObserver(scheduleResize);
    observer.observe(root);
    window.addEventListener("resize", scheduleResize);
    /* uPlot's own double click already auto-scaled both axes; the hold is
       dropped so the next frame follows the run again instead of re-pinning. */
    const resume = () => {
      heldRangeRef.current = null;
      setFollowing(true);
      onViewportChangeRef.current?.(null);
    };
    root.addEventListener("dblclick", resume);
    return () => {
      detachPan();
      root.removeEventListener("dblclick", resume);
      if (resizeFrameRef.current !== null) {
        window.cancelAnimationFrame(resizeFrameRef.current);
        resizeFrameRef.current = null;
      }
      if (redrawFrameRef.current !== null) {
        window.cancelAnimationFrame(redrawFrameRef.current);
        redrawFrameRef.current = null;
      }
      observer.disconnect();
      window.removeEventListener("resize", scheduleResize);
      if (updateFrameRef.current !== null) {
        window.cancelAnimationFrame(updateFrameRef.current);
        updateFrameRef.current = null;
      }
      if (cursorFrameRef.current !== null) {
        window.cancelAnimationFrame(cursorFrameRef.current);
        cursorFrameRef.current = null;
      }
      plot.destroy();
      plotRef.current = null;
      dataSignatureRef.current = null;
    };
    // Recreate the uPlot instance only on STRUCTURAL changes (series set,
    // colors, scales, style, mode, references, declared formats, reset/viewport
    // switches). Data identities (`chartFrame`, `dataSignature`) are
    // deliberately absent: per sample updates flow through `plot.setData` in
    // the effect below. Keying on them destroyed and rebuilt the whole plot DOM
    // on every stream frame, which tripped the browser's scroll anchoring
    // ("too many consecutive adjustments") until it disabled itself and the
    // viewport drifted. The effect body reads the current render's frame for
    // the initial data.
  }, [
    hasData,
    laidOut,
    zoom,
    densePoints,
    style,
    timeAxis,
    booleanAxis,
    uPlotConstructor,
    xScale,
    yScale,
    viewportMode,
    resetKey,
    colorsKey,
    seriesKey,
    yReferenceKey,
    chartMode,
    formatsKey,
    tableLegend,
    syncKey,
  ]);

  // Visibility is this card's own state: toggling a trace shows or hides its
  // series in place, without rebuilding the plot.
  useEffect(() => {
    const plot = plotRef.current;
    if (!plot) return;
    renderedSeries.forEach((item, index) => {
      const show = !hiddenTraces.has(item.key);
      if (plot.series[index + 1]?.show !== show) plot.setSeries(index + 1, { show });
    });
  }, [hiddenTraces, renderedSeries]);

  const legendRows = useMemo(() => {
    if (!legend) return null;
    const frame = backendCartesian;
    const cursorPoints = cursor !== null && frame ? frame.series.map((item) => nearestDrawnPoint(item.x, item.y, cursor.x)) : null;
    const firstPointSeconds = frame?.series[0]?.x[0];
    const fallbackOrigin = Number.isFinite(firstPointSeconds) ? firstPointSeconds : null;
    const axis = {
      timeAxis,
      xFormat,
      xElapsedUnit,
      origin: { runStart: runStartedAt ?? fallbackOrigin, cycleStart: cycleStartedAt ?? fallbackOrigin },
      span: cursor?.span ?? 0,
      widthPx: cursor?.widthPx ?? 0,
    };
    return seriesLegendRows({
      cursorPoints,
      formatX: (x) => formatCursorX(x, axis),
      hidden: hiddenTraces,
      summaries: legend.summaries,
      traces: renderedSeries.map((item, index) => ({
        key: item.key,
        label: item.label,
        color: traceColors[index] ?? colors[series.indexOf(item)] ?? "currentColor",
      })),
    });
  }, [
    backendCartesian,
    colors,
    cursor,
    cycleStartedAt,
    hiddenTraces,
    legend,
    renderedSeries,
    runStartedAt,
    series,
    timeAxis,
    traceColors,
    xElapsedUnit,
    xFormat,
  ]);
  const legendColumns = useMemo(() => seriesLegendColumns(unit), [unit]);

  useEffect(() => {
    markersRef.current = markers;
    promptsRef.current = shownPrompts;
    referenceLinesRef.current = referenceLines;
    referenceBandsRef.current = referenceBands;
    intervalRangesRef.current = intervalRanges;
    if (redrawFrameRef.current !== null) {
      window.cancelAnimationFrame(redrawFrameRef.current);
    }
    redrawFrameRef.current = window.requestAnimationFrame(() => {
      redrawFrameRef.current = null;
      const plot = plotRef.current;
      if (plot?.status === 1) {
        plot.redraw(true, true);
      }
    });
  }, [intervalRanges, markers, referenceBands, referenceLines, shownPrompts]);

  useEffect(() => {
    const plot = plotRef.current;
    if (!hasData || !plot || !chartFrame) {
      return;
    }

    if (updateFrameRef.current !== null) {
      window.cancelAnimationFrame(updateFrameRef.current);
    }
    updateFrameRef.current = window.requestAnimationFrame(() => {
      updateFrameRef.current = null;
      drawnRef.current.source = chartFrame.data;
      const fullRange = () => drawnData(drawnRef.current, null, plot.over.clientWidth, zoom);
      const resetChanged = resetKeyRef.current !== resetKey;
      const viewportModeChanged = viewportModeRef.current !== viewportMode;
      try {
        if (heldRangeRef.current && !resetChanged && !viewportModeChanged) {
          applyHeldViewport(plot, heldRangeRef.current, fullRange(), () => {
            if (viewportMode === "live") setLiveXScale(plot, liveXRange, xScale, timeAxis);
          });
          dataSignatureRef.current = dataSignature;
          return;
        }
        if (!resetChanged && !viewportModeChanged && dataSignatureRef.current === dataSignature) {
          if (viewportMode === "live") {
            setLiveXScale(plot, liveXRange, xScale, timeAxis);
          }
          return;
        }
        if (viewportMode === "live") {
          plot.setData(fullRange(), false);
          setLiveXScale(plot, liveXRange, xScale, timeAxis);
        } else {
          plot.setData(fullRange(), true);
        }
      } catch (error) {
        setPlotLoadError(plotFailureMessage(error));
        return;
      }
      resetKeyRef.current = resetKey;
      viewportModeRef.current = viewportMode;
      dataSignatureRef.current = dataSignature;
    });
  }, [chartFrame, dataSignature, hasData, liveXRange, resetKey, timeAxis, viewportMode, xScale, zoom]);

  const placeholder = displayError
    ? plotUnavailableText(displayError)
    : !hasData
      ? hasTimeline
        ? plotPlaceholderText(
            { stateLabel: stateLabel ?? "Unavailable", stateDetail },
            "The stream is active, but this signal is unavailable in the selected window.",
          )
        : plotPlaceholderText({ stateLabel, stateDetail })
      : null;

  // Keep The Settings Action Mounted As Samples Arrive Or Disappear
  return (
    <article className="plot-card grid min-w-0 max-w-full overflow-hidden gap-2">
      <div className="flex items-center justify-between gap-3">
        <div className="min-w-0">
          <strong className={titleRepeated && !unit ? "sr-only" : "block truncate"} title={formatUnitLabel(title, unit)}>
            {formatUnitLabel(title, unit)}
          </strong>
          {description ? <p className="mt-1 text-xs text-muted-foreground">{description}</p> : null}
          {contextSummary ? <p className="mt-1 text-xs text-muted-foreground">{contextSummary}</p> : null}
        </div>
        <div className="flex items-center gap-2">
          {actions}
          {!placeholder ? (
            <>
              {promptMarkers.length ? (
                <button
                  type="button"
                  aria-pressed={showPrompts}
                  className="plot-prompts rounded-lg border px-2 py-1 text-xs hover:bg-muted focus-visible:outline-2 focus-visible:outline-ring aria-[pressed=false]:text-muted-foreground"
                  onClick={() => {
                    setShowPrompts((current) => !current);
                    setHoveredPrompt(null);
                  }}
                  title="Show When The Operator Was Asked And Answered"
                >
                  Prompts
                </button>
              ) : null}
              <button
                type="button"
                aria-pressed={following}
                className="plot-follow rounded-lg border px-2 py-1 text-xs hover:bg-muted focus-visible:outline-2 focus-visible:outline-ring"
                onClick={() => {
                  heldRangeRef.current = null;
                  setFollowing(true);
                  onViewportChangeRef.current?.(null);
                  const plot = plotRef.current;
                  if (plot && chartFrame) {
                    plot.setData(drawnData(drawnRef.current, null, plot.over.clientWidth, zoom), true);
                    if (viewportMode === "live") setLiveXScale(plot, liveXRange, xScale, timeAxis);
                  }
                }}
              >
                {zoom ? "Reset Zoom" : "Follow"}
              </button>
              <span className="text-xs text-muted-foreground">
                {renderedPointCount} Points
                {downsampledLabel(renderedSeries)}
                {series.length > renderedSeries.length ? ` - ${renderedSeries.length} of ${series.length} series` : ""}
                {xScale === "log" || yScale === "log" ? " - log" : ""}
                {badge ? ` - ${badge}` : ""}
              </span>
            </>
          ) : null}
        </div>
      </div>
      {placeholder ? (
        <PlotPlaceholder {...placeholder} />
      ) : (
        <div className="relative min-w-0 max-w-full">
          <div ref={rootRef} className="min-h-60 min-w-0 max-w-full overflow-hidden [&_.uplot]:max-w-full [&_.uplot]:overflow-hidden" />
          {hoveredPrompt && showPrompts ? (
            <div
              className="pointer-events-none absolute top-1 z-10 max-w-64 -translate-x-1/2 truncate rounded-lg border bg-popover px-2 py-1 text-xs text-popover-foreground shadow-sm"
              role="tooltip"
              style={{ left: `${hoveredPrompt.left}px` }}
            >
              {promptMarkerLabel(hoveredPrompt.marker)}
            </div>
          ) : null}
        </div>
      )}
      {!placeholder && legend && legendRows ? (
        <StatsSummaryTable
          legend={{
            columns: legendColumns,
            rows: legendRows,
            pending: legend.pending,
            onToggle: (key) => setHiddenTraces((current) => toggleTrace(current, key)),
          }}
        />
      ) : null}
    </article>
  );
});

/**
 * The axis font, with its size in pixels.
 *
 * uPlot reads the size back out of the font string with a `px` pattern to
 * place tick labels and scale for the device pixel ratio; a `rem` size from
 * the brand theme matches nothing, the label positions turn NaN and the axes
 * draw their grid but no text. So the theme's size is resolved against the
 * root font size here, and the fallback keeps the axes readable when the
 * variables are absent.
 */
function resolvePlotFont(root: Element): string {
  const styles = window.getComputedStyle(root);
  // Axis text is drawn to canvas, so the touch typography rules in styles.css
  // never reach it: the density is read here instead.
  const touch = document.documentElement.dataset.density === "touch";
  const size = styles.getPropertyValue("--brand-text-xs").trim();
  const family = styles.getPropertyValue("--brand-font-sans").trim() || "sans-serif";
  const px = fontSizePx(size, root);
  return `${touch ? Math.round(px * TOUCH_PLOT_FONT_SCALE) : px}px ${family}`;
}

function fontSizePx(size: string, root: Element): number {
  const value = Number.parseFloat(size);
  if (!Number.isFinite(value) || value <= 0) {
    return 12;
  }
  if (size.endsWith("px")) {
    return Math.round(value);
  }
  if (size.endsWith("rem")) {
    const rootSize = Number.parseFloat(window.getComputedStyle(document.documentElement).fontSize);
    return Math.round(value * (Number.isFinite(rootSize) && rootSize > 0 ? rootSize : 16));
  }
  if (size.endsWith("em")) {
    const ownSize = Number.parseFloat(window.getComputedStyle(root).fontSize);
    return Math.round(value * (Number.isFinite(ownSize) && ownSize > 0 ? ownSize : 16));
  }
  return Math.round(value);
}

/* === Axis gutter === */

/** Measures text at `font`, reusing one offscreen canvas for every call. */
let gutterContext: CanvasRenderingContext2D | null = null;

function measureTextWidth(text: string, font: string): number {
  if (!gutterContext) {
    gutterContext = document.createElement("canvas").getContext("2d");
  }
  if (!gutterContext) {
    // No 2d context (a headless shell without canvas): fall back to a
    // conservative per-character estimate rather than clipping the labels.
    return text.length * Y_AXIS_CHARACTER_WIDTH;
  }
  gutterContext.font = font;
  return gutterContext.measureText(text).width;
}

/**
 * The bottom gutter the x tick labels need.
 *
 * uPlot's default is a fixed 50px, which a time axis at touch density
 * overruns: it draws the date on a second line at 1.5x a font that is itself
 * scaled up, and the second line is then clipped by the card below. The gutter
 * is sized from the lines uPlot is about to draw.
 */
function xAxisGutter(values: string[] | null | undefined, font: string): number {
  const lines = (values ?? []).reduce((most, value) => Math.max(most, value ? value.split("\n").length : 1), 1);
  const size = X_AXIS_TICK_ROOM + Math.ceil(lines * plotFontPx(font) * AXIS_LINE_GAP);
  return Math.min(X_AXIS_MAX_GUTTER, Math.max(X_AXIS_MIN_GUTTER, size));
}

/** The right padding the last x tick label needs to stay inside the card. */
function xAxisRightPad(plot: uPlot, font: string): number {
  const values = (plot?.axes?.[0] as { _values?: string[] } | undefined)?._values ?? [];
  const last = values.at(-1) ?? "";
  const widest = last.split("\n").reduce((width, line) => Math.max(width, measureTextWidth(line, font)), 0);
  return Math.max(X_AXIS_MIN_RIGHT_PAD, Math.ceil(widest / 2) + 2);
}

/** The pixel size uPlot reads back out of the axis font string. */
function plotFontPx(font: string): number {
  const value = Number.parseFloat(font);
  return Number.isFinite(value) && value > 0 ? value : 12;
}

/**
 * The left gutter the widest formatted tick label needs.
 *
 * uPlot's default is a fixed 50px, which silently clips wider labels — a
 * `1.0e-13` tick printed as `.0e-13`, which reads as data rather than as a
 * layout fault. The gutter is sized from the labels uPlot is about to draw,
 * plus its tick marks and gap.
 */
function yAxisGutter(values: string[] | null | undefined, font: string): number {
  const widest = (values ?? []).reduce((width, value) => Math.max(width, value ? measureTextWidth(value, font) : 0), 0);
  return Math.min(Y_AXIS_MAX_GUTTER, Math.max(Y_AXIS_MIN_GUTTER, Math.ceil(widest) + Y_AXIS_LABEL_PADDING));
}

/* === Private Functions === */

export function usableCartesianFrame(
  cartesian: PlotCartesianFrame | null | undefined,
  renderedSeries: PlotSeries[],
  allSeries: PlotSeries[],
): PlotCartesianFrame | null {
  if (!cartesian || !Array.isArray(cartesian.series)) {
    return null;
  }
  if (cartesian.series.length !== allSeries.length || !cartesian.series.every((series) => series.x.length === series.y.length)) {
    return null;
  }

  if (renderedSeries === allSeries) {
    return cartesian;
  }

  // The payload and descriptors share their original order. Apply the same
  // selection to both so labels, colors and values stay paired after limiting.
  const indices = renderedSeries.map((series) => allSeries.indexOf(series));
  if (indices.some((index) => index < 0)) {
    return null;
  }
  const selected = indices.map((index) => cartesian.series[index]);
  return {
    ...cartesian,
    series: selected,
    signature: `${cartesian.signature}:${indices.join(",")}`,
  };
}

function chartFrameFromCartesian(cartesian: PlotCartesianFrame): ChartFrame {
  return {
    data:
      cartesian.mode === "faceted"
        ? ([null, ...cartesian.series.map((series) => [series.x, series.y])] as unknown as uPlot.AlignedData)
        : ([cartesian.series[0]?.x ?? [], ...cartesian.series.map((series) => series.y)] as uPlot.AlignedData),
    mode: cartesian.mode,
    pointCount: cartesian.point_count,
    signature: cartesian.signature,
  };
}

export function liveCartesianXRange(
  cartesian: PlotCartesianFrame | null | undefined,
  windowS: number | null | undefined,
): PlotViewportRange | null {
  // A loop, not a spread: a retained frame can exceed the engine's argument limit.
  let earliest = Number.POSITIVE_INFINITY;
  let latest = Number.NEGATIVE_INFINITY;
  for (const series of cartesian?.series ?? []) {
    for (const value of series.x) {
      if (!Number.isFinite(value)) continue;
      if (value < earliest) earliest = value;
      if (value > latest) latest = value;
    }
  }
  if (!Number.isFinite(latest) || !Number.isFinite(earliest)) {
    return null;
  }

  if (typeof windowS === "number" && Number.isFinite(windowS) && windowS > 0) {
    const fixedMax = earliest + windowS;
    if (latest <= fixedMax) {
      return { min: earliest, max: fixedMax };
    }
    return { min: latest - windowS, max: latest };
  }

  if (earliest === latest) {
    return null;
  }

  return { min: earliest, max: latest };
}

/**
 * Follows the run along a time axis.
 *
 * Only a time axis follows: the live window is seconds, and applied to a
 * position axis it would pin an xy plot to a span its own data decides. The
 * span is also required to be wider than float noise around its bounds; an
 * explicit range one ulp wide gives uPlot an increment its tick loop cannot
 * advance by, and that loop pushes splits until the array overflows.
 */
function setLiveXScale(plot: uPlot, range: PlotViewportRange | null, xScale: DocumentAxisScale, timeAxis: boolean): void {
  if (!timeAxis || !range || !isUsableSpan(range.min, range.max)) {
    return;
  }
  if (xScale === "log" && range.min <= 0) {
    return;
  }
  plot.setScale("x", range);
}

/**
 * Re-pins a held viewport over fresh data: the axes are auto-ranged first so
 * an axis the drag left alone keeps ranging with the data, then each held
 * axis is set back, and an unheld x axis follows the run on a live surface.
 */
function applyHeldViewport(plot: uPlot, held: HeldViewport, data: uPlot.AlignedData, followX: () => void): void {
  plot.setData(data, true);
  if (held.x) plot.setScale("x", held.x);
  else followX();
  if (held.y) plot.setScale("y", held.y);
}

/** Whether `[min, max]` is finite and wider than float noise around its bounds. */
function isUsableSpan(min: number, max: number): boolean {
  if (!Number.isFinite(min) || !Number.isFinite(max)) {
    return false;
  }
  const scale = Math.max(Math.abs(min), Math.abs(max), Number.MIN_VALUE);
  return max - min > scale * 1e-9;
}

/**
 * The data the canvas draws for the x `range` (`null` for the whole extent),
 * reduced to four samples per pixel column of a `columns`-wide plot and
 * recorded as what is now drawn. A faceted frame is drawn whole.
 */
function drawnData(drawn: DrawnWindow, range: PlotViewportRange | null, columns: number, fitPixels: boolean): uPlot.AlignedData {
  const source = drawn.source ?? ([[]] as unknown as uPlot.AlignedData);
  drawn.range = range;
  if (!drawn.aligned) {
    drawn.everySample = true;
    return source;
  }

  const [x, ...traces] = source as unknown as [number[], ...(number | null | undefined)[][]];
  const indices = windowIndices(x, traces, range, columns, fitPixels);
  drawn.everySample = indices === null;
  if (!indices) return source;

  return [indices.map((index) => x[index]), ...traces.map((trace) => indices.map((index) => trace[index] ?? null))] as uPlot.AlignedData;
}

/** Whether what is drawn already resolves the visible x `range`. */
function windowServes(drawn: DrawnWindow, range: PlotViewportRange): boolean {
  if (drawn.everySample) return true;
  const drawnRange = drawn.range;
  if (drawnRange) {
    const tolerance = Math.max(Math.abs(range.max - range.min), Number.MIN_VALUE) * 1e-9;
    return Math.abs(drawnRange.min - range.min) <= tolerance && Math.abs(drawnRange.max - range.max) <= tolerance;
  }
  // The whole-extent reduction serves only a view of the whole extent.
  const x = (drawn.source?.[0] ?? []) as number[];
  return x.length === 0 || (range.min <= x[0] && range.max >= x[x.length - 1]);
}

function plotFailureMessage(error: unknown): string {
  const text = error instanceof Error ? error.message : String(error);
  return `The plot could not be drawn: ${text}`;
}

function intervalRangeData(series: PlotSeries[], scales: { xScale: DocumentAxisScale; yScale: DocumentAxisScale }): IntervalRange[] {
  return series.flatMap((item) =>
    item.points
      .map((point, index) => {
        const x = xValue(point.x, index);
        if (
          !Number.isFinite(x) ||
          !isNumber(point.low) ||
          !isNumber(point.high) ||
          (scales.xScale === "log" && x <= 0) ||
          (scales.yScale === "log" && (point.low <= 0 || point.high <= 0))
        ) {
          return null;
        }
        return {
          x,
          low: Math.min(point.low, point.high),
          high: Math.max(point.low, point.high),
        };
      })
      .filter((range): range is IntervalRange => range !== null),
  );
}

/** The series with the most samples, ranked by the cartesian frame that holds them when present. */
export function limitRenderedSeries(series: PlotSeries[], cartesian?: PlotCartesianFrame | null): PlotSeries[] {
  if (series.length <= MAX_RENDERED_SERIES) {
    return series;
  }
  const sampleCount = (item: PlotSeries) => {
    const frame = cartesian?.series[series.indexOf(item)];
    return frame ? frame.y.filter((value) => value != null).length : item.points.length;
  };
  const counts = new Map(series.map((item) => [item, sampleCount(item)]));
  return [...series].sort((left, right) => (counts.get(right) ?? 0) - (counts.get(left) ?? 0)).slice(0, MAX_RENDERED_SERIES);
}

/**
 * The width the plot is built at, never below the axis gutters.
 *
 * A width smaller than the left axis leaves uPlot a negative plotting area,
 * and its increment search then answers zero, which the tick loop never
 * advances past. The floor keeps a mismeasured root from taking the plot down;
 * the laid-out gate above keeps it from being measured too early at all.
 */
function measuredChartWidth(element: HTMLElement): number {
  const width = element.getBoundingClientRect().width || element.clientWidth;
  return Math.max(MIN_CHART_WIDTH, Math.floor(width));
}

function loadUPlot(): Promise<UPlotConstructor> {
  ensureValidNavigatorLanguage();
  if (!uPlotPromise) {
    uPlotPromise = import("uplot")
      .then((module) => {
        const loaded = module as unknown as {
          default?: UPlotConstructor;
        };
        return loaded.default ?? (module as unknown as UPlotConstructor);
      })
      .catch((error: unknown) => {
        uPlotPromise = null;
        throw error;
      });
  }
  return uPlotPromise;
}

function ensureValidNavigatorLanguage() {
  if (typeof navigator === "undefined" || validLocale(navigator.language)) {
    return;
  }

  const fallback = "en-US";
  patchNavigatorLocale(navigator, fallback);
  const prototype = Object.getPrototypeOf(navigator);
  if (prototype) {
    patchNavigatorLocale(prototype, fallback);
  }
}

function patchNavigatorLocale(target: object, locale: string) {
  try {
    Object.defineProperty(target, "language", {
      configurable: true,
      get: () => locale,
    });
    Object.defineProperty(target, "languages", {
      configurable: true,
      get: () => [locale],
    });
  } catch {
    // Some browser shells expose a non-configurable Navigator prototype.
  }
}

function validLocale(value: unknown): value is string {
  if (typeof value !== "string" || !value.trim()) {
    return false;
  }
  try {
    return Intl.NumberFormat.supportedLocalesOf([value]).length > 0;
  } catch {
    return false;
  }
}

function plotPixelRatio(): number {
  return window.devicePixelRatio || 1;
}

function facetedSeriesPaths(
  style: DocumentPlotStyle,
  showPoints: boolean,
  pointSize: number,
  color: string,
  pointFillColor: string,
): uPlot.Series.PathBuilder {
  return (plot, seriesIdx) => {
    const data = facetedSeriesData(plot, seriesIdx);
    if (!data) {
      return null;
    }

    const [xValues, yValues] = data;
    const { ctx, bbox } = plot;
    const pixelRatio = plotPixelRatio();
    const radius = (pointSize * pixelRatio) / 2;
    const line = style === "points" ? null : new Path2D();
    const points = showPoints ? new Path2D() : null;
    let lineStarted = false;
    let previousTop: number | null = null;

    ctx.save();
    ctx.beginPath();
    ctx.rect(bbox.left, bbox.top, bbox.width, bbox.height);
    ctx.clip();

    for (let index = 0; index < xValues.length; index += 1) {
      const x = xValues[index];
      const y = yValues[index];
      if (!isNumber(x) || !isNumber(y)) {
        lineStarted = false;
        previousTop = null;
        continue;
      }
      const left = plot.valToPos(x, "x", true);
      const top = plot.valToPos(y, "y", true);
      if (line) {
        if (!lineStarted) {
          line.moveTo(left, top);
          lineStarted = true;
        } else if (style === "step" && previousTop !== null) {
          line.lineTo(left, previousTop);
          line.lineTo(left, top);
        } else {
          line.lineTo(left, top);
        }
        previousTop = top;
      }
      if (points) {
        points.moveTo(left + radius, top);
        points.arc(left, top, radius, 0, Math.PI * 2);
      }
    }

    if (line) {
      ctx.strokeStyle = color;
      ctx.lineWidth = 2 * pixelRatio;
      ctx.stroke(line);
    }
    if (points) {
      const alpha = ctx.globalAlpha;
      ctx.fillStyle = pointFillColor;
      ctx.globalAlpha = alpha * 0.82;
      ctx.fill(points);
      ctx.globalAlpha = alpha;
      ctx.strokeStyle = color;
      ctx.lineWidth = 1.5 * pixelRatio;
      ctx.stroke(points);
    }
    ctx.restore();
    return null;
  };
}

function facetedSeriesData(plot: uPlot, seriesIdx: number): [number[], Array<number | null>] | null {
  const data = plot.data[seriesIdx] as unknown;
  if (!Array.isArray(data) || !Array.isArray(data[0]) || !Array.isArray(data[1])) {
    return null;
  }
  return [data[0] as number[], data[1] as Array<number | null>];
}

function drawIntervalRanges(plot: uPlot, ranges: IntervalRange[], color: string) {
  if (!ranges.length) {
    return;
  }
  const { ctx, bbox } = plot;
  const capWidth = 7;
  ctx.save();
  ctx.strokeStyle = color;
  ctx.fillStyle = color;
  ctx.lineWidth = 1.5;
  ranges.forEach((range) => {
    const left = plot.valToPos(range.x, "x", true);
    const top = plot.valToPos(range.high, "y", true);
    const bottom = plot.valToPos(range.low, "y", true);
    if (left < bbox.left || left > bbox.left + bbox.width) {
      return;
    }
    ctx.beginPath();
    ctx.moveTo(left, top);
    ctx.lineTo(left, bottom);
    ctx.moveTo(left - capWidth, top);
    ctx.lineTo(left + capWidth, top);
    ctx.moveTo(left - capWidth, bottom);
    ctx.lineTo(left + capWidth, bottom);
    ctx.stroke();
    const alpha = ctx.globalAlpha;
    ctx.globalAlpha = alpha * 0.15;
    ctx.fillRect(left - 2, top, 4, Math.max(bottom - top, 1));
    ctx.globalAlpha = alpha;
  });
  ctx.restore();
}

function drawMarkers(plot: uPlot, markers: PlotMarker[], element: Element, fallbackColor: string) {
  if (!markers.length) {
    return;
  }
  const { ctx, bbox } = plot;
  ctx.save();
  ctx.setLineDash([4, 4]);
  ctx.lineWidth = 1;
  markers.forEach((marker) => {
    const x = xValue(marker.x, 0);
    if (!Number.isFinite(x)) {
      return;
    }
    const left = Math.round(plot.valToPos(x, "x", true)) + 0.5;
    if (left < bbox.left || left > bbox.left + bbox.width) {
      return;
    }
    ctx.strokeStyle = resolvePlotCssColor(marker.color, element, fallbackColor);
    ctx.beginPath();
    ctx.moveTo(left, bbox.top);
    ctx.lineTo(left, bbox.top + bbox.height);
    ctx.stroke();
  });
  ctx.restore();
}

/**
 * Prompt markers: a vertical line per moment, quieter than declared markers.
 * An asked prompt is dashed under a hollow cap, an answer solid under a
 * filled one, both in the muted axis colour.
 */
function drawPromptMarkers(plot: uPlot, markers: PromptMarker[], color: string) {
  if (!markers.length) {
    return;
  }
  const { ctx, bbox } = plot;
  const ratio = plotPixelRatio();
  const cap = PROMPT_CAP_PX * ratio;
  ctx.save();
  ctx.strokeStyle = color;
  ctx.fillStyle = color;
  ctx.lineWidth = ratio;
  ctx.globalAlpha *= 0.7;
  markers.forEach((marker) => {
    const left = Math.round(plot.valToPos(marker.x, "x", true)) + 0.5;
    if (!Number.isFinite(left) || left < bbox.left || left > bbox.left + bbox.width) {
      return;
    }
    const asked = marker.kind === "asked";
    ctx.setLineDash(asked ? [3 * ratio, 3 * ratio] : []);
    ctx.beginPath();
    ctx.moveTo(left, bbox.top + cap);
    ctx.lineTo(left, bbox.top + bbox.height);
    ctx.stroke();

    ctx.setLineDash([]);
    ctx.beginPath();
    ctx.moveTo(left - cap, bbox.top);
    ctx.lineTo(left + cap, bbox.top);
    ctx.lineTo(left, bbox.top + cap);
    ctx.closePath();
    if (asked) ctx.stroke();
    else ctx.fill();
  });
  ctx.restore();
}

/** The prompt marker within reach of the pointer at `left` (CSS pixels over the plot), if any. */
function nearestPrompt(plot: uPlot, markers: PromptMarker[], left: number): { marker: PromptMarker; left: number } | null {
  let best: { marker: PromptMarker; left: number } | null = null;
  let distance = PROMPT_HOVER_PX + 1;
  for (const marker of markers) {
    const position = plot.valToPos(marker.x, "x");
    const gap = Math.abs(position - left);
    if (Number.isFinite(position) && gap < distance) {
      best = { marker, left: position };
      distance = gap;
    }
  }
  return best;
}

function drawReferenceLines(plot: uPlot, lines: DocumentReferenceLine[], element: Element, fallbackColor: string) {
  const { ctx, bbox } = plot;
  const bottom = bbox.top + bbox.height;
  const right = bbox.left + bbox.width;
  ctx.save();
  ctx.lineWidth = 1;
  lines.forEach((line) => {
    if (!isNumber(line.value)) {
      return;
    }
    ctx.strokeStyle = resolvePlotCssColor(line.color, element, fallbackColor);
    ctx.setLineDash([5, 3]);
    ctx.beginPath();
    if (line.axis === "x") {
      const left = Math.round(plot.valToPos(line.value, "x", true)) + 0.5;
      if (left < bbox.left || left > right) {
        return;
      }
      ctx.moveTo(left, bbox.top);
      ctx.lineTo(left, bottom);
    } else {
      const top = Math.round(plot.valToPos(line.value, "y", true)) + 0.5;
      if (top < bbox.top || top > bottom) {
        return;
      }
      ctx.moveTo(bbox.left, top);
      ctx.lineTo(right, top);
    }
    ctx.stroke();
  });
  ctx.restore();
}

function drawReferenceBands(plot: uPlot, bands: DocumentReferenceBand[], element: Element, fallbackColor: string) {
  const { ctx, bbox } = plot;
  const bottomEdge = bbox.top + bbox.height;
  const rightEdge = bbox.left + bbox.width;
  ctx.save();
  bands.forEach((band) => {
    if (!isNumber(band.min) || !isNumber(band.max)) {
      return;
    }
    ctx.fillStyle = resolvePlotCssColor(band.color, element, fallbackColor);
    const alpha = ctx.globalAlpha;
    ctx.globalAlpha = alpha * 0.15;
    if (band.axis === "x") {
      const left = plot.valToPos(band.min, "x", true);
      const right = plot.valToPos(band.max, "x", true);
      const clampedLeft = clamp(left, bbox.left, rightEdge);
      const clampedRight = clamp(right, bbox.left, rightEdge);
      ctx.fillRect(clampedLeft, bbox.top, clampedRight - clampedLeft, bbox.height);
    } else {
      const top = plot.valToPos(band.max, "y", true);
      const bottom = plot.valToPos(band.min, "y", true);
      const clampedTop = clamp(top, bbox.top, bottomEdge);
      const clampedBottom = clamp(bottom, bbox.top, bottomEdge);
      ctx.fillRect(bbox.left, clampedTop, bbox.width, clampedBottom - clampedTop);
    }
    ctx.globalAlpha = alpha;
  });
  ctx.restore();
}

function referenceYValues(lines: DocumentReferenceLine[], bands: DocumentReferenceBand[], yScale: DocumentAxisScale): number[] {
  const values = [
    ...lines.filter((line) => line.axis === "y").map((line) => line.value),
    ...bands.filter((band) => band.axis === "y").flatMap((band) => [band.min, band.max]),
  ].filter(isNumber);
  return yScale === "log" ? values.filter((value) => value > 0) : values;
}

function paddedYRange(
  dataMin: number,
  dataMax: number,
  referenceValues: number[],
  yScale: DocumentAxisScale,
): [number | null, number | null] {
  if (!Number.isFinite(dataMin) || !Number.isFinite(dataMax)) {
    return [null, null];
  }
  const values = [dataMin, dataMax, ...referenceValues].filter(isNumber);
  const usable = yScale === "log" ? values.filter((value) => value > 0) : values;
  if (!usable.length) {
    return [null, null];
  }
  let min = Math.min(...usable);
  let max = Math.max(...usable);
  if (!isUsableSpan(min, max)) {
    if (yScale === "log") {
      const low = Math.max(Number.MIN_VALUE, min / 1.1);
      const high = max * 1.1;
      return finiteRange(low, high);
    }
    const pad = Math.max(Math.abs(min) * 0.1, 1);
    return finiteRange(min - pad, max + pad);
  }
  if (yScale === "log") {
    return finiteRange(min, max);
  }
  const pad = (max - min) * 0.08;
  min -= pad;
  max += pad;
  return finiteRange(min, max);
}

function finiteRange(min: number, max: number): [number | null, number | null] {
  if (!Number.isFinite(min) || !Number.isFinite(max) || min >= max) {
    return [null, null];
  }
  return [min, max];
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function downsampledLabel(series: PlotSeries[]): string {
  const rawPoints = Math.max(0, ...series.map((item) => item.raw_points));
  return series.some((item) => item.downsampled) ? ` from ${rawPoints}` : "";
}
