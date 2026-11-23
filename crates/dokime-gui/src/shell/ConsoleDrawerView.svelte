<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { ToggleGroup } from "bits-ui";
  import { Eraser, X } from "lucide-svelte";
  import ErrorCard from "../components/ErrorCard.svelte";
  import Button from "../components/ui/Button.svelte";
  import { inputClass } from "../components/ui/input-style";
  import SelectField from "../components/ui/SelectField.svelte";
  import { toggleVariants } from "../components/ui/toggle-variants";
  import { cn } from "../lib/utils";
  import type { ConsoleState } from "../stores/ui-store";
  import type { ConsoleRecords } from "./console-buffer";
  import { type ConsoleScope, consoleLevelOptions, visibleConsoleRecords } from "./console-view-model";
  import ConsoleRecordList from "./ConsoleRecordList.svelte";

  type Props = {
    cycleId: string | null;
    executionId: string | null;
    streamed: ConsoleRecords;
    drawer: ConsoleState;
    setConsoleState: (patch: Partial<ConsoleState>) => void;
    filter: string;
    setFilter: (filter: string) => void;
  };

  const scopes = [
    { value: "instance", label: "Instance" },
    { value: "cycle", label: "Cycle" },
    { value: "execution", label: "Execution" },
  ] as const;
  const minHeight = 120;
  const maxHeight = 900;
  const keyboardStep = 24;

  let { cycleId, executionId, streamed, drawer, setConsoleState, filter, setFilter }: Props = $props();
  let drag: { origin: number; height: number } | null = null;
  const rows = $derived(visibleConsoleRecords(streamed.records, drawer.level, filter));

  function close() {
    drag = null;
    setConsoleState({ open: false });
  }

  function leave(event: KeyboardEvent) {
    if (drawer.open && event.key === "Escape" && !event.defaultPrevented) {
      event.preventDefault();
      close();
    }
  }

  function chooseScope(value: string) {
    const scope = scopes.find((item) => item.value === value);
    if (scope) {
      setConsoleState({ scope: scope.value });
    }
  }

  function scopeDisabled(scope: ConsoleScope) {
    return (scope === "cycle" && !cycleId) || (scope === "execution" && !executionId);
  }

  function resize(value: number) {
    setConsoleState({ height: Math.min(maxHeight, Math.max(minHeight, Math.round(value))) });
  }

  function resizeKey(event: KeyboardEvent) {
    if (event.key === "ArrowUp" || event.key === "ArrowDown") {
      event.preventDefault();
      resize(drawer.height + (event.key === "ArrowUp" ? keyboardStep : -keyboardStep));
    }
  }

  function startDrag(event: PointerEvent & { currentTarget: HTMLButtonElement }) {
    drag = { origin: event.clientY, height: drawer.height };
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  function moveDrag(event: PointerEvent & { currentTarget: HTMLButtonElement }) {
    if (drag && event.currentTarget.hasPointerCapture(event.pointerId)) {
      resize(drag.height + drag.origin - event.clientY);
    }
  }
</script>

<svelte:document onkeydown={leave} />

{#if drawer.open}
  <section
    aria-label="Console"
    class="grid max-h-[80%] min-h-0 grid-rows-[auto_auto_minmax(0,1fr)] border-t bg-card"
    style:height={`${drawer.height}px`}
  >
    <button
      type="button"
      aria-label="Console Height"
      class="h-1 cursor-row-resize touch-none border-0 bg-transparent hover:bg-border"
      onkeydown={resizeKey}
      onpointerdown={startDrag}
      onpointermove={moveDrag}
      onlostpointercapture={() => {
        drag = null;
      }}
    ></button>

    <div class="grid grid-cols-[auto_auto_minmax(0,1fr)_auto_auto] items-center gap-3 border-b px-[var(--shell-inset)] py-2">
      <ToggleGroup.Root
        type="single"
        value={drawer.scope}
        onValueChange={chooseScope}
        aria-label="Console Scope"
        class="flex w-fit items-center gap-2 rounded-lg"
        data-slot="toggle-group"
      >
        {#each scopes as scope (scope.value)}
          <ToggleGroup.Item
            data-slot="toggle-group-item"
            class={toggleVariants({ variant: "outline", class: "shrink-0 focus:z-10 focus-visible:z-10" })}
            disabled={scopeDisabled(scope.value)}
            value={scope.value}
            >{scope.label}</ToggleGroup.Item
          >
        {/each}
      </ToggleGroup.Root>

      <SelectField
        label="Console Level"
        class="w-32"
        size="sm"
        value={drawer.level}
        options={consoleLevelOptions}
        onChange={(level) => setConsoleState({ level })}
      />

      <input
        data-slot="input"
        aria-label="Filter Records"
        class={cn(inputClass, "h-8 min-w-0")}
        oninput={(event) => setFilter(event.currentTarget.value)}
        placeholder="Filter Records"
        value={filter}
      >

      <Button
        onclick={streamed.clear}
        size="sm"
        title="Clear the rows shown here; the backend keeps its records and trace."
        variant="outline"
      >
        <Eraser data-icon="inline-start" />
        Clear
      </Button>
      <Button aria-label="Close Console" onclick={close} size="icon-sm" variant="ghost"><X /></Button>
    </div>

    <div class="grid min-h-0 grid-rows-[auto_minmax(0,1fr)]">
      <div>
        {#if streamed.failure}
          <ErrorCard failure={streamed.failure} />
        {:else if streamed.error}
          <p role="status" class="px-[var(--shell-inset)] py-1 text-xs text-muted-foreground">{streamed.error}</p>
        {/if}
        {#if streamed.dropped > 0n}
          <p class="px-[var(--shell-inset)] py-1 font-mono text-xs text-muted-foreground">
            {streamed.dropped.toString()}
            older {streamed.dropped === 1n ? "event" : "events"} dropped
          </p>
        {/if}
      </div>
      <ConsoleRecordList
        empty="No records at this level yet."
        follow={drawer.follow}
        onFollowChange={(follow) => setConsoleState({ follow })}
        {rows}
      />
    </div>
  </section>
{/if}
