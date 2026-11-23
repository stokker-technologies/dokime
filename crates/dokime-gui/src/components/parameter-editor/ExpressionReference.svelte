<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Popover } from "bits-ui";
  import { CircleHelp } from "lucide-svelte";
  import { buttonVariants } from "../ui/button-variants";

  const examples = [
    { example: "10, 20, 30", result: "10, 20, 30", syntax: "a, b, c" },
    { example: "1..5", result: "1, 2, 3, 4, 5", syntax: "1..5" },
    { example: "0..2:0.5", result: "0, 0.5, 1, 1.5, 2", syntax: "0..10:0.5" },
    { example: "linspace(0, 1, 3)", result: "0, 0.5, 1", syntax: "linspace(start, stop, n)" },
    { example: "arange(0, 3)", result: "0, 1, 2", syntax: "arange(start, stop[, step])" },
  ];
</script>

<Popover.Root>
  <Popover.Trigger
    aria-label="List Expression Help"
    class={buttonVariants({ size: "icon-xs", variant: "ghost", class: "size-6 [[data-density=touch]_&]:size-11" })}
  >
    <CircleHelp aria-hidden="true" />
  </Popover.Trigger>
  <Popover.Portal>
    <Popover.Content
      align="end"
      data-testid="expression-reference"
      class="z-50 grid w-[min(--spacing(152),calc(100vw_-_--spacing(8)))] gap-2 rounded-lg bg-popover p-3 text-popover-foreground ring-1 ring-border"
    >
      <p class="font-sans text-sm font-semibold">Expression Reference</p>
      <dl class="grid grid-cols-[auto_auto_auto] items-baseline gap-x-4 gap-y-1 text-xs">
        {#each examples as { syntax, example, result } (syntax)}
          <div class="contents">
            <dt class="font-mono whitespace-nowrap text-foreground">{syntax}</dt>
            <dd class="font-mono whitespace-nowrap text-muted-foreground">{example}</dd>
            <dd class="font-mono whitespace-nowrap tabular-nums text-muted-foreground">{result}</dd>
          </div>
        {/each}
      </dl>
      <p class="text-xs text-muted-foreground">
        Cartesian parameters cross. Every combination of the expanded lists becomes a case. Zip parameters advance together, one case per
        position.
      </p>
    </Popover.Content>
  </Popover.Portal>
</Popover.Root>
