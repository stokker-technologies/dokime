<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts" generics="Value extends string">
  import { Select } from "bits-ui";
  import { Check, ChevronDown } from "lucide-svelte";
  import { cn } from "../../lib/utils";

  type Props = {
    label: string;
    value: Value;
    options: readonly { value: Value; label: string; disabled?: boolean }[];
    class?: string;
    id?: string;
    name?: string;
    placeholder?: string;
    disabled?: boolean;
    size?: "sm" | "default";
    onChange: (value: Value) => void;
  };

  let { label, value, options, class: style, id, name, placeholder, disabled, size = "default", onChange }: Props = $props();
  const selected = $derived(options.find((option) => option.value === value)?.label);

  function choose(next: string) {
    const option = options.find((item) => item.value === next);
    if (option && !option.disabled) {
      onChange(option.value);
    }
  }
</script>

<Select.Root type="single" {value} {name} {disabled} onValueChange={choose}>
  <Select.Trigger
    {id}
    aria-label={label}
    data-slot="select-trigger"
    data-size={size}
    class={cn(
      "flex h-(--control-height) w-fit items-center justify-between gap-2 rounded-lg border border-input bg-transparent py-2 pr-2 pl-3 text-sm whitespace-nowrap transition-colors outline-none select-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-input/30 dark:hover:bg-input/50",
      style,
    )}
  >
    <span data-slot="select-value" class="line-clamp-1 flex items-center gap-2">{selected ?? placeholder}</span>
    <ChevronDown class="pointer-events-none size-4 shrink-0 text-muted-foreground" />
  </Select.Trigger>
  <Select.Portal>
    <Select.Content
      data-slot="select-content"
      class="z-50 max-h-(--bits-select-content-available-height) min-w-36 overflow-x-hidden overflow-y-auto rounded-lg bg-popover text-popover-foreground ring-1 ring-border"
    >
      <Select.Viewport>
        {#each options as option (option.value)}
          <Select.Item
            value={option.value}
            label={option.label}
            disabled={option.disabled}
            data-slot="select-item"
            class="relative flex w-full cursor-default items-center gap-2 rounded-lg py-1 pr-8 pl-2 text-sm outline-hidden select-none data-highlighted:bg-accent data-highlighted:text-accent-foreground data-disabled:opacity-50 max-lg:min-h-(--control-height) [[data-density=touch]_&]:min-h-(--control-height)"
          >
            {option.label}
            {#if value === option.value}
              <Check class="pointer-events-none absolute right-2 size-4" />
            {/if}
          </Select.Item>
        {/each}
      </Select.Viewport>
    </Select.Content>
  </Select.Portal>
</Select.Root>
