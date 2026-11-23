<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Dialog } from "bits-ui";
  import { ChevronDown, Wifi } from "lucide-svelte";
  import { parseParameterInput } from "../../lib/parameter-parser";
  import type { ParameterDefinition } from "../../lib/types";
  import { cn } from "../../lib/utils";
  import { type WifiChannelChoice, wifiSelectionSummary } from "../start-cycle-form-model";
  import Badge from "../ui/Badge.svelte";
  import Button from "../ui/Button.svelte";
  import { buttonVariants } from "../ui/button-variants";
  import Checkbox from "../ui/Checkbox.svelte";
  import DialogContent from "../ui/DialogContent.svelte";
  import { parameterValueType } from "./model";

  let {
    choices,
    definition,
    inputId,
    raw,
    onChange,
  }: {
    choices: WifiChannelChoice[];
    definition: ParameterDefinition;
    inputId: string;
    raw: string;
    onChange: (raw: string) => void;
  } = $props();
  const parsed = $derived(
    parseParameterInput(raw, {
      ...definition,
      choices: [],
      expansion: { kind: "cartesian" },
      metadata: {},
      value_type: parameterValueType(definition),
      values: [],
    }),
  );
  const selected = $derived(
    new Set(
      parsed.ok
        ? parsed.values.map(String)
        : raw
            .split(",")
            .map((entry) => entry.trim())
            .filter(Boolean),
    ),
  );
  const selectedChoices = $derived(choices.filter((choice) => selected.has(choice.value)));
  const groups = $derived([
    { title: "Auto", channels: choices.filter((choice) => choice.mode === "auto") },
    ...["2.4 GHz", "5 GHz"].map((band) => ({
      title: `${band} Streams`,
      channels: choices.filter((choice) => choice.mode !== "auto" && choice.band === band),
    })),
  ]);

  function setSelected(values: Set<string>) {
    onChange(
      choices
        .map((choice) => choice.value)
        .filter((value) => values.has(value))
        .join(", "),
    );
  }

  function toggle(value: string, checked: boolean) {
    const next = new Set(selected);
    if (checked) {
      next.add(value);
    } else {
      next.delete(value);
    }
    setSelected(next);
  }

  function selectBand(band: string) {
    setSelected(new Set([...selected, ...choices.filter((choice) => choice.band === band).map((choice) => choice.value)]));
  }
</script>

<Dialog.Root>
  <Dialog.Trigger
    id={inputId}
    data-testid="wifi-stream-selector-trigger"
    class={buttonVariants({ variant: "outline", class: "h-[calc(var(--parameter-row-height)_-_--spacing(2))] w-full justify-between" })}
  >
    <span class="flex min-w-0 items-center gap-2"
      ><Wifi aria-hidden="true" size={14} /><span class="truncate">{wifiSelectionSummary(selectedChoices)}</span></span
    >
    <ChevronDown aria-hidden="true" size={14} />
  </Dialog.Trigger>
  <DialogContent class="max-h-[min(88vh,--spacing(184))] overflow-hidden p-0 sm:max-w-3xl">
    <header class="flex flex-col gap-2 px-4 pt-4">
      <Dialog.Title class="font-medium">WiFi Channel</Dialog.Title>
      <Dialog.Description class="text-muted-foreground"
        >Select auto-band options or explicit streams for generated WiFi cases.</Dialog.Description
      >
    </header>
    <div class="flex flex-wrap gap-2 border-b px-4 pb-2">
      <Button onclick={() => selectBand("2.4 GHz")} size="sm" variant="outline">Select 2.4 GHz</Button>
      <Button onclick={() => selectBand("5 GHz")} size="sm" variant="outline">Select 5 GHz</Button>
      <Button onclick={() => onChange("")} size="sm" variant="ghost">Use Default</Button>
    </div>
    <div class="grid max-h-[min(58vh,--spacing(112))] gap-4 overflow-auto px-4 pb-4">
      {#each groups.filter((group) => group.channels.length) as { title, channels } (title)}
        <section class="grid gap-2">
          <div class="flex items-center justify-between gap-3">
            <strong class="text-sm">{title}</strong><Badge variant="secondary">{channels.length}</Badge>
          </div>
          <div class="grid gap-2 [grid-template-columns:repeat(auto-fit,minmax(min(100%,7.5rem),1fr))]">
            {#each channels as choice (choice.value)}
              {@const id = `${inputId}-${choice.value}`}
              {@const checked = selected.has(choice.value)}
              <label
                for={id}
                class={cn(
                  "grid min-h-12 cursor-pointer grid-cols-[auto_minmax(0,1fr)] items-center gap-2 rounded-lg border px-3 py-2 text-sm transition-colors",
                  checked ? "border-primary bg-primary/5" : "bg-background hover:bg-muted/45",
                )}
              >
                <Checkbox {id} {checked} onCheckedChange={(next) => toggle(choice.value, next)} />
                <span class="min-w-0"
                  ><span class="block truncate font-medium">{choice.label}</span>
                  {#if choice.band}
                    <span class="mt-1 inline-flex"><Badge class="px-1 py-0 text-xs" variant="outline">{choice.band}</Badge></span>
                  {/if}
                </span>
              </label>
            {/each}
          </div>
        </section>
      {/each}
    </div>
    <footer class="flex items-center justify-between gap-2 border-t bg-muted/50 p-4">
      <div class="min-w-0 flex-1 text-xs text-muted-foreground">
        {selectedChoices.length ? `${selectedChoices.length} selected` : "Template Default"}
      </div>
      <Dialog.Close class={buttonVariants()}>Done</Dialog.Close>
    </footer>
  </DialogContent>
</Dialog.Root>
