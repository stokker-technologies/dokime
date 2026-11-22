<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Plus, Trash2 } from "lucide-svelte";
  import { promptFieldDefault, type PromptValue } from "../lib/prompts";
  import type { PromptField } from "../lib/types";
  import Button from "./ui/Button.svelte";
  import Checkbox from "./ui/Checkbox.svelte";
  import Input from "./ui/Input.svelte";
  import SelectField from "./ui/SelectField.svelte";

  let {
    id,
    field,
    value,
    onChange,
  }: {
    id: string;
    field: PromptField;
    value?: PromptValue;
    onChange: (value: PromptValue) => void;
  } = $props();
  const resolved = $derived(value ?? promptFieldDefault(field));
  const entries = $derived(Array.isArray(resolved) ? (resolved.length ? resolved : [""]) : [String(resolved)]);
  const options = $derived((field.choices ?? []).map((choice) => ({ value: choice, label: choice })));

  function updateEntry(index: number, next: string) {
    const updated = entries.map((entry, entryIndex) => (entryIndex === index ? next : entry));
    onChange(updated);
  }

  function removeEntry(index: number) {
    const updated = entries.filter((_, entryIndex) => entryIndex !== index);
    onChange(updated.length ? updated : [""]);
  }
</script>

{#snippet scalar(
  fieldId: string | undefined,
  label: string,
  entry: string,
  update: (value: string) => void,
)}
  {#if field.kind === "choice"}
    <SelectField id={fieldId} {label} class="w-full" value={entry} {options} disabled={field.readonly} onChange={update} />
  {:else}
    <Input
      id={fieldId}
      aria-label={label}
      type={field.kind === "text" ? "text" : "number"}
      step={field.kind === "int" ? 1 : "any"}
      value={entry}
      readonly={field.readonly}
      required={field.required !== false}
      oninput={(event) => update(event.currentTarget.value)}
    />
  {/if}
{/snippet}

{#if field.multiple}
  <div class="grid gap-2">
    {#each entries as entry, index}
      <div class="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
        {@render scalar(index === 0 ? id : undefined, `${field.label} Entry ${index + 1}`, entry, (next) => updateEntry(index, next))}
        <Button
          aria-label={`Remove ${field.label} Entry ${index + 1}`}
          disabled={field.readonly || entries.length <= 1}
          size="icon"
          variant="ghost"
          onclick={() => removeEntry(index)}
        >
          <Trash2 size={16} />
        </Button>
      </div>
    {/each}
    <Button class="justify-self-start" disabled={field.readonly} size="sm" variant="outline" onclick={() => onChange([...entries, ""])}>
      <Plus size={14} />
      Add Entry
    </Button>
  </div>
{:else if field.kind === "boolean"}
  <div class="flex min-h-8 items-center gap-2">
    <Checkbox {id} checked={Boolean(resolved)} disabled={field.readonly} onCheckedChange={onChange} />
    <label class="font-medium" for={id}>{resolved ? "Yes" : "No"}</label>
  </div>
{:else}
  {@render scalar(id, field.label, String(resolved), onChange)}
{/if}
