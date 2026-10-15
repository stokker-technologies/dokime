/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* One pending prompt, printed in place: its state, title, instruction, fields
 * and actions, with no box of its own. The surface that hosts it decides the
 * frame; the prompt's own "Run Paused" line already says the run waits.
 */

/* === External Imports === */

import { AlertTriangle, LoaderCircle, Plus, Trash2 } from "lucide-react";
import type { ReactNode } from "react";

/* === Local Imports === */

import { PathPill } from "@/components/PathPill";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { formatNumber } from "@/lib/format";
import {
  type PromptValue,
  type PromptValues,
  promptFieldDefault,
  promptFieldError,
} from "@/lib/prompts";
import type { PromptField, PromptRequest } from "@/lib/types";
import { occurrenceKeys } from "@/lib/utils";

/* === Types === */

type PromptPanelProps = {
  prompt: PromptRequest | undefined;
  submittingAction?: string | null;
  values: PromptValues;
  emergencyAction?: ReactNode;
  onAction: (action: string) => void;
  onValueChange: (field: PromptField, value: PromptValue) => void;
};

/* === Components === */

export function PromptPanel({
  prompt,
  submittingAction = null,
  values,
  emergencyAction,
  onAction,
  onValueChange,
}: PromptPanelProps) {
  if (!prompt) {
    return null;
  }

  if (prompt.kind === "status") {
    return <StatusPromptDialog emergencyAction={emergencyAction} prompt={prompt} />;
  }
  if (prompt.kind === "verification") {
    return <VerificationPromptDialog emergencyAction={emergencyAction} prompt={prompt} />;
  }
  return (
    <PromptDialog
      emergencyAction={emergencyAction}
      prompt={prompt}
      submittingAction={submittingAction}
      values={values}
      onAction={onAction}
      onValueChange={onValueChange}
    />
  );
}

function StatusPromptDialog({
  emergencyAction,
  prompt,
}: {
  emergencyAction?: ReactNode;
  prompt: PromptRequest;
}) {
  return (
    <div
      aria-live="polite"
      aria-labelledby="active-prompt-title"
      className="grid min-w-0 gap-4"
      role="status"
    >
      <div className="flex min-w-0 gap-3">
        <div className="flex size-9 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary">
          <LoaderCircle className="animate-spin" size={18} />
        </div>
        <div className="grid min-w-0 gap-1">
          <p className="text-xs font-bold uppercase text-primary">Working</p>
          <strong id="active-prompt-title" className="text-base [overflow-wrap:anywhere]">
            {prompt.title}
          </strong>
          {prompt.message ? (
            <p className="whitespace-pre-line text-sm text-muted-foreground [overflow-wrap:anywhere]">
              {prompt.message}
            </p>
          ) : null}
        </div>
      </div>
      {emergencyAction ? (
        <div className="flex justify-end border-t pt-3">{emergencyAction}</div>
      ) : null}
    </div>
  );
}

function VerificationPromptDialog({
  emergencyAction,
  prompt,
}: {
  emergencyAction?: ReactNode;
  prompt: PromptRequest;
}) {
  return (
    <section aria-labelledby="active-prompt-title" className="grid min-w-0 gap-4">
      <div className="flex min-w-0 gap-3">
        <div className="flex size-9 shrink-0 items-center justify-center rounded-full bg-destructive/10 text-destructive">
          <LoaderCircle className="animate-spin" size={18} />
        </div>
        <div className="grid min-w-0 gap-1">
          <p className="text-xs font-bold uppercase text-destructive">Waiting For Verification</p>
          <strong id="active-prompt-title" className="text-base [overflow-wrap:anywhere]">
            {prompt.title}
          </strong>
          {prompt.message ? (
            <p className="whitespace-pre-line text-sm text-muted-foreground [overflow-wrap:anywhere]">
              {prompt.message}
            </p>
          ) : null}
        </div>
      </div>

      <div className="rounded-lg border bg-muted/25 p-3">
        <p className="text-xs font-bold uppercase text-muted-foreground">Required Condition</p>
        <p className="mt-1 text-sm font-medium">{verificationConditionLabel(prompt)}</p>
      </div>
      {emergencyAction ? (
        <div className="flex justify-end border-t pt-3">{emergencyAction}</div>
      ) : null}
    </section>
  );
}

function verificationConditionLabel(prompt: PromptRequest) {
  const condition = prompt.verification?.condition;
  if (!condition) {
    return "Verification condition";
  }
  if (condition.kind === "record") {
    const detailLabels = Object.entries(condition.details ?? {}).map(
      ([key, value]) => `${key}=${String(value)}`,
    );
    return detailLabels.length
      ? `${condition.record_type} with ${detailLabels.join(", ")}`
      : condition.record_type;
  }
  const unit = condition.unit ? ` ${condition.unit}` : "";
  const tolerance =
    condition.operator === "within"
      ? ` +/- ${promptNumber(condition.tolerance ?? 0.5)}${unit}`
      : "";
  return `${condition.stream}.${condition.field} ${operatorLabel(
    condition.operator,
  )} ${promptNumber(condition.target)}${unit}${tolerance}`;
}

function operatorLabel(operator: string) {
  if (operator === "within") {
    return "within";
  }
  if (operator === "gte") {
    return ">=";
  }
  if (operator === "lte") {
    return "<=";
  }
  if (operator === "gt") {
    return ">";
  }
  if (operator === "lt") {
    return "<";
  }
  return "=";
}

/* A verification threshold reads as up to three decimals; a number the payload
 * cannot express reads as a dash rather than as empty space. */
function promptNumber(value: number): string {
  return Number.isFinite(value)
    ? formatNumber(value, { kind: "normal", max_fraction_digits: 3 })
    : "-";
}

function PromptDialog({
  prompt,
  submittingAction,
  values,
  emergencyAction,
  onAction,
  onValueChange,
}: {
  prompt: PromptRequest;
  submittingAction: string | null;
  values: PromptValues;
  emergencyAction?: ReactNode;
  onAction: (action: string) => void;
  onValueChange: (field: PromptField, value: PromptValue) => void;
}) {
  const fieldErrors = Object.fromEntries(
    prompt.fields.map((field) => [field.name, promptFieldError(field, values[field.name])]),
  );
  const invalidFields = Object.values(fieldErrors).some(Boolean);

  return (
    <section aria-labelledby="active-prompt-title" className="grid min-w-0 gap-4">
      <div className="flex min-w-0 gap-3">
        <div className="flex size-9 shrink-0 items-center justify-center rounded-full bg-destructive/10 text-destructive">
          <AlertTriangle size={18} />
        </div>
        <div className="grid min-w-0 gap-1">
          <p className="text-xs font-bold uppercase text-destructive">Run Paused</p>
          <strong id="active-prompt-title" className="text-base [overflow-wrap:anywhere]">
            {prompt.title}
          </strong>
          {prompt.message ? <PromptMessage message={prompt.message} /> : null}
        </div>
      </div>

      {prompt.fields.length ? (
        <div className="grid gap-3">
          {prompt.fields.map((field) => (
            <div className="grid gap-1.5" key={field.name}>
              <Label
                className="text-xs font-bold uppercase text-muted-foreground"
                htmlFor={promptFieldId(field)}
              >
                {field.label}
              </Label>
              <PromptFieldControl
                field={field}
                value={values[field.name]}
                onChange={onValueChange}
              />
              {fieldErrors[field.name] ? (
                <p className="text-xs text-destructive" role="status">
                  {fieldErrors[field.name]}
                </p>
              ) : null}
            </div>
          ))}
        </div>
      ) : null}

      <div className="flex flex-wrap justify-between gap-2 border-t pt-3">
        <div>{emergencyAction}</div>
        <div className="flex flex-wrap justify-end gap-2">
          {prompt.actions.map((action) => (
            <Button
              key={action.id}
              variant={action.emphasis}
              disabled={
                submittingAction !== null ||
                (invalidFields && !["cancel", "abort"].includes(action.id))
              }
              onClick={() => onAction(action.id)}
            >
              {submittingAction === action.id ? "Submitting..." : action.label}
            </Button>
          ))}
        </div>
      </div>
    </section>
  );
}

function PromptMessage({ message }: { message: string }) {
  const sections = message
    .split(/\n\s*\n/u)
    .map((section) => section.trim())
    .filter(Boolean);
  if (sections.length <= 1) {
    return (
      <p className="text-sm text-muted-foreground [overflow-wrap:anywhere]">
        <InlinePathText text={message} />
      </p>
    );
  }

  const lastSection = sections.at(-1) ?? "";
  const hasAction = !isTechnicalPromptSection(lastSection);
  const context = sections.at(0) ?? "";
  const technicalSections = sections.slice(1, hasAction ? -1 : undefined);

  return (
    <div className="grid gap-3 text-sm">
      {context ? (
        <p className="text-muted-foreground [overflow-wrap:anywhere]">
          <InlinePathText text={context} />
        </p>
      ) : null}
      {technicalSections.length ? (
        <div className="grid gap-1.5 rounded-md border bg-muted/35 p-3">
          <p className="text-xs font-bold uppercase text-muted-foreground">Technical Detail</p>
          <pre className="whitespace-pre-wrap font-mono text-xs leading-relaxed text-muted-foreground [overflow-wrap:anywhere]">
            {technicalSections.join("\n\n")}
          </pre>
        </div>
      ) : null}
      {hasAction ? (
        <p className="rounded-md border border-destructive/20 bg-destructive/5 p-3 font-medium text-foreground [overflow-wrap:anywhere]">
          <InlinePathText text={lastSection} />
        </p>
      ) : null}
    </div>
  );
}

function InlinePathText({ text }: { text: string }) {
  const parts = text.split(/([A-Za-z]:\\[^\s.,;)]*)/gu);
  return (
    <>
      {occurrenceKeys(parts, (part) => part).map(({ key, value: part }) =>
        /^[A-Za-z]:\\/u.test(part) ? (
          <PathPill key={key} value={part} />
        ) : (
          <span className="whitespace-pre-line" key={key}>
            {part}
          </span>
        ),
      )}
    </>
  );
}

function isTechnicalPromptSection(section: string) {
  const normalized = section.trim().toLowerCase();
  return (
    normalized.startsWith("error:") ||
    normalized.startsWith("traceback") ||
    normalized.startsWith("stack trace") ||
    normalized.includes("\nat ")
  );
}

function PromptFieldControl({
  field,
  value,
  onChange,
}: {
  field: PromptField;
  value: PromptValue | undefined;
  onChange: (field: PromptField, value: PromptValue) => void;
}) {
  const resolvedValue = value ?? promptFieldDefault(field);
  if (field.multiple) {
    return (
      <MultiplePromptFieldControl
        field={field}
        value={Array.isArray(resolvedValue) ? resolvedValue : [String(resolvedValue)]}
        onChange={onChange}
      />
    );
  }
  if (field.kind === "choice") {
    return (
      <Select
        value={String(resolvedValue)}
        disabled={field.readonly}
        onValueChange={(next) => onChange(field, next)}
      >
        <SelectTrigger className="w-full" id={promptFieldId(field)}>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {(field.choices ?? []).map((choice) => (
            <SelectItem key={choice} value={choice}>
              {choice}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    );
  }
  if (field.kind === "boolean") {
    return (
      <div className="flex min-h-8 items-center gap-2">
        <Checkbox
          id={promptFieldId(field)}
          checked={Boolean(resolvedValue)}
          disabled={field.readonly}
          onCheckedChange={(checked) => onChange(field, checked === true)}
        />
        <Label className="font-medium" htmlFor={promptFieldId(field)}>
          {resolvedValue ? "Yes" : "No"}
        </Label>
      </div>
    );
  }
  return (
    <Input
      id={promptFieldId(field)}
      type={field.kind === "text" ? "text" : "number"}
      step={field.kind === "int" ? "1" : "any"}
      value={String(resolvedValue)}
      readOnly={field.readonly}
      required={field.required !== false}
      onChange={(record) => onChange(field, record.target.value)}
    />
  );
}

function MultiplePromptFieldControl({
  field,
  value,
  onChange,
}: {
  field: PromptField;
  value: string[];
  onChange: (field: PromptField, value: PromptValue) => void;
}) {
  const entries = value.length ? value : [""];

  function updateEntry(index: number, next: string) {
    onChange(
      field,
      entries.map((entry, entryIndex) => (entryIndex === index ? next : entry)),
    );
  }

  function addEntry() {
    onChange(field, [...entries, ""]);
  }

  function removeEntry(index: number) {
    const next = entries.filter((_, entryIndex) => entryIndex !== index);
    onChange(field, next.length ? next : [""]);
  }

  return (
    <div className="grid gap-2">
      {occurrenceKeys(entries, (entry) => entry).map(({ key, value: entry }, index) => (
        <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-2" key={`${field.name}-${key}`}>
          {field.kind === "choice" ? (
            <Select
              value={entry}
              disabled={field.readonly}
              onValueChange={(next) => updateEntry(index, next)}
            >
              <SelectTrigger
                aria-label={`${field.label} entry ${index + 1}`}
                className="w-full"
                id={index === 0 ? promptFieldId(field) : undefined}
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {(field.choices ?? []).map((choice) => (
                  <SelectItem key={choice} value={choice}>
                    {choice}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          ) : (
            <Input
              aria-label={`${field.label} entry ${index + 1}`}
              id={index === 0 ? promptFieldId(field) : undefined}
              type={field.kind === "text" ? "text" : "number"}
              step={field.kind === "int" ? "1" : "any"}
              value={entry}
              readOnly={field.readonly}
              required={field.required !== false}
              onChange={(record) => updateEntry(index, record.target.value)}
            />
          )}
          <Button
            aria-label={`Remove ${field.label} entry ${index + 1}`}
            disabled={field.readonly || entries.length <= 1}
            size="icon"
            type="button"
            variant="ghost"
            onClick={() => removeEntry(index)}
          >
            <Trash2 size={16} />
          </Button>
        </div>
      ))}
      <Button
        className="justify-self-start"
        disabled={field.readonly}
        size="sm"
        type="button"
        variant="outline"
        onClick={addEntry}
      >
        <Plus size={14} />
        Add Entry
      </Button>
    </div>
  );
}

function promptFieldId(field: PromptField) {
  return `prompt-field-${field.name}`;
}
