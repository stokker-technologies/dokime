<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import CycleStatusPill from "../cycles-home/CycleStatusPill.svelte";
  import type { OpenCycle } from "./active-cycle-model";

  let { cycle }: { cycle: OpenCycle } = $props();
</script>

<span class="grid min-w-0 grid-cols-[minmax(0,auto)_auto_auto] items-center gap-2">
  <span class="truncate font-medium text-foreground">{cycle.label}</span>
  <span class="font-mono text-muted-foreground tabular-nums">{cycle.shortId}</span>
  <CycleStatusPill label={cycle.statusLabel} tone={cycle.statusTone} />
</span>
