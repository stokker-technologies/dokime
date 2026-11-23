<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { ConnectedSession } from "@/app/native-session/session";
  import PlotWindowControl from "@/features/cycle/PlotWindowControl.svelte";
  import type { createPlotWindow } from "@/features/cycle/plot-window-state";
  import { createLivePlotPayloads, type LivePlotOptions } from "@/plots/live-data.svelte";
  import type { LivePlotState, LivePlotStateMap } from "@/plots/live-frames";

  let {
    options,
    session,
    onRender,
    onPlot,
    windowStore,
  }: {
    options: LivePlotOptions;
    session: ConnectedSession;
    onRender: (states: LivePlotStateMap) => void;
    /** Called Whenever One Plot's Own Reader Re-Runs. */
    onPlot?: (plotId: string, state: LivePlotState | undefined) => void;
    windowStore?: ReturnType<typeof createPlotWindow>;
  } = $props();

  const live = createLivePlotPayloads(
    () => ({ ...options, window: windowStore }),
    () => session,
  );

  $effect(() => onRender(Object.fromEntries(live.states)));
</script>

{#each options.plots as plot (plot.plot_id)}
  {onPlot?.(plot.plot_id, live.states.get(plot.plot_id))}
{/each}
{#if windowStore}
  <PlotWindowControl live elapsedUnit={null} window={windowStore} />
{/if}
