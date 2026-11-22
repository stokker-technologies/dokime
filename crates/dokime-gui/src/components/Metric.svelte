<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Progress } from "bits-ui";
  import { isPathLike } from "../lib/path-display";
  import { exactNumericText } from "../lib/record-display";
  import type { MetricState } from "../lib/types";
  import { cn } from "../lib/utils";
  import AbbreviatedValue from "./AbbreviatedValue.svelte";
  import type { OverviewMetric } from "./document/overview/metric-model";
  import PathPill from "./PathPill.svelte";
  import { badgeVariants } from "./ui/badge-variants";

  let { metricId, label, value, exactValue, rawValue, detail, presentation = "value", state = null }: OverviewMetric = $props();
  const progress = $derived(presentation === "progress" ? progressValue(rawValue, value) : null);
  const status = $derived(stateVariant(state) ?? statusVariant(value.trim().toLowerCase()));
  const variant = $derived(status ?? (presentation === "badge" ? "outline" : null));
  const complete = $derived(exactValue === value && /^-?\d/.test(value));

  function progressValue(rawValue: unknown, formatted: string): number | null {
    if (exactNumericText(rawValue) !== null) {
      return null;
    }
    const candidate = typeof rawValue === "number" ? rawValue : Number.parseFloat(formatted.replace("%", ""));
    if (!Number.isFinite(candidate)) {
      return null;
    }
    return Math.max(0, Math.min(100, candidate));
  }

  function statusVariant(value: string): "default" | "secondary" | "destructive" | null {
    if (["connected", "completed", "passed", "running", "true"].includes(value)) {
      return "default";
    }
    if (["failed", "error", "disconnected", "false", "blocked"].includes(value)) {
      return "destructive";
    }
    if (["unknown", "queued", "skipped", "stopped", "stopping"].includes(value)) {
      return "secondary";
    }
    return null;
  }

  function stateVariant(state?: MetricState | null): "default" | "secondary" | "destructive" | "success" | "warning" | null {
    if (state === "ok") {
      return "success";
    }
    if (state === "warning") {
      return "warning";
    }
    if (state === "critical") {
      return "destructive";
    }
    if (state === "muted") {
      return "secondary";
    }
    return null;
  }

  function stateClass(state?: MetricState | null): string {
    if (state === "ok") {
      return "border-success/30 bg-success/10";
    }
    if (state === "warning") {
      return "border-warning/30 bg-warning/10";
    }
    if (state === "critical") {
      return "border-destructive/30 bg-destructive/10";
    }
    return "";
  }

  function progressClass(state?: MetricState | null): string {
    if (state === "ok") {
      return "[&_[data-slot=progress-indicator]]:bg-success";
    }
    if (state === "warning") {
      return "[&_[data-slot=progress-indicator]]:bg-warning";
    }
    if (state === "critical") {
      return "[&_[data-slot=progress-indicator]]:bg-destructive";
    }
    if (state === "muted") {
      return "[&_[data-slot=progress-indicator]]:bg-muted-foreground";
    }
    return "[&_[data-slot=progress-indicator]]:bg-primary";
  }
</script>

<div
  data-slot="card"
  data-size="sm"
  data-document-metric={metricId}
  data-document-value-present={rawValue !== undefined && rawValue !== null}
  class={cn(
    "group/card flex min-h-18 min-w-0 flex-col overflow-hidden border-t border-border bg-transparent py-2 text-sm text-card-foreground",
    stateClass(state),
  )}
>
  <div data-slot="card-content" class="grid min-w-0 grid-rows-[--spacing(5)_minmax(--spacing(6),auto)_--spacing(3)] gap-1 p-3">
    <span class="min-w-0 self-end truncate text-xs font-bold leading-tight text-muted-foreground" title={label}>{label}</span>
    {#if progress === null}
      <strong class="min-w-0 text-lg leading-tight">
        {#if variant}
          <span
            data-slot="badge"
            data-variant={variant}
            class={cn(badgeVariants({ variant }), "max-w-full truncate", status && "capitalize")}
          >
            {value}
          </span>
        {:else if isPathLike(value)}
          <PathPill {value} />
        {:else}
          <AbbreviatedValue
            class={cn("block min-w-0 max-w-full", complete ? "break-all whitespace-normal" : "truncate", value.length > 10 && "text-base")}
            exact={exactValue ?? value}
            {value}
          />
        {/if}
      </strong>
    {:else}
      <div class="grid min-w-0 gap-1 self-center">
        <Progress.Root
          aria-label="Progress"
          data-slot="progress"
          value={progress}
          class={cn(
            "relative flex h-2 w-full items-center overflow-x-hidden rounded-full bg-background ring-1 ring-border",
            progressClass(state),
          )}
        >
          <div
            data-slot="progress-indicator"
            class="size-full flex-1 bg-primary transition-transform motion-reduce:animate-none"
            style:transform={`translateX(-${100 - progress}%)`}
          ></div>
        </Progress.Root>
        <span class="text-xs font-semibold tabular-nums">{progress.toFixed(0)}%</span>
      </div>
    {/if}
    {#if detail}
      <small class="min-w-0 truncate text-xs leading-tight text-muted-foreground" title={detail}>{detail}</small>
    {/if}
  </div>
</div>
