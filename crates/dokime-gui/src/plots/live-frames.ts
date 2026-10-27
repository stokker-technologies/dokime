/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * Each live plot retains the latest bounded storage-query snapshot. A new
 * snapshot replaces the entire payload; the last rendered frame can remain
 * visible while its watch reconnects, without accumulating sample history.
 */

/* === Local Imports === */

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
  epoch: string | null;
  selection: string | null;
  status: LivePlotStatus;
  error: string | null;
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
  return `Waiting For Data \u00b7 ${seconds} s`;
}

/** One plot's state after a batch of its stream messages, in arrival order. */
export function applyPlotMessages(current: LivePlotState, messages: readonly PlotPayloadStreamMessage[]): LivePlotState {
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
    const sameOrigin = message.epoch === next.epoch && message.selection === next.selection;
    if (message.kind !== "snapshot" && (!sameOrigin || message.revision <= next.revision)) {
      next = { ...next, droppedBatches: next.droppedBatches + 1 };
      continue;
    }

    const payload = retainFrame(message.payload);
    next = {
      payload,
      diagnostics: message.diagnostics ?? null,
      revision: message.revision,
      epoch: message.epoch,
      selection: message.selection,
      status: "live",
      error: null,
      droppedBatches: next.droppedBatches,
      summaryInFlight: summaryInFlight("delivered"),
      waitingSince: null,
      heardAt: null,
    };
  }
  return next;
}

/** Keep One Render Representation Of The Latest Server Snapshot. */
function retainFrame(payload: PlotPayload): PlotPayload {
  if (!payload.cartesian) return payload;
  return {
    ...payload,
    series: payload.series.map((series) => ({ ...series, points: [] })),
  };
}
