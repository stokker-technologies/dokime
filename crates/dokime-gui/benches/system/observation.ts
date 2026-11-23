// Dokime — SPDX-License-Identifier: Apache-2.0
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
  var desktopFrameAge: () => number;
}

export const observation: Observation = {
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
    if (!payload) {
      return 0;
    }
    const data = payload as { cartesian?: object; analysis?: object; series?: object };
    const source = data.cartesian ?? data.analysis ?? data.series ?? payload;
    const held = frameIdentities.get(source);
    if (held) {
      return held;
    }
    const identity = ++nextFrameIdentity;
    frameIdentities.set(source, identity);
    return identity;
  },
};
const frameIdentities = new WeakMap<object, number>();
let nextFrameIdentity = 0;
globalThis.__desktopBench = observation;

// Canvas Calls Are Observed at Their Real Owner, Including uPlot and Contour/Grid Drawing.
export const canvasDraws = new WeakMap<HTMLCanvasElement, number>();
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

export function beginFrameObservation(started: number) {
  previousFrame = started;
}

export function classify(card: HTMLElement) {
  const payload = card.querySelector('[data-bench-has-payload="true"]');
  const placeholder = card.querySelector('[role="alert"], .plot-empty');
  const error = card.dataset.benchError === "true" || (Boolean(payload) && Boolean(placeholder));
  const ready = Boolean(payload?.querySelector(".plot-card")) && !error && !placeholder;
  return {
    ready,
    error,
    message: error ? `${card.dataset.benchPlot}: ${placeholder?.textContent ?? "Plot Data Failed"}` : undefined,
  };
}

function presented(now: number) {
  if (now - previousFrame > 25) {
    observation.frameMisses += Math.max(1, Math.round((now - previousFrame) / (1000 / 60)) - 1);
  }
  previousFrame = now;
  if (document.querySelector<HTMLElement>("main")?.dataset.benchScene !== String(observation.scene)) {
    requestAnimationFrame(presented);
    return;
  }
  const cards = [...document.querySelectorAll<HTMLElement>("[data-bench-plot]")].filter((card) => {
    if (observation.target && card.dataset.benchPlot !== observation.target) {
      return false;
    }
    const bounds = card.getBoundingClientRect();
    return bounds.bottom > 0 && bounds.top < innerHeight && bounds.right > 0 && bounds.left < innerWidth;
  });
  const classified = cards.map((card) => ({ card, state: classify(card) }));
  const failures = classified.filter(({ state }) => state.error);
  observation.visibleFailures = failures.length;
  for (const { state } of failures) {
    const message = state.message;
    if (message && !observation.errors.includes(message)) {
      observation.errors.push(message);
    }
  }
  observation.visibleReady = classified.filter(({ state }) => state.ready).length;
  for (const { card, state } of classified) {
    if (!state.ready) {
      continue;
    }
    const canvases = [...card.querySelectorAll("canvas")];
    if (canvases.length > 0 && !canvases.some((canvas) => (canvasDraws.get(canvas) ?? 0) >= observation.started)) {
      continue;
    }
    const key = card.dataset.benchPlot ?? "";
    if (observation.previous.get(key) === card.dataset.benchRevision) {
      continue;
    }
    const opportunities = (observation.opportunities.get(key) ?? 0) + 1;
    observation.opportunities.set(key, opportunities);
    if (opportunities < 2) {
      continue;
    }
    if (!observation.painted.has(key)) {
      observation.painted.add(key);
      const elapsed = now - observation.started;
      observation.firstVisibleMs ??= elapsed;
      if (cards.length > 0 && cards.every((card) => observation.painted.has(card.dataset.benchPlot ?? ""))) {
        observation.allVisibleMs = elapsed;
      }
    }
  }
  requestAnimationFrame(presented);
}
requestAnimationFrame(presented);
// How Long Ago the Last Animation Frame Ran: A Growing Age Means the WebView Stopped Presenting Frames.
globalThis.desktopFrameAge = () => performance.now() - previousFrame;
