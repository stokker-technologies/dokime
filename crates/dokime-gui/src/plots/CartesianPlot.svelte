<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  /* === External Imports === */

  import { type Snippet, untrack } from "svelte";
  import type uPlot from "uplot";

  /* === Local Imports === */

  import type { ValueFormat } from "@generated/host/model";
  import { markerColor, plotColor, resolvePlotCssColor, resolvePlotThemeColors } from "@/plots/colors";
  import { attachPan, type HeldViewport, heldViewport, PLOT_DRAG_OPTIONS } from "@/plots/viewport";
  import ErrorCard from "../components/ErrorCard.svelte";
  import Button from "../components/ui/Button.svelte";
  import { formatNumber, formatTime, formatUnitLabel, type TimeFormatOrigin } from "../lib/format";
  import type { NotificationFailure } from "../lib/notification-details";
  import { failureOf } from "../lib/notification-details";
  import type { Wire } from "../lib/rpc/canonical";
  import { nearestDrawnPoint, type SeriesSummary, seriesLegendColumns, seriesLegendRows, toggleTrace } from "../lib/statistics";
  import type {
    DocumentAxisScale,
    DocumentPlotStyle,
    DocumentReferenceBand,
    DocumentReferenceLine,
    PlotCartesianFrame,
    PlotMarker,
    PlotSeries,
  } from "../lib/types";
  import { isNumber } from "../lib/utils";
  import { formatBooleanState, formatCursorX, formatElapsedTick } from "./axis";
  import {
    applyFrame,
    chartFrameFromCartesian,
    type IntervalRange,
    intervalRangeData,
    limitRenderedSeries,
    linearXRange,
    liveCartesianXRange,
    liveXWindow,
    paddedYRange,
    referenceYValues,
    usableCartesianFrame,
  } from "./cartesian-frame";
  import { drawMarkerKinds, type MarkerTooltip, markerKinds, markerTooltip } from "./markers";
  import PlotPlaceholder from "./PlotPlaceholder.svelte";
  import { getPlotInteraction } from "./plot-interaction";
  import { plotPlaceholderText, plotUnavailableText } from "./plot-placeholder";
  import type { PromptMarker } from "./prompt-markers";
  import { m4Indices } from "./reduce";
  import StatsSummaryTable from "./StatsSummaryTable.svelte";
  import type { PlotViewportMode } from "./types";

  /* === Constants === */

  const MIN_CHART_WIDTH = 160;
  const CHART_HEIGHT = 230;
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
  const POINT_MARK_RENDER_LIMIT = 600;
  /* Dense marks thin from the density at which uPlot's line path decimates: four per device pixel column. */
  const DECIMATION_DENSITY = 4;
  /* Point mark diameters in CSS pixels: a lone sample must read at 1x DPR on
     either theme, and dense captures keep smaller marks. */
  const DENSE_POINT_MARK_SIZE = 5;
  const DEFAULT_POINT_MARK_SIZE = 8;
  const POINT_BACKGROUND_OUTLINE_WIDTH = 1;
  const SERIES_LINE_WIDTH = 2;
  // Y padding as a share of the spanned values: a range widened by references or
  // interval bounds gets more room, so a band containing every sample keeps both
  // edges clear of the plot frame.
  const REFERENCE_BAND_FILL_ALPHA = 0.2;
  /* The hover mark grows by this many CSS pixels over the drawn mark. */
  const CURSOR_POINT_GROWTH = 2;
  const EMPTY_TRACES: ReadonlySet<string> = new Set();
  const NO_MARKERS: PlotMarker[] = [];
  const NO_PROMPTS: PromptMarker[] = [];

  /* === Types === */

  export type CartesianPlotProps = {
    plotId?: string;
    title: string;
    /** Hide a repeated title only when it has no unit to present beside it. */
    titleRepeated?: boolean;
    cartesian?: PlotCartesianFrame | null;
    description?: string | null;
    series: PlotSeries[];
    markers?: PlotMarker[];
    /** Prompt lifecycle moments, toggled by kind in the plot legend. */
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
    xFormat?: Wire<ValueFormat> | null;
    xLabel?: string;
    intervals?: boolean;
    currentPosition?: boolean;
    localViewport?: boolean;
    /**
     * The unit of an elapsed x axis, set when the plot's declared x column names
     * a duration. Its values are the duration itself, so the axis ticks as
     * numbers with this unit and never as a date.
     */
    xElapsedUnit?: string | null;
    yFormat?: Wire<ValueFormat> | null;
    runStartedAt?: number | null;
    cycleStartedAt?: number | null;
    resetKey?: string;
    /** Per-plot controls the host places beside the follow button. */
    actions?: Snippet;
    /** Controls inside the card directly under the chart, such as playback. */
    controls?: Snippet;
    badge?: string;
    stateLabel?: string;
    stateDetail?: string;
    contextSummary?: string;
    windowS?: number | null;
    viewportMode?: PlotViewportMode;
    /**
     * Replaces the canvas legend with the trace statistics table: one clickable
     * row per trace with the backend's statistics of the shown window.
     */
    legend?: { summaries: SeriesSummary[]; pending: boolean } | null;
  };

  /** Where the pointer sits on x, and what the visible window spans. */
  type PlotCursor = {
    x: number;
    span: number;
    widthPx: number;
  };

  /** A series' drawn point marks, in uPlot's terms: the outline is inset within `size`. */
  export type PointMarks = {
    show: boolean;
    filter?: uPlot.Series.Points.Filter;
    size: number;
    width: number;
    stroke: string;
    fill: string;
  };

  type UPlotConstructor = typeof uPlot;

  /* === State === */

  let uPlotPromise: Promise<UPlotConstructor> | null = null;

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

  /* === Axis Gutter === */

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

  /** The right padding the last of the x tick `values` needs to stay inside the card. */
  function xAxisRightPad(values: readonly string[], font: string): number {
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

  /**
   * The one colour of rendered series `index` for its line, point marks and legend swatch: its
   * declared colour, else the plot palette at the series index, resolved for canvas drawing.
   */
  export function seriesPaletteColor(
    colors: readonly string[],
    palette: readonly string[],
    index: number,
    element?: Element | null,
  ): string {
    return resolvePlotCssColor(colors[index] ?? plotColor(index, palette), element, palette[0]);
  }

  /** Every point mark is a disc in the series colour with a plot-background outline for contrast. */
  export function seriesPointMarks(style: DocumentPlotStyle, densePoints: boolean, color: string, background: string): PointMarks {
    const marks = {
      size: densePoints ? DENSE_POINT_MARK_SIZE : DEFAULT_POINT_MARK_SIZE,
      width: POINT_BACKGROUND_OUTLINE_WIDTH,
      stroke: background,
      fill: color,
    };
    if (style === "points") {
      return { show: true, filter: densePointIndices, ...marks };
    }
    return { show: style === "line_points" && !densePoints, filter: isolatedAlignedPoints, ...marks };
  }

  /** A gap leaves a lone sample without a segment, even when ordinary line marks are off. */
  function isolatedAlignedPoints(plot: uPlot, seriesIdx: number, show: boolean): number[] | null {
    if (show) {
      return null;
    }
    const values = plot.data[seriesIdx] as Array<number | null | undefined>;
    const series = plot.series[seriesIdx];
    const [from, to] = series.idxs ?? [0, values.length - 1];
    const isolated: number[] = [];
    if (series.spanGaps) {
      let single = -1;
      for (let index = 0; index < values.length; index += 1) {
        if (!isNumber(values[index])) {
          continue;
        }
        if (single >= 0) {
          return null;
        }
        single = index;
      }
      if (single >= from && single <= to) {
        isolated.push(single);
      }
    } else {
      for (let index = from; index <= to; index += 1) {
        if (isNumber(values[index]) && !isNumber(values[index - 1]) && !isNumber(values[index + 1])) {
          isolated.push(index);
        }
      }
    }
    return isolated.length ? isolated : null;
  }

  /**
   * A frame the host reduced, drawn with spanned gaps, keeps each device pixel
   * column's first, lowest, highest and last mark once it holds as many marks
   * as uPlot's line path decimates; a full frame keeps every mark.
   */
  function densePointIndices(plot: uPlot, seriesIdx: number, show: boolean): number[] | null {
    const series = plot.series[seriesIdx];
    if (!show || !series.spanGaps) {
      return null;
    }
    const [from, to] = series.idxs ?? [0, -1];
    const columns = Math.ceil(plot.bbox.width);
    if (to - from < DECIMATION_DENSITY * columns) {
      return null;
    }
    return m4Indices(plot.data[0] as number[], plot.data[seriesIdx] as Array<number | null>, from, to + 1, columns);
  }

  /** The hover mark is the series' point mark, a little larger. */
  export function cursorPointOptions(marks: readonly PointMarks[]): uPlot.Cursor.Points {
    // uPlot numbers data series from 1; marks follow the rendered series from 0.
    const mark = (seriesIdx: number) => marks[seriesIdx - 1] ?? marks[0];
    return {
      size: (_plot, seriesIdx) => (mark(seriesIdx)?.size ?? DEFAULT_POINT_MARK_SIZE) + CURSOR_POINT_GROWTH,
      width: POINT_BACKGROUND_OUTLINE_WIDTH,
      stroke: (_plot, seriesIdx) => mark(seriesIdx)?.stroke ?? "",
      fill: (_plot, seriesIdx) => mark(seriesIdx)?.fill ?? "",
    };
  }

  export function facetedSeriesPaths(
    style: DocumentPlotStyle,
    marks: PointMarks,
    color: string,
    currentPosition = false,
  ): uPlot.Series.PathBuilder {
    return (plot, seriesIdx) => {
      const data = facetedSeriesData(plot, seriesIdx);
      if (!data) {
        return null;
      }

      const [xValues, yValues] = data;
      const { ctx, bbox } = plot;
      const pixelRatio = plotPixelRatio();
      // uPlot insets the outline within the mark's size; faceted marks match its aligned marks.
      const radius = ((marks.size - marks.width) * pixelRatio) / 2;
      const line = style === "points" ? null : new Path2D();
      const points = new Path2D();
      let lineStarted = false;
      let previousTop: number | null = null;
      let position: [number, number] | null = null;

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
        if (!Number.isFinite(left) || !Number.isFinite(top)) {
          lineStarted = false;
          previousTop = null;
          continue;
        }
        position = [left, top];
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
        const previous = isNumber(xValues[index - 1]) && isNumber(yValues[index - 1]);
        const next = isNumber(xValues[index + 1]) && isNumber(yValues[index + 1]);
        if (marks.show || (!previous && !next)) {
          points.moveTo(left + radius, top);
          points.arc(left, top, radius, 0, Math.PI * 2);
        }
      }

      if (line) {
        ctx.strokeStyle = color;
        ctx.lineWidth = SERIES_LINE_WIDTH * pixelRatio;
        ctx.stroke(line);
      }
      ctx.fillStyle = marks.fill;
      ctx.fill(points);
      ctx.strokeStyle = marks.stroke;
      ctx.lineWidth = marks.width * pixelRatio;
      ctx.stroke(points);
      if (currentPosition && position) {
        ctx.beginPath();
        ctx.arc(position[0], position[1], radius * 1.8, 0, Math.PI * 2);
        ctx.fillStyle = color;
        ctx.fill();
        ctx.stroke();
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
      const top = Math.round(plot.valToPos(line.value, "y", true)) + 0.5;
      if (top < bbox.top || top > bottom) {
        return;
      }
      ctx.moveTo(bbox.left, top);
      ctx.lineTo(right, top);
      ctx.stroke();
    });
    ctx.restore();
  }

  export function drawReferenceBands(plot: uPlot, bands: DocumentReferenceBand[], element: Element, fallbackColor: string) {
    const { ctx, bbox } = plot;
    const bottomEdge = bbox.top + bbox.height;
    ctx.save();
    bands.forEach((band) => {
      if (!isNumber(band.min) || !isNumber(band.max)) {
        return;
      }
      const color = resolvePlotCssColor(band.color, element, fallbackColor);
      const top = plot.valToPos(band.max, "y", true);
      const bottom = plot.valToPos(band.min, "y", true);
      const clampedTop = clamp(top, bbox.top, bottomEdge);
      const clampedBottom = clamp(bottom, bbox.top, bottomEdge);
      const alpha = ctx.globalAlpha;
      ctx.fillStyle = color;
      ctx.globalAlpha = alpha * REFERENCE_BAND_FILL_ALPHA;
      ctx.fillRect(bbox.left, clampedTop, bbox.width, clampedBottom - clampedTop);
      ctx.globalAlpha = alpha;
      // Solid edges keep a band legible when it spans most of the plot.
      ctx.strokeStyle = color;
      ctx.lineWidth = 1;
      ctx.beginPath();
      for (const edge of [top, bottom]) {
        if (edge < bbox.top || edge > bottomEdge) {
          continue;
        }
        const y = Math.round(edge) + 0.5;
        ctx.moveTo(bbox.left, y);
        ctx.lineTo(bbox.left + bbox.width, y);
      }
      ctx.stroke();
    });
    ctx.restore();
  }

  function clamp(value: number, min: number, max: number): number {
    return Math.min(max, Math.max(min, value));
  }

  function downsampledLabel(series: PlotSeries[]): string {
    const rawPoints = Math.max(0, ...series.map((item) => item.raw_points));
    return series.some((item) => item.downsampled) ? ` from ${rawPoints}` : "";
  }
</script>

<script lang="ts">
  let {
    plotId,
    title,
    titleRepeated = false,
    cartesian,
    description,
    series,
    markers = NO_MARKERS,
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
    controls,
    badge,
    stateLabel,
    stateDetail,
    contextSummary,
    windowS = null,
    viewportMode = "fit",
    legend = null,
    currentPosition = false,
    localViewport = false,
  }: CartesianPlotProps = $props();

  const interaction = getPlotInteraction();
  const zoom = $derived(interaction.zoom);
  const syncKey = $derived(interaction.syncKey);

  let hiddenTraces = $state.raw<ReadonlySet<string>>(EMPTY_TRACES);

  let traceColors = $state.raw<string[]>([]);

  let cursor = $state.raw<PlotCursor | null>(null);

  let cursorFrame: number | null = null;

  const setSummaryWindow = $derived(interaction.measurements?.setSummaryWindow);

  /* Props resolve through the parent's per-revision spec, so effects read
     values derived from them: an equal value re-runs no effect. */
  const summaryPlotId = $derived(!localViewport && !zoom && setSummaryWindow ? plotId : undefined);

  /** Reports the x range a zoom holds, or `null` once the plot follows again. */
  function updateViewport(window: { from: number; to: number } | null) {
    if (summaryPlotId) {
      setSummaryWindow?.(summaryPlotId, window);
    }
  }

  $effect(() => {
    const id = summaryPlotId;
    const reset = setSummaryWindow;
    return id && reset ? () => reset(id, null) : undefined;
  });

  const kinds = $derived(markerKinds(markers, promptMarkers));

  let markerColors: string[] = [];

  let pointMarks: PointMarks[] = [];

  const tableLegend = $derived(legend !== null);

  let following = $state.raw(true);

  let heldRange: HeldViewport | null = null;

  let rootElement: HTMLDivElement | undefined = $state();

  let instance: uPlot | null = null;

  let hoveredMarker = $state.raw<MarkerTooltip | null>(null);

  let lastDataSignature: string | null = null;

  let lastResetKey: string | undefined = untrack(() => resetKey);

  let lastViewportMode: PlotViewportMode = untrack(() => viewportMode);

  /* The chart's border-box width in CSS pixels, as the resize observer last reported it. */
  let chartWidth = MIN_CHART_WIDTH;

  let resizeFrame: number | null = null;

  let redrawFrame: number | null = null;

  /* Whether the queued overlay redraw also re-ranges y for changed references or intervals. */
  let rerangePending = false;

  let plotConstructor = $state.raw<UPlotConstructor | null>(null);

  let plotLoadError = $state.raw<NotificationFailure | null>(null);

  let laidOut = $state.raw(false);

  const renderedSeries = $derived(limitRenderedSeries(series, cartesian));

  const backendCartesian = $derived(usableCartesianFrame(cartesian, renderedSeries, series));

  const chartFrame = $derived(backendCartesian ? chartFrameFromCartesian(backendCartesian, xScale, yScale) : null);

  const seriesKey = $derived(renderedSeries.map((item) => `${series.indexOf(item)}:${item.key}:${item.label}`).join("|"));

  const colorsKey = $derived(colors.join("|"));

  const formatsKey = $derived(
    JSON.stringify({
      x: xFormat ?? null,
      xLabel,
      intervals,
      booleanAxis,
      xElapsed: xElapsedUnit ?? null,
      y: yFormat ?? null,
      series: renderedSeries.map((item) => item.format ?? null),
      runStartedAt: runStartedAt ?? null,
      cycleStartedAt: cycleStartedAt ?? null,
    }),
  );

  const intervalRanges = $derived(intervalRangeData(renderedSeries, { xScale, yScale }));

  const yReferenceValues = $derived(referenceYValues(referenceLines, referenceBands, yScale));

  /* Overlay inputs by value: live revisions rebuild these arrays without changing them. */
  const overlayKey = $derived(JSON.stringify([kinds, referenceLines, referenceBands]));

  /* The values that widen the y range beyond the data. */
  const rangeKey = $derived(JSON.stringify([yReferenceValues, intervalRanges]));

  let lastRangeKey = untrack(() => rangeKey);

  const hasData = $derived(Boolean(chartFrame && chartFrame.pointCount > 0));

  const hasTimeline = $derived(
    renderedSeries.some((item) => item.points.length > 0) || Boolean(backendCartesian?.series.some((item) => item.x.length > 0)),
  );

  const frameError = $derived(
    (cartesian != null || hasTimeline) && !backendCartesian ? "The plot data could not be rendered. Reload the plot to try again." : null,
  );

  const displayError = $derived(
    plotLoadError ?? (frameError ? failureOf(frameError, "Plot Unavailable", ["plot-frame", title, resetKey, frameError].join(":")) : null),
  );

  const chartMode = $derived(chartFrame?.mode ?? null);

  const dataSignature = $derived(chartFrame?.signature ?? null);

  const renderedPointCount = $derived(chartFrame?.pointCount ?? 0);

  const densePoints = $derived(renderedPointCount > POINT_MARK_RENDER_LIMIT);

  const liveWindow = $derived(
    viewportMode === "live" ? liveXWindow(liveCartesianXRange(backendCartesian, windowS), xScale, timeAxis) : null,
  );

  /*
   * What the chart is built from: series set, colours, scales, style, mode,
   * declared formats and reset or viewport switches. Keyed by value, so a
   * revision never rebuilds the chart: a rebuild painted an auto-fitted x axis
   * before the live window returned, and tripped the browser's scroll
   * anchoring until the viewport drifted. Revisions reach the chart through
   * `applyFrame` in the data effect below, references through the y range and
   * point density through the series options it restyles in place.
   */
  const structureKey = $derived(
    JSON.stringify([
      hasData,
      laidOut,
      zoom,
      currentPosition,
      style,
      timeAxis,
      booleanAxis,
      xScale,
      yScale,
      viewportMode,
      resetKey,
      colorsKey,
      seriesKey,
      chartMode,
      formatsKey,
      tableLegend,
      syncKey,
    ]),
  );

  $effect(() => {
    let cancelled = false;
    loadUPlot()
      .then((loadedConstructor) => {
        if (cancelled) {
          return;
        }
        plotConstructor = loadedConstructor;
        plotLoadError = null;
      })
      .catch((error: unknown) => {
        if (cancelled) {
          return;
        }
        plotLoadError = failureOf(error, "Plot Unavailable");
      });
    return () => {
      cancelled = true;
    };
  });

  /* One observer sizes the chart from its border box: its first report lays the chart out, later ones resize it. */
  $effect(() => {
    void [rootElement, hasData];
    return untrack(() => {
      const root = rootElement;
      if (!hasData || !root) {
        laidOut = false;
        return;
      }
      const observer = new ResizeObserver((entries) => {
        const width = entries.at(-1)?.borderBoxSize[0]?.inlineSize ?? 0;
        chartWidth = Math.max(MIN_CHART_WIDTH, Math.floor(width));
        laidOut = width > 0;
        scheduleResize();
      });
      observer.observe(root);
      return () => observer.disconnect();
    });
  });

  /* A burst of reports resizes the chart once, in the next frame; uPlot redraws its paths at the new size. */
  function scheduleResize() {
    if (resizeFrame !== null) {
      return;
    }
    resizeFrame = window.requestAnimationFrame(() => {
      resizeFrame = null;
      const plot = instance;
      if (!plot || plot.width === chartWidth) {
        return;
      }
      hoveredMarker = null;
      try {
        plot.setSize({ width: chartWidth, height: CHART_HEIGHT });
      } catch (error) {
        plotLoadError = failureOf(error, "Plot Unavailable");
      }
    });
  }

  $effect(() => {
    void [rootElement, plotConstructor, structureKey];
    return untrack(() => {
      const CartesianChart = plotConstructor;
      if (!hasData || !laidOut || !rootElement || !chartFrame || !CartesianChart) {
        return;
      }
      const root = rootElement;
      const themeColors = resolvePlotThemeColors(root);
      markerColors = kinds.map((_, index) => markerColor(themeColors.references, index));
      const plotFont = resolvePlotFont(root);
      const resolvedSeriesColors = renderedSeries.map((item) =>
        seriesPaletteColor(colors, themeColors.palette, series.indexOf(item), root),
      );
      const fallbackSeriesColor = seriesPaletteColor([], themeColors.palette, 0, root);
      const faceted = chartFrame.mode === "faceted";
      const seriesColor = (index: number) => resolvedSeriesColors[index] ?? fallbackSeriesColor;
      const marks = renderedSeries.map((_, index) => seriesPointMarks(style, densePoints, seriesColor(index), themeColors.background));
      pointMarks = marks;
      const lineWidth = style === "points" ? 0 : SERIES_LINE_WIDTH;
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
        (format: Wire<ValueFormat> | null): uPlot.Series.Value =>
        (_plot, rawValue) =>
          booleanAxis ? formatBooleanState(rawValue) : rawValue == null ? "--" : formatNumber(rawValue, format);
      // The x axis size callback receives the tick labels the right padding is sized from.
      let xTicks: string[] = [];
      const options: uPlot.Options = {
        width: chartWidth,
        height: CHART_HEIGHT,
        mode: faceted ? 2 : 1,
        hooks: {
          setSelect: [
            (plot) => {
              const held = heldViewport(plot.select, { width: plot.over.clientWidth, height: plot.over.clientHeight }, (position, axis) =>
                plot.posToVal(position, axis),
              );
              if (!held) {
                return;
              }
              heldRange = held;
              following = false;
              // A y-only zoom returns x to its live follow, so the statistics
              // return to the window too.
              updateViewport(held.x ? { from: held.x.min, to: held.x.max } : null);
            },
          ],
          setCursor: [
            (plot) => {
              const left = plot.cursor.left;
              const nextTooltip = markerTooltip(plot, kinds, root.getBoundingClientRect());
              const current = hoveredMarker;
              const sameTooltip =
                current?.key === nextTooltip?.key &&
                current?.label === nextTooltip?.label &&
                current?.left === nextTooltip?.left &&
                current?.width === nextTooltip?.width;
              if (!sameTooltip) {
                hoveredMarker = nextTooltip;
              }
              if (!tableLegend || left == null || left < 0) {
                return;
              }
              const next = {
                x: plot.posToVal(left, "x"),
                span: (plot.scales.x.max ?? 0) - (plot.scales.x.min ?? 0),
                widthPx: plot.over.clientWidth,
              };
              if (cursorFrame !== null) {
                window.cancelAnimationFrame(cursorFrame);
              }
              cursorFrame = window.requestAnimationFrame(() => {
                cursorFrame = null;
                cursor = next;
              });
            },
          ],
          drawClear: [
            (plot) => {
              drawReferenceBands(plot, referenceBands, root, themeColors.foreground);
            },
          ],
          draw: [
            (plot) => {
              drawIntervalRanges(plot, intervalRanges, resolvedSeriesColors[0] ?? fallbackSeriesColor);
              drawMarkerKinds(plot, kinds, markerColors);
              drawReferenceLines(plot, referenceLines, root, themeColors.foreground);
            },
          ],
        },
        // A dragged box is drawn as the selection and becomes a held viewport
        // while data continues arriving; a mostly horizontal or vertical drag
        // zooms that axis alone, a double click returns to the full view.
        cursor: {
          bind: {
            /* A double click drops the hold and follows the run again, overriding uPlot's own auto-scale before it is painted. */
            dblclick: (_plot, target, handler) => (event) => {
              if (event.button === 0 && event.target === target) {
                handler(event);
                follow();
              }
              return null;
            },
          },
          drag: { ...PLOT_DRAG_OPTIONS },
          focus: { prox: -1 },
          points: cursorPointOptions(marks),
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
          x: {
            time: timeAxis && !xElapsedUnit,
            distr: xScale === "log" ? 3 : 1,
            range: currentPosition
              ? (_plot, min, max) => paddedYRange(min, max, [], xScale)
              : xScale === "linear"
                ? (_plot, min, max) => linearXRange(min, max)
                : undefined,
          },
          y: {
            auto: true,
            distr: yScale === "log" ? 3 : 1,
            /* References and interval bounds can arrive mid-run, so the range always reads them; without any, uPlot's own range applies. */
            range: booleanAxis
              ? () => [-0.1, 1.1]
              : (_plot, dataMin, dataMax) => {
                  const widening = [...yReferenceValues, ...intervalRanges.flatMap((range) => [range.low, range.high])];
                  if (widening.length || intervals || currentPosition) {
                    return paddedYRange(dataMin, dataMax, widening, yScale);
                  }
                  if (!Number.isFinite(dataMin) || !Number.isFinite(dataMax)) {
                    return [null, null];
                  }
                  return yScale === "log"
                    ? CartesianChart.rangeLog(dataMin, dataMax, 10, false)
                    : CartesianChart.rangeNum(dataMin, dataMax, 0.1, true);
                },
          },
        },
        /* The last tick label is centred on the plot's right edge, so half of it
           hangs outside the plotting area; uPlot's own 25px is not enough for a
           time label at touch density, where the axis font is scaled up. */
        padding: [null, () => xAxisRightPad(xTicks, plotFont), null, null],
        axes: [
          {
            label: xLabel,
            stroke: themeColors.mutedForeground,
            font: plotFont,
            labelFont: plotFont,
            // Subsecond Clock Labels Must Fit At The Selected Density.
            ...(timeAxis && !xElapsedUnit ? { space: measureTextWidth("00:00:00.000", plotFont) + 12 } : {}),
            grid: { stroke: themeColors.border, width: 1 },
            size: (_plot, values) => {
              xTicks = values ?? [];
              return xAxisGutter(values, plotFont);
            },
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
            stroke: seriesColor(index),
            width: lineWidth,
            // uPlot copies point options once; visibility reads the marks each revision restyles.
            points: faceted ? { show: false } : { ...marks[index], show: () => marks[index].show },
            paths: faceted
              ? facetedSeriesPaths(style, marks[index], seriesColor(index), currentPosition)
              : style === "step"
                ? CartesianChart.paths.stepped?.({ align: 1 })
                : undefined,
          })),
        ] as uPlot.Series[],
      };
      // Keep Chart Initialization Failures Local To This Card.
      let plot: uPlot;
      try {
        plot = new CartesianChart(options, chartFrame.data, root);
      } catch (error) {
        plotLoadError = failureOf(error, "Plot Unavailable");
        return;
      }
      if (lastResetKey !== resetKey || lastViewportMode !== viewportMode) {
        heldRange = null;
        following = true;
        updateViewport(null);
      }
      // uPlot draws in a microtask, so even its first draw shows the held or live window.
      applyFrame(plot, chartFrame.data, heldRange, liveWindow);
      renderedSeries.forEach((item, index) => {
        if (hiddenTraces.has(item.key)) {
          plot.setSeries(index + 1, { show: false });
        }
      });
      if (traceColors.join("|") !== resolvedSeriesColors.join("|")) {
        traceColors = resolvedSeriesColors;
      }
      instance = plot;
      lastDataSignature = dataSignature;
      lastResetKey = resetKey;
      lastViewportMode = viewportMode;
      cancelRedraw();

      const detachPan = attachPan(plot, {
        logX: xScale === "log",
        logY: yScale === "log",
        panY: !timeAxis,
        onPan: (next) => {
          const held = { x: next.x, y: next.y ?? heldRange?.y ?? null };
          // uPlot re-ranges an auto y with every x change, so an x pan pins a held y in the same commit.
          if (!next.y && held.y) {
            plot.setScale("y", held.y);
          }
          heldRange = held;
          following = false;
          updateViewport(held.x ? { from: held.x.min, to: held.x.max } : null);
        },
      });
      return () => {
        detachPan();
        if (resizeFrame !== null) {
          window.cancelAnimationFrame(resizeFrame);
          resizeFrame = null;
        }
        cancelRedraw();
        if (cursorFrame !== null) {
          window.cancelAnimationFrame(cursorFrame);
          cursorFrame = null;
        }
        plot.destroy();
        instance = null;
        lastDataSignature = null;
      };
    });
  });

  $effect(() => {
    void [hiddenTraces, renderedSeries];
    return untrack(() => {
      const plot = instance;
      if (!plot) {
        return;
      }
      renderedSeries.forEach((item, index) => {
        const show = !hiddenTraces.has(item.key);
        if (plot.series[index + 1]?.show !== show) {
          plot.setSeries(index + 1, { show });
        }
      });
    });
  });

  const legendRows = $derived.by(() => {
    if (!tableLegend) {
      return null;
    }
    const frame = backendCartesian;
    const cursorPosition = cursor;
    const cursorPoints = cursorPosition && frame ? frame.series.map((item) => nearestDrawnPoint(item.x, item.y, cursorPosition.x)) : null;
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
      summaries: legend?.summaries ?? [],
      traces: renderedSeries.map((item, index) => ({
        key: item.key,
        label: item.label,
        color: traceColors[index] ?? colors[series.indexOf(item)] ?? "currentColor",
      })),
    });
  });

  const legendColumns = $derived(seriesLegendColumns(unit));

  /* Changed overlays redraw over the drawn paths; changed references or intervals also re-range y unless it is held. */
  $effect(() => {
    void [rootElement, overlayKey, rangeKey];
    untrack(() => {
      const root = rootElement;
      if (root) {
        const theme = resolvePlotThemeColors(root);
        markerColors = kinds.map((_, index) => markerColor(theme.references, index));
      }
      rerangePending ||= rangeKey !== lastRangeKey;
      lastRangeKey = rangeKey;
      if (redrawFrame !== null) {
        return;
      }
      redrawFrame = window.requestAnimationFrame(() => {
        redrawFrame = null;
        const rerange = rerangePending && !heldRange?.y;
        rerangePending = false;
        const plot = instance;
        if (plot?.status === 1) {
          plot.redraw(rerange, false);
        }
      });
    });
  });

  // Live revisions already arrive one commit per animation frame; each is applied once, with its window.
  $effect(() => {
    void dataSignature;
    untrack(() => {
      const plot = instance;
      if (!plot || !chartFrame || lastDataSignature === dataSignature) {
        return;
      }
      // The series restyle in place: a reduced frame spans its gaps and a dense one draws smaller marks.
      renderedSeries.forEach((item, index) => {
        const options = plot.series[index + 1];
        const marks = pointMarks[index];
        Object.assign(marks, seriesPointMarks(style, densePoints, marks.fill, marks.stroke));
        options.spanGaps = item.downsampled === true;
        if (options.points) {
          options.points.size = marks.size;
        }
      });
      try {
        applyFrame(plot, chartFrame.data, heldRange, liveWindow);
      } catch (error) {
        plotLoadError = failureOf(error, "Plot Unavailable");
        return;
      }
      lastDataSignature = dataSignature;
      cancelRedraw();
    });
  });

  /* A commit that ranges y and draws the overlays itself makes a queued overlay redraw redundant. */
  function cancelRedraw() {
    if (redrawFrame !== null) {
      window.cancelAnimationFrame(redrawFrame);
      redrawFrame = null;
    }
    rerangePending = false;
  }

  const placeholder = $derived(
    displayError
      ? plotUnavailableText(displayError.message)
      : !hasData
        ? hasTimeline
          ? plotPlaceholderText(
              { stateLabel: stateLabel ?? "Unavailable", stateDetail },
              "The stream is active, but this signal is unavailable in the selected window.",
            )
          : plotPlaceholderText({ stateLabel, stateDetail })
        : null,
  );
  function follow() {
    heldRange = null;
    following = true;
    updateViewport(null);
    const plot = instance;
    if (plot) {
      applyFrame(plot, plot.data, null, liveWindow);
    }
  }
</script>

<article class="plot-card grid min-w-0 max-w-full overflow-hidden gap-2 rounded-lg border bg-card p-4 text-card-foreground">
  <div class="flex items-center justify-between gap-3">
    <strong class={!placeholder && titleRepeated && !unit ? "sr-only" : "min-w-0 truncate"} title={formatUnitLabel(title, unit)}>
      {formatUnitLabel(title, unit)}
    </strong>
    <div class="flex items-center gap-2">
      {@render actions?.()}
      {#if !placeholder}
        <Button variant="outline" size="xs" type="button" aria-pressed={following} onclick={follow}>
          {zoom ? "Reset Zoom" : "Follow"}
        </Button>
        <span class="text-xs text-muted-foreground">
          {renderedPointCount}
          Points{downsampledLabel(renderedSeries)}
          {series.length > renderedSeries.length ? ` - ${renderedSeries.length} of ${series.length} series` : ""}
          {xScale === "log" || yScale === "log" ? " - Log" : ""}
          {badge ? ` - ${badge}` : ""}
        </span>
      {/if}
    </div>
  </div>
  {#if displayError}
    <ErrorCard failure={displayError} />
  {:else if placeholder}
    <PlotPlaceholder {...placeholder} />
  {:else}
    <div class="relative min-w-0 max-w-full">
      <div
        bind:this={rootElement}
        class="min-h-60 min-w-0 max-w-full overflow-hidden [&_.uplot]:max-w-full [&_.uplot]:overflow-hidden"
      ></div>
      {#if hoveredMarker}
        <div
          class="pointer-events-none absolute top-1 z-10 box-border line-clamp-3 wrap-anywhere rounded-lg border bg-popover px-2 py-1 text-xs text-popover-foreground shadow-sm"
          role="tooltip"
          data-marker-kind={hoveredMarker.key}
          style:left={`${hoveredMarker.left}px`}
          style:width={`${hoveredMarker.width}px`}
        >
          {hoveredMarker.label}
        </div>
      {/if}
    </div>
  {/if}
  {@render controls?.()}
  {#if description}
    <p class="text-xs text-muted-foreground">{description}</p>
  {/if}
  {#if contextSummary}
    <p class="text-xs text-muted-foreground">{contextSummary}</p>
  {/if}
  {#if !placeholder && legendRows}
    <StatsSummaryTable
      legend={{
        columns: legendColumns,
        rows: legendRows,
        pending: legend?.pending ?? false,
        onToggle: (key) => {
          hiddenTraces = toggleTrace(hiddenTraces, key);
        },
      }}
    />
  {/if}
</article>
