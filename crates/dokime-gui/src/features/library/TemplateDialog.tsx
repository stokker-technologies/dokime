/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The materialization dialog (concept 11).
 *
 * Two steps and one outcome: edit the template's parameters, read the cases
 * they generate, then append them to the active cycle. A single case at its
 * defaults skips the reading (`summaryNeeded`). This is the only way cases
 * enter a cycle, so the dialog owns the whole transaction and keeps entered
 * values through validation and RPC failures. Without an active cycle it adds
 * nothing and offers New Cycle. */

/* === External Imports === */

import { useNavigate } from "@tanstack/react-router";
import { Check } from "lucide-react";
import { type ReactNode, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { NotificationMessage } from "@/components/NotificationMessage";
import { toast } from "@/lib/notifications";

/* === Local Imports === */

import { useOperatorArtifactUploadMutation } from "@/components/artifact-upload-mutation";
import { type EditorParameterRow, editorProblemCount, editorRow, seedEditorRows } from "@/components/parameter-editor/model";
import { ParameterEditor } from "@/components/parameter-editor/ParameterEditor";
import {
  buildPayload,
  createPlanSelection,
  type FileRowState,
  initialFileRows,
  registeredParameterRaw,
  titleCaseIdentifier,
} from "@/components/start-cycle-form-model";
import { apiSelection } from "@/components/suite-picker/model";
import { useAppendTestCycleMutation, usePreviewTestCycleMutation } from "@/components/suite-picker/server-state";
import { CopyableTraceabilityValue } from "@/components/traceability/CopyableTraceabilityValue";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { ActiveCycleInline } from "@/features/library/ActiveCycle";
import type { OpenCycle } from "@/features/library/active-cycle-model";
import { caseAppend, caseDeltas, NO_ACTIVE_CYCLE, summaryNeeded } from "@/features/library/template-dialog-model";
import type { FileDefinition, PreviewTestCycleResponse, SuiteTemplateEntry, TestCycleSelection, TestSuiteEntry } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useParameterOverridesStatus, useParameterOverridesStore } from "@/stores/parameter-overrides-store";

/* === Constants === */

const STEP_LABELS: { key: DialogStep; label: string }[] = [
  { key: "parameters", label: "Parameters" },
  { key: "summary", label: "Summary" },
];

/* === Types === */

type TemplateDialogProps = {
  /** The active cycle the cases join; absent, the dialog adds nothing. */
  activeCycle: OpenCycle | null;
  /** Whether any cycle is open, so a missing choice asks for one rather than blocking. */
  hasOpenCycles: boolean;
  /** Opens the active cycle picker. */
  onChangeCycle: () => void;
  /** Opens New Cycle, for when no cycle is open. */
  onNewCycle: () => void;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  suite: TestSuiteEntry;
  template: SuiteTemplateEntry;
};

type DialogStep = "parameters" | "summary";

/* === Components === */

export function TemplateDialog({
  activeCycle,
  hasOpenCycles,
  onChangeCycle,
  onNewCycle,
  onOpenChange,
  open,
  suite,
  template,
}: TemplateDialogProps) {
  const cycleId = activeCycle?.cycleId;
  const cycleLabel = activeCycle?.label;
  const navigate = useNavigate();
  const touch = useTouchDensity();
  const templateId = template.template_id;
  const title = titleCaseIdentifier(template.name);

  const { ready: parametersReady, error: parametersError } = useParameterOverridesStatus();
  const overrides = useParameterOverridesStore((state) => state.overrides[templateId]);
  const setParameterOverride = useParameterOverridesStore((state) => state.setParameterOverride);
  const resetParameterOverride = useParameterOverridesStore((state) => state.resetParameterOverride);
  const clearTemplateOverrides = useParameterOverridesStore((state) => state.clearTemplateOverrides);

  /* Seeded once per opening: the persisted override where there is one, the
     registered default otherwise. Row state lives above the step switch so
     Back and a failed RPC both land on the values the operator typed. */
  const [rows, setRows] = useState<EditorParameterRow[]>(() => seedEditorRows(template, overrides));
  useEffect(() => {
    if (parametersReady) setRows(seedEditorRows(template, overrides));
  }, [parametersReady, template, overrides]);
  const [fileRows, setFileRows] = useState<FileRowState[]>(() => initialFileRows(template));
  const [step, setStep] = useState<DialogStep>("parameters");
  const [preview, setPreview] = useState<PreviewTestCycleResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const { mutateAsync: previewTestCycle } = usePreviewTestCycleMutation();
  const { mutateAsync: appendTestCycle } = useAppendTestCycleMutation();
  const { mutateAsync: uploadArtifact } = useOperatorArtifactUploadMutation();

  const problems = editorProblemCount(rows, fileRows, template.file_definitions);

  const payload = useMemo(
    () =>
      buildPayload(
        rows.map((row) => ({ name: row.name, parsed: row.parsed, raw: row.raw })),
        fileRows,
      ),
    [fileRows, rows],
  );

  const selections = useMemo(
    (): TestCycleSelection[] => [
      apiSelection(
        createPlanSelection({
          files: payload.files,
          suiteId: suite.suite_id,
          suiteName: suite.name,
          template,
          values: payload.values,
        }),
      ),
    ],
    [payload.files, payload.values, suite.name, suite.suite_id, template],
  );
  const append = caseAppend(cycleId, selections);

  function changeRaw(name: string, raw: string) {
    setError(null);
    const definition = rows.find((row) => row.name === name)?.definition;
    if (!definition) return;
    setRows((current) => current.map((row) => (row.name === name ? editorRow(row.definition, raw) : row)));
    /* Typing a value back to its registered default is not an override: a
       stored copy of the default would silently outrank a later change to it. */
    if (definition && raw === registeredParameterRaw(definition)) {
      resetParameterOverride(templateId, definition.parameter_id);
      return;
    }
    setParameterOverride(templateId, definition, raw);
  }

  function resetRow(name: string) {
    setError(null);
    const definition = rows.find((row) => row.name === name)?.definition;
    if (!definition) return;
    setRows((current) =>
      current.map((row) => (row.name === name ? editorRow(row.definition, registeredParameterRaw(row.definition)) : row)),
    );
    resetParameterOverride(templateId, definition.parameter_id);
  }

  function resetAll() {
    if (fileRows.some((row) => row.uploading)) return;
    setError(null);
    setRows((current) => current.map((row) => editorRow(row.definition, registeredParameterRaw(row.definition))));
    clearTemplateOverrides(templateId);
    setFileRows(initialFileRows(template));
  }

  async function uploadFile(definition: FileDefinition, file: File | undefined) {
    if (!file) return;
    setFileRows((current) => current.map((row) => (row.name === definition.name ? { ...row, error: null, uploading: true } : row)));
    try {
      const artifact = await uploadArtifact({
        description: definition.description ?? undefined,
        file,
        filename: file.name,
        groupId: "test-inputs",
        groupLabel: "Test Inputs",
        label: definition.label ?? definition.name,
        mediaType: file.type || "application/octet-stream",
      });
      setFileRows((current) =>
        current.map((row) => (row.name === definition.name ? { ...row, error: null, path: artifact.path, uploading: false } : row)),
      );
    } catch (failure: unknown) {
      setFileRows((current) =>
        current.map((row) => (row.name === definition.name ? { ...row, error: messageOf(failure), uploading: false } : row)),
      );
    }
  }

  /* Next expands the selection through Planning.PreviewSelection with the
     edited values; a failure keeps the operator on step 1 with their input.
     A preview with nothing to review (`summaryNeeded`) adds at once. */
  async function next() {
    if (problems || busy || !append) return;
    setBusy(true);
    setError(null);
    try {
      const expanded = await previewTestCycle({
        cycle_id: append.cycleId,
        parameters: {},
        selections: append.selections,
      });
      setPreview(expanded);
      if (summaryNeeded(expanded, rows)) setStep("summary");
      else await add(expanded);
    } catch (failure: unknown) {
      setError(messageOf(failure));
    } finally {
      setBusy(false);
    }
  }

  async function finishAndAdd() {
    if (busy || !preview) return;
    setBusy(true);
    setError(null);
    try {
      await add(preview);
    } catch (failure: unknown) {
      setError(messageOf(failure));
    } finally {
      setBusy(false);
    }
  }

  async function add(planned: PreviewTestCycleResponse) {
    if (!append) throw new Error(NO_ACTIVE_CYCLE);
    const count = planned.case_count;
    const response = await appendTestCycle(append);
    /* The cases are in the cycle: the dialog has finished, and closes before
       anything that only reports what happened. Announcing first left a
       successful add behind an open dialog whenever the toast threw. */
    onOpenChange(false);
    const label = response.plan?.test_cycle?.title ?? cycleLabel ?? append.cycleId;
    toast.success(`Added ${count} ${count === 1 ? "case" : "cases"} to ${label}`, {
      action: {
        label: "Open Cycle",
        onClick: () => {
          void navigate({ params: { cycleId: append.cycleId }, to: "/cycles/$cycleId" });
        },
      },
    });
  }

  const body = (
    <div className="-mx-4 grid min-h-0 content-start gap-3 overflow-y-auto">
      {step === "parameters" ? (
        <div>
          {!parametersReady ? (
            <div role="status">
              <NotificationMessage source="Parameters" tone={parametersError ? "error" : "info"}>
                {parametersError ?? "Loading remembered parameters…"}
              </NotificationMessage>
              {parametersError ? (
                <Button type="button" variant="ghost" onClick={() => void useParameterOverridesStore.persist.rehydrate()}>
                  Retry
                </Button>
              ) : null}
            </div>
          ) : null}
          <fieldset disabled={!parametersReady}>
            <ParameterEditor
              fileDefinitions={template.file_definitions}
              fileRows={fileRows}
              onChangeRaw={changeRaw}
              onFileChange={(definition, file) => void uploadFile(definition, file)}
              onFilePathChange={(name, path) =>
                setFileRows((current) => current.map((row) => (row.name === name ? { ...row, error: null, path } : row)))
              }
              onResetAll={resetAll}
              onResetFile={(name) =>
                setFileRows((current) =>
                  current.map((row) =>
                    row.name === name && !row.uploading
                      ? {
                          ...row,
                          path: template.file_definitions.find((definition) => definition.name === name)?.default ?? "",
                          error: null,
                        }
                      : row,
                  ),
                )
              }
              onResetRow={resetRow}
              rows={rows}
            />
          </fieldset>
        </div>
      ) : (
        <SummaryStep preview={preview} rows={rows} target={cycleLabel ?? cycleId ?? ""} title={title} />
      )}
      <div className="mx-4">
        <ActiveCycleInline cycle={activeCycle} hasOpenCycles={hasOpenCycles} onChange={onChangeCycle} onNewCycle={onNewCycle} />
      </div>
      {error ? <NotificationMessage>{error}</NotificationMessage> : null}
    </div>
  );

  const footer = (
    <div className="grid grid-cols-2 items-center gap-2 p-4">
      <Button
        className="h-9 w-full"
        disabled={busy}
        onClick={() => (step === "summary" ? setStep("parameters") : onOpenChange(false))}
        type="button"
        variant="outline"
      >
        {step === "summary" ? "Back" : "Cancel"}
      </Button>
      {step === "parameters" ? (
        <span className="grid justify-items-stretch gap-1">
          <Button
            className="h-9 w-full"
            disabled={Boolean(problems) || busy || !parametersReady || !append}
            onClick={() => void next()}
            type="button"
          >
            Next
          </Button>
          {problems ? (
            <NotificationMessage>
              {problems} {problems === 1 ? "problem" : "problems"}
            </NotificationMessage>
          ) : null}
        </span>
      ) : (
        <Button className="h-9 w-full" disabled={busy || !preview || !append} onClick={() => void finishAndAdd()} type="button">
          Finish And Add
        </Button>
      )}
    </div>
  );

  const heading = (
    <>
      <div className="grid gap-1 text-xs">
        {[
          { label: "Suite", value: suite.suite_id },
          { label: "Template", value: template.identifier ?? templateId },
        ].map(({ label, value }) => (
          <div className="flex min-w-0 items-center justify-between gap-2" key={label}>
            <span className="shrink-0 text-muted-foreground">{label}</span>
            <CopyableTraceabilityValue value={value} />
          </div>
        ))}
      </div>
      <StepIndicator step={step} />
      <span className="sr-only">
        Step {step === "parameters" ? 1 : 2} of 2: {step === "parameters" ? "Parameters" : "Summary"}
      </span>
    </>
  );

  if (touch) {
    return (
      <Sheet modal onOpenChange={onOpenChange} open={open}>
        <SheetContent
          className="inset-x-0 grid max-w-none grid-rows-[auto_minmax(0,1fr)_auto] p-0"
          onOpenAutoFocus={focusContent}
          side="right"
        >
          <SheetHeader className="grid gap-2 text-center">
            <SheetTitle className="text-center">{title}</SheetTitle>
            {heading}
          </SheetHeader>
          <div className="min-h-0 overflow-y-auto px-4 py-3">{body}</div>
          <div className="border-t bg-muted/50">{footer}</div>
        </SheetContent>
      </Sheet>
    );
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      {/* A pinned top, not a centred box: the header keeps its place while the
          body grows or shrinks between the two steps. */}
      <DialogContent
        className="top-[6vh] max-h-[88vh] translate-y-0 grid-rows-[auto_minmax(0,1fr)_auto] pb-0 sm:max-w-180"
        onOpenAutoFocus={focusContent}
      >
        <DialogHeader className="items-center gap-2 text-center">
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription className="sr-only">Edit this template's parameters, then review the cases it generates.</DialogDescription>
          {heading}
        </DialogHeader>
        {body}
        <DialogFooter className="-mx-4 border-t sm:justify-stretch">
          <div className="w-full">{footer}</div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/* === Subcomponents === */

/**
 * Two equal slots around a short connector.
 *
 * The slots are equal so the pair stays centred under the title whichever
 * label is longer, and a finished step is ticked rather than merely dimmed:
 * the operator can see at a glance what is behind them and what is ahead.
 */
function StepIndicator({ step }: { step: DialogStep }) {
  const currentIndex = STEP_LABELS.findIndex((entry) => entry.key === step);

  return (
    <ol
      aria-hidden
      className="mx-auto grid w-full max-w-72 grid-cols-(--layout-comparison) items-center text-sm"
      data-testid="step-indicator"
    >
      <StepSlot done={currentIndex > 0} entry={STEP_LABELS[0]} step={step} />
      <li className="border-t" />
      <StepSlot done={currentIndex > 1} entry={STEP_LABELS[1]} step={step} />
    </ol>
  );
}

function StepSlot({ done, entry, step }: { done: boolean; entry: (typeof STEP_LABELS)[number] | undefined; step: DialogStep }) {
  if (!entry) return null;
  const current = entry.key === step;
  return (
    <li className="flex items-center justify-center gap-2">
      {done ? <Check aria-hidden className="size-4 text-primary" /> : null}
      <span className={cn(current ? "font-semibold text-primary" : done ? "text-foreground" : "text-muted-foreground")}>{entry.label}</span>
    </li>
  );
}

/**
 * What "Finish And Add" will materialize, read straight off the preview.
 *
 * Every row is one planned case: a readable name built from the template's own
 * label, and the values that differ from what the template registered, stated
 * as `Label: value unit`. A parameter the operator never touched and an
 * expansion that never fired both say so rather than printing a key.
 */
function SummaryStep({
  preview,
  rows,
  target,
  title,
}: {
  preview: PreviewTestCycleResponse | null;
  rows: EditorParameterRow[];
  target: string;
  title: string;
}): ReactNode {
  if (!preview) {
    return <p className="px-4 text-muted-foreground">No preview was returned.</p>;
  }
  const ordinals = preview.cases.length > 1;

  return (
    <div className="grid content-start gap-2 px-4">
      <div className="grid grid-cols-[minmax(0,1fr)_auto] items-baseline gap-3 border-b pb-1">
        <p className="text-sm text-muted-foreground">
          Target Cycle: <span className="text-foreground">{target}</span>
        </p>
        <p className="text-xs tabular-nums text-muted-foreground">
          {preview.case_count} {preview.case_count === 1 ? "Case" : "Cases"}
        </p>
      </div>
      <ul className="grid" data-testid="summary-cases">
        {preview.cases.map((item, index) => {
          const deltas = caseDeltas(item.parameters, rows);
          return (
            <li
              className="grid min-h-[var(--list-row-height)] grid-cols-[minmax(0,2fr)_minmax(0,3fr)] items-center gap-3 border-b border-border/60 py-2"
              key={item.case_id}
            >
              <span className="min-w-0 text-sm font-medium">{ordinals ? `${title} ${index + 1}` : title}</span>
              <span className="flex min-w-0 flex-wrap items-center gap-1">
                {deltas.length ? (
                  deltas.map((delta) => (
                    <span className="rounded-lg bg-muted px-2 py-1 text-xs text-muted-foreground" key={delta}>
                      {delta}
                    </span>
                  ))
                ) : (
                  <span className="text-xs text-muted-foreground">No Changes From Defaults</span>
                )}
              </span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}

/* === Hooks === */

/**
 * Touch density, read from the `data-density` attribute the shell writes on
 * `html`. That attribute is the one source both the native appearance settings
 * and the ui-store converge on.
 */
export function useTouchDensity(): boolean {
  return useSyncExternalStore(subscribeToDensity, readTouchDensity, () => false);
}

/* === Private Functions === */

function subscribeToDensity(onChange: () => void): () => void {
  if (typeof MutationObserver === "undefined") {
    return () => undefined;
  }
  const observer = new MutationObserver(onChange);
  observer.observe(document.documentElement, {
    attributeFilter: ["data-density"],
    attributes: true,
  });
  return () => observer.disconnect();
}

function readTouchDensity(): boolean {
  return document.documentElement.dataset.density === "touch";
}

/**
 * Opening focuses the dialog itself rather than its first tabbable control.
 *
 * The first control is a copy button under the title; focusing it opened its
 * tooltip over the title on every open. Tab still reaches it, and hover still
 * shows its tooltip.
 */
function focusContent(event: Event) {
  event.preventDefault();
  (event.currentTarget as HTMLElement).focus();
}

function messageOf(failure: unknown): string {
  return failure instanceof Error ? failure.message : String(failure);
}
