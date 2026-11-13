/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/** Repeatable High-Rate Records And Many-Plot Browser Workload. */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Profiler, useState } from "react";
import { createRoot } from "react-dom/client";

// Local Imports
import { Connection } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
import { LivePlotGroupGrid } from "@/components/document/PlotGroupGrid";
import { RecordTimeline } from "@/components/document/RecordTimeline";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { ExecutionStepsCriteria } from "@/features/cycle/ExecutionStepsCriteria";
import { ExecutionSectionsView } from "@/features/cycle/ExecutionSectionsView";
import { usePlotWindow } from "@/features/cycle/PlotWindowSlider";
import { useDocumentView } from "@/features/cycle/use-document-view";
import type { DocumentViewOptions } from "@/features/cycle/document-view-model";
import { documentSurface } from "@/features/cycle/document-view-model";
import type { DocumentViewPlot, PlotPayload, StepExecutionRecord, TestExecution } from "@/lib/types";
import { emptyLivePlotState, type LivePlotStateMap } from "@/plots/live-frames";
import { appendPage, detailReads } from "./responsiveness-source";
import type { RecordTail } from "@/lib/api/native-evidence";
import "@/styles.css";

/* === Re-Exports === */

/* === Constants === */

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
const options: DocumentViewOptions = {
  document: {
    cards: [
      { card_id: "plots", kind: "plots", plots },
      { card_id: "records", kind: "records" },
    ],
    layout: { nodes: ["plots", "records"].map((card_id) => ({ type: "card", card_id, layout: { order: 0 } })) },
  } as DocumentViewOptions["document"],
  execution: null,
  liveSnapshot: null,
  noteElapsedStart: null,
  scope: { test_cycle_id: "cycle" },
  live: true,
  parameters: { case: null, execution: null },
};
const session = { identity: { generation: "1", isCurrent: () => true }, own: () => () => {} } as unknown as ConnectedSession;
const steps = [
  {
    record_type: "step",
    step_id: "capture",
    order: 0,
    label: "Capture",
    status: "running",
    metadata: { label: "Capture" },
    started_at: new Date().toISOString(),
    result: { details: {} },
  },
] as unknown as StepExecutionRecord[];
const counts: Record<string, number> = {};
const durations: Record<string, number> = {};
let setStates: (states: LivePlotStateMap) => void;
let setModal: (open: boolean) => void;
let setTicks: (ticks: boolean) => void;
let viewRenders = 0;
let latestView: ReturnType<typeof useDocumentView>;
const client = new QueryClient();

/* === Types === */

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

/* === Components === */

function Workload() {
  const view = useDocumentView(options);
  latestView = view;
  viewRenders += 1;
  const [states, updateStates] = useState(frame(0));
  const [modal, updateModal] = useState(false);
  const [ticks, updateTicks] = useState(false);
  setStates = updateStates;
  setModal = updateModal;
  setTicks = updateTicks;
  if (ticks) return <TickWorkload view={view} states={states} />;
  return (
    <>
      <Profiler id="steps" onRender={profile}>
        <ExecutionStepsCriteria steps={steps} criteria={[]} />
      </Profiler>
      <div style={{ height: 180, overflow: "auto" }}>
        <Profiler id="records" onRender={profile}>
          <RecordTimeline records={view.records} />
        </Profiler>
      </div>
      <div className="grid grid-cols-2 gap-2">
        <Profiler id="plots" onRender={profile}>
          <LivePlotGroupGrid
            documentPlots={plots}
            liveStates={states}
            records={view.records}
            title="Measurements"
            showGroupHeaders={false}
          />
        </Profiler>
      </div>
      <Button id="transition-probe" style={{ position: "fixed", right: 0, bottom: 0 }}>
        Refresh
      </Button>
      <Dialog open={modal} onOpenChange={updateModal}>
        <DialogContent>
          <DialogTitle>Measurements</DialogTitle>
        </DialogContent>
      </Dialog>
    </>
  );
}

function TickWorkload({ view, states }: { view: ReturnType<typeof useDocumentView>; states: LivePlotStateMap }) {
  const plotWindow = usePlotWindow();
  const execution = { status: "running", started_at: steps[0].started_at, finished_at: null, steps } as TestExecution;
  const surface = documentSurface({ ...options, snapshot: null, plotWindow: plotWindow.option }, view, false);
  return (
    <Profiler id="page" onRender={profile}>
      <ExecutionSectionsView
        plan={null}
        row={{
          key: "execution",
          case: { caseId: "case", templateId: "template" } as never,
          execution,
          ordinal: 1,
          rerunOf: null,
          newest: true,
        }}
        live
        label="Measurement"
        titlePrefix=""
        trail={["Cycle", "Measurement"]}
        onRename={null}
        started={execution}
        surface={surface}
        pane={{ kind: "document", staleReason: null }}
        plotWindow={plotWindow}
        documentView={() => (
          <Profiler id="plots" onRender={profile}>
            <LivePlotGroupGrid documentPlots={plots} liveStates={states} records={view.records} title="Measurements" />
          </Profiler>
        )}
      />
    </Profiler>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */

function profile(id: string, _phase: string, duration: number) {
  counts[id] = (counts[id] ?? 0) + 1;
  durations[id] = (durations[id] ?? 0) + duration;
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
  setTicks(false);
  await settled();
  const previous = latestView;
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
  reactRoot.unmount();
  await settled();
  return query?.state.data === previous;
}

async function scroll() {
  setModal(false);
  await settled();
  const target = document.querySelector<HTMLElement>(`[data-plot-viewport="${plots.at(-1)?.plot_id}"]`);
  if (!target) throw new Error("Last plot is missing");
  target.scrollIntoView({ block: "center" });
  await settled();
  const top = target.getBoundingClientRect().top;
  const height = document.documentElement.scrollHeight;
  const mounted = Boolean(target.querySelector("[data-live-mounted]"));
  setStates(frame(999));
  await settled();
  return { mounted, top, nextTop: target.getBoundingClientRect().top, height, nextHeight: document.documentElement.scrollHeight };
}

async function run(mode: string) {
  setTicks(mode === "ticks");
  setModal(mode.endsWith("modal"));
  await settled();
  const overlay = document.querySelector<HTMLElement>('[data-slot="dialog-overlay"]');
  if (overlay) overlay.style.backdropFilter = mode === "blur-modal" ? "blur(4px)" : "";
  const controls = document.querySelectorAll<HTMLElement>(
    '[data-slot="button"], [data-slot="badge"], [data-slot="switch"], [data-slot="dropdown-menu-item"]',
  );
  for (const control of controls)
    control.style.transitionProperty = mode === "narrow-transitions" ? "color,background-color,border-color,opacity" : "";
  const input = document.querySelector<HTMLInputElement>('input[aria-label="Search Records"]');
  if (!input) throw new Error("Record search is missing");
  const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
  setter?.call(input, mode === "search" ? "absent*payload" : "");
  input.dispatchEvent(new Event("input", { bubbles: true }));
  await settled();
  for (const key of Object.keys(counts)) {
    counts[key] = 0;
    durations[key] = 0;
  }
  viewRenders = 0;
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
          setStates(frame(++revision));
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
    viewRendersPerRecord: viewRenders / records,
    detailReads: detailReads - readsBefore,
    counts: { ...counts },
    renderMs: { ...durations },
    canvases: document.querySelectorAll("canvas").length,
    plotCount: plots.length,
    scrollHeight: document.documentElement.scrollHeight,
  };
}

/* === Entry Point === */

const root = document.getElementById("root");
if (!root) throw new Error("Responsiveness root is missing");
const reactRoot = createRoot(root);
reactRoot.render(
  <QueryClientProvider client={client}>
    <Connection.Provider value={session}>
      <Workload />
    </Connection.Provider>
  </QueryClientProvider>,
);
window.responsiveness = { run, burst, cancelPending, scroll };
