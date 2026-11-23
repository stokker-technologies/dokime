<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Checkbox } from "bits-ui";
  import { Check } from "lucide-svelte";
  import type { ComponentProps } from "svelte";
  import { cn } from "../../lib/utils";

  let { class: style, ...props }: Omit<ComponentProps<typeof Checkbox.Root>, "children" | "child"> = $props();
</script>

<Checkbox.Root
  data-slot="checkbox"
  class={cn(
    "peer relative flex size-4 shrink-0 items-center justify-center rounded-lg border border-input outline-none transition-colors after:absolute after:-inset-x-3 after:-inset-y-2 after:content-[''] focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 dark:bg-input/30 data-[state=checked]:border-primary data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground max-lg:size-7 [[data-density=touch]_&]:size-7",
    style,
  )}
  {...props}
>
  {#snippet children({
    checked,
    indeterminate,
  })}
    {#if checked || indeterminate}
      <Check aria-hidden="true" class="size-4" />
    {/if}
  {/snippet}
</Checkbox.Root>
