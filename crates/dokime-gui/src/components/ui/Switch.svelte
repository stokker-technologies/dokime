<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Switch, type SwitchRootProps } from "bits-ui";
  import { cn } from "../../lib/utils";

  type Props = SwitchRootProps & { size?: "sm" | "default" };
  let { checked = $bindable(false), class: style, size = "default", ...attributes }: Props = $props();
</script>

<Switch.Root
  bind:checked
  data-slot="switch"
  data-size={size}
  class={cn(
    "peer group/switch relative inline-flex shrink-0 items-center rounded-full border border-transparent transition-all outline-none after:content-[''] after:absolute after:-inset-x-3 after:-inset-y-3 after:min-h-11 focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 data-[size=default]:h-5 data-[size=default]:w-8 data-[size=sm]:h-4 data-[size=sm]:w-6 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input dark:data-[state=unchecked]:bg-input/80 data-disabled:cursor-not-allowed data-disabled:opacity-50",
    style,
  )}
  {...attributes}
>
  <Switch.Thumb
    data-slot="switch-thumb"
    class="pointer-events-none block rounded-full bg-background ring-0 transition-transform group-data-[size=default]/switch:size-4 group-data-[size=sm]/switch:size-3 data-[state=checked]:translate-x-[calc(100%-2px)] dark:data-[state=checked]:bg-primary-foreground data-[state=unchecked]:translate-x-0 dark:data-[state=unchecked]:bg-foreground"
  />
</Switch.Root>
