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
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { SettingsFieldStatus } from "@/features/settings/SettingsFieldStatus";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { useAutosave } from "@/features/settings/settings-autosave";
import type {
  PluginRuntimeCredentialBindings,
  SecretReference,
  Settings,
  SettingsEntry,
  SettingValues,
} from "./installed-plugin-contracts";

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
  onSaveSecret: (id: string, secret: string) => void;
  savingSecret: boolean;
  bindings: PluginRuntimeCredentialBindings;
};

/**
 * The host owns controls for the semantic plugin catalog.
 *
 * A plugin's settings are settings, so they sit on the same two-column grid as
 * every other settings section — `label (260px) | value`, the description in
 * the label cell, one row height — rather than a stack of labels over
 * full-width fields. The schema carries no groups today, so there are no group
 * eyebrows to draw; when `SettingsEntry` gains one, the eyebrow goes here.
 */
export function InstalledPluginSettingsForm({
  schema,
  values,
  discovered,
  onSubmit,
  onSaveSecret,
  savingSecret,
  bindings,
}: Props) {
  // The draft starts from the discovered values written over the saved ones, so
  // a field the plugin filled in shows what it found. Nothing is confirmed
  // twice: a change writes itself, debounced for text and at once for a switch
  // or a select, and the row says what became of it.
  const [draft, setDraft] = useState<SettingValues>({ ...values, ...discovered });
  const latest = useRef(draft);
  latest.current = draft;
  useEffect(() => setDraft({ ...values, ...discovered }), [values, discovered]);
  const fields = schema.contributions ?? [];
  if (!fields.length)
    return (
      <p className="px-4 py-3 text-sm text-muted-foreground">
        This program has no configurable settings.
      </p>
    );

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
    <div className="grid">
      {fields.map((field) => (
        <SettingField
          configured={Boolean(bindings.bindings?.[field.id]?.configured)}
          discovered={Object.hasOwn(discovered, field.id)}
          field={field}
          key={field.id}
          onSave={(value) => onSubmit(write(field.id, value))}
          onSaveSecret={(secret) => onSaveSecret(field.id, secret)}
          savingSecret={savingSecret}
          value={draft[field.id]}
        />
      ))}
      {(bindings.missing_required ?? []).length > 0 && (
        <p className="px-4 pt-3 text-xs text-destructive">
          Missing required secrets: {(bindings.missing_required ?? []).join(", ")}
        </p>
      )}
    </div>
  );
}

function SettingField({
  field,
  value,
  onSave,
  onSaveSecret,
  savingSecret,
  configured,
  discovered,
}: {
  field: SettingsEntry;
  value: SettingValues[string] | undefined;
  onSave: (value: SettingValues[string] | undefined) => Promise<void>;
  onSaveSecret: (secret: string) => void;
  savingSecret: boolean;
  configured: boolean;
  discovered: boolean;
}) {
  const id = useId();
  const autosave = useAutosave<SettingValues[string] | undefined>(onSave);
  const onChange = autosave.change;
  const onCommit = autosave.commit;
  let control: ReactNode;
  switch (field.kind) {
    case "boolean":
      control = (
        <Switch
          id={id}
          checked={typeof value === "boolean" ? value : field.default}
          onCheckedChange={onCommit}
        />
      );
      break;
    case "text_area":
      control = (
        <Textarea
          id={id}
          required={field.required}
          value={typeof value === "string" ? value : field.default}
          onChange={(event) => onChange(event.target.value)}
        />
      );
      break;
    case "path":
      control = (
        <Input
          id={id}
          required={field.required}
          value={typeof value === "string" ? value : field.default}
          onChange={(event) => onChange(event.target.value)}
        />
      );
      break;
    case "number":
      control = (
        <Input
          id={id}
          type="number"
          required
          min={field.min ?? undefined}
          max={field.max ?? undefined}
          step={field.step ?? "any"}
          value={typeof value === "number" ? value : (field.default ?? "")}
          onChange={(event) =>
            onChange(event.target.value === "" ? undefined : event.target.valueAsNumber)
          }
        />
      );
      break;
    case "multiple_choice":
      control = (
        <select
          className="h-9 rounded-md border bg-background px-3 text-sm"
          id={id}
          value={typeof value === "string" ? value : field.default}
          onChange={(event) => onCommit(event.target.value)}
        >
          {field.choices.map((choice) => (
            <option key={choice.value} value={choice.value}>
              {choice.title}
            </option>
          ))}
        </select>
      );
      break;
    case "secret_reference":
      control = (
        <SecretField
          id={id}
          reference={secretReference(value) ?? field.default ?? undefined}
          onChange={onCommit}
          onSave={onSaveSecret}
          saving={savingSecret}
          configured={configured}
        />
      );
      break;
  }
  return (
    <SettingsRow
      className="border-b border-border/60 last:border-b-0"
      description={field.description}
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
      <div className="grid min-w-0 gap-2">{control}</div>
    </SettingsRow>
  );
}

function SecretField({
  id,
  reference,
  onChange,
  onSave,
  saving,
  configured,
}: {
  id: string;
  reference?: SecretReference;
  onChange: (value: SecretReference | undefined) => void;
  onSave: (secret: string) => void;
  saving: boolean;
  configured: boolean;
}) {
  const [secret, setSecret] = useState("");
  const kind = reference?.kind ?? "stored_credential";
  return (
    <div className="grid gap-2">
      <select
        className="h-9 rounded-md border bg-background px-3 text-sm"
        aria-label="Secret reference source"
        value={kind}
        onChange={(event) =>
          onChange(
            event.target.value === "environment"
              ? { kind: "environment", variable: "" }
              : { kind: "stored_credential", credential_id: "" },
          )
        }
      >
        <option value="stored_credential">Stored credential</option>
        <option value="environment">Environment variable</option>
      </select>
      <Input
        id={id}
        autoComplete="off"
        placeholder={kind === "environment" ? "ENVIRONMENT_VARIABLE" : "Stored credential ID"}
        value={
          reference?.kind === "environment" ? reference.variable : (reference?.credential_id ?? "")
        }
        onChange={(event) =>
          onChange(
            event.target.value === ""
              ? undefined
              : kind === "environment"
                ? { kind, variable: event.target.value }
                : { kind, credential_id: event.target.value },
          )
        }
      />
      {kind === "stored_credential" && (
        <div className="flex gap-2">
          <Input
            aria-label="Secret value"
            type="password"
            autoComplete="new-password"
            placeholder={configured ? "Replace stored secret" : "Enter secret"}
            value={secret}
            onChange={(event) => setSecret(event.target.value)}
          />
          <Button
            type="button"
            disabled={saving || !secret}
            onClick={() => {
              onSave(secret);
              setSecret("");
            }}
          >
            {configured ? "Replace" : "Save"}
          </Button>
        </div>
      )}
    </div>
  );
}

function secretReference(value: SettingValues[string] | undefined): SecretReference | undefined {
  if (!value || typeof value !== "object" || Array.isArray(value)) return undefined;
  if (value.kind === "environment" && typeof value.variable === "string") {
    return { kind: "environment", variable: value.variable };
  }
  if (value.kind === "stored_credential" && typeof value.credential_id === "string") {
    return { kind: "stored_credential", credential_id: value.credential_id };
  }
  return undefined;
}
