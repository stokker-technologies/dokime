<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import Input from "../../components/ui/Input.svelte";
  import Switch from "../../components/ui/Switch.svelte";
  import Textarea from "../../components/ui/Textarea.svelte";
  import type { SettingsFieldsProps } from "./settings-fields";

  let { idPrefix, onChange, readOnly = {}, titleInvalid = false, values }: SettingsFieldsProps = $props();
  const generated = $props.id();
  const id = $derived(idPrefix ?? generated);
  const label = "flex items-center gap-2 text-sm leading-none font-medium select-none";
</script>

<div class="grid gap-4">
  <div class="grid gap-2">
    <label class={label} for={`${id}-title`}>Title</label>
    <Input
      aria-invalid={titleInvalid}
      disabled={Boolean(readOnly.title)}
      id={`${id}-title`}
      oninput={(event) => onChange({ title: event.currentTarget.value })}
      value={values.title}
    />
    {#if readOnly.title}
      <p class="text-xs text-muted-foreground">{readOnly.title}</p>
    {/if}
  </div>
  {#if values.description !== undefined}
    <div class="grid gap-2">
      <label class={label} for={`${id}-description`}>Description</label>
      <Textarea
        disabled={Boolean(readOnly.description)}
        id={`${id}-description`}
        oninput={(event) => onChange({ description: event.currentTarget.value })}
        placeholder="Optional"
        value={values.description}
      />
      {#if readOnly.description}
        <p class="text-xs text-muted-foreground">{readOnly.description}</p>
      {/if}
    </div>
  {/if}
  {#if values.continueAutomatically !== undefined}
    <div class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-1">
      <label class={label} for={`${id}-continue-automatically`}>Continue Automatically</label>
      <Switch
        checked={values.continueAutomatically}
        disabled={Boolean(readOnly.continueAutomatically)}
        id={`${id}-continue-automatically`}
        onCheckedChange={(continueAutomatically) => onChange({ continueAutomatically })}
      />
      <p class="col-span-2 text-xs text-muted-foreground">
        {readOnly.continueAutomatically ?? "After an execution completes, the next unstarted one starts, whatever its verdict."}
      </p>
    </div>
  {/if}
</div>
