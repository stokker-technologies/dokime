<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { SettingDeclaration } from "@generated/host/model";
  import { onDestroy, untrack } from "svelte";
  import Input from "../../../components/ui/Input.svelte";
  import Switch from "../../../components/ui/Switch.svelte";
  import { formatPluginSetting, parsePluginSetting, readPluginSetting } from "../../../lib/api/native-settings";
  import { required, type Wire } from "../../../lib/rpc/canonical";
  import { createDebouncedWriter, GUI_SETTINGS_SAVED_MS, type AutosaveState } from "../settings-autosave";
  import SettingsFieldStatus from "../SettingsFieldStatus.svelte";
  import SettingsRow from "../SettingsRow.svelte";
  import type { SettingValues } from "./installed-plugin-contracts";

  let {
    field,
    value,
    onSave,
  }: {
    field: Wire<SettingDeclaration>;
    value: SettingValues[string] | undefined;
    onSave: (value: SettingValues[string]) => Promise<void>;
  } = $props();
  const id = $props.id();
  const defaultValue = $derived(required(field.default, "Read Plugin Setting Default"));
  const kind = $derived(defaultValue.value.case);
  const numeric = $derived(kind === "number" || kind === "integer");
  let fieldValue = $state<string | boolean>(untrack(() => display(value ?? readPluginSetting(defaultValue))));
  let invalid = $state<string>();
  let saveState = $state.raw<AutosaveState>({ kind: "idle" });
  let edited = false;
  let version = 0;
  let last: { value: SettingValues[string] } | undefined;
  const writer = createDebouncedWriter<SettingValues[string]>(
    async (next) => {
      const saving = version;
      await onSave(next);
      if (saving === version) {
        edited = false;
        fieldValue = display(next);
      }
    },
    (next) => (saveState = next),
  );

  $effect(() => {
    const current = display(value ?? readPluginSetting(defaultValue));
    if (!edited) {
      fieldValue = current;
    }
  });
  $effect(() => {
    if (saveState.kind === "saved") {
      const timer = setTimeout(() => (saveState = { kind: "idle" }), GUI_SETTINGS_SAVED_MS);
      return () => clearTimeout(timer);
    }
  });
  onDestroy(() => writer.flush());

  function display(input: SettingValues[string]) {
    const setting = parsePluginSetting(field, input);
    return kind === "flag" ? readPluginSetting(setting) === true : formatPluginSetting(setting);
  }

  function retry() {
    if (last) {
      writer.reset();
      writer.change(last.value);
    }
  }

  function change(next: string | boolean) {
    edited = true;
    version++;
    fieldValue = next;
    try {
      last = { value: readPluginSetting(parsePluginSetting(field, next)) };
      invalid = undefined;
      retry();
    } catch (error) {
      invalid = error instanceof Error ? error.message : String(error);
      writer.cancel();
      last = undefined;
      saveState = { kind: "idle" };
    }
  }
</script>

<SettingsRow description={field.description} error={invalid} for={id} label={kind === "duration" ? `${field.title} [s]` : field.title}>
  {#snippet status()}
    <SettingsFieldStatus onRetry={retry} state={saveState} />
  {/snippet}
  <div class="grid w-full min-w-0 gap-2">
    {#if kind === "flag"}
      <Switch {id} checked={fieldValue === true} onCheckedChange={change} />
    {:else}
      <Input
        {id}
        type={numeric ? "number" : "text"}
        inputmode={kind === "integer" ? "numeric" : kind === "number" ? "decimal" : undefined}
        class={kind === "text" ? undefined : "text-right tabular-nums"}
        step={kind === "integer" ? "1" : kind === "number" ? "any" : undefined}
        min={numeric && field.minimum ? String(readPluginSetting(field.minimum)) : undefined}
        max={numeric && field.maximum ? String(readPluginSetting(field.maximum)) : undefined}
        aria-invalid={Boolean(invalid)}
        value={String(fieldValue)}
        oninput={(event) => change(event.currentTarget.value)}
      />
    {/if}
  </div>
</SettingsRow>
