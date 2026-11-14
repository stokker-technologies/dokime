// Dokime — SPDX-License-Identifier: Apache-2.0
import { QueryClientProvider } from "@tanstack/react-query";
import { Profiler, useLayoutEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { Connection } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
import { NativeSession } from "@/app/native-session/session";
import { fetchNativeDocument, mapNativeDocument } from "@/lib/api/native-documents";
import { commands } from "@/lib/desktop/commands";
import { LivePlotGroupGrid } from "@/components/document/PlotGroupGrid";
import type { DocumentViewPlot, RecordScope } from "@/lib/types";
import { useLivePlotPayloads } from "@/plots/live-data";
import { useSummaryWindowStore } from "@/stores/summary-window-store";
import "@/styles.css";

type Observation = {
  stages: Record<string, number[]>;
  bytes: number;
  queueDepth: number;
  frameMisses: number;
  firstVisibleMs: number | null;
  allVisibleMs: number | null;
  errors: string[];
  loadedPlots: number;
  failedPlots: number;
  visibleFailures: number;
  visibleReady: number;
  target: string | null;
  scene: number;
  painted: Set<string>;
  previous: Map<string, string>;
  opportunities: Map<string, number>;
  started: number;
  stallUntil: number;
  sample(stage: string, ms: number): void;
  arrival(bytes: number, depth: number): void;
  frame(payload: object | null | undefined): number;
};
declare global {
  var __desktopBench: Observation;
  var desktopCase: (count: number, condition: string, execution: string, cycle: string, from: number, to: number) => Promise<void>;
  var desktopBegin: (condition: string) => void;
  var desktopAction: (condition: string, from: number, to: number) => void;
  var desktopResult: (statusOnly?: boolean) => object;
  var desktopInspect: (plotId: string) => { ready: boolean; error: boolean; drawn: boolean; message?: string };
  var desktopTraverse: (count: number, deadlineMs: number) => Promise<object>;
}

const observation: Observation = {
  stages: {},
  bytes: 0,
  queueDepth: 0,
  frameMisses: 0,
  firstVisibleMs: null,
  allVisibleMs: null,
  errors: [],
  loadedPlots: 0,
  failedPlots: 0,
  visibleFailures: 0,
  visibleReady: 0,
  target: null,
  scene: 0,
  painted: new Set(),
  previous: new Map(),
  opportunities: new Map(),
  started: performance.now(),
  stallUntil: 0,
  sample(stage, ms) {
    this.stages[stage] ??= [];
    this.stages[stage].push(ms);
  },
  arrival(bytes, depth) {
    this.bytes += bytes;
    this.queueDepth = Math.max(this.queueDepth, depth);
  },
  frame(payload) {
    if (!payload) return 0;
    const data = payload as { cartesian?: object; analysis?: object; series?: object };
    const source = data.cartesian ?? data.analysis ?? data.series ?? payload;
    const held = frameIdentities.get(source);
    if (held) return held;
    const identity = ++nextFrameIdentity;
    frameIdentities.set(source, identity);
    return identity;
  },
};
const frameIdentities = new WeakMap<object, number>();
let nextFrameIdentity = 0;
globalThis.__desktopBench = observation;

// Canvas Calls Are Observed At Their Real Owner, Including uPlot And Contour/Grid Drawing.
const canvasDraws = new WeakMap<HTMLCanvasElement, number>();
for (const method of [
  "stroke",
  "fill",
  "fillRect",
  "strokeRect",
  "clearRect",
  "fillText",
  "strokeText",
  "drawImage",
  "putImageData",
] as const) {
  const original = CanvasRenderingContext2D.prototype[method] as (...args: unknown[]) => unknown;
  Object.defineProperty(CanvasRenderingContext2D.prototype, method, {
    value: function (this: CanvasRenderingContext2D, ...args: unknown[]) {
      const started = performance.now();
      try {
        return original.apply(this, args);
      } finally {
        observation.sample("canvas_draw", performance.now() - started);
        canvasDraws.set(this.canvas, performance.now());
      }
    },
  });
}

let previousFrame = performance.now();
let nextScene = 0;
function presented(now: number) {
  if (now - previousFrame > 25) observation.frameMisses += Math.max(1, Math.round((now - previousFrame) / (1000 / 60)) - 1);
  previousFrame = now;
  if (document.querySelector<HTMLElement>("main")?.dataset.benchScene !== String(observation.scene)) {
    requestAnimationFrame(presented);
    return;
  }
  const cards = [...document.querySelectorAll<HTMLElement>("[data-bench-plot]")].filter((card) => {
    if (observation.target && card.dataset.benchPlot !== observation.target) return false;
    const bounds = card.getBoundingClientRect();
    return bounds.bottom > 0 && bounds.top < innerHeight && bounds.right > 0 && bounds.left < innerWidth;
  });
  const failures = cards.filter(
    (card) =>
      card.dataset.benchError === "true" ||
      (card.querySelector('[data-bench-has-payload="true"]') && card.querySelector('[role="alert"], .plot-empty')),
  );
  observation.visibleFailures = failures.length;
  for (const card of failures) {
    const message = `${card.dataset.benchPlot}: ${card.querySelector('[role="alert"], .plot-empty')?.textContent ?? "Plot Data Failed"}`;
    if (!observation.errors.includes(message)) observation.errors.push(message);
  }
  observation.visibleReady = cards.filter(
    (card) => card.querySelector('[data-bench-has-payload="true"] .plot-card') && card.dataset.benchError !== "true",
  ).length;
  for (const card of cards) {
    if (
      !card.querySelector('[data-bench-has-payload="true"] .plot-card') ||
      card.dataset.benchError === "true" ||
      card.querySelector('[role="alert"], .plot-empty')
    )
      continue;
    const canvases = [...card.querySelectorAll("canvas")];
    if (canvases.length > 0 && !canvases.some((canvas) => (canvasDraws.get(canvas) ?? 0) >= observation.started)) continue;
    const key = card.dataset.benchPlot ?? "";
    if (observation.previous.get(key) === card.dataset.benchRevision) continue;
    const opportunities = (observation.opportunities.get(key) ?? 0) + 1;
    observation.opportunities.set(key, opportunities);
    if (opportunities < 2) continue;
    if (!observation.painted.has(key)) {
      observation.painted.add(key);
      const elapsed = now - observation.started;
      observation.firstVisibleMs ??= elapsed;
      if (cards.length > 0 && cards.every((card) => observation.painted.has(card.dataset.benchPlot ?? "")))
        observation.allVisibleMs = elapsed;
    }
  }
  requestAnimationFrame(presented);
}
requestAnimationFrame(presented);

function Workspace({ plots, scope }: { plots: DocumentViewPlot[]; scope: RecordScope }) {
  const states = useLivePlotPayloads({ enabled: true, plots, scope, windowS: 259200 });
  useLayoutEffect(() => {
    const refused = (state: (typeof states)[string]) => state.status === "error" || Boolean(state.failure && !state.payload);
    observation.errors = Object.entries(states)
      .filter(([, state]) => refused(state))
      .map(([plotId, state]) => `${plotId}: ${state.error ?? state.failure?.message ?? "Plot Data Failed"}`);
    observation.loadedPlots = Object.values(states).filter((state) => state.payload && !refused(state)).length;
    observation.failedPlots = Object.values(states).filter(refused).length;
    const started = performance.now();
    requestAnimationFrame(() => requestAnimationFrame(() => observation.sample("presentation_opportunity", performance.now() - started)));
  }, [states]);
  return (
    <Profiler id="plots" onRender={(_id, _phase, duration) => observation.sample("frontend_state", duration)}>
      <LivePlotGroupGrid documentPlots={plots} liveStates={states} title="Mixed Plots" showGroupHeaders={false} />
    </Profiler>
  );
}

function Surface({ session }: { session: ConnectedSession }) {
  const [selection, setSelection] = useState<{ plots: DocumentViewPlot[]; scope: RecordScope; key: number } | null>(null);
  globalThis.desktopBegin = (condition) => {
    observation.previous = new Map(
      [...document.querySelectorAll<HTMLElement>("[data-bench-plot]")].map((card) => [
        card.dataset.benchPlot ?? "",
        card.dataset.benchRevision ?? "",
      ]),
    );
    observation.stages = {};
    observation.target = condition === "zoom" ? "desktop.plot.0" : null;
    if (condition === "resize") observation.previous.clear();
    observation.bytes = 0;
    observation.queueDepth = 0;
    observation.frameMisses = 0;
    observation.firstVisibleMs = null;
    observation.allVisibleMs = null;
    observation.painted.clear();
    observation.opportunities.clear();
    observation.visibleFailures = 0;
    observation.visibleReady = 0;
    observation.started = performance.now();
    previousFrame = observation.started;
    observation.stallUntil = condition === "stalled" ? performance.now() + 1500 : 0;
  };
  globalThis.desktopCase = async (count, condition, execution, cycle, from, to) => {
    window.scrollTo(0, 0);
    globalThis.desktopBegin(condition);
    observation.errors = [];
    observation.loadedPlots = 0;
    observation.failedPlots = 0;
    observation.scene = ++nextScene;
    observation.previous.clear();
    const backend = session.snapshot.connection.backend;
    if (!backend) throw new Error("Read Benchmark Document: Connect Its Native Host");
    const document = await fetchNativeDocument(
      { ...session.identity, own: session.own },
      {
        cycleId: cycle,
        executionId: execution,
        documentId: `desktop-${count}`,
        epoch: backend.epoch,
      },
    );
    const plots = mapNativeDocument(document).cards.flatMap((card) => card.plots);
    const windows = Object.fromEntries(
      plots.map((plot) => [
        plot.plot_id,
        plot.plot_type === "time_series" || plot.plot_type === "frequency_series" ? { from, to } : { from: 0, to: 100 },
      ]),
    );
    useSummaryWindowStore.setState({ windows });
    setSelection({ plots, scope: { test_cycle_id: cycle, test_execution_id: execution }, key: observation.scene });
  };
  globalThis.desktopAction = (condition, from, to) => {
    globalThis.desktopBegin(condition);
    if (condition === "zoom") useSummaryWindowStore.getState().setSummaryWindow("desktop.plot.0", { from, to });
    if (condition === "navigation") {
      const retained = selection;
      const scene = ++nextScene;
      observation.scene = scene;
      observation.previous.clear();
      setSelection(null);
      setTimeout(() => {
        window.scrollTo(0, 0);
        setSelection(retained ? { ...retained, key: scene } : null);
      }, 0);
    }
  };
  globalThis.desktopResult = (statusOnly = false) => ({
    errors: observation.errors,
    loadedPlots: observation.loadedPlots,
    failedPlots: observation.failedPlots,
    visibleFailures: observation.visibleFailures,
    visibleReady: observation.visibleReady,
    paintedCount: observation.painted.size,
    target: observation.target,
    firstVisibleMs: observation.firstVisibleMs,
    allVisibleMs: observation.allVisibleMs,
    ...(statusOnly
      ? {}
      : {
          stages: observation.stages,
          bytes: observation.bytes,
          queueDepth: observation.queueDepth,
          frameMisses: observation.frameMisses,
          painted: [...observation.painted],
          elapsedMs: performance.now() - observation.started,
        }),
  });
  globalThis.desktopInspect = (plotId) => {
    const card = [...document.querySelectorAll<HTMLElement>("[data-bench-plot]")].find((card) => card.dataset.benchPlot === plotId);
    if (!card) return { ready: false, error: false, drawn: false };
    card.scrollIntoView({ block: "center" });
    const canvases = [...card.querySelectorAll("canvas")];
    const placeholder = card.querySelector('[role="alert"], .plot-empty');
    const rendered = card.querySelector('[data-bench-has-payload="true"]');
    const error = card.dataset.benchError === "true" || (Boolean(rendered) && Boolean(placeholder));
    return {
      ready: Boolean(rendered?.querySelector(".plot-card")) && !placeholder,
      error,
      message: error ? `${plotId}: ${placeholder?.textContent ?? "Plot Data Failed"}` : undefined,
      drawn: canvases.length === 0 || canvases.some((canvas) => canvasDraws.has(canvas)),
    };
  };
  globalThis.desktopTraverse = async (count, deadlineMs) => {
    const started = performance.now();
    const stageOffsets = Object.fromEntries(Object.entries(observation.stages).map(([stage, values]) => [stage, values.length]));
    const cards = [];
    for (let index = 0; index < count; index++) {
      const plotId = `desktop.plot.${index}`;
      const cardStarted = performance.now();
      let opportunities = 0;
      while (true) {
        if (performance.now() - cardStarted > deadlineMs) throw new Error(`Draw Document ${plotId}: No Completed Drawing`);
        const state = globalThis.desktopInspect(plotId);
        if (state.error) throw new Error(`Draw Document: ${state.message}`);
        if (state.ready && state.drawn) opportunities++;
        else opportunities = 0;
        if (opportunities >= 2) break;
        await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
      }
      cards.push({ plotId, waitMs: performance.now() - cardStarted });
    }
    return {
      verifiedPlots: count,
      traversalMs: performance.now() - started,
      traversalCards: cards,
      traversalStages: Object.fromEntries(
        Object.entries(observation.stages).map(([stage, values]) => [stage, values.slice(stageOffsets[stage] ?? 0)]),
      ),
    };
  };
  return (
    <Connection.Provider value={session}>
      <QueryClientProvider client={session.queryClient}>
        <main data-bench-scene={selection?.key} className="grid grid-cols-2 gap-2 p-2" style={{ width: "100%" }}>
          {selection && <Workspace key={selection.key} plots={selection.plots} scope={selection.scope} />}
        </main>
      </QueryClientProvider>
    </Connection.Provider>
  );
}

const owner = new NativeSession();
const launch = await commands.desktopSessionLaunch();
if (!launch.target) throw new Error("The benchmark requires its owned fixture endpoint");
await owner.select({ kind: "attach", request: { target: launch.target, expectedBackendId: launch.expectedBackendId } });
const connected = owner.snapshot();
if (connected.phase !== "connected") throw new Error(`Native fixture connection failed: ${JSON.stringify(connected)}`);
const container = document.getElementById("root");
if (!container) throw new Error("The benchmark entry point is missing its root");
createRoot(container).render(<Surface session={connected.session} />);
