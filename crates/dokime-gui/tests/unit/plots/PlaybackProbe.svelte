<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { plotPlayback } from "@/plots/playback.svelte";

  let { samples, capture }: { samples: number[]; capture: (playback: ReturnType<typeof plotPlayback<number[]>>) => void } = $props();
  const playback = plotPlayback(
    () => samples,
    () => samples.map((sample) => `1970-01-01T00:00:0${sample}Z`),
  );
  $effect(() => capture(playback));
</script>
