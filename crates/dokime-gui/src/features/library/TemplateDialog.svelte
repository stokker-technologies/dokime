<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createMutation } from "@tanstack/svelte-query";
  import { Dialog, Tooltip } from "bits-ui";
  import { Check } from "lucide-svelte";
  import { untrack } from "svelte";
  import type { ConnectedSession } from "../../app/native-session/session";
  import { operatorArtifactUploadOptions } from "../../components/artifact-upload-mutation";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import { type EditorParameterRow, editorProblems, editorRow, seedEditorRows } from "../../components/parameter-editor/model";
  import ParameterEditor from "../../components/parameter-editor/ParameterEditor.svelte";
  import {
    buildPayload,
    createPlanSelection,
    type FileRowState,
    initialFileRows,
    registeredParameterRaw,
    titleCaseIdentifier,
  } from "../../components/start-cycle-form-model";
  import { apiSelection } from "../../components/suite-picker/model";
  import { appendTestCycleOptions, previewTestCycleOptions } from "../../components/suite-picker/server-state";
  import CopyableTraceabilityValue from "../../components/traceability/CopyableTraceabilityValue.svelte";
  import Button from "../../components/ui/Button.svelte";
  import DialogContent from "../../components/ui/DialogContent.svelte";
  import { failureOf } from "../../lib/notification-details";
  import NewCycleDialog from "../cycles-home/NewCycleDialog.svelte";
  import { toast } from "../../lib/notifications";
  import type { FileDefinition, PreviewTestCycleResponse, SuiteTemplateEntry, TestSuiteEntry } from "../../lib/types";
  import { parameterOverridesStatus, parameterOverridesStore } from "../../stores/parameter-overrides-store";
  import { storeState } from "../../stores/store-state";
  import ActiveCycleInline from "./ActiveCycleInline.svelte";
  import type { OpenCycle } from "./active-cycle-model";
  import { caseAppend, caseDeltas, NO_ACTIVE_CYCLE, summaryNeeded } from "./template-dialog-model";
  import TemplateProcedure from "./TemplateProcedure.svelte";

  let {
    activeCycle,
    hasOpenCycles,
    onChangeCycle,
    onCycleCreated,
    onOpenChange,
    openCycle,
    open,
    session,
    suite,
    template,
  }: {
    activeCycle: OpenCycle | null;
    hasOpenCycles: boolean;
    onChangeCycle: () => void;
    onCycleCreated: (cycleId: string) => void;
    onOpenChange: (open: boolean) => void;
    openCycle: (cycleId: string) => void;
    open: boolean;
    session: ConnectedSession;
    suite: TestSuiteEntry;
    template: SuiteTemplateEntry;
  } = $props();
  const title = $derived(titleCaseIdentifier(template.name));
  let creating = $state(false);
  const parameters = storeState(parameterOverridesStatus, (state) => state);
  const overrides = storeState(parameterOverridesStore, (state) => state.overrides);
  const { setParameterOverride, resetParameterOverride, clearTemplateOverrides } = parameterOverridesStore.getState();
  const previewMutation = createMutation(
    () => previewTestCycleOptions(session.identity),
    () => session.queryClient,
  );
  const appendMutation = createMutation(
    () => appendTestCycleOptions(session),
    () => session.queryClient,
  );
  const uploadMutation = createMutation(
    () => operatorArtifactUploadOptions(session),
    () => session.queryClient,
  );

  let rows = $state.raw<EditorParameterRow[]>(untrack(() => seedEditorRows(template, overrides()[template.template_id])));
  let fileRows = $state.raw<FileRowState[]>(untrack(() => initialFileRows(template)));
  let step = $state<"parameters" | "summary">("parameters");
  let preview = $state.raw<PreviewTestCycleResponse | null>(null);
  let error = $state.raw<unknown>();
  let busy = $state(false);
  let body: HTMLDivElement;

  // Hydration Can Finish After The Dialog Opens. Restore The Remembered Values Then.
  $effect(() => {
    if (parameters().ready) {
      rows = seedEditorRows(template, overrides()[template.template_id]);
    }
  });

  const validation = $derived(
    failureOf(
      {
        notificationFailure: {
          reason: "validation",
          operation: "Cases.Create",
          problems: editorProblems(rows, fileRows, template.file_definitions),
        },
      },
      "Template Inputs Need Attention",
    ),
  );
  const problems = $derived(validation.problems?.length ?? 0);
  const payload = $derived(
    buildPayload(
      rows.map(({ name, parsed, raw }) => ({ name, parsed, raw })),
      fileRows,
    ),
  );
  const selections = $derived([
    apiSelection(
      createPlanSelection({
        files: payload.files,
        suiteId: suite.suite_id,
        suiteName: suite.name,
        template,
        values: payload.values,
      }),
    ),
  ]);
  const append = $derived(caseAppend(activeCycle?.cycleId, selections));

  function changeRaw(name: string, raw: string) {
    error = undefined;
    const definition = rows.find((row) => row.name === name)?.definition;
    if (!definition) {
      return;
    }
    rows = rows.map((row) => (row.name === name ? editorRow(row.definition, raw) : row));
    if (raw === registeredParameterRaw(definition)) {
      resetParameterOverride(template.template_id, definition.parameter_id);
    } else {
      setParameterOverride(template.template_id, definition, raw);
    }
  }

  function resetRow(name: string) {
    const definition = rows.find((row) => row.name === name)?.definition;
    if (definition) {
      changeRaw(name, registeredParameterRaw(definition));
    }
  }

  function resetAll() {
    if (!fileRows.some((row) => row.uploading)) {
      error = undefined;
      rows = rows.map((row) => editorRow(row.definition, registeredParameterRaw(row.definition)));
      clearTemplateOverrides(template.template_id);
      fileRows = initialFileRows(template);
    }
  }

  function changeFile(name: string, change: Partial<FileRowState>) {
    fileRows = fileRows.map((row) => (row.name === name ? { ...row, ...change } : row));
  }

  function resetFile(name: string) {
    if (!fileRows.find((row) => row.name === name)?.uploading) {
      changeFile(name, { path: template.file_definitions.find((definition) => definition.name === name)?.default ?? "", error: null });
    }
  }

  async function uploadFile(definition: FileDefinition, file: File | undefined) {
    if (!file) {
      return;
    }
    changeFile(definition.name, { error: null, uploading: true });
    try {
      const artifact = await uploadMutation.mutateAsync({
        description: definition.description ?? undefined,
        file,
        filename: file.name,
        groupId: "test-inputs",
        groupLabel: "Test Inputs",
        label: definition.label ?? definition.name,
        mediaType: file.type || "application/octet-stream",
      });
      changeFile(definition.name, { error: null, path: artifact.path, uploading: false });
    } catch (reason) {
      const failure = failureOf(reason, "Operation Failed");
      changeFile(definition.name, { error: failure.message, failure, uploading: false });
    }
  }

  async function submit(reviewed?: PreviewTestCycleResponse) {
    const destination = append;
    if (!destination || problems || busy) {
      return;
    }
    busy = true;
    error = undefined;
    try {
      if (!reviewed) {
        const expanded = await previewMutation.mutateAsync({
          cycle_id: destination.cycleId,
          parameters: {},
          selections: destination.selections,
        });
        preview = expanded;
        if (summaryNeeded(expanded, rows)) {
          step = "summary";
          return;
        }
      }
      const plan = reviewed ?? preview;
      const current = append;
      if (!plan || !current) {
        throw new Error(NO_ACTIVE_CYCLE);
      }
      await appendMutation.mutateAsync(current);
      // Close Before Reporting Success So An Unavailable Toast Cannot Leave An Applied Dialog Open.
      onOpenChange(false);
      const count = plan.case_count;
      const label = activeCycle?.label ?? current.cycleId;
      toast.success(`Added ${count} ${count === 1 ? "Case" : "Cases"} to ${label}`, {
        action: { label: "Open Cycle", onClick: () => openCycle(current.cycleId) },
      });
    } catch (failure) {
      error = failure;
    } finally {
      busy = false;
    }
  }

  function focusContent(event: Event) {
    event.preventDefault();
    body?.closest<HTMLElement>("[role=dialog]")?.focus();
  }
</script>

{#snippet summary(
  plan: PreviewTestCycleResponse,
)}
  <div class="grid content-start gap-2 px-4">
    <div class="grid grid-cols-[minmax(0,1fr)_auto] items-baseline gap-3 border-b pb-1">
      <p class="text-sm text-muted-foreground">
        Target Cycle: <span class="text-foreground">{activeCycle?.label ?? activeCycle?.cycleId ?? ""}</span>
      </p>
      <p class="text-xs tabular-nums text-muted-foreground">{plan.case_count} {plan.case_count === 1 ? "Case" : "Cases"}</p>
    </div>
    <ul class="grid" data-testid="summary-cases">
      {#each plan.cases as item, index (item.case_id)}
        {@const deltas = caseDeltas(item.parameters, rows)}
        <li
          class="grid min-h-[var(--list-row-height)] grid-cols-[minmax(0,2fr)_minmax(0,3fr)] items-center gap-3 border-b border-border/60 py-2"
        >
          <span class="min-w-0 text-sm font-medium">{plan.cases.length > 1 ? `${title} ${index + 1}` : title}</span>
          <span class="flex min-w-0 flex-wrap items-center gap-1">
            {#each deltas as delta (delta)}
              <span class="rounded-lg bg-muted px-2 py-1 text-xs text-muted-foreground">{delta}</span>
            {:else}
              <span class="text-xs text-muted-foreground">No Changes From Defaults</span>
            {/each}
          </span>
        </li>
      {/each}
    </ul>
  </div>
{/snippet}

<Dialog.Root {open} {onOpenChange}>
  <DialogContent
    class="top-[6vh] max-h-[88vh] translate-y-0 grid-rows-[auto_minmax(0,1fr)_auto] pb-0 sm:max-w-180
    [[data-density=touch]_&]:inset-0 [[data-density=touch]_&]:max-h-none [[data-density=touch]_&]:max-w-none
    [[data-density=touch]_&]:translate-x-0 [[data-density=touch]_&]:rounded-none"
    onOpenAutoFocus={focusContent}
  >
    <header class="flex flex-col items-center gap-2 text-center">
      <Dialog.Title class="font-medium">{title}</Dialog.Title>
      <Dialog.Description class="sr-only">Edit this template’s parameters, then review the cases it generates.</Dialog.Description>
      <Tooltip.Provider>
        <div class="grid w-full gap-1 text-xs">
          {#each [
            { label: "Suite", value: suite.suite_id },
            { label: "Template", value: template.identifier ?? template.template_id },
          ] as item (item.label)}
            <div class="flex min-w-0 items-center justify-between gap-2">
              <span class="shrink-0 text-muted-foreground">{item.label}</span>
              <CopyableTraceabilityValue value={item.value} />
            </div>
          {/each}
        </div>
      </Tooltip.Provider>
      <ol
        aria-hidden="true"
        class="mx-auto grid w-full max-w-72 grid-cols-(--layout-comparison) items-center text-sm"
        data-testid="step-indicator"
      >
        <li class="flex items-center justify-center gap-2">
          {#if step === "summary"}
            <Check aria-hidden="true" class="size-4 text-primary" />
          {/if}
          <span class={step === "parameters" ? "font-semibold text-primary" : "text-foreground"}>Parameters</span>
        </li>
        <li class="border-t"></li>
        <li class={step === "summary" ? "font-semibold text-primary" : "text-muted-foreground"}>Summary</li>
      </ol>
      <span class="sr-only">Step {step === "parameters" ? 1 : 2} Of 2: {step === "parameters" ? "Parameters" : "Summary"}</span>
    </header>
    <div bind:this={body} class="-mx-4 grid min-h-0 content-start gap-3 overflow-y-auto">
      <TemplateProcedure {template} />
      {#if step === "parameters"}
        <div>
          {#if !parameters().ready}
            {@const failure = parameters().error}
            <div role="status">
              {#if failure}
                <ErrorCard {failure} />
                <Button variant="ghost" onclick={() => void parameterOverridesStore.persist.rehydrate()}>Retry</Button>
              {:else}
                <p class="text-sm text-muted-foreground">Loading Remembered Parameters…</p>
              {/if}
            </div>
          {/if}
          <fieldset disabled={!parameters().ready}>
            <ParameterEditor
              fileDefinitions={template.file_definitions}
              {fileRows}
              onChangeRaw={changeRaw}
              onFileChange={(definition, file) => void uploadFile(definition, file)}
              onFilePathChange={(name, path) => changeFile(name, { error: null, path })}
              onResetAll={resetAll}
              onResetFile={resetFile}
              onResetRow={resetRow}
              {rows}
            />
          </fieldset>
        </div>
      {:else if preview}
        {@render summary(preview)}
      {/if}
      <div class="mx-4">
        <ActiveCycleInline cycle={activeCycle} {hasOpenCycles} onChange={onChangeCycle} onNewCycle={() => (creating = true)} />
      </div>
      <!-- Nested in This Dialog's Tree, So It Stacks Above It as a Child Layer -->
      <NewCycleDialog {session} open={creating} onOpenChange={(next) => (creating = next)} onCreated={onCycleCreated} />
      {#if error}
        <ErrorCard notificationContext="action" failure={failureOf(error, "Operation Failed")} />
      {/if}
    </div>
    <footer class="-mx-4 grid grid-cols-2 items-center gap-2 rounded-b-lg border-t bg-muted/50 p-4">
      <Button
        class="h-9 w-full"
        disabled={busy}
        variant="outline"
        onclick={() => {
          if (step === "summary") step = "parameters";
          else onOpenChange(false);
        }}
        >{step === "summary" ? "Back" : "Cancel"}</Button
      >
      {#if step === "parameters"}
        <div class="grid justify-items-stretch gap-1">
          <Button class="h-9 w-full" disabled={Boolean(problems) || busy || !parameters().ready || !append} onclick={() => void submit()}
            >Next</Button
          >
          {#if problems}
            <ErrorCard failure={validation} />
          {/if}
        </div>
      {:else if preview}
        <Button data-testid="template-add" class="h-9 w-full" disabled={busy || !append} onclick={() => void submit(preview ?? undefined)}
          >Finish and Add</Button
        >
      {/if}
    </footer>
  </DialogContent>
</Dialog.Root>
