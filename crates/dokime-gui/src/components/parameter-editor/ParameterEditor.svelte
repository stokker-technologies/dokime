<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Tooltip } from "bits-ui";
  import { Clipboard, RotateCcw } from "lucide-svelte";
  import { failureOf } from "../../lib/notification-details";
  import { toast } from "../../lib/notifications";
  import type { FileDefinition } from "../../lib/types";
  import { cn } from "../../lib/utils";
  import ErrorCard from "../ErrorCard.svelte";
  import { type FileRowState, parameterInputId } from "../start-cycle-form-model";
  import Button from "../ui/Button.svelte";
  import { buttonVariants } from "../ui/button-variants";
  import ExpressionReference from "./ExpressionReference.svelte";
  import FileValueCell from "./FileValueCell.svelte";
  import {
    acceptsExpressions,
    type EditorParameterRow,
    editorProblemCount,
    expansionStatement,
    groupEditorRows,
    isEditedRow,
    unitColumnCharacters,
  } from "./model";
  import ParameterValueCell from "./ParameterValueCell.svelte";

  let {
    fileDefinitions,
    fileRows,
    onChangeRaw,
    onFileChange,
    onFilePathChange,
    onResetAll,
    onResetRow,
    onResetFile,
    rows,
  }: {
    fileDefinitions: FileDefinition[];
    fileRows: FileRowState[];
    onChangeRaw: (name: string, raw: string) => void;
    onFileChange: (definition: FileDefinition, file: File | undefined) => void;
    onFilePathChange: (name: string, path: string) => void;
    onResetAll: () => void;
    onResetRow: (name: string) => void;
    onResetFile: (name: string) => void;
    rows: EditorParameterRow[];
  } = $props();
  const groups = $derived(groupEditorRows(rows));
  const edited = $derived(
    rows.some(isEditedRow) ||
      fileDefinitions.some((definition) => {
        const row = fileRows.find((candidate) => candidate.name === definition.name);
        return row && (row.path !== (definition.default ?? "") || row.error !== null);
      }),
  );
  const problems = $derived(editorProblemCount(rows, fileRows, fileDefinitions));
  const rowStyle =
    "grid min-h-[var(--parameter-row-height)] grid-cols-[minmax(0,2fr)_minmax(0,3fr)_--spacing(10)] items-center gap-3 border-b border-l-2 border-border/60 px-4 py-1";
  const groupStyle = "px-4 pb-1 text-xs font-semibold tracking-wide text-muted-foreground uppercase";

  async function copyParameterId(parameterId: string) {
    try {
      await navigator.clipboard.writeText(parameterId);
      toast.success("Parameter ID Copied");
    } catch (error) {
      toast.error(failureOf(error, "Could Not Copy Parameter ID"));
    }
  }

  function resetTargetLabel(row: EditorParameterRow): string {
    const raw = row.defaultRaw.trim();
    const unit = row.definition.unit?.trim();
    return raw ? (unit ? `${raw} ${unit}` : raw) : "Template Default";
  }
</script>

{#snippet labelCell(
  label: string,
  id: string,
  description?: string | null,
)}
  <div class="grid min-w-0 gap-1">
    <label class="min-w-0 text-sm leading-snug font-medium" for={id} title={label}>{label}</label>
    {#if description}
      <p class="text-xs leading-snug text-muted-foreground">{description}</p>
    {/if}
  </div>
{/snippet}

{#snippet iconAction(
  label: string,
  hint: string,
  Icon: typeof RotateCcw,
  onclick: () => void,
  testId?: string,
  disabled?: boolean,
)}
  <Tooltip.Root>
    <Tooltip.Trigger
      aria-label={label}
      {disabled}
      {onclick}
      data-testid={testId}
      class={buttonVariants({ variant: "ghost", size: "icon-sm" })}
    >
      <Icon aria-hidden="true" />
    </Tooltip.Trigger>
    <Tooltip.Portal>
      <Tooltip.Content class="z-50 rounded-lg bg-popover px-3 py-2 text-xs text-popover-foreground ring-1 ring-border"
        >{hint}</Tooltip.Content
      >
    </Tooltip.Portal>
  </Tooltip.Root>
{/snippet}

<Tooltip.Provider>
  <div
    class="grid content-start gap-2 [--parameter-row-height:2.5rem] [[data-density=touch]_&]:[--parameter-row-height:3rem]"
    style:--parameter-unit-width={`${unitColumnCharacters(rows)}ch`}
  >
    <div class="flex items-baseline justify-between gap-3 border-b px-4 pb-1">
      <p class="text-xs text-muted-foreground">
        {problems
          ? `${problems} ${problems === 1 ? "problem" : "problems"} to fix before the summary.`
          : "Values you change are remembered for this template."}
      </p>
      <Button disabled={!edited || fileRows.some((row) => row.uploading)} onclick={onResetAll} size="sm" variant="ghost">Reset All</Button>
    </div>
    <div class="grid" data-testid="parameter-editor-grid">
      {#each groups as group, index (group.label)}
        <section class="grid">
          <p class={[groupStyle, index === 0 ? "pt-1" : "pt-4"]}>{group.label}</p>
          {#each group.rows as row (row.definition.parameter_id)}
            {@const definition = row.definition}
            {@const label = definition.label ?? row.name}
            {@const changed = isEditedRow(row)}
            {@const expansion = expansionStatement(definition)}
            {@const error = row.parsed.ok ? null : row.parsed.error}
            <div
              class={cn(rowStyle, changed ? "border-l-primary" : "border-l-transparent")}
              data-edited={changed ? "true" : undefined}
              data-testid={`parameter-row-${row.name}`}
            >
              {@render labelCell(label, parameterInputId(definition.parameter_id), definition.description)}
              <div class="grid min-w-0 gap-1">
                <div class="flex min-w-0 items-center gap-1">
                  <div class="min-w-0 flex-1"><ParameterValueCell onChange={(raw) => onChangeRaw(row.name, raw)} {row} /></div>
                  {@render iconAction(
                    `Copy ${label} Parameter ID`,
                    definition.parameter_id,
                    Clipboard,
                    () => void copyParameterId(definition.parameter_id),
                  )}
                  {#if acceptsExpressions(row)}
                    <ExpressionReference />
                  {:else}
                    <span class="size-6 [[data-density=touch]_&]:size-11"></span>
                  {/if}
                </div>
                {#if error}
                  <ErrorCard failure={failureOf(error, "Operation Failed")} />
                {:else if expansion}
                  <p class="flex flex-wrap items-center gap-2 text-xs leading-snug text-muted-foreground">
                    {#if expansion.badge}
                      <span
                        class="pointer-events-none rounded-lg bg-muted px-2 py-1 text-xs font-semibold tracking-wide text-muted-foreground uppercase select-none"
                        >{expansion.badge}</span
                      >
                    {/if}
                    <span>{expansion.sentence}</span>
                  </p>
                {/if}
              </div>
              <div class="justify-self-center">
                {#if changed || error}
                  {@const label = `Reset to ${resetTargetLabel(row)}`}
                  {@render iconAction(label, label, RotateCcw, () => onResetRow(row.name), "parameter-reset")}
                {/if}
              </div>
            </div>
          {/each}
        </section>
      {/each}
      {#if fileDefinitions.length}
        <section class="grid">
          <p class={[groupStyle, groups.length ? "pt-4" : "pt-1"]}>Files</p>
          {#each fileDefinitions as definition (definition.name)}
            {@const row = fileRows.find((candidate) => candidate.name === definition.name) ?? {
              error: null,
              name: definition.name,
              path: definition.default ?? "",
              uploading: false,
            }}
            {@const label = definition.label ?? definition.name}
            <div class={cn(rowStyle, "border-l-transparent")}>
              {@render labelCell(label, `${parameterInputId(definition.name)}-file`, definition.description)}
              <div class="grid min-w-0 gap-1">
                <div class="flex min-w-0 items-center gap-1">
                  <div class="min-w-0 flex-1">
                    <FileValueCell
                      {definition}
                      {row}
                      onFileChange={(file) => onFileChange(definition, file)}
                      onPathChange={(path) => onFilePathChange(definition.name, path)}
                    />
                  </div>
                  <span class="size-6"></span>
                </div>
                {#if row.error}
                  <ErrorCard notificationContext="action" failure={failureOf(row.failure, "Operation Failed")} />
                {/if}
              </div>
              <div class="justify-self-center">
                {#if row.path !== (definition.default ?? "") || row.error}
                  {@render iconAction(
                    `Reset ${label} to Template Default`,
                    "Reset to Template Default",
                    RotateCcw,
                    () => onResetFile(definition.name),
                    undefined,
                    row.uploading,
                  )}
                {/if}
              </div>
            </div>
          {/each}
        </section>
      {/if}
    </div>
  </div>
</Tooltip.Provider>
