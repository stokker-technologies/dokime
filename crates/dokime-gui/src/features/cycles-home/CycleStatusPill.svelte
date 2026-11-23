<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import Badge from "../../components/ui/Badge.svelte";
  import { CYCLE_STATE_LABELS, STATUS_PILL_CLASS } from "../cycle/cycle-workspace-model";
  import type { CycleRow } from "./cycles-model";

  let { label, tone }: { label: string; tone: CycleRow["statusTone"] } = $props();
  const variants = { good: "success", warn: "warning", bad: "destructive", neutral: "outline" } as const;
</script>

<span class="inline-grid align-middle" data-slot="cycle-status">
  {#each CYCLE_STATE_LABELS as sizer (sizer)}
    <Badge aria-hidden="true" class={`${STATUS_PILL_CLASS} invisible [grid-area:1/1]`} data-sizer="" variant="outline">{sizer}</Badge>
  {/each}
  <Badge class={`${STATUS_PILL_CLASS} w-full [grid-area:1/1]`} variant={variants[tone]}>{label}</Badge>
</span>
