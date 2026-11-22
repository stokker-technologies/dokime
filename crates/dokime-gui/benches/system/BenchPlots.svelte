<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { EvidenceScope } from "@generated/host/model";
  import { tick } from "svelte";
  import type { ConnectedSession } from "@/app/native-session/session";
  import PlotGroupGrid from "@/components/document/PlotGroupGrid.svelte";
  import type { Wire } from "@/lib/rpc/canonical";
  import type { DocumentViewPlot } from "@/lib/types";
  import { createLivePlotPayloads } from "@/plots/live-data.svelte";
  import type { LivePlotState } from "@/plots/live-frames";
  import { summaryWindowStore } from "@/stores/summary-window-store";

  let {
    plots,
    scope,
    session,
  }: {
    plots: DocumentViewPlot[];
    scope: Wire<EvidenceScope>;
    session: ConnectedSession;
  } = $props();
  const live = createLivePlotPayloads(
    () => ({ enabled: true, plots, scope, windowS: 259200 }),
    () => session,
  );

  $effect.pre(() => {
    void live.states;
    const started = performance.now();
    void tick().then(() => globalThis.__desktopBench.sample("svelte_update", performance.now() - started));
  });

  $effect(() => {
    const observation = globalThis.__desktopBench;
    const states = Object.entries(live.states);
    const refused = (state: LivePlotState) => state.status === "error" || Boolean(state.failure && !state.payload);
    const failures = states.filter(([, state]) => refused(state));
    observation.errors = failures.map(([plotId, state]) => `${plotId}: ${state.error ?? state.failure?.message ?? "Plot Data Failed"}`);
    observation.loadedPlots = states.filter(([, state]) => state.payload && !refused(state)).length;
    observation.failedPlots = failures.length;

    const started = performance.now();
    requestAnimationFrame(() => requestAnimationFrame(() => observation.sample("presentation_opportunity", performance.now() - started)));
  });
</script>

<PlotGroupGrid
  documentPlots={plots}
  liveStates={live.states}
  measurements={summaryWindowStore.getInitialState()}
  title="Mixed Plots"
  showGroupHeaders={false}
/>
