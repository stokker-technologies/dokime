/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { failureOf } from "@/lib/notification-details";
import { type MutableRefObject, useEffect, useMemo, useRef, useState } from "react";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/context";
import type { PlotRefreshState } from "@/features/cycle/PlotWindowSlider";
import { fetchNativePlotPayloads, type NativePlotStream, openNativePlotStream, type PlotPayloadRequestInput } from "@/lib/api/native-plots";
import { NativeApiLimitError } from "@/lib/api/native-values";
import { toast } from "@/lib/notifications";
import { summaryInFlight } from "@/lib/statistics";
import type { DocumentViewPlot, PlotPayloadStreamMessage, RecordScope } from "@/lib/types";
import { applyDocumentPlotMessages, applyPendingNotice, emptyLivePlotState, type LivePlotStateMap } from "@/plots/live-frames";
import { DEFAULT_PLOT_MAX_POINTS, NATIVE_PLOT_MAX_POINTS } from "@/plots/reduce";
import { type SummaryWindow, useSummaryWindowStore } from "@/stores/summary-window-store";

/* === Constants === */

const GUI_HIDDEN_PLOT_FLUSH_MS = 1_000;
const GUI_RECONNECT_INITIAL_MS = 1_000;
const GUI_RECONNECT_MAX_MS = 10_000;

const HIDDEN_PLOT_STREAM_FLUSH_MS = GUI_HIDDEN_PLOT_FLUSH_MS;
const RECONNECT_INITIAL_MS = GUI_RECONNECT_INITIAL_MS;
const RECONNECT_MAX_MS = GUI_RECONNECT_MAX_MS;

/* === Types === */

/* === Public Functions === */

export function useLivePlotPayloads({
  enabled,
  plots,
  renderActive = true,
  refresh,
  scope,
  windowS,
}: {
  enabled: boolean;
  plots: DocumentViewPlot[];
  renderActive?: boolean;
  refresh?: PlotRefreshState;
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
  // Zoom And Width Changes Request Detail While The Previous Frame Stays Visible.
  const summaryWindows = useSummaryWindowStore((store) => store.windows);
  const panelWidths = useSummaryWindowStore((store) => store.widths);
  const panelHeights = useSummaryWindowStore((store) => store.heights);
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
  const scopeWindowKey = useMemo(() => [scopeKey, windowS ?? "all"].join("|"), [scopeKey, windowS]);
  const dataKey = useMemo(() => [plotRequestKey, scopeWindowKey].join("|"), [plotRequestKey, scopeWindowKey]);

  // All Captures Query Settings On Entry Or Explicit Refresh.
  const automatic = windowS != null;
  const refreshKey = refresh?.key ?? 0;
  const currentPlots = useRef(plots);
  currentPlots.current = plots;
  const selectionKey = automatic ? dataKey : `${scopeWindowKey}|${refreshKey}`;
  const requestSelection = useMemo(() => ({ key: selectionKey, plots: currentPlots.current }), [selectionKey]);
  const requestPlots = requestSelection.plots;
  const summaryKey = useMemo(
    () => JSON.stringify(Object.fromEntries(requestPlots.map((plot) => [plot.plot_id, summaryWindows[plot.plot_id]]))),
    [requestPlots, summaryWindows],
  );
  const widthKey = JSON.stringify(Object.fromEntries(requestPlots.map((plot) => [plot.plot_id, [panelWidths[plot.plot_id], panelHeights[plot.plot_id]]])));
  const requestedWidths = useMemo(() => JSON.parse(widthKey) as Record<string, [number | null, number | null] | undefined>, [widthKey]);
  const requestedSummaryWindows = useMemo(() => JSON.parse(summaryKey) as Record<string, SummaryWindow | undefined>, [summaryKey]);
  const requestDataKey = automatic ? dataKey : `${requestPlots.map(documentPlotRequestKey).join(";")}|${scopeWindowKey}`;
  const requestKey = automatic ? `${dataKey}|${summaryKey}|${widthKey}` : `${scopeWindowKey}|${summaryKey}|${widthKey}`;
  const requestVisible = !automatic || pageVisible;
  const onRefreshChange = refresh?.onChange;
  const inputsRef = useRef({ plots: requestPlots, requestedSummaryWindows, requestedWidths, dataKey: requestDataKey, scopeWindowKey });
  inputsRef.current = { plots: requestPlots, requestedSummaryWindows, requestedWidths, dataKey: requestDataKey, scopeWindowKey };
  const requestInputs = useMemo(() => ({ ...inputsRef.current, key: `${requestKey}|${refreshKey}` }), [requestKey, refreshKey]);

  const dataKeyRef = useRef<string | null>(null);
  const viewKeyRef = useRef<string | null>(null);
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
    const { plots: activePlots, requestedSummaryWindows, requestedWidths, dataKey, scopeWindowKey } = requestInputs;
    const dataChanged = dataKeyRef.current !== dataKey;
    dataKeyRef.current = dataKey;
    const viewChanged = viewKeyRef.current !== scopeWindowKey;
    viewKeyRef.current = scopeWindowKey;

    const testCycleId = requestScope?.test_cycle_id;
    if (!enabled || !testCycleId || activePlots.length === 0) {
      onRefreshChange?.({ refreshing: false, updatedAt: null });
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
    if (!requestVisible) {
      onRefreshChange?.({ refreshing: false });
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
        panelWidth: requestedWidths[plot.plot_id]?.[0] ?? undefined,
        panelHeight: requestedWidths[plot.plot_id]?.[1] ?? undefined,
      }),
    );
    let active = true;
    let stream: NativePlotStream | null = null;
    let releaseStream: (() => void) | undefined;
    let reopenTimer: number | null = null;
    let reopenDelayMs = RECONNECT_INITIAL_MS;
    const received = new Set<string>();
    let failed = false;
    let awaitingRefresh = true;
    onRefreshChange?.({ refreshing: true, ...(viewChanged ? { updatedAt: null } : {}) });

    const reportRefreshFailure = (error: unknown) => {
      received.clear();
      awaitingRefresh = false;
      onRefreshChange?.({ refreshing: false });
      toast.error(failureOf(error, "Plot Refresh Failed"), { context: "background" });
      failed = true;
    };

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

    let queued: LivePlotStateMap = {};
    const flush = () => {
      flushFrameRef.current = null;
      flushTimerRef.current = null;
      if (!active || Object.keys(queued).length === 0) return;
      const updates = queued;
      queued = {};
      // Only Complete Revisions Reach The Visible Model, In One Shared Commit.
      commit((current) => ({ ...current, ...updates }));
    };

    const enqueue = (messages: readonly PlotPayloadStreamMessage[]) => {
      if (!active || messages.length === 0) return;
      reopenDelayMs = RECONNECT_INITIAL_MS;
      const next = applyDocumentPlotMessages({ ...modelRef.current, ...queued }, messages);
      for (const message of messages) {
        const plotState = next[message.plot_id];
        if (!plotState) continue;
        // Retain Only The Latest State Per Plot While Rendering Is Inactive.
        queued[message.plot_id] = plotState;
        if (plotState.status === "error") {
          const error = plotState.failure ?? failureOf(plotState.error, "Plot Refresh Failed");
          if (automatic) reportRefreshFailure(error);
          else {
            toast.error(failureOf(error, "Plot Refresh Failed"), { context: "background" });
            failed = true;
          }
        } else {
          received.add(message.plot_id);
          if (activePlots.every((plot) => received.has(plot.plot_id))) {
            awaitingRefresh = false;
            onRefreshChange?.({ refreshing: false, updatedAt: Date.now() });
            received.clear();
            failed = false;
          }
        }
      }
      if (flushFrameRef.current !== null || flushTimerRef.current !== null) return;
      if (renderActiveRef.current) {
        flushFrameRef.current = window.requestAnimationFrame(flush);
        return;
      }
      flushTimerRef.current = window.setTimeout(flush, HIDDEN_PLOT_STREAM_FLUSH_MS);
    };

    const markStreamEnded = () => {
      if (!active || !nativeSession.identity.isCurrent()) {
        return;
      }
      clearScheduledFlush(flushFrameRef, flushTimerRef);
      flush();
      if (awaitingRefresh) reportRefreshFailure(new Error("The plot stream ended before the view finished refreshing."));
      onRefreshChange?.({ refreshing: false });
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
      clearScheduledFlush(flushFrameRef, flushTimerRef);
      flush();
      const terminal = error instanceof NativeApiLimitError;
      reportRefreshFailure(error);
      commit((current) =>
        Object.fromEntries(
          activePlots.map((plot) => [
            plot.plot_id,
            {
              ...(current[plot.plot_id] ?? emptyLivePlotState()),
              error: failureOf(error, "Plot Stream Failed").message,
              failure: failureOf(error, "Plot Stream Failed"),
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
          // Decode The Whole Revision Before Scheduling Its One Shared Commit.
          const updates: PlotPayloadStreamMessage[] = [];
          for (const update of message.updates) {
            if (update.kind === "pending") {
              reopenDelayMs = RECONNECT_INITIAL_MS;
              const now = Date.now();
              commit((current) => applyPendingNotice(current, update.plot_id, now));
            } else {
              updates.push(update);
            }
          }
          enqueue(updates);
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

    const readSnapshot = async () => {
      // Each plot owns one deadline and one transport response budget. Deliver
      // completed plots immediately, even while another plot is still reading.
      await Promise.all(
        requests.map(async (request, index) => {
          const plotId = activePlots[index].plot_id;
          try {
            const envelopes = await fetchNativePlotPayloads(nativeSession.identity, [request], { signal: controller.signal });
            if (!active || controller.signal.aborted || !nativeSession.identity.isCurrent()) return;
            if (envelopes.length !== 1 || envelopes[0].plot_id !== plotId) {
              throw new Error("The plot refresh returned an incomplete plot.");
            }
            enqueue([{ ...envelopes[0], kind: "snapshot" }]);
          } catch (error) {
            if (!active || isAbortError(error) || !nativeSession.identity.isCurrent()) return;
            toast.error(failureOf(error, "Plot Refresh Failed"), { context: "background" });
            failed = true;
            commit((current) => ({
              ...current,
              [plotId]: {
                ...(current[plotId] ?? emptyLivePlotState()),
                status: "error",
                error: failureOf(error, "Plot Stream Failed").message,
                failure: failureOf(error, "Plot Stream Failed"),
                summaryInFlight: false,
              },
            }));
          }
        }),
      );
      if (!active || controller.signal.aborted || !nativeSession.identity.isCurrent()) return;
      awaitingRefresh = false;
      onRefreshChange?.({ refreshing: false, ...(!failed ? { updatedAt: Date.now() } : {}) });
    };

    if (automatic) void consumeStream();
    else void readSnapshot();

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
  }, [requestInputs, onRefreshChange, enabled, requestScope, windowS, requestVisible, nativeSession, automatic]);

  return state;
}

export function plotRequestForDocumentPlot(
  plot: DocumentViewPlot,
  options: {
    scope: RecordScope | null;
    windowS?: number | null;
    summaryWindow?: SummaryWindow | null;
    panelWidth?: number;
    panelHeight?: number;
  },
): PlotPayloadRequestInput {
  const maximum = displayPointBudget(plot.display_points);
  return {
    plotId: plot.plot_id,
    stream: plot.stream,
    yKeys: plot.y,
    markerRecordTypes: plot.marker_record_types ?? [],
    filters: plot.filters ?? [],
    xKey: plot.x ?? undefined,
    plotType: plot.plot_type,
    reduction: plot.reduction,
    render: plot.render,
    grid: plot.grid,
    style: plot.style ?? "line",
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
    // All always has a finite display budget, including declarations asking
    // for every raw sample. Timed windows keep their declared budget.
    maxPoints: options.windowS == null && maximum === 0 ? DEFAULT_PLOT_MAX_POINTS : maximum,
    technique: plot.downsample ?? "auto",
    windowS: options.windowS ?? null,
    summaryWindow: options.summaryWindow ?? null,
    viewport: options.summaryWindow ?? null,
    panelWidth: options.panelWidth,
    panelHeight: options.panelHeight,
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

/* === Private Functions === */

function documentPlotRequestKey(plot: DocumentViewPlot): string {
  return JSON.stringify({ ...plotRequestForDocumentPlot(plot, { scope: null }), maxPoints: displayPointBudget(plot.display_points) });
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
