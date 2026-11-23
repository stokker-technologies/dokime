<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { orderedInstants } from "@/plots/instants";
  import PlotPlayback from "@/plots/PlotPlayback.svelte";
  import { plotPlayback } from "@/plots/playback.svelte";

  type Playback = ReturnType<typeof plotPlayback<string[]>>;
  let { incoming, capture }: { incoming: string[]; capture: (trajectory: Playback, grid: Playback) => void } = $props();
  const available = $derived(orderedInstants(incoming));
  const trajectory = plotPlayback(
    () => incoming,
    () => available,
  );
  const grid = plotPlayback(
    () => incoming,
    () => available,
  );
  $effect(() => capture(trajectory, grid));
</script>

<PlotPlayback title="Trajectory" playback={trajectory} />
<PlotPlayback title="Grid" playback={grid} />
