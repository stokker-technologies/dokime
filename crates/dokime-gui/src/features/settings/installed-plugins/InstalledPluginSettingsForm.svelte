<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { Settings, SettingValues } from "./installed-plugin-contracts";
  import PluginSettingField from "./PluginSettingField.svelte";

  let {
    schema,
    values,
    onSubmit,
  }: {
    schema: Settings;
    values: SettingValues;
    onSubmit: (values: SettingValues) => Promise<void>;
  } = $props();
  const fields = $derived((schema.contributions ?? []).filter((entry) => entry.kind === "setting"));
  let writing = Promise.resolve();

  function submit(name: string, value: SettingValues[string]) {
    const save = writing.then(() => onSubmit({ [name]: value }));
    writing = save.catch(() => undefined);
    return save;
  }
</script>

{#each fields as { declaration } (declaration.name)}
  <PluginSettingField field={declaration} value={values[declaration.name]} onSave={(value) => submit(declaration.name, value)} />
{:else}
  <p class="px-4 py-3 text-sm text-muted-foreground">This program has no configurable settings.</p>
{/each}
