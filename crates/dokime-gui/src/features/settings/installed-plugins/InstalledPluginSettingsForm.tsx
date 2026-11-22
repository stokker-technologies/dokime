// Dokime — SPDX-License-Identifier: Apache-2.0

/* === Imports === */

import { useEffect, useId, useRef, useState } from "react";
import type { SettingDeclaration } from "@host/model";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@dokime/document/components/ui/textarea";
import { SettingsFieldStatus } from "@/features/settings/SettingsFieldStatus";
import { SettingsRow } from "@/features/settings/SettingsRow";
import { useAutosave } from "@/features/settings/settings-autosave";
import type { Settings, SettingValues } from "./installed-plugin-contracts";
import { parsePluginSetting, readPluginSetting } from "@/lib/api/native-settings";
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
  if (!fields.length) return <p className="px-4 py-3 text-sm text-muted-foreground">This program has no configurable settings.</p>;

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

function SettingField({ field, value, onSave }: {
  field: Wire<SettingDeclaration>;
  value: SettingValues[string] | undefined;
  onSave: (value: SettingValues[string]) => Promise<void>;
}) {
  const id = useId();
  const defaultValue = required(field.default, "Read Plugin Setting Default");
  const [fieldValue, setFieldValue] = useState(value ?? readPluginSetting(defaultValue));
  const [invalid, setInvalid] = useState<string>();
  const edited = useRef(false);
  const version = useRef(0);
  useEffect(() => {
    if (!edited.current) setFieldValue(value ?? readPluginSetting(defaultValue));
  }, [value, defaultValue]);
  const autosave = useAutosave<SettingValues[string]>(async (next) => {
    const saving = version.current;
    await onSave(next);
    if (saving === version.current) edited.current = false;
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
      label={field.title}
      status={<SettingsFieldStatus onRetry={autosave.retry} state={autosave.state} />}
    >
      <div className="grid w-full min-w-0 gap-2">
        {defaultValue.value.case === "flag" ? (
          <Switch id={id} checked={fieldValue === true} onCheckedChange={change} />
        ) : (
          <Textarea id={id} aria-invalid={Boolean(invalid)} value={String(fieldValue ?? "")} onChange={(event) => change(event.target.value)} />
        )}
      </div>
    </SettingsRow>
  );
}
