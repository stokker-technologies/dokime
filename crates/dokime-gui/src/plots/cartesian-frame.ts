/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import type uPlot from "uplot";

/* === Local Imports === */

import { xValue } from "./axis";
import type { DocumentAxisScale, DocumentReferenceBand, DocumentReferenceLine, PlotCartesianFrame, PlotSeries } from "../lib/types";
import { isNumber } from "../lib/utils";
import { windowIndices } from "./reduce";
import type { HeldViewport } from "./viewport";

/* === Constants === */

const MAX_RENDERED_SERIES = 48;
const DATA_RANGE_PADDING = 0.08;
const REFERENCE_RANGE_PADDING = 0.15;

/* === Types === */

export type IntervalRange = {
  x: number;
  low: number;
  high: number;
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
export type DrawnWindow = {
  source: uPlot.AlignedData | null;
  aligned: boolean;
  preserveSamples: boolean;
  range: PlotViewportRange | null;
  everySample: boolean;
};

/* === Functions === */

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

export function chartFrameFromCartesian(cartesian: PlotCartesianFrame, xScale: DocumentAxisScale, yScale: DocumentAxisScale): ChartFrame {
  let pointCount = 0;
  for (const series of cartesian.series) {
    for (let index = 0; index < series.x.length; index += 1) {
      const x = series.x[index];
      const y = series.y[index];
      const finitePair = Number.isFinite(x) && isNumber(y);
      const withinScales = (xScale !== "log" || x > 0) && (yScale !== "log" || (y !== null && y > 0));
      if (finitePair && withinScales) {
        pointCount += 1;
      }
    }
  }
  return {
    data:
      cartesian.mode === "faceted"
        ? ([null, ...cartesian.series.map((series) => [series.x, series.y])] as unknown as uPlot.AlignedData)
        : ([cartesian.series[0]?.x ?? [], ...cartesian.series.map((series) => series.y)] as uPlot.AlignedData),
    mode: cartesian.mode,
    pointCount,
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
      if (!Number.isFinite(value)) {
        continue;
      }
      if (value < earliest) {
        earliest = value;
      }
      if (value > latest) {
        latest = value;
      }
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
export function setLiveXScale(plot: uPlot, range: PlotViewportRange | null, xScale: DocumentAxisScale, timeAxis: boolean): void {
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
export function applyHeldViewport(plot: uPlot, held: HeldViewport, data: uPlot.AlignedData, followX: () => void): void {
  plot.setData(data, true);
  if (held.x) {
    plot.setScale("x", held.x);
  } else {
    followX();
  }
  if (held.y) {
    plot.setScale("y", held.y);
  }
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
export function drawnData(drawn: DrawnWindow, range: PlotViewportRange | null, columns: number, fitPixels: boolean): uPlot.AlignedData {
  const source = drawn.source ?? ([[]] as unknown as uPlot.AlignedData);
  drawn.range = range;
  if (!drawn.aligned || drawn.preserveSamples) {
    drawn.everySample = true;
    return source;
  }

  const [x, ...traces] = source as unknown as [number[], ...(number | null | undefined)[][]];
  const indices = windowIndices(x, traces, range, columns, fitPixels);
  drawn.everySample = indices === null;
  if (!indices) {
    return source;
  }

  return [indices.map((index) => x[index]), ...traces.map((trace) => indices.map((index) => trace[index] ?? null))] as uPlot.AlignedData;
}

/** Whether what is drawn already resolves the visible x `range`. */
export function windowServes(drawn: DrawnWindow, range: PlotViewportRange): boolean {
  if (drawn.everySample) {
    return true;
  }
  const drawnRange = drawn.range;
  if (drawnRange) {
    const tolerance = Math.max(Math.abs(range.max - range.min), Number.MIN_VALUE) * 1e-9;
    return Math.abs(drawnRange.min - range.min) <= tolerance && Math.abs(drawnRange.max - range.max) <= tolerance;
  }
  // The whole-extent reduction serves only a view of the whole extent.
  const x = (drawn.source?.[0] ?? []) as number[];
  return x.length === 0 || (range.min <= x[0] && range.max >= x[x.length - 1]);
}

export function intervalRangeData(series: PlotSeries[], scales: { xScale: DocumentAxisScale; yScale: DocumentAxisScale }): IntervalRange[] {
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
export function referenceYValues(lines: DocumentReferenceLine[], bands: DocumentReferenceBand[], yScale: DocumentAxisScale): number[] {
  const values = [...lines.map((line) => line.value), ...bands.flatMap((band) => [band.min, band.max])].filter(isNumber);
  return yScale === "log" ? values.filter((value) => value > 0) : values;
}

export function paddedYRange(
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
  const pad = (max - min) * (referenceValues.length ? REFERENCE_RANGE_PADDING : DATA_RANGE_PADDING);
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
