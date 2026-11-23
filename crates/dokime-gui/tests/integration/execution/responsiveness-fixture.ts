// Dokime — SPDX-License-Identifier: Apache-2.0
import { QueryClient } from "@tanstack/svelte-query";
import { mount, unmount } from "svelte";
import type { ConnectedSession } from "@/app/native-session/session";
import type { DocumentSurfaceOptions } from "@/features/cycle/document-view-model";
import type { RecordTail } from "@/lib/api/native-evidence";
import type { DocumentViewPlot, PlotPayload } from "@/lib/types";
import { emptyLivePlotState, type LivePlotStateMap } from "@/plots/live-frames";
import ResponsivenessWorkload from "./ResponsivenessWorkload.svelte";
import { appendPage, detailReads } from "./responsiveness-source";
import "../../../src/desktop.css";

const plots: DocumentViewPlot[] = Array.from({ length: 24 }, (_, index) => ({
  plot_id: `plot-${index}`,
  title: `Plot ${index}`,
  stream: "pulse",
  plot_type: "time_series",
  style: "line",
  x_scale: "linear",
  y_scale: "linear",
  x: "timestamp",
  y: ["value"],
  display_points: 2_000,
  fft: { window: "hann", detrend: "mean", level_unit: "linear", include_dc: false },
}));
const options: Omit<DocumentSurfaceOptions, "plotWindow"> = {
  document: {
    cards: [
      { card_id: "plots", kind: "plots", plots },
      { card_id: "records", kind: "records" },
    ],
    layout: { nodes: ["plots", "records"].map((card_id) => ({ type: "card", card_id, layout: { order: 0 } })) },
  } as DocumentSurfaceOptions["document"],
  execution: null,
  snapshot: null,
  noteElapsedStart: null,
  scope: { cycleId: "cycle" },
  live: true,
  parameters: { case: null, execution: null },
};
const client = new QueryClient();
const session = {
  identity: { generation: "1", isCurrent: () => true },
  own: () => () => {},
  queryClient: client,
} as unknown as ConnectedSession;

declare global {
  interface Window {
    responsiveness: {
      run: (mode: string) => Promise<Record<string, unknown>>;
      burst: (pages?: number) => Promise<Record<string, unknown>>;
      cancelPending: () => Promise<boolean>;
      scroll: () => Promise<Record<string, unknown>>;
    };
  }
}

function frame(revision: number): LivePlotStateMap {
  const x = Array.from({ length: 2_000 }, (_, index) => 1_800_000_000 + index);
  const y = x.map((_, index) => Math.sin(index / 20 + revision / 10));
  const payload = {
    stream: "pulse",
    x_key: "timestamp",
    generated_at: "2026-11-01T00:00:00Z",
    max_points: 2_000,
    window_s: 300,
    series: [
      { key: "value", label: "Value", points: x.map((at, index) => ({ x: at, y: y[index] })), raw_points: x.length, downsampled: false },
    ],
    cartesian: { mode: "aligned", series: [{ x, y }], point_count: x.length, signature: `frame-${revision}` },
  } as PlotPayload;
  return Object.fromEntries(plots.map((plot) => [plot.plot_id, { ...emptyLivePlotState(), status: "live", payload, revision }]));
}

async function settled() {
  for (let index = 0; index < 8; index += 1) await new Promise(requestAnimationFrame);
}

async function burst(pages = 20) {
  workload.setTicks(false);
  await settled();
  const previous = workload.currentView();
  let updates = 0;
  const unsubscribe = client.getQueryCache().subscribe((event) => {
    if (event.type === "updated" && event.action.type === "success" && event.query.queryKey.includes("records")) updates += 1;
  });
  const expected = Array.from({ length: pages }, appendPage);
  await settled();
  unsubscribe();
  const tail = client
    .getQueryCache()
    .findAll()
    .find((query) => query.queryKey.includes("records"))?.state.data as RecordTail;
  const latestView = workload.currentView();
  return {
    updates,
    expected,
    actual: tail.records.slice(-expected.length).map((record) => record.record_id),
    through: tail.cursor?.positions[0]?.sequence.toString(),
    stableProjection:
      previous.sections === latestView.sections &&
      previous.notes === latestView.notes &&
      previous.parameterContext === latestView.parameterContext,
  };
}

async function cancelPending() {
  const query = client
    .getQueryCache()
    .findAll()
    .find((query) => query.queryKey.includes("records"));
  const previous = query?.state.data;
  appendPage();
  for (let index = 0; index < 20; index += 1) await Promise.resolve();
  await unmount(workload);
  await settled();
  return query?.state.data === previous;
}

async function scroll() {
  workload.setModal(false);
  await settled();
  const target = document.querySelector<HTMLElement>(`[data-plot-viewport="${plots.at(-1)?.plot_id}"]`);
  if (!target) {
    throw new Error("Last plot is missing");
  }
  target.scrollIntoView({ block: "center" });
  await settled();
  const top = target.getBoundingClientRect().top;
  const height = document.documentElement.scrollHeight;
  const mounted = Boolean(target.querySelector("[data-live-mounted]"));
  workload.setStates(frame(999));
  await settled();
  return { mounted, top, nextTop: target.getBoundingClientRect().top, height, nextHeight: document.documentElement.scrollHeight };
}

async function run(mode: string) {
  workload.setTicks(mode === "ticks");
  workload.setModal(mode.endsWith("modal"));
  await settled();
  const overlay = document.querySelector<HTMLElement>('[data-slot="dialog-overlay"]');
  if (overlay) overlay.style.backdropFilter = mode === "blur-modal" ? "blur(4px)" : "";
  const controls = document.querySelectorAll<HTMLElement>(
    '[data-slot="button"], [data-slot="badge"], [data-slot="switch"], [data-slot="dropdown-menu-item"]',
  );
  for (const control of controls)
    control.style.transitionProperty = mode === "narrow-transitions" ? "color,background-color,border-color,opacity" : "";
  const input = document.querySelector<HTMLInputElement>('input[aria-label="Search Records"]');
  if (!input) {
    throw new Error("Record search is missing");
  }
  const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
  setter?.call(input, mode === "search" ? "absent*payload" : "");
  input.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();
  const frames: number[] = [];
  const longTasks: number[] = [];
  const observer = new PerformanceObserver((list) => {
    for (const entry of list.getEntries()) longTasks.push(entry.duration);
  });
  observer.observe({ type: "longtask" });
  let active = true;
  let previous = performance.now();
  const sample = (now: number) => {
    frames.push(now - previous);
    previous = now;
    if (active) requestAnimationFrame(sample);
  };
  requestAnimationFrame(sample);
  let records = 0;
  let revision = 0;
  const begin = performance.now();
  const readsBefore = detailReads;
  const recordTimer =
    mode === "ticks"
      ? undefined
      : window.setInterval(() => {
          appendPage();
          records += 1;
        }, 2);
  const plotTimer =
    mode === "ticks"
      ? undefined
      : window.setInterval(() => {
          workload.setStates(frame(++revision));
        }, 100);
  const transitionTimer = mode.includes("transitions")
    ? window.setInterval(() => {
        const button = document.getElementById("transition-probe") as HTMLButtonElement;
        button.disabled = !button.disabled;
      }, 100)
    : undefined;
  await new Promise((resolve) => setTimeout(resolve, 3_000));
  clearInterval(recordTimer);
  clearInterval(plotTimer);
  clearInterval(transitionTimer);
  await settled();
  active = false;
  observer.disconnect();
  const seconds = (performance.now() - begin) / 1_000;
  frames.sort((a, b) => a - b);
  return {
    mode,
    records,
    seconds,
    longTasksPerSecond: longTasks.length / seconds,
    frameP95: frames[Math.floor(frames.length * 0.95)],
    frameMean: frames.reduce((a, b) => a + b, 0) / frames.length,
    detailReads: detailReads - readsBefore,
    canvases: document.querySelectorAll("canvas").length,
    plotCount: plots.length,
    scrollHeight: document.documentElement.scrollHeight,
  };
}

const root = document.getElementById("root");
if (!root) {
  throw new Error("Responsiveness root is missing");
}
const workload = mount(ResponsivenessWorkload, {
  target: root,
  props: { session, plots, options, initialFrames: frame(0), synchronous: new URL(location.href).searchParams.has("synchronous") },
});
window.responsiveness = { run, burst, cancelPending, scroll };
