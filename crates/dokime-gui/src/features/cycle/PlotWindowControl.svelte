<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { RefreshCw, TimerReset } from "lucide-svelte";
  import Button from "../../components/ui/Button.svelte";
  import Slider from "../../components/ui/Slider.svelte";
  import { plotWindowDetail, plotWindows, plotWindowTitle, plotWindowValueLabel, type PlotWindowState } from "./plot-window";

  type Props = { elapsedUnit: string | null; live: boolean; window: PlotWindowState };
  let { elapsedUnit, live, window }: Props = $props();

  const index = $derived(window.index);
  let draft = $derived(index);
  const option = $derived(plotWindows[draft] ?? window.option);
  const title = $derived(plotWindowTitle(elapsedUnit));
  const detail = $derived(plotWindowDetail(option, elapsedUnit));
  const updated = $derived(
    window.refresh.updatedAt === null
      ? "Not Updated Yet"
      : `Updated ${new Date(window.refresh.updatedAt).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit", hour12: false })}`,
  );
</script>

{#if live}
  <div class="flex flex-wrap items-center justify-end gap-3">
    <Button
      aria-busy={window.refresh.refreshing}
      disabled={window.refresh.refreshing}
      onclick={window.refresh.request}
      size="sm"
      variant="outline"
    >
      <RefreshCw class={window.refresh.refreshing ? "animate-spin" : undefined} size={14} />
      {window.refresh.refreshing ? "Refreshing…" : "Refresh"}
    </Button>
    <span class="whitespace-nowrap text-xs tabular-nums text-muted-foreground" role="status">{updated}</span>
    <div class="w-56 xl:w-80">
      <div class="grid h-full min-w-0 grid-cols-(--layout-plot-window) items-center gap-2">
        <span class="inline-flex min-w-0 items-center gap-2 text-xs font-bold uppercase text-muted-foreground">
          <TimerReset class="shrink-0" size={14} />
          <span class="truncate">{title}</span>
        </span>
        <Slider
          aria-label={title}
          aria-valuetext={detail}
          class="min-w-0"
          max={plotWindows.length - 1}
          min={0}
          step={1}
          value={[draft]}
          onValueChange={([value]) => {
            draft = value ?? 0;
          }}
          onValueCommit={([value]) => {
            draft = value ?? 0;
            window.choose(draft);
          }}
        />
        <span class="whitespace-nowrap text-right text-sm font-semibold tabular-nums" title={detail}>
          {plotWindowValueLabel(option, elapsedUnit)}
        </span>
      </div>
    </div>
  </div>
{/if}
