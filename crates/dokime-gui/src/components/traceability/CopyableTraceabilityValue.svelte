<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Tooltip } from "bits-ui";
  import { buttonVariants } from "../ui/button-variants";
  import { cn } from "../../lib/utils";
  import { copyTraceabilityText } from "./clipboard";

  let { value }: { value: string } = $props();
  const text = $derived(String(value ?? ""));
  const long = $derived(text.length > 24);
</script>

<Tooltip.Provider delayDuration={0}>
  <Tooltip.Root>
    <Tooltip.Trigger
      data-slot="button"
      data-variant="plain"
      data-size="inline"
      aria-label={`Copy ${text} to Clipboard`}
      class={cn(
        buttonVariants({ variant: "plain", size: "inline" }),
        "block min-w-0 max-w-full truncate text-right text-xs font-medium underline-offset-2 hover:underline",
      )}
      onclick={() => void copyTraceabilityText(text)}
    >
      {text}
    </Tooltip.Trigger>
    <Tooltip.Portal>
      <Tooltip.Content
        role="tooltip"
        data-slot="tooltip-content"
        side="top"
        align="end"
        class="z-50 w-fit max-w-[min(--spacing(128),calc(100vw_-_--spacing(8)))] rounded-lg bg-foreground px-3 py-2 text-left text-xs break-all whitespace-normal text-background"
      >
        {long ? `Copy: ${text}` : "Copy to Clipboard"}
        <Tooltip.Arrow class="fill-foreground" />
      </Tooltip.Content>
    </Tooltip.Portal>
  </Tooltip.Root>
</Tooltip.Provider>
