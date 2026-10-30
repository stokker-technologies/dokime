/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { type ReactNode, useEffect, useId, useRef, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { SettingsFieldStatus } from "@/features/settings/SettingsFieldStatus";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { useAutosave } from "@/features/settings/settings-autosave";
import type { Settings, SettingsEntry, SettingValues } from "./installed-plugin-contracts";

type Props = {
  schema: Settings;
  values: SettingValues;
  /**
   * What the plugin discovered for the fields nobody saved.
   *
   * Offered in the form rather than merged into the saved values: the field
   * shows the discovered value with a marker, and submitting the form is what
   * turns it into an edited one.
   */
  discovered: SettingValues;
  /** Resolves when the write landed, so each row can state its own outcome. */
  onSubmit: (values: SettingValues) => Promise<void>;
};

/**
 * The host owns controls for the semantic plugin catalog.
 *
 * A plugin's settings are settings, so they sit on the same two-column grid as
 * every other settings section — `label | value`, the description in
 * the label cell, one row height — rather than a stack of labels over
 * full-width fields. The schema carries no groups today, so there are no group
 * eyebrows to draw; when `SettingsEntry` gains one, the eyebrow goes here.
 */
export function InstalledPluginSettingsForm({ schema, values, discovered, onSubmit }: Props) {
  // The draft starts from the discovered values written over the saved ones, so
  // a field the plugin filled in shows what it found. Nothing is confirmed
  // twice: every change writes after the same debounce, and the row reports it.
  const [draft, setDraft] = useState<SettingValues>({ ...values, ...discovered });
  const latest = useRef(draft);
  latest.current = draft;
  useEffect(() => setDraft({ ...values, ...discovered }), [values, discovered]);
  const fields = (schema.contributions ?? []).filter((field) => field.kind !== "secret");
  if (!fields.length) return <p className="px-4 py-3 text-sm text-muted-foreground">This program has no configurable settings.</p>;

  /* One field's write is the whole values map with that field replaced: the
     mutation takes the map, and the row that changed is the one that reports. */
  function write(id: string, value: SettingValues[string] | undefined): SettingValues {
    const next = { ...latest.current };
    if (value === undefined) delete next[id];
    else next[id] = value;
    latest.current = next;
    setDraft(next);
    return next;
  }

  return (
    <>
      {fields.map((field) => (
        <SettingField
          discovered={Object.hasOwn(discovered, field.id)}
          field={field}
          key={field.id}
          onSave={(value) => onSubmit(write(field.id, value))}
          value={draft[field.id]}
        />
      ))}
    </>
  );
}

function SettingField({
  field,
  value,
  onSave,
  discovered,
}: {
  field: Exclude<SettingsEntry, { kind: "secret" }>;
  value: SettingValues[string] | undefined;
  onSave: (value: SettingValues[string] | undefined) => Promise<void>;
  discovered: boolean;
}) {
  const id = useId();
  const [fieldValue, setFieldValue] = useState(value);
  useEffect(() => setFieldValue(value), [value]);
  const [invalid, setInvalid] = useState(false);
  const autosave = useAutosave<SettingValues[string] | undefined>(onSave);
  const onChange = (next: SettingValues[string] | undefined) => {
    setFieldValue(next);
    autosave.change(next);
  };
  let control: ReactNode;
  switch (field.kind) {
    case "boolean":
      control = <Switch id={id} checked={typeof fieldValue === "boolean" ? fieldValue : field.default} onCheckedChange={onChange} />;
      break;
    case "text_area":
      control = (
        <Textarea
          id={id}
          required={field.required}
          value={typeof fieldValue === "string" ? fieldValue : field.default}
          onChange={(event) => onChange(event.target.value)}
        />
      );
      break;
    case "path":
      control = (
        <Input
          id={id}
          required={field.required}
          value={typeof fieldValue === "string" ? fieldValue : field.default}
          onChange={(event) => onChange(event.target.value)}
        />
      );
      break;
    case "number":
      control = (
        <Input
          id={id}
          className="text-right tabular-nums"
          type="number"
          required
          min={field.min ?? undefined}
          max={field.max ?? undefined}
          step={field.step ?? "any"}
          value={typeof fieldValue === "number" ? fieldValue : (field.default ?? "")}
          onChange={(event) => {
            if (!event.currentTarget.validity.valid || !Number.isFinite(event.target.valueAsNumber)) {
              setInvalid(true);
              return;
            }
            setInvalid(false);
            onChange(event.target.valueAsNumber);
          }}
        />
      );
      break;
    case "multiple_choice":
      control = (
        <select
          className="h-[var(--control-height)] w-full rounded-lg border border-input bg-background px-3 text-sm focus-visible:outline-2 focus-visible:outline-ring"
          id={id}
          value={typeof fieldValue === "string" ? fieldValue : field.default}
          onChange={(event) => onChange(event.target.value)}
        >
          {field.choices.map((choice) => (
            <option key={choice.value} value={choice.value}>
              {choice.title}
            </option>
          ))}
        </select>
      );
      break;
  }
  return (
    <SettingsRow
      description={field.description}
      error={invalid ? "Enter a number within the allowed range." : undefined}
      htmlFor={id}
      label={field.title}
      status={
        <span className="flex items-center gap-2">
          {discovered ? (
            <Badge variant="secondary" title="Found by the program. Changing it keeps it.">
              Discovered
            </Badge>
          ) : null}
          <SettingsFieldStatus onRetry={autosave.retry} state={autosave.state} />
        </span>
      }
    >
      <div className="grid w-full min-w-0 gap-2">{control}</div>
    </SettingsRow>
  );
}
