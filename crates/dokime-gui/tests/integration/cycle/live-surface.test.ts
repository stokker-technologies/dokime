// @vitest-environment happy-dom
/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Live Updates Reach Only What They Change
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { notifyManager, QueriesObserver, QueryClient, QueryObserver } from "@tanstack/svelte-query";
import { render } from "@testing-library/svelte";
import { tick } from "svelte";
import { onTestFinished, test, vi } from "vitest";

// Local Imports
import type { ConnectedSession } from "@/app/native-session/session";
import { createPlotWindow } from "@/features/cycle/plot-window-state";
import type { NativePlotStreamMessage } from "@/lib/api/native-plots";
import { documentCard, documentView } from "@/lib/document-presentation";
import type { DocumentViewCard, DocumentViewPayload, DocumentViewPlot, HubLiveState, PlotPayload, TestExecution } from "@/lib/types";
import { documentPlotSpec } from "@/plots/grouping";
import { BUILT_IN_REPORT_DEFAULTS } from "@/plots/plot-settings";
import LiveSurfaceProbe from "./LiveSurfaceProbe.svelte";

/* === Constants === */

const backend = vi.hoisted(() => ({ fetch: vi.fn(), watch: vi.fn() }));
vi.mock("@/lib/api/native-document-session", () => ({ fetchNativeDocument: (...args: unknown[]) => backend.fetch(...args) }));
vi.mock("@/lib/api/native-documents", () => ({
  mapNativeDocument: (capture: { view: DocumentViewPayload }) => capture.view,
  mapNativeDocumentCard: () => undefined,
  nativeDocumentCardKey: (_capture: unknown, id: string) => id,
}));
vi.mock("@/lib/api/native-annotations", () => ({ fetchNativeNotes: async () => [], fetchNativeHealth: async () => [] }));
vi.mock("@/lib/api/native-content", () => ({ fetchNativeExecutionFiles: async () => [] }));
vi.mock("@/lib/api/native-observation", () => ({ fetchNativeTraceability: async () => null }));
vi.mock("@/lib/api/native-plots", async (original) => ({
  ...(await original<typeof import("@/lib/api/native-plots")>()),
  openNativePlotStream: (...args: unknown[]) => backend.watch(...args),
}));
vi.mock("@/plots/grouping", async (original) => {
  const actual = await original<typeof import("@/plots/grouping")>();
  return { ...actual, documentPlotSpec: vi.fn(actual.documentPlotSpec) };
});

/** The Viewport Of Every Desktop Panel: Panels Share One Observer, Created Once Per Module. */
const viewport = { notify: (() => {}) as IntersectionObserverCallback, panels: new Set<Element>() };

/* === Classes === */

class ViewportObserver {
  constructor(callback: IntersectionObserverCallback) {
    viewport.notify = callback;
  }
  observe(target: Element) {
    viewport.panels.add(target);
  }
  unobserve(target: Element) {
    viewport.panels.delete(target);
  }
}

/* === Private Functions === */

function connect() {
  notifyManager.setScheduler(queueMicrotask);
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const session = {
    identity: { generation: "1", isCurrent: () => true },
    own: () => () => {},
    snapshot: { connection: { backend: { epoch: "epoch" } } },
    queryClient,
  } as unknown as ConnectedSession;
  return { queryClient, session };
}

/** Each Read Of The Structure Returns Equal Content In New Objects. */
function serve(cards: DocumentViewCard[]) {
  const view = documentView("default", { identifier: "default", label: "Fixture" }, cards, {
    nodes: cards.map((card) => ({ type: "card" as const, card_id: card.card_id, layout: card.layout })),
  });
  backend.fetch.mockImplementation(async (_session, _request, _signal, _previous, publish) => {
    const capture = { view: structuredClone(view), bindings: new Map() };
    publish?.(capture);
    return capture;
  });
}

function plotStream() {
  const messages: NativePlotStreamMessage[] = [];
  let wake: (() => void) | undefined;
  let ended = false;
  const end = () => {
    ended = true;
    wake?.();
  };
  return {
    push(message: NativePlotStreamMessage) {
      messages.push(message);
      wake?.();
    },
    /** The Host Ends The Watch. */
    end,
    cancel: vi.fn(async () => end()),
    async *[Symbol.asyncIterator]() {
      while (!ended) {
        const message = messages.shift();
        if (message) {
          yield message;
        } else {
          await new Promise<void>((resolve) => {
            wake = resolve;
          });
        }
      }
    },
  };
}

function frame(plot_id: string, revision: number, kind: "snapshot" | "replace", x = [0, 1]) {
  const payload = {
    stream: "pulse",
    x_key: "timestamp",
    generated_at: "2026-11-01T00:00:00Z",
    max_points: 100,
    window_s: 300,
    series: [{ key: "trace-0", label: "Trace", points: [], raw_points: x.length, downsampled: false }],
    cartesian: {
      mode: "aligned",
      series: [{ x, y: x.map(() => revision) }],
      point_count: x.length,
      signature: `${plot_id}-${revision}`,
    },
  } as unknown as PlotPayload;
  return {
    kind,
    revision,
    plot_id,
    epoch: "epoch",
    selection: plot_id,
    payload,
    warnings: [],
  } as NativePlotStreamMessage["updates"][number];
}

/** A Running Execution's Desktop Document Whose Plots Scroll In And Out Of View. */
async function scrollingSurface(kinds: Record<string, DocumentViewPlot["plot_type"]>) {
  vi.useFakeTimers();
  vi.stubGlobal("IntersectionObserver", ViewportObserver);
  let page: DocumentVisibilityState = "visible";
  Object.defineProperty(document, "visibilityState", { configurable: true, get: () => page });
  vi.spyOn(window, "requestAnimationFrame").mockImplementation((callback) => setTimeout(() => callback(0), 16) as unknown as number);
  vi.spyOn(window, "cancelAnimationFrame").mockImplementation((handle) => clearTimeout(handle));
  const plots = Object.entries(kinds).map(
    ([plot_id, plot_type]): DocumentViewPlot => ({
      plot_id,
      title: plot_id,
      stream: "pulse",
      plot_type,
      style: "line",
      x_scale: "linear",
      y_scale: "linear",
      x: "timestamp",
      y: ["trace-0"],
      fft: { window: "hann", detrend: "mean", level_unit: "linear", include_dc: false },
    }),
  );
  serve([documentCard({ identifier: "plots", label: "Plots" }, "plots", { plots })]);
  const watches: { requests: { plotId: string; maxPoints?: bigint; panelWidth?: number }[]; stream: ReturnType<typeof plotStream> }[] = [];
  backend.watch.mockImplementation(async (_identity, requests) => {
    const stream = plotStream();
    watches.push({ requests, stream });
    return stream;
  });
  const { session } = connect();
  const desktop = { session, plotPalette: ["#123456"], reportDefaults: BUILT_IN_REPORT_DEFAULTS };
  const probe = render(LiveSurfaceProbe, { session, plotWindow: createPlotWindow(), desktop });
  onTestFinished(() => {
    probe.unmount();
    Reflect.deleteProperty(document, "visibilityState");
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });
  // Settling Runs Pending Work Without Letting Any Time Pass.
  const settle = async () => {
    await tick();
    await vi.advanceTimersByTimeAsync(0);
  };
  await vi.waitFor(() => assert.equal(viewport.panels.size, plots.length));
  return {
    probe,
    settle,
    watches,
    watch: (plot: string) => watches.findLast((entry) => entry.requests.some((request) => request.plotId === plot))?.stream,
    shown: (plot: string) => probe.container.querySelector(`[data-plot-viewport="${plot}"]`)?.textContent?.replace(/\s+/g, " "),
    async scroll(visible: Record<string, boolean>) {
      const entries = [...viewport.panels].flatMap((target) => {
        const plot = (target as HTMLElement).dataset.plotViewport ?? "";
        return plot in visible ? [{ target, isIntersecting: visible[plot] } as IntersectionObserverEntry] : [];
      });
      viewport.notify(entries, {} as IntersectionObserver);
      await settle();
    },
    async turnPage(state: DocumentVisibilityState) {
      page = state;
      document.dispatchEvent(new Event("visibilitychange"));
      await settle();
    },
  };
}

/* === Entry Point === */

// A Refresh Status, A Live Snapshot, An Equal Scope Or An Unchanged Structure Never Re-Runs A Query's Options.
test("Live Updates Keep Every Query's Options", async () => {
  const { queryClient, session } = connect();
  serve(
    ["notes", "health", "traceability", "artifacts", "records"].map((kind) =>
      documentCard({ identifier: kind, label: kind }, kind as "notes", {}),
    ),
  );
  const setOptions = vi.spyOn(QueryObserver.prototype, "setOptions");
  const setQueries = vi.spyOn(QueriesObserver.prototype, "setQueries");
  onTestFinished(() => {
    vi.restoreAllMocks();
  });
  const plotWindow = createPlotWindow();
  const probe = render(LiveSurfaceProbe, { session, plotWindow });
  await vi.waitFor(() =>
    assert.ok(
      queryClient
        .getQueryCache()
        .getAll()
        .every((query) => query.state.status === "success"),
    ),
  );
  await tick();
  const queries = queryClient.getQueryCache().getAll().length;
  assert.equal(queries, 6, "the document and its five evidence queries");
  setOptions.mockClear();
  setQueries.mockClear();
  const stream = { name: "pulse" } as unknown as HubLiveState["telemetry"][string];
  const sections = probe.component.sections();
  const changes = {
    refresh: async () => plotWindow.getState().refresh.onChange({ refreshing: false, updatedAt: 1 }),
    snapshot: async () => probe.component.setSnapshot({ telemetry: { pulse: stream } } as unknown as HubLiveState),
    scope: async () => probe.component.setScope({ cycleId: "cycle", executionId: "execution" }),
    structure: () => queryClient.refetchQueries({ queryKey: ["workflow", "documents"] }),
  };
  for (const [name, change] of Object.entries(changes)) {
    await change();
    await tick();
    assert.deepEqual(
      setOptions.mock.calls.map(([options]) => options.queryKey),
      [],
      `${name} keeps every query's options`,
    );
    assert.equal(setQueries.mock.calls.length, 0, `${name} keeps the card queries`);
  }
  assert.deepEqual(probe.component.streams(), [stream]);
  assert.equal(probe.component.sections(), sections, "an unchanged structure keeps its sections");
  assert.ok(backend.fetch.mock.calls.length > 1);
  assert.equal(queryClient.getQueryCache().getAll().length, queries);
  probe.unmount();
});

// The Desktop Document Rebuilds A Plot Only When Its Own Inputs Change.
test("A Live Revision Rebuilds Only Its Own Plot", async () => {
  vi.useFakeTimers();
  vi.stubGlobal("IntersectionObserver", undefined);
  vi.spyOn(window, "requestAnimationFrame").mockImplementation((callback) => setTimeout(() => callback(0), 16) as unknown as number);
  vi.spyOn(window, "cancelAnimationFrame").mockImplementation((handle) => clearTimeout(handle));
  onTestFinished(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });
  const { session } = connect();
  const plots = ["alpha", "beta"].map(
    (plot_id): DocumentViewPlot => ({
      plot_id,
      title: plot_id,
      stream: "pulse",
      plot_type: "time_series",
      style: "line",
      x_scale: "linear",
      y_scale: "linear",
      x: "timestamp",
      y: ["trace-0"],
      fft: { window: "hann", detrend: "mean", level_unit: "linear", include_dc: false },
    }),
  );
  serve([documentCard({ identifier: "plots", label: "Plots" }, "plots", { plots })]);
  const streams: ReturnType<typeof plotStream>[] = [];
  backend.watch.mockImplementation(async () => {
    const stream = plotStream();
    streams.push(stream);
    return stream;
  });
  const plotWindow = createPlotWindow();
  const desktop = { session, plotPalette: ["#123456"], reportDefaults: BUILT_IN_REPORT_DEFAULTS };
  const probe = render(LiveSurfaceProbe, { session, plotWindow, desktop });
  await vi.waitFor(() => assert.equal(streams.length, 1));
  streams[0].push({ updates: plots.map((plot) => frame(plot.plot_id, 1, "snapshot")) });
  await vi.advanceTimersByTimeAsync(20);
  await tick();
  assert.equal(probe.container.querySelectorAll("[data-live-mounted]").length, 2, "both plots draw");

  const spec = vi.mocked(documentPlotSpec);
  spec.mockClear();
  plotWindow.getState().refresh.onChange({ refreshing: false, updatedAt: 9 });
  probe.component.setSnapshot({ telemetry: { other: { stream: "other" } } } as unknown as HubLiveState);
  await tick();
  assert.equal(spec.mock.calls.length, 0, "a refresh status and an unplotted stream rebuild no plot");

  streams[0].push({ updates: [frame("alpha", 2, "replace")] });
  await vi.advanceTimersByTimeAsync(20);
  await tick();
  assert.deepEqual(
    spec.mock.calls.map(([input]) => input.plot.plot_id),
    ["alpha"],
  );
  probe.unmount();
});

// Scrolling Back Draws A Retained Frame At Once; Leaving And Returning Never Reads As A Stream Fault.
test("Scrolling Back Redraws the Last Frame at Once and Reopens Every Watch a Running Execution Needs", async () => {
  const { probe, scroll, settle, shown, turnPage, watch, watches } = await scrollingSurface({ alpha: "time_series", beta: "xy" });
  await scroll({ alpha: true, beta: true });
  assert.equal(watches.length, 2, "each plot kind holds its own watch");
  assert.match(shown("alpha") ?? "", /Loading Plot/, "a plot without any frame yet is loading");

  watch("alpha")?.push({ updates: [frame("alpha", 1, "snapshot")] });
  watch("beta")?.push({ updates: [frame("beta", 1, "snapshot", [])] });
  await vi.advanceTimersByTimeAsync(20);
  assert.match(shown("alpha") ?? "", /2 Points/);
  assert.match(shown("beta") ?? "", /Waiting for Samples/);

  // Scrolling Away Closes Alpha's Watch; The Page Leaving The Screen Closes Every Watch.
  await scroll({ alpha: false });
  assert.equal(watch("alpha")?.cancel.mock.calls.length, 1, "an off-screen plot holds no watch");
  await turnPage("hidden");
  assert.equal(watch("beta")?.cancel.mock.calls.length, 1, "a hidden page holds no watch");
  await turnPage("visible");
  assert.equal(watches.length, 3, "the shown plot reopens its watch");
  assert.match(shown("beta") ?? "", /Waiting for Samples/, "a routine reopen is no stream fault");

  // The Returning Plot Draws Its Retained Frame Before Its Reopened Watch Sends Anything.
  const spec = vi.mocked(documentPlotSpec);
  spec.mockClear();
  await scroll({ alpha: true });
  assert.equal(watches.length, 4, "the returning plot reopens its watch");
  assert.match(shown("alpha") ?? "", /2 Points/, "the retained frame is drawn with no time elapsed");
  assert.doesNotMatch(shown("alpha") ?? "", /Loading/);
  assert.deepEqual(new Set(spec.mock.calls.map(([input]) => input.plot.plot_id)), new Set(["alpha"]), "only the returning plot rebuilds");

  // A Watch The Host Ends Reopens While The Execution Runs And Stays Closed Once It Has Finished.
  watch("beta")?.push({ updates: [frame("beta", 1, "snapshot", [])] });
  await vi.advanceTimersByTimeAsync(20);
  watch("beta")?.end();
  await vi.advanceTimersByTimeAsync(1_000);
  assert.equal(watches.length, 5, "a running execution reopens the watch the host ended");
  probe.component.setExecution({ status: "passed", steps: [] } as unknown as TestExecution);
  watch("beta")?.push({ updates: [frame("beta", 1, "snapshot", [])] });
  await vi.advanceTimersByTimeAsync(20);
  watch("beta")?.end();
  await vi.advanceTimersByTimeAsync(60_000);
  await settle();
  assert.equal(watches.length, 5, "a finished execution's watch stays closed");
  assert.match(shown("beta") ?? "", /Waiting for Samples/, "a final frame reads as final");
});

// The Desktop Measures Every Card Before It Mounts, So A First Scroll-In Changes No Request.
test("A Plot Scrolling In for the First Time Joins Its Group's Running Watch", async () => {
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (this: HTMLElement) {
    const width = this.dataset.plotViewport || this.dataset.plotId ? 320 : 0;
    return { width, height: 0, x: 0, y: 0, top: 0, left: 0, right: width, bottom: 0, toJSON: () => ({}) };
  });
  const { scroll, shown, watches } = await scrollingSurface({ alpha: "time_series", beta: "time_series" });
  await scroll({ alpha: true, beta: false });
  assert.equal(watches.length, 1, "plots of one kind share a watch");
  assert.deepEqual(
    watches[0].requests.map((request) => [request.maxPoints, request.panelWidth]),
    [
      [4n * 320n, undefined],
      [4n * 320n, undefined],
    ],
    "the watch opens sized to every plot's width",
  );
  watches[0].stream.push({ updates: ["alpha", "beta"].map((plot) => frame(plot, 1, "snapshot")) });
  await vi.advanceTimersByTimeAsync(1_000);
  await scroll({ beta: true });
  assert.equal(watches.length, 1, "a first scroll-in reopens no watch");
  assert.equal(watches[0].stream.cancel.mock.calls.length, 0);
  assert.match(shown("beta") ?? "", /2 Points/, "the plot draws the frame its group already served");
});
