<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Dialog, DropdownMenu } from "bits-ui";
  import { Ellipsis } from "lucide-svelte";
  import { untrack } from "svelte";
  import ErrorCard from "../../../components/ErrorCard.svelte";
  import Button from "../../../components/ui/Button.svelte";
  import { buttonVariants } from "../../../components/ui/button-variants";
  import Checkbox from "../../../components/ui/Checkbox.svelte";
  import DialogContent from "../../../components/ui/DialogContent.svelte";
  import {
    exportSecrets,
    importSecrets,
    pickSecretArchive,
    previewSecrets,
    type SecretPreview,
    type SecretSnapshot,
  } from "../../../lib/api/native-secrets";
  import { failureOf } from "../../../lib/notification-details";
  import type { SessionIdentity } from "../../../lib/rpc/transport";

  let {
    session,
    catalog,
    onImported,
  }: {
    session: SessionIdentity;
    catalog: SecretSnapshot;
    onImported: (catalog: SecretSnapshot) => void;
  } = $props();
  const id = $props.id();
  const menuItem =
    "cursor-default rounded-lg px-2 py-1 text-sm outline-none data-highlighted:bg-accent data-highlighted:text-accent-foreground data-disabled:opacity-50";
  let mode = $state<"export" | "import" | null>(null);
  let generation = $state(untrack(() => catalog.generation));
  let selected = $state.raw(new Set<string>());
  let data = $state.raw<Uint8Array | null>(null);
  let preview = $state.raw<SecretPreview | null>(null);
  let busy = $state(false);
  let error = $state.raw<unknown>();
  let notice = $state<string>();

  function open(next: typeof mode) {
    mode = next;
    generation = catalog.generation;
    selected = new Set();
    data = null;
    preview = null;
    error = undefined;
    notice = undefined;
  }

  function select(identifier: string, checked: boolean) {
    const next = new Set(selected);
    if (checked) {
      next.add(identifier);
    } else {
      next.delete(identifier);
    }
    selected = next;
  }

  async function run(work: () => Promise<void>) {
    busy = true;
    error = undefined;
    notice = undefined;
    try {
      await work();
    } catch (failure) {
      if (session.isCurrent()) {
        error = failure;
      }
    } finally {
      if (session.isCurrent()) {
        busy = false;
      }
    }
  }

  function submitExport(event: SubmitEvent) {
    event.preventDefault();
    void run(async () => {
      const path = await exportSecrets(session, [...selected], generation);
      if (session.isCurrent()) {
        notice = path ? `Saved ${path}` : "Export Cancelled";
      }
    });
  }

  function pick() {
    void run(async () => {
      preview = null;
      data = null;
      const bytes = await pickSecretArchive(session);
      if (session.isCurrent()) {
        data = bytes;
      }
    });
  }

  function submitImport(event: SubmitEvent) {
    event.preventDefault();
    const bytes = data;
    if (!bytes) {
      return;
    }
    void run(async () => {
      if (!preview) {
        const next = await previewSecrets(session, bytes);
        if (session.isCurrent()) {
          preview = next;
          selected = new Set(next.entries.filter((entry) => !entry.exists).map((entry) => entry.metadata.identifier));
        }
        return;
      }
      const next = await importSecrets(session, bytes, preview, selected);
      if (session.isCurrent()) {
        onImported(next);
        data = null;
        preview = null;
        notice = "Selected Secrets Imported";
      }
    });
  }
</script>

{#snippet choice(
  identifier: string,
  label: string,
)}
  <label for={`${id}-${identifier}`} class="flex items-center gap-2 text-sm">
    <Checkbox
      id={`${id}-${identifier}`}
      disabled={busy}
      checked={selected.has(identifier)}
      onCheckedChange={(checked) => select(identifier, checked)}
    />
    {label}
  </label>
{/snippet}

<DropdownMenu.Root>
  <DropdownMenu.Trigger class={buttonVariants({ variant: "ghost", size: "icon-sm" })} aria-label="Secret Transfer Actions">
    <Ellipsis />
  </DropdownMenu.Trigger>
  <DropdownMenu.Portal>
    <DropdownMenu.Content align="end" class="z-50 rounded-lg bg-popover p-1 text-popover-foreground ring-1 ring-border">
      <DropdownMenu.Item class={menuItem} disabled={busy || !catalog.entries.length} onSelect={() => open("export")}>
        Export Selected Secrets
      </DropdownMenu.Item>
      <DropdownMenu.Item class={menuItem} disabled={busy} onSelect={() => open("import")}>Import Secrets</DropdownMenu.Item>
    </DropdownMenu.Content>
  </DropdownMenu.Portal>
</DropdownMenu.Root>
{#if notice}
  <p role="status" class="whitespace-pre-wrap text-sm text-muted-foreground">{notice}</p>
{/if}
<Dialog.Root
  open={mode !== null}
  onOpenChange={(isOpen) => {
    if (!isOpen && !busy) open(null);
  }}
>
  <DialogContent class="max-h-[85dvh] overflow-y-auto">
    <Dialog.Title class="font-medium">{mode === "export" ? "Export Secrets" : "Import Secrets"}</Dialog.Title>
    {#if error}
      <ErrorCard notificationContext="action" failure={failureOf(error, "Operation Failed")} />
    {/if}
    {#if mode === "export"}
      <form class="grid gap-3" onsubmit={submitExport}>
        <p class="text-sm text-muted-foreground">
          Choose identifiers to save in a Base64 .b64 file. Anyone with the file can read its secrets. Each transfer holds at most 512 KiB
          of secret data and metadata.
        </p>
        <fieldset class="grid gap-2" disabled={busy}>
          <legend class="mb-2 text-sm font-medium">Identifiers to Export</legend>
          {#each catalog.entries as { metadata } (metadata.identifier)}
            {@render choice(metadata.identifier, metadata.identifier)}
          {/each}
        </fieldset>
        <div class="flex gap-2">
          <Button type="submit" disabled={busy || !selected.size}>Save Secret File</Button>
          <Button disabled={busy} variant="outline" onclick={() => open(null)}>Cancel</Button>
        </div>
      </form>
    {:else if mode === "import"}
      <form class="grid gap-3" onsubmit={submitImport}>
        <Button disabled={busy} variant="outline" onclick={pick}>Choose Secret Archive</Button>
        {#if data}
          <p class="text-sm">Secret Archive Selected ({data.byteLength} bytes)</p>
        {/if}
        {#if preview}
          <p class="text-sm">Import Into {preview.hostLabel} · {preview.hostId}</p>
          <fieldset class="grid gap-2" disabled={busy}>
            <legend class="mb-2 text-sm font-medium">Review Every Identifier</legend>
            {#each preview.entries as { metadata, exists } (metadata.identifier)}
              {@render choice(metadata.identifier, `${exists ? "Replace" : "Import"} ${metadata.identifier} (${metadata.label})`)}
            {/each}
          </fieldset>
          <p class="text-sm text-muted-foreground">
            Unchecked entries are skipped. Existing identifiers start unchecked. Apply only the identifiers you have reviewed.
          </p>
        {/if}
        <div class="flex gap-2">
          <Button type="submit" disabled={busy || !data}>{preview ? "Apply Reviewed Import" : "Preview Import"}</Button>
          <Button disabled={busy} variant="outline" onclick={() => open(null)}>Cancel</Button>
        </div>
      </form>
    {/if}
  </DialogContent>
</Dialog.Root>
