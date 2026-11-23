<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { FileDefinition } from "../../lib/types";
  import { type FileRowState, parameterInputId } from "../start-cycle-form-model";
  import Button from "../ui/Button.svelte";
  import Input from "../ui/Input.svelte";

  let {
    definition,
    row,
    onPathChange,
    onFileChange,
  }: {
    definition: FileDefinition;
    row: FileRowState;
    onPathChange: (path: string) => void;
    onFileChange: (file: File | undefined) => void;
  } = $props();
  const label = $derived(definition.label ?? definition.name);
  let picker: HTMLInputElement;
</script>

<div class="flex h-[calc(var(--parameter-row-height)_-_--spacing(2))] items-center gap-2">
  <Input
    aria-invalid={Boolean(row.error) || (definition.required && !(row.path.trim() || definition.default?.trim()))}
    aria-label={`${label} Path`}
    class="h-[calc(var(--parameter-row-height)_-_--spacing(2))] min-w-0 flex-1 font-mono text-xs"
    disabled={row.uploading}
    id={`${parameterInputId(definition.name)}-file`}
    oninput={(event) => onPathChange(event.currentTarget.value)}
    placeholder={definition.default ?? "No File Chosen"}
    spellcheck={false}
    value={row.path}
  />
  <input
    accept={definition.media_types?.join(",")}
    aria-label={`${label} File`}
    class="sr-only"
    disabled={row.uploading}
    onchange={(event) => {
      const file = event.currentTarget.files?.[0];
      event.currentTarget.value = "";
      onFileChange(file);
    }}
    bind:this={picker}
    tabindex={-1}
    type="file"
  >
  <Button aria-label={`Choose ${label}`} disabled={row.uploading} onclick={() => picker.click()} size="sm" variant="outline">
    {row.uploading ? "Choosing…" : "Choose"}
  </Button>
</div>
