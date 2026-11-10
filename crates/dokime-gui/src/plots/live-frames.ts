/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * Live Plots Retain One Bounded Frame, Replacing Changed Time Buckets.
 * Exact Statistics Advance Independently Of The Drawn Revision.
 */

/* === Local Imports === */

import { failureOf, type NotificationFailure } from "@/lib/notification-details";
import { summaryInFlight } from "@/lib/statistics";
import type { PlotPayload, PlotPayloadDiagnostics, PlotPayloadStreamMessage } from "@/lib/types";

/* === Types === */

export type LivePlotStatus = "idle" | "loading" | "waiting" | "live" | "stale" | "reconnecting" | "error";

export type LivePlotState = {
  /** The retained frame the renderer draws. */
  payload: PlotPayload | null;
  diagnostics: PlotPayloadDiagnostics | null;
  /** The latest applied revision; older frames are counted as dropped. */
  revision: number;
  statisticsRevision: number;
  epoch: string | null;
  selection: string | null;
  status: LivePlotStatus;
  error: string | null;
  failure?: NotificationFailure;
  droppedBatches: number;
  /** A window, zoom or plot change asked for summaries not yet delivered. */
  summaryInFlight: boolean;
  /** When the watch first said this plot is subscribed and reading, in epoch ms. */
  waitingSince?: number | null;
  /** When the watch last said so; each keepalive moves it and re-renders the wait. */
  heardAt?: number | null;
};

export type LivePlotStateMap = Record<string, LivePlotState>;

/* === Public Functions === */

export function emptyLivePlotState(): LivePlotState {
  return {
    payload: null,
    diagnostics: null,
    revision: 0,
    statisticsRevision: 0,
    epoch: null,
    selection: null,
    status: "idle",
    error: null,
    droppedBatches: 0,
    summaryInFlight: false,
    waitingSince: null,
    heardAt: null,
  };
}

/**
 * A plot after the watch's pending notice at `now`: still without a frame,
 * it waits quietly, counting from the first notice; a plot that already draws
 * a frame, or reports an error, is left as it is.
 */
export function markPlotPending(current: LivePlotState, now: number): LivePlotState {
  if (current.payload || current.status === "error") return current;
  return { ...current, status: "waiting", error: null, waitingSince: current.waitingSince ?? now, heardAt: now };
}

/**
 * The plots after the watch's pending notice for `plotId`. Only a plot of this
 * watch moves; a plot it no longer requests is left out of the map.
 */
export function applyPendingNotice(current: LivePlotStateMap, plotId: string, now: number): LivePlotStateMap {
  const plot = current[plotId];
  if (!plot) return current;
  const next = markPlotPending(plot, now);
  return next === plot ? current : { ...current, [plotId]: next };
}

/** The quiet wait a pending plot shows, with how long it has waited. */
export function plotWaitingLabel(state: Pick<LivePlotState, "waitingSince" | "heardAt">): string {
  const since = state.waitingSince ?? null;
  const heard = state.heardAt ?? null;
  const seconds = since !== null && heard !== null ? Math.max(0, Math.round((heard - since) / 1_000)) : 0;
  return `Waiting for Data \u00b7 ${seconds} s`;
}

/** One plot's state after a batch of its stream messages, in arrival order. */
export function applyPlotMessages(current: LivePlotState, messages: readonly PlotPayloadStreamMessage[]): LivePlotState {
  let next = current;
  for (const message of messages) {
    // An error frame reports one plot's failure and carries no revision of its own.
    if (message.kind === "error") {
      next = {
        ...next,
        failure: failureOf({ message: message.warnings[0] ?? "Plot stream returned an error." }, "Plot Stream Failed"),
        error: message.warnings[0] ?? "Plot stream returned an error.",
        status: "error",
        summaryInFlight: summaryInFlight("failed"),
      };
      continue;
    }
    // A snapshot opens its own origin: an empty plot's first one is revision 0.
    const sameOrigin = message.epoch === next.epoch && message.selection === next.selection;
    if (message.kind === "statistics") {
      if (!sameOrigin || !next.payload || message.revision < next.statisticsRevision) {
        next = { ...next, droppedBatches: next.droppedBatches + 1 };
        continue;
      }
      next = {
        ...next,
        payload: { ...next.payload, summaries: message.payload.summaries },
        statisticsRevision: message.revision,
        summaryInFlight: false,
      };
      continue;
    }
    if (message.kind !== "snapshot" && (!sameOrigin || message.revision <= next.revision)) {
      next = { ...next, droppedBatches: next.droppedBatches + 1 };
      continue;
    }

    const payload = message.kind === "delta" ? applyBucketDelta(next.payload, message.payload) : message.payload;
    const summaries = payload.summaries?.length ? payload.summaries : sameOrigin ? next.payload?.summaries : [];
    next = {
      payload: { ...payload, summaries },
      statisticsRevision: message.payload.summaries?.length ? message.revision : sameOrigin ? next.statisticsRevision : 0,
      diagnostics: message.diagnostics ?? null,
      revision: message.revision,
      epoch: message.epoch,
      selection: message.selection,
      status: "live",
      error: null,
      droppedBatches: next.droppedBatches,
      summaryInFlight: !message.payload.summaries?.length,
      waitingSince: null,
      heardAt: null,
    };
  }
  return next;
}

/** Commit All Arrived Frames To One Document State Without Mutating The Visible State. */
export function applyDocumentPlotMessages(current: LivePlotStateMap, messages: readonly PlotPayloadStreamMessage[]): LivePlotStateMap {
  if (messages.length === 0) return current;
  const next = { ...current };
  for (const message of messages) {
    const previous = next[message.plot_id];
    if (!previous) continue;
    try {
      next[message.plot_id] = applyPlotMessages(previous, [message]);
    } catch (error) {
      const failure = failureOf(error, "Plot Stream Failed");
      next[message.plot_id] = { ...previous, status: "error", error: failure.message, failure };
    }
  }
  return next;
}

/** Replace Complete Buckets, Including Expired Buckets With No Replacement Rows. */
function applyBucketDelta(current: PlotPayload | null, delta: PlotPayload): PlotPayload {
  const previous = current?.cartesian;
  const incoming = delta.cartesian;
  const width = delta.bucket_width_s;
  if (
    !current ||
    !previous ||
    !incoming ||
    !width ||
    current.bucket_width_s !== width ||
    previous.mode !== "aligned" ||
    incoming.mode !== "aligned" ||
    current.series.length !== delta.series.length ||
    current.series.some((series, index) => series.key !== delta.series[index].key)
  ) {
    throw new Error("Plot delta does not match its retained frame.");
  }
  const changed = new Set(delta.replaced_buckets ?? []);
  const window = delta.bucket_window;
  const rows = new Map<number, Array<number | null>>();
  for (const [index, x] of (previous.series[0]?.x ?? []).entries()) {
    if (changed.has(Math.floor(x / width)) || (window && (x < window.from || x > window.to))) continue;
    rows.set(
      x,
      previous.series.map((column) => column.y[index] ?? null),
    );
  }
  for (const [index, x] of (incoming.series[0]?.x ?? []).entries()) {
    if (!changed.has(Math.floor(x / width)) || (window && (x < window.from || x > window.to))) continue;
    rows.set(
      x,
      incoming.series.map((column) => column.y[index] ?? null),
    );
  }
  const time = [...rows.keys()].sort((a, b) => a - b);
  return {
    ...delta,
    summaries: current.summaries,
    cartesian: {
      ...incoming,
      series: incoming.series.map((_, column) => ({ x: time, y: time.map((x) => rows.get(x)?.[column] ?? null) })),
    },
    series: delta.series.map((series) => ({ ...series, points: [] })),
  };
}
