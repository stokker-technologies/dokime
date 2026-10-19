/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The materialization dialog's parameter step (concept 11a).
 *
 * One grid for the whole step, `label | value | reset`, hairline rules between
 * rows and quiet eyebrows for the parameter groups. No cards, no per-row
 * frames: alignment carries the structure. */

/* === External Imports === */

import { Clipboard, RotateCcw } from "lucide-react";
import type { CSSProperties } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import type { FileRowState } from "@/components/start-cycle-form-model";
import { parameterInputId } from "@/components/start-cycle-form-model";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import type { FileDefinition } from "@/lib/types";
import { cn } from "@/lib/utils";
import {
  acceptsExpressions,
  type EditorParameterRow,
  editorProblemCount,
  expansionStatement,
  groupEditorRows,
  isEditedRow,
  unitColumnCharacters,
} from "./model";
import { ExpressionReferencePopover, FileValueCell, ParameterValueCell } from "./parameter-cells";

/* === Constants === */

/** 40px rows, 48px at touch; the whole grid reads off this one measure. */
const GRID_DENSITY = "[--parameter-row-height:2.5rem] [[data-density=touch]_&]:[--parameter-row-height:3rem]";

const GRID_COLUMNS = "grid-cols-[minmax(0,2fr)_minmax(0,3fr)_2.5rem]";

/**
 * One inset for every row of every group.
 *
 * The 2px edge rule is part of the inset, so an edited row and an untouched
 * row start their labels on the same line: 14px of padding behind a 2px rule
 * is the 16px the concept asks for.
 */
const ROW_INSET = "border-l-2 pr-4 pl-[0.875rem]";

/** The eyebrow above a parameter group; the first one does not need the air. */
const GROUP_HEADING = "px-4 pb-1 text-xs font-semibold tracking-wide text-muted-foreground uppercase";

/* === Types === */

export type ParameterEditorProps = {
  fileDefinitions: FileDefinition[];
  fileRows: FileRowState[];
  onChangeRaw: (name: string, raw: string) => void;
  onFileChange: (definition: FileDefinition, file: File | undefined) => void;
  onFilePathChange: (name: string, path: string) => void;
  onResetAll: () => void;
  onResetRow: (name: string) => void;
  onResetFile: (name: string) => void;
  rows: EditorParameterRow[];
};

/* === Components === */

export function ParameterEditor({
  fileDefinitions,
  fileRows,
  onChangeRaw,
  onFileChange,
  onFilePathChange,
  onResetAll,
  onResetRow,
  onResetFile,
  rows,
}: ParameterEditorProps) {
  const groups = groupEditorRows(rows);
  const edited =
    rows.some(isEditedRow) ||
    fileDefinitions.some((definition) => {
      const row = fileRows.find((candidate) => candidate.name === definition.name);
      return row && (row.path !== (definition.default ?? "") || row.error !== null);
    });
  const problems = editorProblemCount(rows, fileRows, fileDefinitions);

  return (
    <div
      className={cn("grid content-start gap-2", GRID_DENSITY)}
      style={{ "--parameter-unit-width": `${unitColumnCharacters(rows)}ch` } as CSSProperties}
    >
      <div className="flex items-baseline justify-between gap-3 border-b px-4 pb-1">
        <p className="text-xs text-muted-foreground">
          {problems
            ? `${problems} ${problems === 1 ? "problem" : "problems"} to fix before the summary.`
            : "Values you change are remembered for this template."}
        </p>
        <Button disabled={!edited || fileRows.some((row) => row.uploading)} onClick={onResetAll} size="sm" type="button" variant="ghost">
          Reset All
        </Button>
      </div>

      <div className="grid" data-testid="parameter-editor-grid">
        {groups.map((group, index) => (
          <section className="grid" key={group.label}>
            <p className={cn(GROUP_HEADING, index === 0 ? "pt-1" : "pt-5")}>{group.label}</p>
            {group.rows.map((row) => (
              <ParameterEditorRow key={row.definition.parameter_id} onChangeRaw={onChangeRaw} onResetRow={onResetRow} row={row} />
            ))}
          </section>
        ))}

        {fileDefinitions.length ? (
          <section className="grid">
            <p className={cn(GROUP_HEADING, groups.length === 0 ? "pt-1" : "pt-5")}>Files</p>
            {fileDefinitions.map((definition) => {
              const row = fileRows.find((candidate) => candidate.name === definition.name) ?? {
                error: null,
                name: definition.name,
                path: definition.default ?? "",
                uploading: false,
              };
              return (
                <div
                  className={cn(
                    "grid min-h-[var(--parameter-row-height)] items-center gap-3 border-b border-border/60 py-1",
                    ROW_INSET,
                    "border-l-transparent",
                    GRID_COLUMNS,
                  )}
                  key={definition.name}
                >
                  <ParameterLabelCell
                    description={definition.description ?? null}
                    inputId={`${parameterInputId(definition.name)}-file`}
                    label={definition.label ?? definition.name}
                  />
                  <div className="grid min-w-0 gap-0.5">
                    {/* The same reserved help slot a parameter row keeps, so
                        every field of the step shares one right edge. */}
                    <div className="flex min-w-0 items-center gap-1">
                      <div className="min-w-0 flex-1">
                        <FileValueCell
                          definition={definition}
                          onFileChange={(file) => onFileChange(definition, file)}
                          onPathChange={(path) => onFilePathChange(definition.name, path)}
                          row={row}
                        />
                      </div>
                      <span className="size-6" />
                    </div>
                    {row.error ? (
                      <p className="text-xs text-destructive" role="alert">
                        {row.error}
                      </p>
                    ) : null}
                  </div>
                  <div className="justify-self-center">
                    {row.path !== (definition.default ?? "") || row.error ? (
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            aria-label={`Reset ${definition.label ?? definition.name} To Template Default`}
                            disabled={row.uploading}
                            onClick={() => onResetFile(definition.name)}
                            size="icon-sm"
                            type="button"
                            variant="ghost"
                          >
                            <RotateCcw aria-hidden />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>Reset To Template Default</TooltipContent>
                      </Tooltip>
                    ) : null}
                  </div>
                </div>
              );
            })}
          </section>
        ) : null}
      </div>
    </div>
  );
}

/* === Subcomponents === */

function ParameterEditorRow({
  row,
  onChangeRaw,
  onResetRow,
}: {
  row: EditorParameterRow;
  onChangeRaw: (name: string, raw: string) => void;
  onResetRow: (name: string) => void;
}) {
  const { definition } = row;
  const edited = isEditedRow(row);
  const expansion = expansionStatement(definition);
  const error = row.parsed.ok ? null : row.parsed.error;

  return (
    <div
      className={cn(
        "grid min-h-[var(--parameter-row-height)] items-center gap-3 border-b border-border/60 py-1",
        ROW_INSET,
        // The gilt rule marks what this operator changed, at a glance.
        edited ? "border-l-primary" : "border-l-transparent",
        GRID_COLUMNS,
      )}
      data-edited={edited ? "true" : undefined}
      data-testid={`parameter-row-${row.name}`}
    >
      <ParameterLabelCell
        description={definition.description ?? null}
        inputId={parameterInputId(definition.parameter_id)}
        label={definition.label ?? row.name}
      />

      <div className="grid min-w-0 gap-0.5">
        {/* The help sits beside the field, not inside it: the value column
            keeps its edge and the icon stays on one vertical down the step. */}
        <div className="flex min-w-0 items-center gap-1">
          <div className="min-w-0 flex-1">
            <ParameterValueCell onChange={(raw) => onChangeRaw(row.name, raw)} row={row} />
          </div>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                aria-label={`Copy ${definition.label ?? row.name} Parameter ID`}
                onClick={() => {
                  void copyParameterId(definition.parameter_id);
                }}
                size="icon-sm"
                type="button"
                variant="ghost"
              >
                <Clipboard aria-hidden />
              </Button>
            </TooltipTrigger>
            <TooltipContent className="font-mono">{definition.parameter_id}</TooltipContent>
          </Tooltip>
          {/* The help keeps its own slot beside the field whether or not the
              row has one, so it never competes with Reset for the same x. */}
          {acceptsExpressions(row) ? <ExpressionReferencePopover /> : <span className="size-6 [[data-density=touch]_&]:size-11" />}
        </div>
        {error ? (
          <p className="text-xs text-destructive" role="alert">
            {error}
          </p>
        ) : expansion ? (
          <p className="flex flex-wrap items-center gap-1.5 text-xs leading-snug text-muted-foreground">
            {expansion.badge ? (
              /* A statement, not a control: no border, no hover, no pointer. */
              <span className="pointer-events-none rounded-sm bg-muted px-1.5 py-0.5 text-[0.7rem] font-semibold tracking-wide text-muted-foreground uppercase select-none">
                {expansion.badge}
              </span>
            ) : null}
            <span>{expansion.sentence}</span>
          </p>
        ) : null}
      </div>

      <div className="justify-self-center">
        {edited || error ? (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                aria-label={`Reset To ${resetTargetLabel(row)}`}
                onClick={() => onResetRow(row.name)}
                size="icon-sm"
                type="button"
                variant="ghost"
              >
                <RotateCcw aria-hidden />
              </Button>
            </TooltipTrigger>
            <TooltipContent>{`Reset To ${resetTargetLabel(row)}`}</TooltipContent>
          </Tooltip>
        ) : null}
      </div>
    </div>
  );
}

function ParameterLabelCell({ description, inputId, label }: { description: string | null; inputId: string; label: string }) {
  return (
    <div className="grid min-w-0 gap-0.5">
      <div className="flex min-w-0 items-center gap-1">
        <Label className="min-w-0 text-sm leading-snug font-medium" htmlFor={inputId} title={label}>
          {label}
        </Label>
      </div>
      {description ? (
        /* The description is the row's second line and may be more: rows grow
           to fit it rather than cutting the sentence that explains the
           parameter. */
        <p className="text-xs leading-snug text-muted-foreground">{description}</p>
      ) : null}
    </div>
  );
}

/* === Private Functions === */
async function copyParameterId(parameterId: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(parameterId);
    toast.success("Parameter ID copied");
  } catch {
    toast.error("Could not copy parameter ID");
  }
}

/** The accessible name says what the reset restores, with the unit when there is one. */
function resetTargetLabel(row: EditorParameterRow): string {
  const raw = row.defaultRaw.trim();
  const unit = row.definition.unit?.trim() ?? "";
  if (!raw) {
    return "Template Default";
  }
  return unit ? `${raw} ${unit}` : raw;
}
