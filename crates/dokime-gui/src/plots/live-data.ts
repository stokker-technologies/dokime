/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { type MutableRefObject, useEffect, useMemo, useRef, useState } from "react";

import { useNativeSessionOptional } from "@/app/native-session/context";
import { type NativePlotStream, openNativePlotStream, type PlotPayloadRequestInput } from "@/lib/api/native-plots";
import { NativeApiLimitError } from "@/lib/api/native-values";
import { summaryInFlight } from "@/lib/statistics";
import type { DocumentViewPlot, PlotPayloadStreamMessage, RecordScope } from "@/lib/types";
import { applyPendingNotice, applyPlotMessages, emptyLivePlotState, type LivePlotStateMap } from "@/plots/live-frames";
import { DEFAULT_PLOT_MAX_POINTS, NATIVE_PLOT_MAX_POINTS } from "@/plots/reduce";
import { type SummaryWindow, useSummaryWindowStore } from "@/stores/summary-window-store";

const GUI_HIDDEN_PLOT_FLUSH_MS = 1_000;
const GUI_RECONNECT_INITIAL_MS = 1_000;
const GUI_RECONNECT_MAX_MS = 10_000;

const HIDDEN_PLOT_STREAM_FLUSH_MS = GUI_HIDDEN_PLOT_FLUSH_MS;
const RECONNECT_INITIAL_MS = GUI_RECONNECT_INITIAL_MS;
const RECONNECT_MAX_MS = GUI_RECONNECT_MAX_MS;

export function useLivePlotPayloads({
  enabled,
  plots,
  renderActive = true,
  scope,
  windowS,
}: {
  enabled: boolean;
  plots: DocumentViewPlot[];
  renderActive?: boolean;
  scope: RecordScope | null;
  windowS?: number | null;
}): LivePlotStateMap {
  const pageVisible = usePageVisible();
  const nativeSession = useNativeSessionOptional();
  if (enabled && !nativeSession) {
    throw new Error("A connected native backend is required for live plots.");
  }
  const [state, setState] = useState<LivePlotStateMap>({});
  // The latest query snapshot waits here until React paints it.
  const modelRef = useRef<LivePlotStateMap>({});
  const plotRequestKey = useMemo(() => plots.map(documentPlotRequestKey).join(";"), [plots]);
  // A zoom changes which rows a plot's statistics cover, not the rows it
  // draws: the stream reopens for it while the drawn frame stays on screen.
  const summaryWindows = useSummaryWindowStore((store) => store.windows);
  const summaryKey = useMemo(
    () => JSON.stringify(Object.fromEntries(plots.map((plot) => [plot.plot_id, summaryWindows[plot.plot_id]]))),
    [plots, summaryWindows],
  );
  const requestedSummaryWindows = useMemo(() => JSON.parse(summaryKey) as Record<string, SummaryWindow | undefined>, [summaryKey]);
  const scopeFields = useMemo(
    () => ({
      test_case_id: scope?.test_case_id ?? null,
      criterion_id: scope?.criterion_id ?? null,
      session_id: scope?.session_id ?? null,
      suite_id: scope?.suite_id ?? null,
      step_id: scope?.step_id ?? null,
      template_id: scope?.template_id ?? null,
      test_cycle_id: scope?.test_cycle_id ?? null,
      test_execution_id: scope?.test_execution_id ?? null,
    }),
    [
      scope?.test_case_id,
      scope?.criterion_id,
      scope?.session_id,
      scope?.suite_id,
      scope?.step_id,
      scope?.template_id,
      scope?.test_cycle_id,
      scope?.test_execution_id,
    ],
  );
  const scopeKey = useMemo(() => recordScopeKey(scopeFields), [scopeFields]);
  const requestScope = useMemo(() => cloneRecordScope(scopeFields), [scopeFields]);
  const scopeWindowKey = useMemo(
    () => [scopeKey, windowS === undefined ? "plot-default" : windowS === null ? "all" : windowS].join("|"),
    [scopeKey, windowS],
  );
  const dataKey = useMemo(() => [plotRequestKey, scopeWindowKey].join("|"), [plotRequestKey, scopeWindowKey]);

  const plotsRef = useRef(plots);
  plotsRef.current = plots;
  const dataKeyRef = useRef<string | null>(null);
  const flushFrameRef = useRef<number | null>(null);
  const flushTimerRef = useRef<number | null>(null);
  const renderActiveRef = useRef(renderActive);

  useEffect(() => {
    renderActiveRef.current = renderActive;
  }, [renderActive]);

  useEffect(() => {
    const commit = (update: (current: LivePlotStateMap) => LivePlotStateMap) => {
      modelRef.current = update(modelRef.current);
      setState(modelRef.current);
    };
    clearScheduledFlush(flushFrameRef, flushTimerRef);
    const activePlots = plotsRef.current;
    const dataChanged = dataKeyRef.current !== dataKey;
    dataKeyRef.current = dataKey;

    const testCycleId = requestScope?.test_cycle_id;
    if (!enabled || !testCycleId || activePlots.length === 0) {
      commit((current) => {
        if (!testCycleId || activePlots.length === 0) {
          return {};
        }
        return Object.fromEntries(
          activePlots.map((plot) => {
            const currentPlot = current[plot.plot_id] ?? emptyLivePlotState();
            return [plot.plot_id, currentPlot.payload ? { ...currentPlot, status: "stale" as const } : currentPlot];
          }),
        );
      });
      return;
    }
    if (!pageVisible) {
      commit((current) =>
        Object.fromEntries(
          activePlots.map((plot) => {
            const currentPlot = current[plot.plot_id] ?? emptyLivePlotState();
            return [plot.plot_id, currentPlot.payload ? { ...currentPlot, status: "stale" as const } : currentPlot];
          }),
        ),
      );
      return;
    }

    if (!nativeSession) {
      return;
    }

    const controller = new AbortController();
    const requests = activePlots.map((plot) =>
      plotRequestForDocumentPlot(plot, {
        scope: requestScope,
        windowS,
        summaryWindow: requestedSummaryWindows[plot.plot_id] ?? null,
      }),
    );
    let active = true;
    let stream: NativePlotStream | null = null;
    let releaseStream: (() => void) | undefined;
    let reopenTimer: number | null = null;
    let reopenDelayMs = RECONNECT_INITIAL_MS;

    // A new watch numbers its frames from its own origin.
    commit((current) =>
      Object.fromEntries(
        activePlots.map((plot) => {
          const currentPlot = dataChanged ? emptyLivePlotState() : (current[plot.plot_id] ?? emptyLivePlotState());
          return [
            plot.plot_id,
            {
              ...currentPlot,
              revision: 0,
              error: null,
              status: currentPlot.payload ? "stale" : "loading",
              summaryInFlight: summaryInFlight("requested"),
            },
          ];
        }),
      ),
    );

    const flush = () => {
      flushFrameRef.current = null;
      flushTimerRef.current = null;
      if (active) setState(modelRef.current);
    };

    const enqueue = (message: PlotPayloadStreamMessage) => {
      if (!active) {
        return;
      }
      // A frame of any kind proves the watch is healthy again.
      reopenDelayMs = RECONNECT_INITIAL_MS;
      modelRef.current = applyMessageById(modelRef.current, message);
      if (flushFrameRef.current !== null || flushTimerRef.current !== null) {
        return;
      }
      if (message.kind === "error") {
        flushTimerRef.current = window.setTimeout(flush, 0);
        return;
      }
      if (renderActiveRef.current) {
        flushFrameRef.current = window.requestAnimationFrame(() => flush());
        return;
      }
      flushTimerRef.current = window.setTimeout(flush, HIDDEN_PLOT_STREAM_FLUSH_MS);
    };

    const markStreamEnded = () => {
      if (!active || !nativeSession.identity.isCurrent()) {
        return;
      }
      commit((current) =>
        Object.fromEntries(
          activePlots.map((plot) => {
            const plotState = current[plot.plot_id] ?? emptyLivePlotState();
            return [
              plot.plot_id,
              {
                ...plotState,
                status: plotState.status === "error" ? "error" : ("stale" as const),
                summaryInFlight: plotState.summaryInFlight && summaryInFlight("ended"),
              },
            ];
          }),
        ),
      );
    };

    /**
     * A failed watch keeps the last frame on screen and reopens with
     * backoff; only an admission refusal is terminal, since no retry changes it.
     */
    const markStreamError = (error: unknown) => {
      if (!active || !nativeSession.identity.isCurrent()) {
        return;
      }
      const terminal = error instanceof NativeApiLimitError;
      console.error(`Plot stream failed (${grpcStatusCode(error) ?? "no status"}): ${errorMessage(error)}`);
      commit((current) =>
        Object.fromEntries(
          activePlots.map((plot) => [
            plot.plot_id,
            {
              ...(current[plot.plot_id] ?? emptyLivePlotState()),
              error: errorMessage(error),
              status: terminal ? ("error" as const) : ("reconnecting" as const),
            },
          ]),
        ),
      );
      if (!terminal) scheduleReopen();
    };

    const scheduleReopen = () => {
      if (!active || reopenTimer !== null || !nativeSession.identity.isCurrent()) {
        return;
      }
      reopenTimer = window.setTimeout(() => {
        reopenTimer = null;
        // The reopened watch resends snapshots from its own revision origin.
        commit((current) => Object.fromEntries(Object.entries(current).map(([plotId, plot]) => [plotId, { ...plot, revision: 0 }])));
        void consumeStream();
      }, reopenDelayMs);
      reopenDelayMs = Math.min(reopenDelayMs * 2, RECONNECT_MAX_MS);
    };

    const consumeStream = async () => {
      if (!active || controller.signal.aborted) {
        return;
      }
      try {
        // Watch requests receive initial snapshots from the server
        // after subscription, avoiding a snapshot/watch race.
        stream = await openNativePlotStream(nativeSession.identity, requests, {
          signal: controller.signal,
        });
        if (!active || controller.signal.aborted || !nativeSession.identity.isCurrent()) {
          await stream.cancel();
          return;
        }
        releaseStream = nativeSession.own(() => {
          if (stream) void stream.cancel().catch(() => undefined);
        });
        for await (const message of stream) {
          if (!active || controller.signal.aborted || !nativeSession.identity.isCurrent()) {
            break;
          }
          // The watch is subscribed and still reading: a quiet wait, never an error.
          if (message.kind === "pending") {
            reopenDelayMs = RECONNECT_INITIAL_MS;
            const now = Date.now();
            commit((current) => applyPendingNotice(current, message.plot_id, now));
            continue;
          }
          enqueue(message);
        }
        markStreamEnded();
      } catch (exc: unknown) {
        if (!isAbortError(exc)) {
          markStreamError(exc);
        }
      } finally {
        releaseStream?.();
        releaseStream = undefined;
        stream = null;
      }
    };

    void consumeStream();

    return () => {
      active = false;
      controller.abort();
      if (reopenTimer !== null) {
        window.clearTimeout(reopenTimer);
        reopenTimer = null;
      }
      releaseStream?.();
      releaseStream = undefined;
      if (stream) void stream.cancel().catch(() => undefined);
      clearScheduledFlush(flushFrameRef, flushTimerRef);
    };
  }, [dataKey, requestedSummaryWindows, requestScope?.test_cycle_id, enabled, requestScope, windowS, pageVisible, nativeSession]);

  return state;
}

export function plotRequestForDocumentPlot(
  plot: DocumentViewPlot,
  options: {
    scope: RecordScope | null;
    windowS?: number | null;
    summaryWindow?: SummaryWindow | null;
  },
): PlotPayloadRequestInput {
  return {
    plotId: plot.plot_id,
    stream: plot.stream,
    yKeys: plot.y,
    markerRecordTypes: plot.marker_record_types ?? [],
    filters: plot.filters ?? [],
    xKey: plot.x ?? undefined,
    plotType: plot.plot_type,
    xScale: plot.x_scale ?? "linear",
    yScale: plot.y_scale ?? "linear",
    fft: plot.fft,
    analysis: plot.analysis,
    interval: plot.interval ?? undefined,
    histogram: plot.histogram ?? undefined,
    zExpression: plot.z_expression ?? undefined,
    seriesBy: plot.series_by ?? undefined,
    xExpression: plot.x_expression ?? undefined,
    yExpressions: plot.y_expressions ?? [],
    seriesByExpression: plot.series_by_expression ?? undefined,
    zStream: plot.z_stream ?? undefined,
    zField: plot.z_field ?? undefined,
    unit: plot.unit ?? undefined,
    // The display budget applies to EVERY window, including "All": the canvas
    // cannot resolve more points than this, and an unbounded request made the
    // full-range window ship every stored raw sample (slow payload, slow
    // render) instead of a server-downsampled frame.
    maxPoints: displayPointBudget(plot.display_points),
    technique: plot.downsample ?? "auto",
    windowS: options.windowS === undefined ? plot.live_window_s : options.windowS,
    summaryWindow: options.summaryWindow ?? null,
    scope: options.scope,
  };
}

/** The request budget a plot's declared `display_points` resolves to. */
export function displayPointBudget(displayPoints: number | null | undefined): number {
  if (typeof displayPoints !== "number" || !Number.isFinite(displayPoints)) {
    return DEFAULT_PLOT_MAX_POINTS;
  }
  return Math.min(Math.max(0, Math.trunc(displayPoints)), NATIVE_PLOT_MAX_POINTS);
}

function applyMessageById(current: LivePlotStateMap, message: PlotPayloadStreamMessage): LivePlotStateMap {
  const previous = current[message.plot_id] ?? emptyLivePlotState();
  try {
    return { ...current, [message.plot_id]: applyPlotMessages(previous, [message]) };
  } catch (failure) {
    return { ...current, [message.plot_id]: { ...previous, status: "error", error: errorMessage(failure) } };
  }
}

function documentPlotRequestKey(plot: DocumentViewPlot): string {
  return JSON.stringify(plotRequestForDocumentPlot(plot, { scope: null }));
}

function recordScopeKey(scope: RecordScope | null): string {
  return [
    scope?.test_cycle_id ?? "",
    scope?.test_execution_id ?? "",
    scope?.test_case_id ?? "",
    scope?.session_id ?? "",
    scope?.suite_id ?? "",
    scope?.template_id ?? "",
    scope?.step_id ?? "",
    scope?.criterion_id ?? "",
  ].join(":");
}

function cloneRecordScope(scope: RecordScope | null): RecordScope | null {
  if (!scope) {
    return null;
  }
  if (
    !scope.test_cycle_id &&
    !scope.test_execution_id &&
    !scope.test_case_id &&
    !scope.session_id &&
    !scope.suite_id &&
    !scope.template_id &&
    !scope.step_id &&
    !scope.criterion_id
  ) {
    return null;
  }
  return {
    test_cycle_id: scope.test_cycle_id ?? null,
    test_execution_id: scope.test_execution_id ?? null,
    test_case_id: scope.test_case_id ?? null,
    session_id: scope.session_id ?? null,
    suite_id: scope.suite_id ?? null,
    template_id: scope.template_id ?? null,
    step_id: scope.step_id ?? null,
    criterion_id: scope.criterion_id ?? null,
  };
}

function isAbortError(value: unknown): boolean {
  return value instanceof DOMException && value.name === "AbortError";
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** The gRPC status code a normalized native RPC error carries, when it does. */
function grpcStatusCode(error: unknown): number | null {
  if (!error || typeof error !== "object") return null;
  const candidate = error as { kind?: unknown; detail?: { code?: unknown } };
  return candidate.kind === "grpc" && typeof candidate.detail?.code === "number" ? candidate.detail.code : null;
}

function clearScheduledFlush(flushFrameRef: MutableRefObject<number | null>, flushTimerRef: MutableRefObject<number | null>): void {
  if (flushFrameRef.current !== null) {
    window.cancelAnimationFrame(flushFrameRef.current);
    flushFrameRef.current = null;
  }
  if (flushTimerRef.current !== null) {
    window.clearTimeout(flushTimerRef.current);
    flushTimerRef.current = null;
  }
}

function usePageVisible(): boolean {
  const [visible, setVisible] = useState(() => document.visibilityState !== "hidden");

  useEffect(() => {
    const handleVisibilityChange = () => {
      setVisible(document.visibilityState !== "hidden");
    };
    document.addEventListener("visibilitychange", handleVisibilityChange);
    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, []);

  return visible;
}
