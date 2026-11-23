<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts" module>
  import type { SecretEntry } from "../../../lib/api/native-secrets";

  export type Editor = { kind: "new"; identifier?: string } | { kind: "edit"; entry: SecretEntry };

  const SECRETS_DESCRIPTION_BYTES = 2_048;
  const SECRETS_LABEL_BYTES = 256;
  const SECRETS_IDENTIFIER_BYTES = 128;
  const SECRETS_SECRET_BYTES = 65_536;
</script>

<script lang="ts">
  import { Dialog } from "bits-ui";
  import { untrack } from "svelte";
  import ErrorCard from "../../../components/ErrorCard.svelte";
  import Button from "../../../components/ui/Button.svelte";
  import DialogContent from "../../../components/ui/DialogContent.svelte";
  import Input from "../../../components/ui/Input.svelte";
  import { failureOf } from "../../../lib/notification-details";

  let {
    editor,
    busy,
    cancel,
    submit,
    error,
  }: {
    editor: Editor;
    busy: boolean;
    cancel: () => void;
    submit: (metadata: SecretEntry["metadata"], value: string | undefined) => Promise<void>;
    error?: unknown;
  } = $props();
  const id = $props.id();
  const initial = untrack(() =>
    editor.kind === "edit"
      ? editor.entry.metadata
      : {
          identifier: editor.identifier,
          label: editor.identifier,
          description: undefined,
        },
  );
  let identifier = $state(initial.identifier ?? "");
  let label = $state(initial.label ?? "");
  let description = $state(initial.description ?? "");
  let value = $state("");

  function save(event: SubmitEvent) {
    event.preventDefault();
    const entered = value || undefined;
    value = "";
    void submit({ identifier, label, description: description || undefined }, entered);
  }
</script>

<Dialog.Root
  open
  onOpenChange={(open) => {
    if (!open && !busy) cancel();
  }}
>
  <DialogContent>
    <header class="flex flex-col gap-2">
      <Dialog.Title class="font-medium">{editor.kind === "new" ? "Add Secret" : "Edit Secret"}</Dialog.Title>
      <Dialog.Description class="text-muted-foreground">Values stay in the host’s OS keyring.</Dialog.Description>
    </header>
    {#if error}
      <ErrorCard notificationContext="action" failure={failureOf(error, "Operation Failed")} />
    {/if}
    <form class="grid gap-3" onsubmit={save}>
      <label class="text-sm font-medium" for={`${id}-identifier`}>Identifier</label>
      <Input
        id={`${id}-identifier`}
        class="font-mono"
        required
        pattern={"(?!\\.{1,2}$)[A-Za-z0-9._\\-]+"}
        maxlength={SECRETS_IDENTIFIER_BYTES}
        value={identifier}
        disabled={busy || editor.kind === "edit"}
        oninput={(event) => (identifier = event.currentTarget.value)}
      />
      <label class="text-sm font-medium" for={`${id}-label`}>Label</label>
      <Input
        id={`${id}-label`}
        required
        maxlength={SECRETS_LABEL_BYTES}
        value={label}
        disabled={busy}
        oninput={(event) => (label = event.currentTarget.value)}
      />
      <label class="text-sm font-medium" for={`${id}-description`}>Description</label>
      <Input
        id={`${id}-description`}
        maxlength={SECRETS_DESCRIPTION_BYTES}
        value={description}
        disabled={busy}
        oninput={(event) => (description = event.currentTarget.value)}
      />
      <label class="text-sm font-medium" for={`${id}-value`}
        >{editor.kind === "edit" ? "Replacement Value (Leave Empty to Keep)" : "Value"}</label
      >
      <Input
        id={`${id}-value`}
        type="password"
        autocomplete="new-password"
        required={editor.kind === "new"}
        maxlength={SECRETS_SECRET_BYTES}
        {value}
        disabled={busy}
        oninput={(event) => (value = event.currentTarget.value)}
      />
      <div class="flex gap-2">
        <Button type="submit" disabled={busy}>Save Secret</Button>
        <Button disabled={busy} variant="outline" onclick={cancel}>Cancel</Button>
      </div>
    </form>
  </DialogContent>
</Dialog.Root>
