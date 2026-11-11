/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
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
  unapplied?: string[];
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
export function InstalledPluginSettingsForm({ schema, values, discovered, onSubmit, unapplied = [] }: Props) {
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
          unapplied={unapplied.includes(field.id)}
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
  unapplied,
}: {
  field: Exclude<SettingsEntry, { kind: "secret" }>;
  value: SettingValues[string] | undefined;
  onSave: (value: SettingValues[string] | undefined) => Promise<void>;
  discovered: boolean;
  unapplied: boolean;
}) {
  const id = useId();
  const [fieldValue, setFieldValue] = useState(value);
  const [numberText, setNumberText] = useState(String(value ?? (field.kind === "number" ? (field.default ?? "") : "")));
  const edited = useRef(false);
  const version = useRef(0);
  useEffect(() => {
    if (edited.current) return;
    setFieldValue(value);
    if (field.kind === "number") setNumberText(String(value ?? field.default ?? ""));
  }, [value, field]);
  const [invalid, setInvalid] = useState(false);
  const autosave = useAutosave<SettingValues[string] | undefined>(async (next) => {
    const saving = version.current;
    await onSave(next);
    if (saving === version.current) edited.current = false;
  });
  const onChange = (next: SettingValues[string] | undefined) => {
    edited.current = true;
    version.current++;
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
          value={numberText}
          onChange={(event) => {
            edited.current = true;
            version.current++;
            setNumberText(event.target.value);
            if (!event.currentTarget.validity.valid || !Number.isFinite(event.target.valueAsNumber)) {
              setInvalid(true);
              autosave.cancel();
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
        <Select
          value={String(
            field.choices.findIndex((choice) => choice.value === (typeof fieldValue === "string" ? fieldValue : field.default)),
          )}
          onValueChange={(value) => onChange(field.choices[Number(value)].value)}
        >
          <SelectTrigger className="w-full" id={id}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {field.choices.map((choice, index) => (
              <SelectItem key={choice.value} value={String(index)}>
                {choice.title}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      );
      break;
  }
  return (
    <SettingsRow
      className={unapplied ? "border-l-2 border-orange-500 bg-orange-500/10" : undefined}
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
          {unapplied ? (
            <Badge
              variant="warning"
              className="border-orange-500/30 bg-orange-500/10 text-orange-700 dark:text-orange-300"
              data-setting-unapplied={field.id}
            >
              Applied on Reload
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
