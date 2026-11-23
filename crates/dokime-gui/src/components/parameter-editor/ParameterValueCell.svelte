<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { ToggleGroup } from "bits-ui";
  import { parameterInputId, wifiChannelChoices } from "../start-cycle-form-model";
  import Input from "../ui/Input.svelte";
  import SelectField from "../ui/SelectField.svelte";
  import Switch from "../ui/Switch.svelte";
  import { toggleVariants } from "../ui/toggle-variants";
  import { type EditorParameterRow, isExpandedParameter, parameterChoiceList, parameterValueType } from "./model";
  import ListChipsCell from "./ListChipsCell.svelte";
  import WifiChannelCell from "./WifiChannelCell.svelte";

  let { row, onChange }: { row: EditorParameterRow; onChange: (raw: string) => void } = $props();
  const cellHeight = "h-[calc(var(--parameter-row-height)_-_--spacing(2))]";
  const itemClass = toggleVariants({
    variant: "outline",
    class: "shrink-0 rounded-none px-2 not-first:border-l-0 first:rounded-l-lg last:rounded-r-lg focus:z-10",
  });
  const definition = $derived(row.definition);
  const inputId = $derived(parameterInputId(definition.parameter_id));
  const invalid = $derived(!row.parsed.ok);
  const choices = $derived(parameterChoiceList(definition));
  const checked = $derived(row.raw.trim().toLowerCase() === "true");
  const expanded = $derived(isExpandedParameter(definition));
  const unit = $derived(definition.unit?.trim() ?? "");
  const valueType = $derived(parameterValueType(definition));
  const integer = $derived(["int", "integer", "unsigned"].includes(valueType));
  const numeric = $derived(row.kind === "number");
  const selected = $derived(
    new Set(
      row.raw
        .split(",")
        .map((entry) => entry.trim())
        .filter(Boolean),
    ),
  );

  function chooseMany(values: string[]) {
    onChange(
      choices
        .map((choice) => choice.value)
        .filter((value) => values.includes(value))
        .join(", "),
    );
  }
</script>

{#snippet segmentedOptions()}
  {#each choices as choice (choice.value)}
    <ToggleGroup.Item class={itemClass} value={choice.value} data-slot="toggle-group-item">{choice.label}</ToggleGroup.Item>
  {/each}
{/snippet}

{#if row.kind === "boolean"}
  <div class={["flex items-center gap-2", cellHeight]}>
    <Switch {checked} id={inputId} onCheckedChange={(next) => onChange(next ? "true" : "false")} />
    <label class="text-sm text-muted-foreground" for={inputId}>{checked ? "On" : "Off"}</label>
  </div>
{:else if row.kind === "list"}
  <ListChipsCell {definition} {invalid} raw={row.raw} {onChange} />
{:else if row.kind === "segmented"}
  {#if expanded}
    <ToggleGroup.Root
      type="multiple"
      id={inputId}
      class={["flex w-fit flex-wrap items-center rounded-lg", cellHeight]}
      data-slot="toggle-group"
      value={choices.map((choice) => choice.value).filter((value) => selected.has(value))}
      onValueChange={chooseMany}
    >
      {@render segmentedOptions()}
    </ToggleGroup.Root>
  {:else}
    <ToggleGroup.Root
      type="single"
      id={inputId}
      class={["flex w-fit flex-wrap items-center rounded-lg", cellHeight]}
      data-slot="toggle-group"
      value={row.raw.trim()}
      onValueChange={(value) => {
        if (value) onChange(value);
      }}
    >
      {@render segmentedOptions()}
    </ToggleGroup.Root>
  {/if}
{:else if row.kind === "select"}
  <SelectField
    id={inputId}
    label={definition.label ?? definition.name}
    class={`w-full ${cellHeight}`}
    options={choices}
    value={row.raw.trim()}
    placeholder="Select a Value"
    {onChange}
  />
{:else if row.kind === "wifi"}
  <WifiChannelCell {definition} choices={wifiChannelChoices(definition, choices)} {inputId} raw={row.raw} {onChange} />
{:else}
  <div class="relative min-w-0">
    <Input
      aria-invalid={invalid}
      class={["w-full", cellHeight, numeric && "text-right font-mono tabular-nums"].filter(Boolean).join(" ")}
      data-testid={`parameter-input-${definition.name}`}
      id={inputId}
      inputmode={numeric ? (integer ? "numeric" : "decimal") : undefined}
      type={numeric && !expanded ? "number" : "text"}
      min={numeric && !expanded ? (definition.min ?? (valueType === "unsigned" ? 0 : undefined)) : undefined}
      max={numeric && !expanded ? (definition.max ?? undefined) : undefined}
      step={numeric && !expanded ? (integer ? 1 : "any") : undefined}
      oninput={(event) => onChange(event.currentTarget.value)}
      spellcheck={false}
      style={numeric ? "padding-right: calc(var(--parameter-unit-width, 3ch) + 1.25rem)" : undefined}
      value={row.raw}
    />
    {#if numeric && unit}
      <span
        aria-hidden="true"
        class="pointer-events-none absolute inset-y-0 right-2 flex w-[var(--parameter-unit-width,3ch)] items-center truncate font-mono text-xs text-muted-foreground"
        >{unit}</span
      >
    {:else if unit}
      <span class="sr-only">{unit}</span>
    {/if}
  </div>
{/if}
