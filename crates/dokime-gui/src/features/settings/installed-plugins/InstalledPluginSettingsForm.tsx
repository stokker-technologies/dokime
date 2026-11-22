// Dokime — SPDX-License-Identifier: Apache-2.0

/* === Imports === */

import { useEffect, useId, useRef, useState } from "react";
import type { SettingDeclaration } from "@generated/host/model";
import { Switch } from "@/components/ui/switch";
import { Input } from "@dokime/plots/components/ui/input";
import { SettingsFieldStatus } from "@/features/settings/SettingsFieldStatus";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { useAutosave } from "@/features/settings/settings-autosave";
import type { Settings, SettingValues } from "./installed-plugin-contracts";
import { formatPluginSetting, parsePluginSetting, readPluginSetting } from "@/lib/api/native-settings";
import { required, type Wire } from "@dokime/plots/lib/rpc/canonical";

/* === Types === */

type Props = {
  schema: Settings;
  values: SettingValues;
  onSubmit: (values: SettingValues) => Promise<void>;
};

/* === Public Functions === */

export function InstalledPluginSettingsForm({ schema, values, onSubmit }: Props) {
  const writing = useRef(Promise.resolve());
  const fields = (schema.contributions ?? []).filter((field) => field.kind === "setting");
  if (!fields.length) {
    return <p className="px-4 py-3 text-sm text-muted-foreground">This program has no configurable settings.</p>;
  }

  return fields.map(({ declaration }) => (
    <SettingField
      field={declaration}
      key={declaration.name}
      value={values[declaration.name]}
      onSave={(value) => {
        const save = writing.current.then(() => onSubmit({ [declaration.name]: value }));
        writing.current = save.catch(() => undefined);
        return save;
      }}
    />
  ));
}

/* === Private Functions === */

function SettingField({
  field,
  value,
  onSave,
}: {
  field: Wire<SettingDeclaration>;
  value: SettingValues[string] | undefined;
  onSave: (value: SettingValues[string]) => Promise<void>;
}) {
  const id = useId();
  const defaultValue = required(field.default, "Read Plugin Setting Default");
  const kind = defaultValue.value.case;
  const current = parsePluginSetting(field, value ?? readPluginSetting(defaultValue));
  const [fieldValue, setFieldValue] = useState<string | boolean>(() =>
    kind === "flag" ? readPluginSetting(current) === true : formatPluginSetting(current),
  );
  const [invalid, setInvalid] = useState<string>();
  const edited = useRef(false);
  const version = useRef(0);
  useEffect(() => {
    if (edited.current) {
      return;
    }
    const setting = parsePluginSetting(field, value ?? readPluginSetting(defaultValue));
    setFieldValue(kind === "flag" ? readPluginSetting(setting) === true : formatPluginSetting(setting));
  }, [value, defaultValue, field, kind]);
  const autosave = useAutosave<SettingValues[string]>(async (next) => {
    const saving = version.current;
    await onSave(next);
    if (saving === version.current) {
      edited.current = false;
      const setting = parsePluginSetting(field, next);
      setFieldValue(kind === "flag" ? readPluginSetting(setting) === true : formatPluginSetting(setting));
    }
  });

  function change(next: string | boolean) {
    edited.current = true;
    version.current++;
    setFieldValue(next);
    try {
      const parsed = parsePluginSetting(field, next);
      setInvalid(undefined);
      autosave.change(readPluginSetting(parsed));
    } catch (error) {
      setInvalid(error instanceof Error ? error.message : String(error));
      autosave.cancel();
    }
  }

  return (
    <SettingsRow
      description={field.description}
      error={invalid}
      htmlFor={id}
      label={kind === "duration" ? `${field.title} [s]` : field.title}
      status={<SettingsFieldStatus onRetry={autosave.retry} state={autosave.state} />}
    >
      <div className="grid w-full min-w-0 gap-2">
        {kind === "flag" ? (
          <Switch id={id} checked={fieldValue === true} onCheckedChange={change} />
        ) : (
          <Input
            id={id}
            type={kind === "number" || kind === "integer" ? "number" : "text"}
            inputMode={kind === "integer" ? "numeric" : kind === "number" ? "decimal" : undefined}
            className={kind === "text" ? undefined : "text-right tabular-nums"}
            step={kind === "integer" ? "1" : kind === "number" ? "any" : undefined}
            min={kind === "integer" || kind === "number" ? field.minimum && String(readPluginSetting(field.minimum)) : undefined}
            max={kind === "integer" || kind === "number" ? field.maximum && String(readPluginSetting(field.maximum)) : undefined}
            aria-invalid={Boolean(invalid)}
            value={String(fieldValue)}
            onChange={(event) => change(event.target.value)}
          />
        )}
      </div>
    </SettingsRow>
  );
}
