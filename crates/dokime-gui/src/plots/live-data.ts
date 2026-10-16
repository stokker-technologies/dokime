/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { type MutableRefObject, useEffect, useMemo, useRef, useState } from "react";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import {
  type NativePlotStream,
  openNativePlotStream,
  type PlotPayloadRequestInput,
} from "@/lib/api/native-plots";
import { NativeApiLimitError } from "@/lib/api/native-values";
import { summaryInFlight } from "@/lib/statistics";
import type {
  DocumentDownsampleTechnique,
  DocumentViewPlot,
  PlotPayloadStreamMessage,
  RecordScope,
} from "@/lib/types";
import { applyPlotMessages, emptyLivePlotState, type LivePlotStateMap } from "@/plots/live-frames";
import {
  DEFAULT_PLOT_MAX_POINTS,
  NATIVE_PLOT_MAX_POINTS,
  type ReduceTechnique,
} from "@/plots/reduce";
import { type SummaryWindow, useSummaryWindowStore } from "@/stores/summary-window-store";

/* === Constants === */

const HIDDEN_PLOT_STREAM_FLUSH_MS = 1_000;
/** How long a plot may wait for its first frame before the wait is reported. */
const FIRST_FRAME_TIMEOUT_MS = 15_000;
const RECONNECT_INITIAL_MS = 1_000;
const RECONNECT_MAX_MS = 10_000;

/* === Types === */

type TechniqueByPlot = Record<string, ReduceTechnique>;

/* === Public Functions === */

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
  const streamEnabled = enabled && pageVisible;
  const [state, setState] = useState<LivePlotStateMap>({});
  // The retained frames live here and are committed to React state whole, so
  // a frame is applied exactly once and its arrays can grow in place.
  const modelRef = useRef<LivePlotStateMap>({});
  const plotRequestKey = useMemo(() => plots.map(documentPlotRequestKey).join(";"), [plots]);
  // A zoom changes which rows a plot's statistics cover, not the rows it
  // draws: the stream reopens for it while the drawn frame stays on screen.
  const summaryWindows = useSummaryWindowStore((store) => store.windows);
  const summaryKey = useMemo(
    () =>
      JSON.stringify(
        Object.fromEntries(plots.map((plot) => [plot.plot_id, summaryWindows[plot.plot_id]])),
      ),
    [plots, summaryWindows],
  );
  const requestedSummaryWindows = useMemo(
    () => JSON.parse(summaryKey) as Record<string, SummaryWindow | undefined>,
    [summaryKey],
  );
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
    () =>
      [scopeKey, windowS === undefined ? "plot-default" : windowS === null ? "all" : windowS].join(
        "|",
      ),
    [scopeKey, windowS],
  );
  const dataKey = useMemo(
    () => [plotRequestKey, scopeWindowKey].join("|"),
    [plotRequestKey, scopeWindowKey],
  );
  const _descriptorKey = useMemo(
    () => [dataKey, streamEnabled ? "enabled" : "disabled"].join("|"),
    [dataKey, streamEnabled],
  );

  const plotsRef = useRef(plots);
  plotsRef.current = plots;
  const dataKeyRef = useRef<string | null>(null);
  const pendingMessagesRef = useRef<PlotPayloadStreamMessage[]>([]);
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
    pendingMessagesRef.current = [];
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
            return [
              plot.plot_id,
              currentPlot.payload ? { ...currentPlot, status: "stale" as const } : currentPlot,
            ];
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
            return [
              plot.plot_id,
              currentPlot.payload ? { ...currentPlot, status: "stale" as const } : currentPlot,
            ];
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
    const techniques: TechniqueByPlot = Object.fromEntries(
      activePlots.map((plot) => [plot.plot_id, reduceTechnique(plot.downsample)]),
    );
    let active = true;
    let stream: NativePlotStream | null = null;
    let releaseStream: (() => void) | undefined;
    let reopenTimer: number | null = null;
    let reopenDelayMs = RECONNECT_INITIAL_MS;
    let firstFrameTimer: number | null = null;

    // A new watch numbers its frames from its own origin.
    commit((current) =>
      Object.fromEntries(
        activePlots.map((plot) => {
          const currentPlot = dataChanged
            ? emptyLivePlotState()
            : (current[plot.plot_id] ?? emptyLivePlotState());
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
      const messages = pendingMessagesRef.current;
      pendingMessagesRef.current = [];
      if (!messages.length || !active) {
        return;
      }
      commit((current) => applyMessagesById(current, messages, techniques));
    };

    const enqueue = (message: PlotPayloadStreamMessage) => {
      if (!active) {
        return;
      }
      // A frame of any kind proves the watch is healthy again.
      reopenDelayMs = RECONNECT_INITIAL_MS;
      pendingMessagesRef.current.push(message);
      if (flushFrameRef.current !== null || flushTimerRef.current !== null) {
        return;
      }
      if (message.kind === "snapshot" || message.kind === "replace" || message.kind === "error") {
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
     * A failed watch keeps the last frame on screen and reopens cursorless with
     * backoff; only an admission refusal is terminal, since no retry changes it.
     */
    const markStreamError = (error: unknown) => {
      if (!active || !nativeSession.identity.isCurrent()) {
        return;
      }
      const terminal = error instanceof NativeApiLimitError;
      console.error(
        `Plot stream failed (${grpcStatusCode(error) ?? "no status"}): ${errorMessage(error)}`,
      );
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
        commit((current) =>
          Object.fromEntries(
            Object.entries(current).map(([plotId, plot]) => [plotId, { ...plot, revision: 0 }]),
          ),
        );
        void consumeStream();
      }, reopenDelayMs);
      reopenDelayMs = Math.min(reopenDelayMs * 2, RECONNECT_MAX_MS);
    };

    const consumeStream = async () => {
      if (!active || controller.signal.aborted) {
        return;
      }
      try {
        // Cursorless watch requests receive initial snapshots from the server
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

    /**
     * A plot still without a frame after the deadline reports why: the stream
     * error when there is one, the silence otherwise. A later frame recovers it.
     */
    firstFrameTimer = window.setTimeout(() => {
      firstFrameTimer = null;
      if (!active) return;
      commit((current) =>
        Object.fromEntries(
          Object.entries(current).map(([plotId, plot]) => [
            plotId,
            plot.payload || (plot.status !== "loading" && plot.status !== "reconnecting")
              ? plot
              : {
                  ...plot,
                  status: "error" as const,
                  error:
                    plot.error ??
                    `No plot frame arrived within ${FIRST_FRAME_TIMEOUT_MS / 1_000} s.`,
                },
          ]),
        ),
      );
    }, FIRST_FRAME_TIMEOUT_MS);

    void consumeStream();

    return () => {
      active = false;
      controller.abort();
      if (firstFrameTimer !== null) {
        window.clearTimeout(firstFrameTimer);
        firstFrameTimer = null;
      }
      if (reopenTimer !== null) {
        window.clearTimeout(reopenTimer);
        reopenTimer = null;
      }
      releaseStream?.();
      releaseStream = undefined;
      if (stream) void stream.cancel().catch(() => undefined);
      clearScheduledFlush(flushFrameRef, flushTimerRef);
      pendingMessagesRef.current = [];
    };
  }, [
    dataKey,
    requestedSummaryWindows,
    requestScope?.test_cycle_id,
    enabled,
    requestScope,
    windowS,
    pageVisible,
    nativeSession,
  ]);

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
    // full-range window ship every retained raw sample (slow payload, slow
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

/* === Private Functions === */

function applyMessagesById(
  current: LivePlotStateMap,
  messages: PlotPayloadStreamMessage[],
  techniques: TechniqueByPlot,
): LivePlotStateMap {
  const messagesByPlot = new Map<string, PlotPayloadStreamMessage[]>();
  for (const message of messages) {
    const plotMessages = messagesByPlot.get(message.plot_id);
    if (plotMessages) {
      plotMessages.push(message);
    } else {
      messagesByPlot.set(message.plot_id, [message]);
    }
  }

  const next = { ...current };
  messagesByPlot.forEach((plotMessages, plotId) => {
    const previous = next[plotId] ?? emptyLivePlotState();
    try {
      next[plotId] = applyPlotMessages(previous, plotMessages, techniques[plotId] ?? "auto");
    } catch (failure) {
      next[plotId] = { ...previous, status: "error", error: errorMessage(failure) };
    }
  });
  return next;
}

function reduceTechnique(value: DocumentDownsampleTechnique | null | undefined): ReduceTechnique {
  return value ?? "auto";
}

function documentPlotRequestKey(plot: DocumentViewPlot): string {
  return [
    plot.plot_id,
    plot.stream,
    plot.plot_type,
    plot.style,
    plot.x_scale,
    plot.y_scale,
    plot.x ?? "",
    plot.z_stream ?? "",
    plot.z_field ?? "",
    plot.x_expression ? JSON.stringify(plot.x_expression) : "",
    plot.y_expressions ? JSON.stringify(plot.y_expressions) : "",
    plot.series_by_expression ? JSON.stringify(plot.series_by_expression) : "",
    plot.filters ? JSON.stringify(plot.filters) : "",
    plot.analysis ? JSON.stringify(plot.analysis) : "",
    plot.unit ?? "",
    plot.series_by ?? "",
    plot.live_window_s ?? "",
    plot.display_points ?? "",
    plot.downsample ?? "",
    plot.y.join(","),
    (plot.marker_record_types ?? []).join(","),
  ].join(":");
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
  return candidate.kind === "grpc" && typeof candidate.detail?.code === "number"
    ? candidate.detail.code
    : null;
}

function clearScheduledFlush(
  flushFrameRef: MutableRefObject<number | null>,
  flushTimerRef: MutableRefObject<number | null>,
): void {
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
