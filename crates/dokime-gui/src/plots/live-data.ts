/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import type { EvidenceScope } from "@host/model";
import { cutKey, type Wire } from "@dokime/plots/lib/rpc/canonical";
import { failureOf } from "@dokime/plots/lib/notification-details";
import { type MutableRefObject, useEffect, useMemo, useRef, useState } from "react";

import { create } from "@bufbuild/protobuf";
import { validatedExpression } from "@dokime/plots/expression";
import { plotType } from "@dokime/plots/lib/api/native-plot-projection";
import { jsonToNativeValue } from "@dokime/plots/lib/host/values";
import {
  AxisScale,
  FilterOperator,
  type PlotRequest,
  BarArrangement,
  ContourRender,
  Downsample,
  FftDetrend,
  FftWindow,
  FrequencyLevelUnit,
  LineStyle,
  PlotRequestSchema,
  Reduction,
  XyInterval,
} from "@host/model";

import { useNativeSessionOptional } from "@/app/native-session/context";
import type { PlotRefreshState } from "@dokime/document/features/cycle/PlotWindowSlider";
import { fetchNativePlotPayloads, NATIVE_PLOT_MAX_REQUESTS, type NativePlotStream, openNativePlotStream } from "@/lib/api/native-plots";
import { NativeApiLimitError } from "@dokime/plots/lib/api/native-values";
import { toast } from "@dokime/plots/lib/notifications";
import { summaryInFlight } from "@dokime/plots/lib/statistics";
import type { DocumentAxisScale, DocumentDownsampleTechnique, DocumentViewPlot, PlotPayloadStreamMessage } from "@dokime/plots/lib/types";
import { applyDocumentPlotMessages, applyPendingNotice, emptyLivePlotState, type LivePlotStateMap } from "@dokime/plots/live-frames";
import { DEFAULT_PLOT_MAX_POINTS, NATIVE_PLOT_MAX_POINTS } from "@dokime/plots/reduce";
import { type SummaryWindow, useSummaryWindowStore } from "@dokime/plots/stores/summary-window-store";

/* === Constants === */

const HIDDEN_PLOT_STREAM_FLUSH_MS = 1_000;
const RECONNECT_INITIAL_MS = 1_000;
const RECONNECT_MAX_MS = 10_000;

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
  scope: Wire<EvidenceScope> | null;
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
  const visible = useSummaryWindowStore((store) => store.visible);
  const scopeKey = cutKey(scope);
  // Equivalent Canonical Scopes Keep Their Subscription Across Query Refreshes.
  const scopeRef = useRef({ key: scopeKey, value: scope });
  if (scopeRef.current.key !== scopeKey) {
    scopeRef.current = { key: scopeKey, value: scope };
  }
  const requestScope = scopeRef.current.value;
  const generation = nativeSession?.identity.generation;
  const scopeWindowKey = useMemo(() => [generation, scopeKey, windowS ?? "all"].join("|"), [generation, scopeKey, windowS]);
  const dataKey = useMemo(() => [plotRequestKey, scopeWindowKey].join("|"), [plotRequestKey, scopeWindowKey]);

  // All Captures Query Settings On Entry Or Explicit Refresh.
  const automatic = windowS != null;
  const refreshKey = refresh?.key ?? 0;
  const currentPlots = useRef(plots);
  currentPlots.current = plots;
  const selectionKey = automatic ? dataKey : `${scopeWindowKey}|${refreshKey}`;
  const requestSelection = useMemo(() => ({ key: selectionKey, plots: currentPlots.current }), [selectionKey]);
  const requestPlots = requestSelection.plots;
  // Bound Each Stable Run By Kind So Off-Screen Analyses Cannot Delay The First Charts.
  const requestGroups = useMemo(() => {
    const groups: DocumentViewPlot[][] = [];
    for (const plot of requestPlots) {
      const last = groups.at(-1);
      if (!last || last.length === NATIVE_PLOT_MAX_REQUESTS || last[0].plot_type !== plot.plot_type) {
        groups.push([plot]);
      } else last.push(plot);
    }
    return groups;
  }, [requestPlots]);
  const visibleGroups = requestGroups.map(
    (group) => typeof IntersectionObserver === "undefined" || group.some((plot) => visible[plot.plot_id] === true),
  );
  // Visible Groups Finish Loading Before Prefetch Starts.
  const visibleReady = requestGroups.every((group, index) => !visibleGroups[index] || group.every((plot) => state[plot.plot_id]?.payload));
  const shownGroups = requestGroups.filter((_, index) => visibleGroups[index] || (visibleReady && visibleGroups[index - 1]));
  const shownPlots = shownGroups.flat();
  const shownKey = shownPlots.map((plot) => plot.plot_id).join(";");
  const summaryKey = useMemo(
    () => JSON.stringify(Object.fromEntries(requestPlots.map((plot) => [plot.plot_id, summaryWindows[plot.plot_id]]))),
    [requestPlots, summaryWindows],
  );
  const widthKey = JSON.stringify(
    Object.fromEntries(requestPlots.map((plot) => [plot.plot_id, [panelWidths[plot.plot_id], panelHeights[plot.plot_id]]])),
  );
  const requestedWidths = useMemo(() => JSON.parse(widthKey) as Record<string, [number | null, number | null] | undefined>, [widthKey]);
  const requestedSummaryWindows = useMemo(() => JSON.parse(summaryKey) as Record<string, SummaryWindow | undefined>, [summaryKey]);
  const requestDataKey = automatic ? dataKey : `${requestPlots.map(documentPlotRequestKey).join(";")}|${scopeWindowKey}`;
  const requestKey = automatic
    ? `${dataKey}|${summaryKey}|${widthKey}|${shownKey}`
    : `${scopeWindowKey}|${summaryKey}|${widthKey}|${shownKey}`;
  const requestVisible = !automatic || pageVisible;
  const onRefreshChange = refresh?.onChange;
  const inputsRef = useRef({
    plots: shownPlots,
    groups: shownGroups,
    requestedSummaryWindows,
    requestedWidths,
    dataKey: requestDataKey,
    scopeWindowKey,
  });
  inputsRef.current = {
    plots: shownPlots,
    groups: shownGroups,
    requestedSummaryWindows,
    requestedWidths,
    dataKey: requestDataKey,
    scopeWindowKey,
  };
  const requestInputs = useMemo(() => ({ ...inputsRef.current, key: `${requestKey}|${refreshKey}` }), [requestKey, refreshKey]);

  const dataKeyRef = useRef<string | null>(null);
  const viewKeyRef = useRef<string | null>(null);
  const refreshFailuresRef = useRef(new Set<string>());
  const groupsRef = useRef(new Map<string, () => void>());
  const pendingRef = useRef(new Set<string>());
  const renderActiveRef = useRef(renderActive);

  useEffect(() => {
    renderActiveRef.current = renderActive;
  }, [renderActive]);

  useEffect(
    () => () => {
      for (const dispose of groupsRef.current.values()) {
        dispose();
      }
      groupsRef.current.clear();
      pendingRef.current.clear();
    },
    [],
  );

  useEffect(() => {
    const commit = (update: (current: LivePlotStateMap) => LivePlotStateMap) => {
      modelRef.current = update(modelRef.current);
      setState(modelRef.current);
    };
    const {
      plots: selectedPlots,
      groups: selectedGroups,
      requestedSummaryWindows,
      requestedWidths,
      dataKey,
      scopeWindowKey,
    } = requestInputs;
    const dataChanged = dataKeyRef.current !== dataKey;
    if (dataChanged) {
      refreshFailuresRef.current.clear();
    }
    dataKeyRef.current = dataKey;
    const viewChanged = viewKeyRef.current !== scopeWindowKey;
    viewKeyRef.current = scopeWindowKey;

    const testCycleId = requestScope?.cycleId;
    if (!enabled || !testCycleId || selectedPlots.length === 0 || !requestVisible) {
      for (const dispose of groupsRef.current.values()) {
        dispose();
      }
      groupsRef.current.clear();
    }
    if (!enabled || !testCycleId || selectedPlots.length === 0) {
      onRefreshChange?.({ refreshing: false, updatedAt: null });
      commit((current) => {
        if (!testCycleId || requestPlots.length === 0) {
          return {};
        }
        return Object.fromEntries(
          requestPlots.map((plot) => {
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
          selectedPlots.map((plot) => {
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

    const desired = new Map<string, { plots: DocumentViewPlot[]; requests: PlotRequest[] }>();
    for (const plots of selectedGroups) {
      const requests = plots.map((plot) => {
        const width = requestedWidths[plot.plot_id]?.[0];
        const maximum = displayPointBudget(plot.display_points) || DEFAULT_PLOT_MAX_POINTS;
        return plotRequestForDocumentPlot(plot, {
          scope: requestScope,
          windowS,
          summaryWindow: requestedSummaryWindows[plot.plot_id] ?? null,
          // Widths Above The Point Ceiling Do Not Change The Served Selection.
          panelWidth: !automatic || (width && width * 4 < maximum) ? (width ?? undefined) : undefined,
          panelHeight: plot.plot_type === "contour" ? (requestedWidths[plot.plot_id]?.[1] ?? undefined) : undefined,
        });
      });
      desired.set(cutKey([nativeSession.identity.generation, requests, refreshKey]), { plots, requests });
    }
    for (const [key, dispose] of groupsRef.current) {
      if (desired.has(key)) {
        continue;
      }
      dispose();
      groupsRef.current.delete(key);
    }
    if (dataChanged)
      commit((current) => Object.fromEntries(Object.entries(current).filter(([id]) => requestPlots.some((plot) => plot.plot_id === id))));
    for (const [key, group] of desired) {
      if (groupsRef.current.has(key)) {
        continue;
      }
      const activePlots = group.plots;
      const requests = group.requests;
      const flushFrameRef = { current: null as number | null };
      const flushTimerRef = { current: null as number | null };
      const controller = new AbortController();
      let active = true;
      const streams = new Set<NativePlotStream>();
      const releases = new Set<() => void>();
      for (const plot of activePlots) {
        pendingRef.current.add(plot.plot_id);
      }
      let reopenTimer: number | null = null;
      let reopenDelayMs = RECONNECT_INITIAL_MS;
      const received = new Set<string>();
      let awaitingRefresh = true;
      onRefreshChange?.({ refreshing: true, ...(viewChanged ? { updatedAt: null } : {}) });

      const reportRefreshFailure = (error: unknown, plotId = "stream") => {
        received.clear();
        awaitingRefresh = false;
        for (const plot of activePlots) {
          pendingRef.current.delete(plot.plot_id);
        }
        onRefreshChange?.({ refreshing: pendingRef.current.size > 0 });
        const failure = failureOf(error, "Plot Refresh Failed");
        if (!refreshFailuresRef.current.has(plotId)) {
          refreshFailuresRef.current.add(plotId);
          toast.error(failure, { context: "background" });
        }
      };

      // A new watch numbers its frames from its own origin.
      commit((current) => ({
        ...current,
        ...Object.fromEntries(
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
      }));

      let queued: LivePlotStateMap = {};
      const flush = () => {
        flushFrameRef.current = null;
        flushTimerRef.current = null;
        if (!active || !nativeSession.identity.isCurrent() || Object.keys(queued).length === 0) {
          return;
        }
        const updates = queued;
        queued = {};
        // Only Complete Revisions Reach The Visible Model, In One Shared Commit.
        commit((current) => ({ ...current, ...updates }));
      };

      const enqueue = (messages: readonly PlotPayloadStreamMessage[]) => {
        if (!active || messages.length === 0) {
          return;
        }
        reopenDelayMs = RECONNECT_INITIAL_MS;
        const next = applyDocumentPlotMessages({ ...modelRef.current, ...queued }, messages);
        for (const message of messages) {
          const plotState = next[message.plot_id];
          if (!plotState) {
            continue;
          }
          // Retain Only The Latest State Per Plot While Rendering Is Inactive.
          queued[message.plot_id] = plotState;
          if (plotState.status === "error") {
            const error = plotState.failure ?? failureOf(plotState.error, "Plot Refresh Failed");
            reportRefreshFailure(error, message.plot_id);
          } else {
            refreshFailuresRef.current.delete(message.plot_id);
            refreshFailuresRef.current.delete("stream");
            received.add(message.plot_id);
            if (activePlots.every((plot) => received.has(plot.plot_id))) {
              awaitingRefresh = false;
              for (const plot of activePlots) {
                pendingRef.current.delete(plot.plot_id);
              }
              onRefreshChange?.({ refreshing: pendingRef.current.size > 0, updatedAt: Date.now() });
              received.clear();
            }
          }
        }
        if (flushFrameRef.current !== null || flushTimerRef.current !== null) {
          return;
        }
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
        if (awaitingRefresh) {
          reportRefreshFailure(new Error("The plot stream ended before the view finished refreshing."));
        }
        onRefreshChange?.({ refreshing: pendingRef.current.size > 0 });
        commit((current) => ({
          ...current,
          ...Object.fromEntries(
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
        }));
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
        commit((current) => ({
          ...current,
          ...Object.fromEntries(
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
        }));
        if (!terminal) {
          scheduleReopen();
        }
      };

      const scheduleReopen = () => {
        if (!active || reopenTimer !== null || !nativeSession.identity.isCurrent()) {
          return;
        }
        reopenTimer = window.setTimeout(() => {
          reopenTimer = null;
          // The reopened watch resends snapshots from its own revision origin.
          commit((current) => ({
            ...current,
            ...Object.fromEntries(activePlots.map((plot) => [plot.plot_id, { ...current[plot.plot_id], revision: 0 }])),
          }));
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
          const stream = await openNativePlotStream(nativeSession.identity, requests, { signal: controller.signal });
          streams.add(stream);
          const release = nativeSession.own(() => void stream.cancel().catch(() => undefined));
          releases.add(release);
          try {
            if (!active || controller.signal.aborted || !nativeSession.identity.isCurrent()) {
              return;
            }
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
          } finally {
            release();
            releases.delete(release);
            if (streams.delete(stream)) {
              await stream.cancel();
            }
          }
          markStreamEnded();
        } catch (exc: unknown) {
          if (!isAbortError(exc)) {
            markStreamError(exc);
          }
        }
      };

      const readSnapshot = async () => {
        try {
          const envelopes = await fetchNativePlotPayloads(nativeSession.identity, requests, { signal: controller.signal });
          if (!active || controller.signal.aborted || !nativeSession.identity.isCurrent()) {
            return;
          }
          const expected = new Set(activePlots.map((plot) => plot.plot_id));
          if (envelopes.length !== expected.size || envelopes.some((envelope) => !expected.delete(envelope.plot_id))) {
            throw new Error("Read Plot Group: Refresh the Incomplete Plot Selection");
          }
          enqueue(envelopes.map((envelope) => ({ ...envelope, kind: "snapshot" })));
        } catch (error) {
          if (!active || isAbortError(error) || !nativeSession.identity.isCurrent()) {
            return;
          }
          reportRefreshFailure(error);
          const failure = failureOf(error, "Plot Refresh Failed");
          commit((current) => ({
            ...current,
            ...Object.fromEntries(
              activePlots.map((plot) => [
                plot.plot_id,
                {
                  ...(current[plot.plot_id] ?? emptyLivePlotState()),
                  status: "error",
                  error: failure.message,
                  failure,
                  summaryInFlight: false,
                },
              ]),
            ),
          }));
        }
      };

      if (automatic) {
        void consumeStream();
      } else void readSnapshot();

      groupsRef.current.set(key, () => {
        active = false;
        for (const plot of activePlots) {
          pendingRef.current.delete(plot.plot_id);
        }
        controller.abort();
        if (reopenTimer !== null) {
          window.clearTimeout(reopenTimer);
          reopenTimer = null;
        }
        for (const release of releases) {
          release();
        }
        for (const stream of streams) {
          void stream.cancel().catch(() => undefined);
        }
        streams.clear();
        clearScheduledFlush(flushFrameRef, flushTimerRef);
      });
    }
  }, [requestInputs, requestPlots, refreshKey, onRefreshChange, enabled, requestScope, windowS, requestVisible, nativeSession, automatic]);

  return state;
}

export function plotRequestForDocumentPlot(
  plot: DocumentViewPlot,
  options: {
    scope: Wire<EvidenceScope> | null;
    windowS?: number | null;
    summaryWindow?: SummaryWindow | null;
    panelWidth?: number;
    panelHeight?: number;
  },
): PlotRequest {
  const declared = displayPointBudget(plot.display_points);
  const maximum = options.windowS == null && declared === 0 ? DEFAULT_PLOT_MAX_POINTS : declared;
  return create(PlotRequestSchema, {
    arrangement: plot.arrangement ? (plot.arrangement === "grouped" ? BarArrangement.GROUPED : BarArrangement.STACKED) : undefined,
    pareto: plot.pareto ?? false,
    hole: plot.hole ?? undefined,
    xyInterval: plot.xy_interval ? XyInterval.SIGMA : undefined,
    render: plot.render === "bands" ? ContourRender.BANDS : ContourRender.CELLS,
    reduction: plot.reduction ? { mean: Reduction.MEAN, min: Reduction.MIN, max: Reduction.MAX }[plot.reduction] : undefined,
    grid: plot.grid
      ? {
          row: plot.grid.row ? validatedExpression(plot.grid.row) : undefined,
          column: plot.grid.column ? validatedExpression(plot.grid.column) : undefined,
          frame: plot.grid.frame ? validatedExpression(plot.grid.frame) : undefined,
          rows: plot.grid.rows.map((entry) => ({ ...entry, group: entry.group ?? undefined })),
          columns: plot.grid.columns.map((entry) => ({ ...entry, group: entry.group ?? undefined })),
          highlights: plot.grid.highlights.map((highlight) => ({
            row: validatedExpression(highlight.row),
            column: validatedExpression(highlight.column),
            label: highlight.label,
          })),
          summaryLabel: plot.grid.summaryLabel,
        }
      : undefined,
    panelHeight: options.panelHeight,

    ...(plot.plot_id == null ? {} : { plotId: plot.plot_id }),
    plotType: plotType(plot.plot_type),
    style: {
      line: LineStyle.LINE,
      points: LineStyle.POINTS,
      line_points: LineStyle.LINE_POINTS,
      step: LineStyle.STEP,
    }[plot.style ?? "line"],
    xScale: axisScale(plot.x_scale),
    yScale: axisScale(plot.y_scale),
    ...(plot.z_expression == null ? {} : { zExpression: validatedExpression(plot.z_expression) }),
    ...(plot.fft == null
      ? {}
      : {
          fft: {
            inputSamples: plot.fft.input_samples ?? undefined,
            sampleRateHz: plot.fft.sample_rate_hz ?? undefined,
            window: plot.fft.window === "rectangular" ? FftWindow.RECTANGULAR : FftWindow.HANN,
            detrend: plot.fft.detrend === "none" ? FftDetrend.NONE : FftDetrend.MEAN,
            levelUnit: {
              linear: FrequencyLevelUnit.LINEAR,
              dB: FrequencyLevelUnit.DB,
              dBu: FrequencyLevelUnit.DBU,
              dBm: FrequencyLevelUnit.DBM,
            }[plot.fft.level_unit],
            includeDc: plot.fft.include_dc,
            minFrequencyHz: plot.fft.min_frequency_hz ?? undefined,
            maxFrequencyHz: plot.fft.max_frequency_hz ?? undefined,
          },
        }),
    ...(plot.interval == null
      ? {}
      : {
          interval: {
            binWidth: plot.interval.bin_width,
            sigma: plot.interval.sigma,
            clipToObserved: plot.interval.clip_to_observed,
          },
        }),
    ...(plot.histogram == null
      ? {}
      : {
          histogram: {
            bins: boundedUint64(plot.histogram.bins, "histogram bins"),
            unit: plot.histogram.unit ?? undefined,
          },
        }),
    stream: plot.stream,
    yKeys: plot.y,
    markerRecordTypes: plot.marker_record_types ?? [],
    filters: (plot.filters ?? []).map(plotFilter),
    ...(plot.x == null ? {} : { xKey: plot.x }),
    ...(plot.series_by == null ? {} : { seriesBy: plot.series_by }),
    ...(plot.x_expression == null ? {} : { xExpression: validatedExpression(plot.x_expression) }),
    yExpressions: (plot.y_expressions ?? []).map(validatedExpression),
    ...(plot.series_by_expression == null ? {} : { seriesByExpression: validatedExpression(plot.series_by_expression) }),
    ...(plot.z_stream == null ? {} : { zStream: plot.z_stream }),
    ...(plot.z_field == null ? {} : { zField: plot.z_field }),
    ...(plot.unit == null ? {} : { unit: plot.unit }),
    ...(maximum == null ? {} : { maxPoints: boundedPlotPoints(maximum) }),
    ...(options.windowS == null ? {} : { windowS: boundedWindow(options.windowS) }),
    ...(options.summaryWindow == null ? {} : { summaryWindow: summaryWindow(options.summaryWindow) }),
    ...(options.summaryWindow == null ? {} : { viewport: summaryWindow(options.summaryWindow) }),
    ...(options.panelWidth == null ? {} : { panelWidth: boundedPanelWidth(options.panelWidth) }),
    scope: options.scope ?? undefined,
    technique: downsampleTechnique(plot.downsample),
  });
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
  return cutKey({ ...plotRequestForDocumentPlot(plot, { scope: null }), maxPoints: displayPointBudget(plot.display_points) });
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

function plotFilter(value: NonNullable<DocumentViewPlot["filters"]>[number]): Wire<PlotRequest>["filters"][number] {
  return {
    expression: validatedExpression(value.expression),
    operator: filterOperator(value.operator),
    ...(value.value === undefined ? {} : { value: jsonToNativeValue(value.value) }),
  };
}

function downsampleTechnique(value?: DocumentDownsampleTechnique | null): Downsample {
  return value === "raw" ? Downsample.RAW : value === "min_max" ? Downsample.MIN_MAX : value === "lttb" ? Downsample.LTTB : Downsample.AUTO;
}

function axisScale(value?: DocumentAxisScale | null): AxisScale {
  if (value === "log") {
    return AxisScale.LOG;
  }
  if (value === "linear" || value == null) {
    return AxisScale.LINEAR;
  }
  throw new Error(`Unsupported plot axis scale: ${String(value)}`);
}

function filterOperator(value: NonNullable<DocumentViewPlot["filters"]>[number]["operator"]): FilterOperator {
  const operator = {
    eq: FilterOperator.EQ,
    ne: FilterOperator.NE,
    gt: FilterOperator.GT,
    gte: FilterOperator.GE,
    lt: FilterOperator.LT,
    lte: FilterOperator.LE,
    contains: FilterOperator.CONTAINS,
    exists: FilterOperator.EXISTS,
  }[value];
  if (operator === undefined) {
    throw new Error(`Unsupported plot filter operator: ${String(value)}`);
  }
  return operator;
}

function boundedUint64(value: number, label: string): bigint {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new NativeApiLimitError(label, String(value));
  }
  return BigInt(value);
}

function boundedPanelWidth(value: number): number {
  if (!Number.isSafeInteger(value) || value < 1 || value > 0xffff_ffff) {
    throw new Error("Native plot panel_width must be a positive 32-bit integer.");
  }
  return value;
}

function boundedPlotPoints(value: number): bigint {
  if (!Number.isSafeInteger(value) || value < 0 || value > NATIVE_PLOT_MAX_POINTS) {
    throw new NativeApiLimitError("Plots request", `max_points must be an integer between 0 and ${NATIVE_PLOT_MAX_POINTS}`);
  }
  return BigInt(value);
}

function summaryWindow(window: { from: number; to: number }): { from: number; to: number } {
  if (!Number.isFinite(window.from) || !Number.isFinite(window.to) || window.from > window.to) {
    throw new Error("Native plot summary_window must be a finite range with from <= to.");
  }
  return { from: window.from, to: window.to };
}

function boundedWindow(value: number): number {
  if (!Number.isFinite(value) || value < 0) {
    throw new Error("Native plot window_s must be finite and non-negative.");
  }
  return value;
}
