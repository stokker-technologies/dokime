<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Dialog, Select } from "bits-ui";
  import { CheckIcon, ChevronDownIcon, Settings2, XIcon } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import ErrorCard from "../components/ErrorCard.svelte";
  import Button from "../components/ui/Button.svelte";
  import { buttonVariants } from "../components/ui/button-variants";
  import { formatDecimal } from "../lib/format";
  import { failureOf } from "../lib/notification-details";
  import type { DocumentDownsampleTechnique, DocumentViewPlot } from "../lib/types";
  import { BUILT_IN_REPORT_DEFAULTS, type PlotDisplaySettings, type ReportDefaults } from "./plot-settings";
  import { NATIVE_PLOT_MAX_POINTS } from "./reduce";

  let {
    children,
    plot,
    onSubmit,
    defaults = BUILT_IN_REPORT_DEFAULTS,
  }: {
    children: Snippet<[Snippet]>;
    plot: DocumentViewPlot;
    onSubmit: (settings: PlotDisplaySettings) => Promise<void>;
    defaults?: ReportDefaults;
  } = $props();

  const techniques: { value: DocumentDownsampleTechnique; label: string }[] = [
    { value: "auto", label: "Auto" },
    { value: "lttb", label: "Largest Triangle (LTTB)" },
    { value: "min_max", label: "Min / Max Buckets" },
    { value: "raw", label: "Raw Samples" },
  ];
  const options = $derived(
    techniques.map((option) =>
      option.value === "auto" && defaults.plotDownsample !== "auto"
        ? { ...option, label: `Default (${techniques.find(({ value }) => value === defaults.plotDownsample)?.label})` }
        : option,
    ),
  );
  const id = $props.id();
  let open = $state(false);
  let pointsRaw = $state("");
  let technique = $state<DocumentDownsampleTechnique>("auto");
  let busy = $state(false);
  let error = $state.raw<unknown>(null);
  const points = $derived(parseDisplayPoints(pointsRaw));
  const invalid = $derived(points === undefined);

  function changeOpen(next: boolean) {
    if (next) {
      pointsRaw = plot.display_points == null ? "" : String(plot.display_points);
      technique = plot.downsample ?? "auto";
      error = null;
    }
    open = next;
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (points === undefined || busy) {
      return;
    }
    busy = true;
    error = null;
    try {
      await onSubmit({ display_points: points, downsample: technique });
      open = false;
    } catch (cause: unknown) {
      error = cause;
    } finally {
      busy = false;
    }
  }

  function parseDisplayPoints(raw: string): number | null | undefined {
    const text = raw.trim();
    if (!text) {
      return null;
    }
    const value = Number(text);
    if (!Number.isSafeInteger(value) || value < 0 || value > NATIVE_PLOT_MAX_POINTS) {
      return undefined;
    }
    return value;
  }
</script>

{#snippet trigger()}
  <Dialog.Trigger
    aria-label={`Plot Settings for ${plot.title}`}
    data-testid="plot-settings"
    data-slot="button"
    data-size="icon-sm"
    data-variant="ghost"
    title="Plot Settings"
    class={buttonVariants({ variant: "ghost", size: "icon-sm" })}
  >
    <Settings2 />
  </Dialog.Trigger>
{/snippet}

<Dialog.Root bind:open={() => open, changeOpen}>
  {@render children(trigger)}
  <Dialog.Portal>
    <Dialog.Overlay data-slot="dialog-overlay" class="fixed inset-0 isolate z-50 bg-black/10" />
    <Dialog.Content
      data-slot="dialog-content"
      class="fixed top-1/2 left-1/2 z-50 grid w-full max-w-[calc(100%_-_--spacing(8))] grid-cols-[minmax(0,1fr)] -translate-x-1/2 -translate-y-1/2 gap-4 rounded-lg bg-popover p-4 text-sm text-popover-foreground ring-1 ring-border outline-none sm:max-w-md"
    >
      <form class="grid gap-4" onsubmit={submit}>
        <div data-slot="dialog-header" class="flex flex-col gap-2">
          <Dialog.Title data-slot="dialog-title" class="font-sans text-base leading-none font-medium">Plot Settings</Dialog.Title>
          <Dialog.Description data-slot="dialog-description" class="text-sm text-muted-foreground">
            {plot.title}. Saved with the running cycle, so the sealed report draws the same points.
          </Dialog.Description>
        </div>

        <div class="grid grid-cols-[minmax(0,--spacing(36))_minmax(0,1fr)] items-center gap-x-3 gap-y-3">
          <label class="text-sm font-medium" for={`${id}-points`}>Display Points</label>
          <input
            data-slot="input"
            class="h-(--control-height) w-full min-w-0 rounded-lg border border-input bg-transparent px-3 py-1 text-base outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 aria-invalid:border-destructive md:text-sm dark:bg-input/30"
            aria-invalid={invalid || undefined}
            id={`${id}-points`}
            inputmode="numeric"
            max={NATIVE_PLOT_MAX_POINTS}
            min={0}
            oninput={(event) => {
              pointsRaw = event.currentTarget.value;
            }}
            placeholder={`Default (${formatDecimal(defaults.plotPoints)})`}
            step={1}
            type="number"
            value={pointsRaw}
          >
          <label class="text-sm font-medium" for={`${id}-technique`}>Reduction</label>
          <Select.Root
            type="single"
            value={technique}
            onValueChange={(value) => {
              technique = value as DocumentDownsampleTechnique;
            }}
          >
            <Select.Trigger
              id={`${id}-technique`}
              data-slot="select-trigger"
              class="flex h-(--control-height) w-full items-center justify-between gap-2 rounded-lg border border-input bg-transparent py-2 pr-2 pl-3 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 dark:bg-input/30"
            >
              {options.find(({ value }) => value === technique)?.label}
              <ChevronDownIcon class="size-4 text-muted-foreground" />
            </Select.Trigger>
            <Select.Content class="z-50 min-w-36 rounded-lg bg-popover p-1 text-popover-foreground ring-1 ring-border" align="start">
              {#each options as { label, value } (value)}
                <Select.Item
                  {value}
                  {label}
                  class="relative flex items-center rounded-lg px-2 py-1 pr-8 text-sm data-highlighted:bg-accent"
                >
                  {label}
                  {#if value === technique}
                    <CheckIcon class="absolute right-2 size-4" />
                  {/if}
                </Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </div>

        <p class="text-xs text-muted-foreground">
          {invalid
            ? `Enter between 1 and ${formatDecimal(NATIVE_PLOT_MAX_POINTS)} points, use 0 for every sample, or leave empty for the default.`
            : `Leave the points empty for the host default, or use 0 for every sample; a long live run is compacted near ${formatDecimal(NATIVE_PLOT_MAX_POINTS)} samples per trace.`}
        </p>
        {#if error}
          <ErrorCard notificationContext="action" failure={failureOf(error, "Operation Failed")} />
        {/if}

        <div
          data-slot="dialog-footer"
          class="-mx-4 -mb-4 flex min-w-0 flex-col-reverse gap-2 rounded-b-lg border-t bg-muted/50 p-4 sm:flex-row sm:flex-wrap sm:justify-end"
        >
          <Dialog.Close class={buttonVariants({ variant: "outline" })}>Cancel</Dialog.Close>
          <Button disabled={invalid || busy} type="submit">{busy ? "Saving" : "Apply"}</Button>
        </div>
      </form>
      <Dialog.Close class={`${buttonVariants({ variant: "ghost", size: "icon-sm" })} absolute top-2 right-2`} aria-label="Close">
        <XIcon />
      </Dialog.Close>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
