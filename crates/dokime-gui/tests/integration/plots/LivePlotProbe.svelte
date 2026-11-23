<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createSubscriber } from "svelte/reactivity";
  import type { ConnectedSession } from "@/app/native-session/session";
  import PlotWindowControl from "@/features/cycle/PlotWindowControl.svelte";
  import type { createPlotWindow } from "@/features/cycle/plot-window-state";
  import { createLivePlotPayloads, type LivePlotOptions } from "@/plots/live-data.svelte";
  import type { LivePlotStateMap } from "@/plots/live-frames";

  let {
    options,
    session,
    onRender,
    windowStore,
  }: {
    options: LivePlotOptions;
    session: ConnectedSession;
    onRender: (states: LivePlotStateMap) => void;
    windowStore?: ReturnType<typeof createPlotWindow>;
  } = $props();

  const subscribe = createSubscriber((update) => windowStore?.subscribe(update));
  const windowState = $derived.by(() => {
    subscribe();
    return windowStore?.getState();
  });
  const live = createLivePlotPayloads(
    () => (windowState ? { ...options, windowS: windowState.option.value, refresh: windowState.refresh } : options),
    () => session,
  );

  $effect(() => onRender(live.states));
</script>

{#if windowState}
  <PlotWindowControl live elapsedUnit={null} window={windowState} />
{/if}
