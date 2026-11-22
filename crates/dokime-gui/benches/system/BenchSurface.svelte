<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { EvidenceScope } from "@generated/host/model";
  import type { ConnectedSession } from "@/app/native-session/session";
  import { fetchNativeDocument } from "@/lib/api/native-document-session";
  import { mapNativeDocument } from "@/lib/api/native-documents";
  import type { Wire } from "@/lib/rpc/canonical";
  import type { DocumentViewPlot } from "@/lib/types";
  import { summaryWindowStore } from "@/stores/summary-window-store";
  import BenchPlots from "./BenchPlots.svelte";
  import { beginFrameObservation, canvasDraws, classify, observation } from "./observation";

  let { session }: { session: ConnectedSession } = $props();
  let selection = $state.raw<{ plots: DocumentViewPlot[]; scope: Wire<EvidenceScope>; key: number } | null>(null);
  let nextScene = 0;

  globalThis.desktopBegin = (condition) => {
    observation.previous = new Map(
      [...document.querySelectorAll<HTMLElement>("[data-bench-plot]")].map((card) => [
        card.dataset.benchPlot ?? "",
        card.dataset.benchRevision ?? "",
      ]),
    );
    observation.stages = {};
    observation.target = condition === "zoom" ? "desktop.plot.0" : null;
    if (condition === "resize") {
      observation.previous.clear();
    }
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
    beginFrameObservation(observation.started);
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
    if (!backend) {
      throw new Error("Read Benchmark Document: Connect Its Native Host");
    }
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
    summaryWindowStore.setState({ windows });
    selection = { plots, scope: { cycleId: cycle, executionId: execution }, key: observation.scene };
  };
  globalThis.desktopAction = (condition, from, to) => {
    globalThis.desktopBegin(condition);
    if (condition === "zoom") {
      summaryWindowStore.getState().setSummaryWindow("desktop.plot.0", { from, to });
    }
    if (condition === "navigation") {
      const retained = selection;
      const scene = ++nextScene;
      observation.scene = scene;
      observation.previous.clear();
      selection = null;
      setTimeout(() => {
        window.scrollTo(0, 0);
        selection = retained ? { ...retained, key: scene } : null;
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
    if (!card) {
      return { ready: false, error: false, drawn: false };
    }
    card.scrollIntoView({ block: "center" });
    const canvases = [...card.querySelectorAll("canvas")];
    const state = classify(card);
    return {
      ...state,
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
        if (performance.now() - cardStarted > deadlineMs) {
          throw new Error(`Draw Document ${plotId}: No Completed Drawing`);
        }
        const state = globalThis.desktopInspect(plotId);
        if (state.error) {
          throw new Error(`Draw Document: ${state.message}`);
        }
        if (state.ready && state.drawn) {
          opportunities++;
        } else {
          opportunities = 0;
        }
        if (opportunities >= 2) {
          break;
        }
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
</script>

<main data-bench-scene={selection?.key} class="grid grid-cols-2 gap-2 p-2" style:width="100%">
  {#if selection}
    {#key selection.key}
      <BenchPlots plots={selection.plots} scope={selection.scope} {session} />
    {/key}
  {/if}
</main>
