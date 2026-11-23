<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Plus, X } from "lucide-svelte";
  import {
    emptyListItem,
    expandChipValue,
    formatListCandidates,
    type ListChip,
    listCandidates,
    listChips,
  } from "../../lib/parameter-parser";
  import type { ParameterDefinition } from "../../lib/types";
  import { cn } from "../../lib/utils";
  import Button from "../ui/Button.svelte";
  import Input from "../ui/Input.svelte";
  import { isExpandedParameter, parameterValueType } from "./model";

  let {
    definition,
    invalid,
    raw,
    onChange,
  }: { definition: ParameterDefinition; invalid: boolean; raw: string; onChange: (raw: string) => void } = $props();
  const elementType = $derived((definition.list?.element_type ?? parameterValueType(definition)).toLowerCase());
  const label = $derived(definition.label ?? definition.name);
  const expands = $derived(isExpandedParameter(definition));
  const candidates = $derived(listCandidates(raw, elementType, expands));
  const chips = $derived(listChips(candidates));

  function write(next: unknown[][]) {
    onChange(formatListCandidates(next, expands));
  }

  function replace(chip: ListChip, values: unknown[]) {
    write(
      candidates.map((candidate, position) =>
        position === chip.candidate ? candidate.flatMap((value, index) => (index === chip.index ? values : [value])) : candidate,
      ),
    );
  }

  // Half-Typed Ranges Survive Until Commit, When They Expand In Their Candidate.
  function commit(chip: ListChip, text: string) {
    const expanded = expandChipValue(text, elementType);
    if (expanded && expands && (candidates[chip.candidate]?.length ?? 0) === 1) {
      write(candidates.flatMap((candidate, position) => (position === chip.candidate ? expanded.map((value) => [value]) : [candidate])));
    } else if (expanded) {
      replace(chip, expanded);
    } else {
      write(candidates);
    }
  }

  function add() {
    const empty = emptyListItem(elementType);
    write(expands ? [...candidates, [empty]] : [[...(candidates[0] ?? []), empty]]);
  }
</script>

<div
  aria-invalid={invalid}
  data-testid={`parameter-chips-${definition.name}`}
  class={cn(
    "flex min-h-[calc(var(--parameter-row-height)_-_--spacing(2))] flex-wrap items-center gap-2 rounded-lg border bg-background px-2 py-1",
    invalid && "border-destructive",
  )}
>
  {#each chips as chip, position (`${chip.candidate}-${chip.index}`)}
    <span class="inline-flex h-6 items-center gap-1 rounded-lg border bg-muted/40 pl-2 [[data-density=touch]_&]:h-7">
      <Input
        aria-label={`${label} Value ${position + 1}`}
        class="h-auto w-auto min-w-6 rounded-none border-0 bg-transparent p-0 font-mono text-xs tabular-nums"
        onblur={(event) => commit(chip, event.currentTarget.value)}
        oninput={(event) => replace(chip, [event.currentTarget.value])}
        onkeydown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            commit(chip, event.currentTarget.value);
          }
        }}
        size={Math.max(String(chip.value).length, 1)}
        spellcheck={false}
        value={String(chip.value)}
      />
      <Button
        aria-label={`Remove ${label} Value ${position + 1}`}
        class="relative size-5 rounded-l-none [[data-density=touch]_&]:after:absolute [[data-density=touch]_&]:after:-inset-2.5 [[data-density=touch]_&]:after:content-['']"
        onclick={() => replace(chip, [])}
        size="icon-xs"
        variant="ghost"
        ><X aria-hidden="true" /></Button
      >
    </span>
  {/each}
  <Button aria-label={`Add ${label} Value`} class="size-6 [[data-density=touch]_&]:size-11" onclick={add} size="icon-xs" variant="ghost">
    <Plus aria-hidden="true" />
  </Button>
</div>
