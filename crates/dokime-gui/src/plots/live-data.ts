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
  NATIVE_PLOT_MAX_POINTS,
  type NativePlotStream,
  openNativePlotStream,
  type PlotPayloadRequestInput,
} from "@/lib/api/native-plots";
import { NativeApiLimitError } from "@/lib/api/native-values";
import type {
  DocumentDownsampleTechnique,
  DocumentViewPlot,
  PlotCartesianFrame,
  PlotMarker,
  PlotPayload,
  PlotPayloadCursor,
  PlotPayloadDiagnostics,
  PlotPayloadStreamMessage,
  PlotPoint,
  RecordScope,
} from "@/lib/types";
import { xValue } from "@/plots/axis";
import {
  DEFAULT_PLOT_MAX_POINTS,
  RAW_RING_FACTOR,
  type ReduceTechnique,
  reduceIndices,
  retainedStart,
} from "@/plots/reduce";

/* === Constants === */

const HIDDEN_PLOT_STREAM_FLUSH_MS = 1_000;
const MAX_RETAINED_MARKERS = 500;
const RECONNECT_INITIAL_MS = 1_000;
const RECONNECT_MAX_MS = 10_000;

/* === Types === */

export type LivePlotStatus =
  | "idle"
  | "loading"
  | "live"
  | "stale"
  | "reconnecting"
  | "error"
  /** A frozen report left the plot out to fit its budget. */
  | "omitted";

export type LivePlotState = {
  /** The reduced frame the renderer draws. */
  payload: PlotPayload | null;
  /** Every retained sample behind `payload`: windowed, raw-capped, not reduced. */
  buffer: PlotPayload | null;
  cursor: PlotPayloadCursor | null;
  diagnostics: PlotPayloadDiagnostics | null;
  revision: number;
  status: LivePlotStatus;
  error: string | null;
  droppedBatches: number;
};

export type LivePlotStateMap = Record<string, LivePlotState>;

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
  const plotRequestKey = useMemo(() => plots.map(documentPlotRequestKey).join(";"), [plots]);
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
  const latestRevisionRef = useRef<Record<string, number>>({});
  const pendingMessagesRef = useRef<PlotPayloadStreamMessage[]>([]);
  const flushFrameRef = useRef<number | null>(null);
  const flushTimerRef = useRef<number | null>(null);
  const renderActiveRef = useRef(renderActive);

  useEffect(() => {
    renderActiveRef.current = renderActive;
  }, [renderActive]);

  useEffect(() => {
    latestRevisionRef.current = {};
    pendingMessagesRef.current = [];
    clearScheduledFlush(flushFrameRef, flushTimerRef);
    const activePlots = plotsRef.current;
    const dataChanged = dataKeyRef.current !== dataKey;
    dataKeyRef.current = dataKey;

    const testCycleId = requestScope?.test_cycle_id;
    if (!enabled || !testCycleId || activePlots.length === 0) {
      setState((current) => {
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
      setState((current) =>
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
      plotRequestForDocumentPlot(plot, { scope: requestScope, windowS }),
    );
    const techniques: TechniqueByPlot = Object.fromEntries(
      activePlots.map((plot) => [plot.plot_id, reduceTechnique(plot.downsample)]),
    );
    let active = true;
    let stream: NativePlotStream | null = null;
    let releaseStream: (() => void) | undefined;
    let reopenTimer: number | null = null;
    let reopenDelayMs = RECONNECT_INITIAL_MS;

    setState((current) =>
      Object.fromEntries(
        activePlots.map((plot) => {
          const currentPlot = dataChanged
            ? emptyLivePlotState()
            : (current[plot.plot_id] ?? emptyLivePlotState());
          return [
            plot.plot_id,
            {
              ...currentPlot,
              error: null,
              status: currentPlot.payload ? "stale" : "loading",
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
      setState((current) =>
        applyPlotStreamMessagesById(current, messages, latestRevisionRef, techniques),
      );
    };

    const enqueue = (message: PlotPayloadStreamMessage) => {
      if (!active) {
        return;
      }
      // A frame of any kind proves the watch is healthy again.
      reopenDelayMs = RECONNECT_INITIAL_MS;
      // An error frame reports one plot's failure and carries no revision of
      // its own; it is applied as-is rather than compacted away.
      const latestRevision = latestRevisionRef.current[message.plot_id] ?? 0;
      if (message.kind !== "error" && message.revision < latestRevision) {
        setState((current) => ({
          ...current,
          [message.plot_id]: {
            ...(current[message.plot_id] ?? emptyLivePlotState()),
            droppedBatches: (current[message.plot_id]?.droppedBatches ?? 0) + 1,
          },
        }));
        return;
      }
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
      setState((current) =>
        Object.fromEntries(
          activePlots.map((plot) => {
            const plotState = current[plot.plot_id] ?? emptyLivePlotState();
            return [
              plot.plot_id,
              {
                ...plotState,
                status: plotState.status === "error" ? "error" : ("stale" as const),
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
      setState((current) =>
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
        latestRevisionRef.current = {};
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
      pendingMessagesRef.current = [];
    };
  }, [
    dataKey,
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
  options: { scope: RecordScope | null; windowS?: number | null },
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

export function emptyLivePlotState(): LivePlotState {
  return {
    payload: null,
    buffer: null,
    cursor: null,
    diagnostics: null,
    revision: 0,
    status: "idle",
    error: null,
    droppedBatches: 0,
  };
}

/**
 * Grow the retained buffer with one append frame: the trailing window applies
 * first, then the raw cap. Nothing is reduced here; `reducePlotPayload` draws
 * the display frame from the result.
 */
export function appendPlotPayload(current: PlotPayload, update: PlotPayload): PlotPayload {
  const incoming = new Map(update.series.map((series) => [series.key, series]));
  const maxPoints = plotPointBudget(update.max_points, current.max_points);
  const rawCap = rawRingCap(maxPoints);
  const windowS = update.window_s ?? current.window_s ?? null;
  const series = current.series.map((existing) => {
    const appended = incoming.get(existing.key);
    if (!appended) {
      return existing;
    }
    incoming.delete(existing.key);
    return {
      ...existing,
      points: retainPlotPoints([...existing.points, ...appended.points], windowS, rawCap),
      raw_points: existing.raw_points + appended.raw_points,
      downsampled: existing.downsampled || appended.downsampled,
    };
  });
  incoming.forEach((appended) => {
    series.push({
      ...appended,
      points: retainPlotPoints(appended.points, windowS, rawCap),
    });
  });

  return {
    ...current,
    generated_at: update.generated_at,
    analysis: update.analysis ?? current.analysis,
    context: { ...(current.context ?? {}), ...(update.context ?? {}) },
    markers: trimPlotMarkers([...(current.markers ?? []), ...(update.markers ?? [])], windowS),
    max_points: maxPoints ?? 0,
    window_s: windowS,
    series,
    cartesian: appendCartesianFrame(current.cartesian, update.cartesian, windowS, rawCap),
  };
}

/**
 * The display frame of a retained buffer: each series over its budget is
 * reduced with the plot's technique on its time axis. A buffer within budget
 * is returned as-is, so the common case allocates nothing.
 */
export function reducePlotPayload(buffer: PlotPayload, technique: ReduceTechnique): PlotPayload {
  const budget = plotPointBudget(buffer.max_points);
  const options = { budget, technique };
  const cartesian = buffer.cartesian;
  const aligned =
    cartesian != null &&
    cartesian.series.length === buffer.series.length &&
    cartesian.series.every((frame, index) => frame.x.length === buffer.series[index].points.length);
  const selections = buffer.series.map((series, index) =>
    aligned
      ? reduceIndices(cartesian.series[index].x, cartesian.series[index].y, options)
      : reduceIndices(
          series.points.map((point, at) => xValue(point.x, at)),
          series.points.map((point) => point.y),
          options,
        ),
  );
  if (selections.every((selection) => selection === null)) {
    return buffer;
  }

  const series = buffer.series.map((item, index) => {
    const selection = selections[index];
    return selection ? { ...item, points: pick(item.points, selection), downsampled: true } : item;
  });
  const reducedCartesian =
    cartesian && aligned
      ? rebuildCartesianFrame(
          cartesian,
          cartesian.series.map((frame, index) => {
            const selection = selections[index];
            return selection ? { x: pick(frame.x, selection), y: pick(frame.y, selection) } : frame;
          }),
        )
      : cartesian;
  return { ...buffer, series, cartesian: reducedCartesian };
}

/* === Private Functions === */

function applyPlotStreamMessages(
  current: LivePlotState,
  messages: PlotPayloadStreamMessage[],
  latestRevisionRef: MutableRefObject<number>,
  technique: ReduceTechnique,
): LivePlotState {
  let next = current;
  for (const message of messages) {
    if (message.kind === "error") {
      next = {
        ...next,
        error: message.warnings[0] ?? "Plot stream returned an error.",
        status: "error",
      };
      continue;
    }
    if (message.revision <= latestRevisionRef.current) {
      next = {
        ...next,
        droppedBatches: next.droppedBatches + 1,
      };
      continue;
    }
    latestRevisionRef.current = message.revision;
    const buffer =
      message.kind === "append" && next.buffer
        ? appendPlotPayload(next.buffer, message.payload)
        : retainPlotPayload(message.payload);
    next = {
      payload: reducePlotPayload(buffer, technique),
      buffer,
      cursor: message.cursor,
      diagnostics: message.diagnostics ?? next.diagnostics,
      revision: message.revision,
      status: "live",
      error: null,
      droppedBatches: next.droppedBatches,
    };
  }
  return next;
}

function applyPlotStreamMessagesById(
  current: LivePlotStateMap,
  messages: PlotPayloadStreamMessage[],
  latestRevisionRef: MutableRefObject<Record<string, number>>,
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
  if (messagesByPlot.size === 0) {
    return current;
  }
  const next = { ...current };
  messagesByPlot.forEach((plotMessages, plotId) => {
    const latestForPlot = {
      get current() {
        return latestRevisionRef.current[plotId] ?? 0;
      },
      set current(value: number) {
        latestRevisionRef.current[plotId] = value;
      },
    } as MutableRefObject<number>;
    const previous = next[plotId] ?? emptyLivePlotState();
    try {
      next[plotId] = applyPlotStreamMessages(
        previous,
        plotMessages,
        latestForPlot,
        techniques[plotId] ?? "auto",
      );
    } catch (failure) {
      next[plotId] = { ...previous, status: "error", error: errorMessage(failure) };
    }
  });

  return next;
}

/** A snapshot or replace frame becomes the whole buffer, bounded like an append. */
function retainPlotPayload(payload: PlotPayload): PlotPayload {
  const windowS = payload.window_s ?? null;
  const rawCap = rawRingCap(plotPointBudget(payload.max_points));
  return {
    ...payload,
    series: payload.series.map((series) => ({
      ...series,
      points: retainPlotPoints(series.points, windowS, rawCap),
    })),
    markers: trimPlotMarkers(payload.markers ?? [], windowS),
    cartesian: payload.cartesian
      ? rebuildCartesianFrame(
          payload.cartesian,
          payload.cartesian.series.map((frame) => retainCartesianSeries(frame, windowS, rawCap)),
        )
      : payload.cartesian,
  };
}

function appendCartesianFrame(
  current: PlotCartesianFrame | null | undefined,
  update: PlotCartesianFrame | null | undefined,
  windowS: number | null,
  rawCap: number,
): PlotCartesianFrame | null | undefined {
  if (!current || !update) {
    return update ?? current;
  }
  if (current.mode !== update.mode || current.series.length !== update.series.length) {
    return update;
  }

  const series = current.series.map((existing, index) =>
    retainCartesianSeries(
      {
        x: [...existing.x, ...update.series[index].x],
        y: [...existing.y, ...update.series[index].y],
      },
      windowS,
      rawCap,
    ),
  );
  return rebuildCartesianFrame(update, series);
}

function rebuildCartesianFrame(
  frame: PlotCartesianFrame,
  series: PlotCartesianFrame["series"],
): PlotCartesianFrame {
  return {
    ...frame,
    series,
    x_domain: numericDomain(series.flatMap((item) => item.x)),
    y_domain: numericDomain(
      series.flatMap((item) => item.y.filter((value): value is number => Number.isFinite(value))),
    ),
    point_count: series.reduce(
      (count, item) => count + item.y.filter((value) => Number.isFinite(value)).length,
      0,
    ),
    signature: cartesianSignature(series),
  };
}

function retainCartesianSeries(
  series: PlotCartesianFrame["series"][number],
  windowS: number | null,
  rawCap: number,
): PlotCartesianFrame["series"][number] {
  const start = retainedStart(series.x, windowS, rawCap);
  if (series.x.length - start > NATIVE_PLOT_MAX_POINTS) {
    throw new Error(
      "Raw plot exceeds the retained point limit; select a display budget or shorter window.",
    );
  }
  return start === 0 ? series : { x: series.x.slice(start), y: series.y.slice(start) };
}

function retainPlotPoints(
  points: PlotPoint[],
  windowS: number | null,
  rawCap: number,
): PlotPoint[] {
  const start = retainedStart(
    points.map((point, index) => xValue(point.x, index)),
    windowS,
    rawCap,
  );
  if (points.length - start > NATIVE_PLOT_MAX_POINTS) {
    throw new Error(
      "Raw plot exceeds the retained point limit; select a display budget or shorter window.",
    );
  }
  return start === 0 ? points : points.slice(start);
}

function pick<T>(values: readonly T[], indices: readonly number[]): T[] {
  return indices.map((index) => values[index]);
}

function numericDomain(values: number[]): { min: number; max: number } | null {
  const finite = values.filter(Number.isFinite);
  if (!finite.length) {
    return null;
  }
  return {
    min: Math.min(...finite),
    max: Math.max(...finite),
  };
}

function cartesianSignature(series: PlotCartesianFrame["series"]): string {
  return JSON.stringify(series.map(({ x, y }) => [x, y]));
}

/** The request budget a payload carries; `null` is the raw escape hatch. */
function plotPointBudget(...candidates: Array<number | null | undefined>): number | null {
  for (const candidate of candidates) {
    if (typeof candidate !== "number") {
      continue;
    }
    if (candidate <= 0) {
      return null;
    }
    if (Number.isFinite(candidate)) {
      return Math.max(1, candidate);
    }
  }
  return DEFAULT_PLOT_MAX_POINTS;
}

/** Raw samples kept behind a budget; an unbudgeted plot keeps the host's own cap. */
function rawRingCap(budget: number | null): number {
  return budget === null ? 0 : Math.min(NATIVE_PLOT_MAX_POINTS, budget * RAW_RING_FACTOR);
}

function reduceTechnique(value: DocumentDownsampleTechnique | null | undefined): ReduceTechnique {
  return value ?? "auto";
}

function trimPlotMarkers(markers: PlotMarker[], windowS: number | null): PlotMarker[] {
  const byBudget =
    markers.length > MAX_RETAINED_MARKERS ? markers.slice(-MAX_RETAINED_MARKERS) : markers;
  if (!windowS || byBudget.length < 2) {
    return byBudget;
  }
  const latest = Math.max(
    ...byBudget.map((marker, index) => xValue(marker.x, index)).filter(Number.isFinite),
  );
  if (!Number.isFinite(latest)) {
    return byBudget;
  }
  const cutoff = latest - windowS;
  return byBudget.filter((marker, index) => xValue(marker.x, index) >= cutoff);
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
