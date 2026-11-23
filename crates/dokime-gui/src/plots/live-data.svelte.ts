// Dokime — SPDX-License-Identifier: Apache-2.0
import type { EvidenceScope, PlotRequest } from "@generated/host/model";
import { onMount, untrack } from "svelte";
import { createSubscriber, SvelteMap } from "svelte/reactivity";
import type { ConnectedSession } from "@/app/native-session/session";
import type { PlotRefreshState } from "@/features/cycle/plot-window";
import type { PlotWindow } from "@/features/cycle/plot-window-state";
import { fetchNativePlotPayloads, NATIVE_PLOT_MAX_REQUESTS, type NativePlotStream, openNativePlotStream } from "@/lib/api/native-plots";
import { NativeApiLimitError } from "@/lib/api/native-values";
import { failureOf } from "@/lib/notification-details";
import { isHostReopen } from "@/lib/rpc/errors";
import { toast } from "@/lib/notifications";
import { cutKey, type Wire } from "@/lib/rpc/canonical";
import { summaryInFlight } from "@/lib/statistics";
import type { DocumentViewPlot, PlotPayloadStreamMessage } from "@/lib/types";
import { summaryWindowStore } from "@/stores/summary-window-store";
import { displayPointBudget, plotRequestForDocumentPlot } from "./live-data";
import {
  applyDocumentPlotMessages,
  applyPendingNotice,
  emptyLivePlotState,
  type LivePlotState,
  type LivePlotStateMap,
  type LivePlotStates,
} from "./live-frames";
import { DEFAULT_PLOT_MAX_POINTS } from "./reduce";

const HIDDEN_PLOT_STREAM_FLUSH_MS = 1_000;
const RECONNECT_INITIAL_MS = 1_000;
const RECONNECT_MAX_MS = 10_000;

export type LivePlotOptions = {
  enabled: boolean;
  plots: DocumentViewPlot[];
  renderActive?: boolean;
  scope: Wire<EvidenceScope> | null;
  /** The Window And Refresh Control; Without One Every Sample Is Read Once. */
  window?: PlotWindow;
};

export function createLivePlotPayloads(options: () => LivePlotOptions, getSession: () => ConnectedSession | null) {
  const states = new SvelteMap<string, LivePlotState>();
  let model: LivePlotStateMap = {};
  let pageVisible = $state(document.visibilityState !== "hidden");
  const subscribe = createSubscriber((update) => summaryWindowStore.subscribe(update));
  const measurements = $derived.by(() => {
    subscribe();
    return summaryWindowStore.getState();
  });

  const session = $derived(getSession());
  const enabled = $derived(options().enabled);
  const plots = $derived(options().plots);
  const scope = $derived(options().scope);
  const plotWindow = $derived(options().window);
  const windowS = $derived(plotWindow?.option().value);
  const automatic = $derived(windowS != null);
  const refreshKey = $derived(plotWindow?.refreshKey() ?? 0);
  const requestVisible = $derived(!automatic || pageVisible);
  const onRefreshChange = $derived(plotWindow?.getState().refresh.onChange);
  const renderActive = $derived(options().renderActive ?? true);
  const visible = $derived(measurements.visible);

  const scopeKey = $derived(cutKey(scope));
  // Equivalent Canonical Scopes Keep Their Subscription Across Query Refreshes.
  const requestScope = $derived.by(() => {
    void scopeKey;
    return untrack(() => scope);
  });
  const scopeWindowKey = $derived([session?.identity.generation, scopeKey, windowS ?? "all"].join("|"));
  const plotRequestKey = $derived(plots.map(documentPlotRequestKey).join(";"));
  const dataKey = $derived([plotRequestKey, scopeWindowKey].join("|"));

  // All Captures Query Settings On Entry Or Explicit Refresh.
  const selectionKey = $derived(automatic ? dataKey : `${scopeWindowKey}|${refreshKey}`);
  const requestPlots = $derived.by(() => {
    void selectionKey;
    return untrack(() => plots);
  });
  // Bound Each Stable Run By Kind So Off-Screen Analyses Cannot Delay The First Charts.
  const requestGroups = $derived.by(() => {
    const groups: DocumentViewPlot[][] = [];
    for (const plot of requestPlots) {
      const last = groups.at(-1);
      if (!last || last.length === NATIVE_PLOT_MAX_REQUESTS || last[0].plot_type !== plot.plot_type) {
        groups.push([plot]);
      } else {
        last.push(plot);
      }
    }
    return groups;
  });
  const visibleGroups = $derived(
    requestGroups.map((group) => typeof IntersectionObserver === "undefined" || group.some((plot) => visible[plot.plot_id] === true)),
  );
  // Visible Groups Finish Loading Before Prefetch Starts.
  const visibleReady = $derived(
    requestGroups.every((group, index) => !visibleGroups[index] || group.every((plot) => states.get(plot.plot_id)?.payload)),
  );
  const shownGroups = $derived(requestGroups.filter((_, index) => visibleGroups[index] || (visibleReady && visibleGroups[index - 1])));
  const shownPlots = $derived(shownGroups.flat());
  const shownKey = $derived(shownPlots.map((plot) => plot.plot_id).join(";"));
  const summaryKey = $derived(
    JSON.stringify(Object.fromEntries(requestPlots.map((plot) => [plot.plot_id, measurements.windows[plot.plot_id]]))),
  );
  const widthKey = $derived(
    JSON.stringify(
      Object.fromEntries(
        requestPlots.map((plot) => [plot.plot_id, [measurements.widths[plot.plot_id], measurements.heights[plot.plot_id]]]),
      ),
    ),
  );
  const requestDataKey = $derived(automatic ? dataKey : `${requestPlots.map(documentPlotRequestKey).join(";")}|${scopeWindowKey}`);
  const requestKey = $derived(
    automatic ? `${dataKey}|${summaryKey}|${widthKey}|${shownKey}` : `${scopeWindowKey}|${summaryKey}|${widthKey}|${shownKey}`,
  );

  let previousDataKey: string | null = null;
  let previousWindowKey: string | null = null;
  const refreshFailures = new Set<string>();
  const groups = new Map<string, { dispose: () => void; reschedule: () => void }>();
  const pending = new Set<string>();
  let painting = untrack(() => renderActive);
  let visiblePanels = untrack(() => visible);

  onMount(() => {
    function updateVisibility() {
      pageVisible = document.visibilityState !== "hidden";
    }
    document.addEventListener("visibilitychange", updateVisibility);
    return () => document.removeEventListener("visibilitychange", updateVisibility);
  });

  $effect(() => {
    painting = renderActive;
    visiblePanels = visible;
    for (const group of groups.values()) {
      group.reschedule();
    }
  });

  $effect(() => () => {
    for (const group of groups.values()) {
      group.dispose();
    }
    groups.clear();
    pending.clear();
  });

  $effect(() => {
    void requestKey;
    const nativeSession = session;
    const activeOptions = { enabled, requestScope, windowS, requestVisible, automatic, refreshKey, onRefreshChange, requestPlots };
    // The Stream Selection Is Keyed By Its Inputs, Independent Of Render Commits.
    untrack(() => reconcile(nativeSession, activeOptions));
  });

  // Only Plots Whose State Changed Reach Their Readers.
  function commit(update: (current: LivePlotStateMap) => LivePlotStateMap) {
    const previous = model;
    model = update(model);
    for (const [id, plot] of Object.entries(model)) {
      if (previous[id] !== plot) {
        states.set(id, plot);
      }
    }
    for (const id of Object.keys(previous)) {
      if (!Object.hasOwn(model, id)) {
        states.delete(id);
      }
    }
  }

  function reconcile(
    nativeSession: ConnectedSession | null,
    {
      enabled,
      requestScope,
      windowS,
      requestVisible,
      automatic,
      refreshKey,
      onRefreshChange,
      requestPlots,
    }: {
      enabled: boolean;
      requestScope: LivePlotOptions["scope"];
      windowS: number | null | undefined;
      requestVisible: boolean;
      automatic: boolean;
      refreshKey: number;
      onRefreshChange?: PlotRefreshState["onChange"];
      requestPlots: DocumentViewPlot[];
    },
  ) {
    if (enabled && !nativeSession) {
      throw new Error("Connect to a Backend Before Loading Live Plots");
    }

    const selectedPlots = shownPlots;
    const selectedGroups = shownGroups;
    const { windows, widths, heights } = measurements;
    const dataChanged = previousDataKey !== requestDataKey;
    if (dataChanged) {
      refreshFailures.clear();
    }
    previousDataKey = requestDataKey;
    const viewChanged = previousWindowKey !== scopeWindowKey;
    previousWindowKey = scopeWindowKey;

    const testCycleId = requestScope?.cycleId;
    if (!enabled || !testCycleId || selectedPlots.length === 0 || !requestVisible) {
      for (const group of groups.values()) {
        group.dispose();
      }
      groups.clear();
    }
    // A Paused Watch Keeps Every Retained Frame And Its Status For The Plot's Return.
    if (!enabled || !testCycleId || selectedPlots.length === 0) {
      onRefreshChange?.({ refreshing: false, updatedAt: null });
      commit((current) =>
        testCycleId ? Object.fromEntries(requestPlots.map((plot) => [plot.plot_id, current[plot.plot_id] ?? emptyLivePlotState()])) : {},
      );
      return;
    }
    if (!requestVisible) {
      onRefreshChange?.({ refreshing: false });
      return;
    }

    if (!nativeSession) {
      return;
    }

    const desired = new Map<string, { plots: DocumentViewPlot[]; requests: PlotRequest[] }>();
    for (const plots of selectedGroups) {
      const requests = plots.map((plot) => {
        const width = widths[plot.plot_id];
        const maximum = displayPointBudget(plot.display_points) || DEFAULT_PLOT_MAX_POINTS;
        return plotRequestForDocumentPlot(plot, {
          scope: requestScope,
          windowS,
          summaryWindow: windows[plot.plot_id] ?? null,
          // Widths Above The Point Ceiling Do Not Change The Served Selection.
          panelWidth: !automatic || (width && width * 4 < maximum) ? (width ?? undefined) : undefined,
          panelHeight: plot.plot_type === "contour" ? heights[plot.plot_id] : undefined,
        });
      });
      desired.set(cutKey([nativeSession.identity.generation, requests, refreshKey]), { plots, requests });
    }
    for (const [key, group] of groups) {
      if (desired.has(key)) {
        continue;
      }
      group.dispose();
      groups.delete(key);
    }
    if (dataChanged) {
      commit((current) => Object.fromEntries(Object.entries(current).filter(([id]) => requestPlots.some((plot) => plot.plot_id === id))));
    }
    for (const [key, group] of desired) {
      if (groups.has(key)) {
        continue;
      }
      const activePlots = group.plots;
      const requests = group.requests;
      let flushFrame: number | null = null;
      let flushTimer: number | null = null;
      const controller = new AbortController();
      let active = true;
      const streams = new Set<NativePlotStream>();
      const releases = new Set<() => void>();
      for (const plot of activePlots) {
        pending.add(plot.plot_id);
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
          pending.delete(plot.plot_id);
        }
        onRefreshChange?.({ refreshing: pending.size > 0 });
        const failure = failureOf(error, "Plot Refresh Failed");
        if (!refreshFailures.has(plotId)) {
          refreshFailures.add(plotId);
          toast.error(failure, { context: "background" });
        }
      };

      // A new watch numbers its frames from its own origin; a retained frame stays live until the host says otherwise.
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
                status: currentPlot.payload ? "live" : "loading",
                summaryInFlight: summaryInFlight("requested"),
              },
            ];
          }),
        ),
      }));

      const queued: LivePlotStateMap = {};
      let queuedUpdatedAt: number | null = null;
      const isVisible = (id: string) =>
        painting &&
        // Initial Payloads Complete Visible-Group Admission Before Off-Screen Updates Are Throttled.
        (typeof IntersectionObserver === "undefined" || visiblePanels[id] === true || !model[id]?.payload);
      const flush = (visible?: boolean) => {
        if (visible !== false) {
          flushFrame = null;
        }
        if (visible !== true) {
          flushTimer = null;
        }
        if (!active || !nativeSession.identity.isCurrent()) {
          return;
        }
        const updates: LivePlotStateMap = {};
        for (const [id, state] of Object.entries(queued)) {
          if (visible !== undefined && isVisible(id) !== visible) {
            continue;
          }
          updates[id] = state;
          delete queued[id];
        }
        if (Object.keys(updates).length > 0) {
          // Only Complete Revisions Reach The Visible Model, In One Shared Commit.
          commit((current) => ({ ...current, ...updates }));
        }
        if (queuedUpdatedAt !== null) {
          onRefreshChange?.({ refreshing: pending.size > 0, updatedAt: queuedUpdatedAt });
          queuedUpdatedAt = null;
        }
      };
      function clearScheduledFlush() {
        if (flushFrame !== null) {
          window.cancelAnimationFrame(flushFrame);
          flushFrame = null;
        }
        if (flushTimer !== null) {
          window.clearTimeout(flushTimer);
          flushTimer = null;
        }
      }

      const scheduleFlush = () => {
        const ids = Object.keys(queued);
        if (ids.some(isVisible) && flushFrame === null) {
          flushFrame = window.requestAnimationFrame(() => flush(true));
        }
        if (ids.some((id) => !isVisible(id)) && flushTimer === null) {
          flushTimer = window.setTimeout(() => flush(false), HIDDEN_PLOT_STREAM_FLUSH_MS);
        }
      };

      const enqueue = (messages: readonly PlotPayloadStreamMessage[]) => {
        if (!active || messages.length === 0) {
          return;
        }
        reopenDelayMs = RECONNECT_INITIAL_MS;
        const next = applyDocumentPlotMessages({ ...model, ...queued }, messages);
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
            refreshFailures.delete(message.plot_id);
            refreshFailures.delete("stream");
            received.add(message.plot_id);
            if (activePlots.every((plot) => received.has(plot.plot_id))) {
              awaitingRefresh = false;
              for (const plot of activePlots) {
                pending.delete(plot.plot_id);
              }
              queuedUpdatedAt = Date.now();
              received.clear();
            }
          }
        }
        scheduleFlush();
      };

      const markStreamEnded = () => {
        if (!active || !nativeSession.identity.isCurrent()) {
          return;
        }
        clearScheduledFlush();
        flush();
        if (awaitingRefresh) {
          reportRefreshFailure(new Error("The plot stream ended before the view finished refreshing."));
        }
        onRefreshChange?.({ refreshing: pending.size > 0 });
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
        clearScheduledFlush();
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
          reopen();
        }, reopenDelayMs);
        reopenDelayMs = Math.min(reopenDelayMs * 2, RECONNECT_MAX_MS);
      };

      const reopen = () => {
        // The reopened watch resends snapshots from its own revision origin.
        commit((current) => ({
          ...current,
          ...Object.fromEntries(activePlots.map((plot) => [plot.plot_id, { ...current[plot.plot_id], revision: 0 }])),
        }));
        void consumeStream();
      };

      // A Resync Or A Host Deadline Asks To Reopen At Once: Not A Failure, No Back-Off.
      const reopenAtOnce = () => {
        clearScheduledFlush();
        flush();
        reopenDelayMs = RECONNECT_INITIAL_MS;
        reopen();
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
          let resync = false;
          try {
            if (!active || controller.signal.aborted || !nativeSession.identity.isCurrent()) {
              return;
            }
            for await (const message of stream) {
              if (!active || controller.signal.aborted || !nativeSession.identity.isCurrent()) {
                break;
              }
              if (message.resync) {
                resync = true;
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
          if (resync && active && nativeSession.identity.isCurrent()) {
            reopenAtOnce();
            return;
          }
          markStreamEnded();
        } catch (exc: unknown) {
          if (isHostReopen(exc) && active && nativeSession.identity.isCurrent()) {
            reopenAtOnce();
          } else if (!isAbortError(exc)) {
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
      } else {
        void readSnapshot();
      }

      groups.set(key, {
        reschedule: () => {
          clearScheduledFlush();
          scheduleFlush();
        },
        dispose: () => {
          active = false;
          for (const plot of activePlots) {
            pending.delete(plot.plot_id);
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
          clearScheduledFlush();
        },
      });
    }
  }

  return { states: states as LivePlotStates };
}

function documentPlotRequestKey(plot: DocumentViewPlot): string {
  return cutKey({ ...plotRequestForDocumentPlot(plot, { scope: null }), maxPoints: displayPointBudget(plot.display_points) });
}

function isAbortError(value: unknown): boolean {
  return value instanceof DOMException && value.name === "AbortError";
}
