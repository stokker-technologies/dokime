<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Slider } from "bits-ui";
  import type { SliderMultipleRootProps } from "bits-ui";
  import { cn } from "../../lib/utils";

  type Props = Omit<SliderMultipleRootProps, "children" | "child" | "type">;
  let {
    class: className,
    "aria-label": label,
    "aria-valuetext": valueText,
    disabled = false,
    min = 0,
    max = 100,
    value = $bindable<number[]>(),
    ...attributes
  }: Props = $props();
  const values = $derived(value ?? [min, max]);
</script>

<Slider.Root
  type="multiple"
  value={values}
  {min}
  {disabled}
  aria-label={label}
  aria-disabled={disabled || undefined}
  {max}
  data-slot="slider"
  class={cn(
    "[[data-density=touch]_&]:h-(--control-height) relative flex w-full touch-none items-center select-none data-disabled:opacity-50",
    className,
  )}
  {...attributes}
>
  {#snippet children({
    thumbItems,
  })}
    <span data-slot="slider-track" class="relative h-1.5 w-full grow overflow-hidden rounded-full bg-muted">
      <Slider.Range data-slot="slider-range" class="absolute h-full bg-primary" />
    </span>
    {#each thumbItems as { index } (index)}
      <Slider.Thumb
        {index}
        aria-label={label}
        aria-valuetext={valueText}
        data-slot="slider-thumb"
        class="[[data-density=touch]_&]:size-6 block size-4 rounded-full border border-primary bg-background shadow-sm transition-colors focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50"
      />
    {/each}
  {/snippet}
</Slider.Root>
