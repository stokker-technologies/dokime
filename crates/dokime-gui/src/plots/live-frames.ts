/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * The retained frame of each live plot and how stream messages change it.
 *
 * A plot retains exactly one representation of its samples: the backend's
 * cartesian frame when the plot draws one, its series points otherwise. A
 * Snapshot or Replace frame becomes the whole state. An Append frame grows the
 * retained cartesian arrays in place, evicts samples older than the window from
 * the front, and compacts the arrays back to the display budget once they hold
 * twice that, so retention is bounded by the budget for every window,
 * including "All". The arrays of a retained frame are owned by this model:
 * earlier states share them and must not be read for data.
 */

/* === Local Imports === */

import { summaryInFlight } from "@/lib/statistics";
import type {
  PlotCartesianFrame,
  PlotMarker,
  PlotPayload,
  PlotPayloadCursor,
  PlotPayloadDiagnostics,
  PlotPayloadStreamMessage,
  PlotSeries,
} from "@/lib/types";
import { xValue } from "@/plots/axis";
import { DEFAULT_PLOT_MAX_POINTS, NATIVE_PLOT_MAX_POINTS, type ReduceTechnique, reduceIndices } from "@/plots/reduce";

/* === Constants === */

const MAX_RETAINED_MARKERS = 500;

/* === Types === */

export type LivePlotStatus = "idle" | "loading" | "live" | "stale" | "reconnecting" | "error";

export type LivePlotState = {
  /** The retained frame the renderer draws. */
  payload: PlotPayload | null;
  cursor: PlotPayloadCursor | null;
  diagnostics: PlotPayloadDiagnostics | null;
  /** The latest applied revision; older frames are counted as dropped. */
  revision: number;
  status: LivePlotStatus;
  error: string | null;
  droppedBatches: number;
  /** A window, zoom or plot change asked for summaries not yet delivered. */
  summaryInFlight: boolean;
};

export type LivePlotStateMap = Record<string, LivePlotState>;

type CartesianSeries = PlotCartesianFrame["series"][number];

/* === Public Functions === */

export function emptyLivePlotState(): LivePlotState {
  return {
    payload: null,
    cursor: null,
    diagnostics: null,
    revision: 0,
    status: "idle",
    error: null,
    droppedBatches: 0,
    summaryInFlight: false,
  };
}

/** One plot's state after a batch of its stream messages, in arrival order. */
export function applyPlotMessages(
  current: LivePlotState,
  messages: readonly PlotPayloadStreamMessage[],
  technique: ReduceTechnique,
): LivePlotState {
  let next = current;
  for (const message of messages) {
    // An error frame reports one plot's failure and carries no revision of its own.
    if (message.kind === "error") {
      next = {
        ...next,
        error: message.warnings[0] ?? "Plot stream returned an error.",
        status: "error",
        summaryInFlight: summaryInFlight("failed"),
      };
      continue;
    }
    // A snapshot opens its own origin: an empty plot's first one is revision 0.
    if (message.kind !== "snapshot" && message.revision <= next.revision) {
      next = { ...next, droppedBatches: next.droppedBatches + 1 };
      continue;
    }

    const payload =
      message.kind === "append" && next.payload
        ? appendFrame(next.payload, message.payload, message.revision, technique)
        : retainFrame(message.payload, technique);
    next = {
      payload,
      cursor: message.cursor,
      diagnostics: message.diagnostics ?? next.diagnostics,
      revision: message.revision,
      status: "live",
      error: null,
      droppedBatches: next.droppedBatches,
      summaryInFlight: summaryInFlight("delivered"),
    };
  }
  return next;
}

/**
 * A Snapshot or Replace frame as the whole retained state. Its series points
 * are dropped when it carries a cartesian frame, which is then the only copy.
 */
export function retainFrame(payload: PlotPayload, technique: ReduceTechnique): PlotPayload {
  const windowS = payload.window_s ?? null;
  const markers = trimMarkers(payload.markers ?? [], windowS);
  const cartesian = payload.cartesian;
  if (!cartesian) return { ...payload, markers };

  const series = cartesian.series.map((item) => ({ x: [...item.x], y: [...item.y] }));
  const frame = { ...cartesian, series };
  const compacted = compactFrame(frame, retainedLimit(payload.max_points), technique);
  return {
    ...payload,
    markers,
    series: payload.series.map((item) => ({
      ...item,
      points: [],
      downsampled: item.downsampled || compacted,
    })),
    cartesian: frame,
  };
}

/**
 * The retained frame grown by one Append frame. A frame whose structure
 * changed (mode, series set, or no cartesian frame) starts over from the
 * update, since its samples cannot line up with the retained ones.
 */
export function appendFrame(current: PlotPayload, update: PlotPayload, revision: number, technique: ReduceTechnique): PlotPayload {
  const base = current.cartesian;
  const delta = update.cartesian;
  if (!base || !delta || base.mode !== delta.mode || !sameSeries(current.series, update.series)) {
    return retainFrame(update, technique);
  }

  // Grow The Retained Arrays
  for (let index = 0; index < base.series.length; index += 1) {
    pushAll(base.series[index].x, delta.series[index].x);
    pushAll(base.series[index].y, delta.series[index].y);
  }
  const frame: PlotCartesianFrame = {
    ...base,
    series: base.series.map((item) => ({ x: item.x, y: item.y })),
    point_count: base.point_count + delta.point_count,
  };

  // Evict, Then Compact
  const windowS = update.window_s ?? current.window_s ?? null;
  evictBefore(frame, windowS);
  const maxPoints = update.max_points ?? current.max_points;
  const compacted = compactFrame(frame, retainedLimit(maxPoints), technique);
  frame.signature = frameSignature(frame, revision);

  return {
    ...current,
    generated_at: update.generated_at,
    analysis: update.analysis ?? current.analysis,
    // An append frame's summaries already cover the whole window.
    summaries: update.summaries ?? current.summaries,
    context: { ...(current.context ?? {}), ...(update.context ?? {}) },
    markers: trimMarkers([...(current.markers ?? []), ...(update.markers ?? [])], windowS),
    max_points: maxPoints,
    window_s: windowS,
    series: current.series.map((item, index) => ({
      ...item,
      raw_points: item.raw_points + update.series[index].raw_points,
      downsampled: item.downsampled || update.series[index].downsampled || compacted,
    })),
    cartesian: frame,
  };
}

/** Samples a series retains before compaction: the budget, or half the host ceiling. */
export function retainedLimit(maxPoints: number | null | undefined): number {
  if (typeof maxPoints !== "number" || !Number.isFinite(maxPoints)) return DEFAULT_PLOT_MAX_POINTS;
  if (maxPoints <= 0) return NATIVE_PLOT_MAX_POINTS / 2;
  return Math.min(Math.max(1, Math.trunc(maxPoints)), NATIVE_PLOT_MAX_POINTS);
}

/* === Private Functions === */

/**
 * Compacts each x axis that holds more than twice its capacity back to the
 * budget; the capacity of an aligned axis is the budget for each series on it.
 */
function compactFrame(frame: PlotCartesianFrame, limit: number, technique: ReduceTechnique) {
  let compacted = false;
  for (const group of axisGroups(frame)) {
    const x = group[0].x;
    if (x.length <= 2 * limit * group.length) continue;
    const kept = reduceIndices(
      x,
      group.map((item) => item.y),
      limit,
      technique,
    );
    if (!kept) continue;
    for (const item of group) {
      item.x = kept.map((index) => item.x[index]);
      item.y = kept.map((index) => item.y[index]);
    }
    compacted = true;
  }
  if (compacted) frame.point_count = countPoints(frame);
  return compacted;
}

/** Drops samples older than the window from the front of each x axis. */
function evictBefore(frame: PlotCartesianFrame, windowS: number | null) {
  if (windowS === null || !(windowS > 0)) return;
  let evicted = 0;
  for (const group of axisGroups(frame)) {
    const x = group[0].x;
    if (x.length < 2) continue;
    const cutoff = x[x.length - 1] - windowS;
    if (!(x[0] < cutoff)) continue;
    const start = Math.min(firstAtOrAfter(x, cutoff), x.length - 1);
    for (const item of group) {
      for (let index = 0; index < start; index += 1) if (item.y[index] != null) evicted += 1;
      item.x.splice(0, start);
      item.y.splice(0, start);
    }
  }
  frame.point_count -= evicted;
}

/** Series that share one x axis: all of them when aligned, each alone when faceted. */
function axisGroups(frame: PlotCartesianFrame): CartesianSeries[][] {
  if (!frame.series.length) return [];
  return frame.mode === "aligned" ? [frame.series] : frame.series.map((item) => [item]);
}

/** The first index whose x is at or after `cutoff`; x ascends. */
function firstAtOrAfter(x: readonly number[], cutoff: number): number {
  let low = 0;
  let high = x.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if (x[middle] < cutoff) low = middle + 1;
    else high = middle;
  }
  return low;
}

function countPoints(frame: PlotCartesianFrame): number {
  let count = 0;
  for (const item of frame.series) for (const value of item.y) if (value != null) count += 1;
  return count;
}

/** Changes whenever the drawn samples do; computed from the ends, not every sample. */
function frameSignature(frame: PlotCartesianFrame, revision: number): string {
  const x = frame.series[0]?.x ?? [];
  return `append:${revision}:${x.length}:${x[0] ?? ""}:${x.at(-1) ?? ""}:${frame.point_count}`;
}

function pushAll<T>(target: T[], values: readonly T[]) {
  for (const value of values) target.push(value);
}

function sameSeries(current: readonly PlotSeries[], update: readonly PlotSeries[]): boolean {
  return current.length === update.length && current.every((item, index) => item.key === update[index].key);
}

function trimMarkers(markers: PlotMarker[], windowS: number | null): PlotMarker[] {
  const byBudget = markers.length > MAX_RETAINED_MARKERS ? markers.slice(-MAX_RETAINED_MARKERS) : markers;
  if (!windowS || byBudget.length < 2) return byBudget;

  let latest = Number.NEGATIVE_INFINITY;
  byBudget.forEach((marker, index) => {
    const x = xValue(marker.x, index);
    if (Number.isFinite(x) && x > latest) latest = x;
  });
  if (!Number.isFinite(latest)) return byBudget;

  const cutoff = latest - windowS;
  return byBudget.filter((marker, index) => xValue(marker.x, index) >= cutoff);
}
