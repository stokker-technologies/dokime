<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Dialog } from "bits-ui";
  import { untrack } from "svelte";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import Button from "../../components/ui/Button.svelte";
  import DialogContent from "../../components/ui/DialogContent.svelte";
  import { failureOf } from "../../lib/notification-details";
  import { EMPTY_SETTINGS_TITLE_ERROR, type SettingsDialogProps, type SettingsValues } from "./settings-fields";
  import SettingsFields from "./SettingsFields.svelte";

  let {
    description,
    onOpenChange,
    onSubmit,
    open,
    readOnly = {},
    submitLabel = "Save Settings",
    title,
    values,
  }: SettingsDialogProps = $props();
  let draft = $state.raw(untrack(() => values));
  let error = $state.raw<unknown>(null);
  let saving = $state(false);
  const editable = $derived(Object.keys(values).some((key) => !readOnly[key as keyof SettingsValues]));

  $effect(() => {
    if (open) {
      draft = untrack(() => values);
      error = null;
    }
  });

  function change(patch: Partial<SettingsValues>) {
    error = null;
    draft = { ...draft, ...patch };
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (saving || !editable) {
      return;
    }
    const title = draft.title.trim();
    if (!title) {
      error = EMPTY_SETTINGS_TITLE_ERROR;
      return;
    }
    saving = true;
    try {
      await onSubmit({ ...draft, title });
      onOpenChange(false);
    } catch (failure: unknown) {
      error = failure;
    } finally {
      saving = false;
    }
  }
</script>

<Dialog.Root {open} onOpenChange={(next) => !saving && onOpenChange(next)}>
  <DialogContent>
    <form class="grid gap-4" onsubmit={submit}>
      <div class="flex flex-col gap-2 pr-8 text-center sm:text-left" data-slot="dialog-header">
        <Dialog.Title class="font-sans text-base leading-none font-medium">{title}</Dialog.Title>
        <Dialog.Description class="text-sm text-muted-foreground">{description}</Dialog.Description>
      </div>
      <SettingsFields onChange={change} {readOnly} titleInvalid={error === EMPTY_SETTINGS_TITLE_ERROR} values={draft} />
      {#if error}
        <ErrorCard notificationContext="action" failure={failureOf(error, "Operation Failed")} />
      {/if}
      <div
        class="-mx-4 -mb-4 flex flex-col-reverse gap-2 rounded-b-lg border-t bg-muted/50 p-4 sm:flex-row sm:justify-end"
        data-slot="dialog-footer"
      >
        <Button disabled={saving} onclick={() => onOpenChange(false)} variant="outline">{editable ? "Cancel" : "Close"}</Button>
        {#if editable}
          <Button disabled={saving} type="submit">{submitLabel}</Button>
        {/if}
      </div>
    </form>
  </DialogContent>
</Dialog.Root>
