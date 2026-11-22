<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Dialog } from "bits-ui";
  import { onMount } from "svelte";
  import { relativeTime } from "../../components/document/document-overview-model";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import FailureConsequences from "../../components/FailureConsequences.svelte";
  import Button from "../../components/ui/Button.svelte";
  import { buttonVariants } from "../../components/ui/button-variants";
  import DialogContent from "../../components/ui/DialogContent.svelte";
  import { failureOf, notificationDetailsText } from "../../lib/notification-details";
  import {
    getNotificationDetails,
    openNotificationDetails,
    subscribeNotificationDetails,
    toast,
    type NotificationRecord,
  } from "../../lib/notifications";

  let record = $state.raw<NotificationRecord | null>(null);
  let copyStatus = $state("");
  let opener: HTMLElement | null = null;
  const timestamp = $derived(record ? new Date(record.at).toISOString() : "");

  onMount(() => {
    function update() {
      record = getNotificationDetails();
      copyStatus = "";
    }
    const unsubscribe = subscribeNotificationDetails(update);
    update();
    return unsubscribe;
  });

  function close(open: boolean) {
    if (!open) {
      openNotificationDetails(null);
    }
  }

  async function copy() {
    const entry = record;
    if (!entry) {
      return;
    }
    try {
      await navigator.clipboard.writeText(notificationDetailsText(entry, relativeTime(timestamp)));
      if (record === entry) {
        copyStatus = "Copied";
      }
    } catch (error: unknown) {
      toast.error(failureOf(error, "Could Not Copy Notification"));
      if (record === entry) {
        copyStatus = "Could not copy. Try again.";
      }
    }
  }
</script>

<Dialog.Root open={Boolean(record)} onOpenChange={close}>
  <DialogContent
    class="z-[110] max-h-[85dvh] grid-rows-[auto_minmax(0,1fr)_auto] sm:max-w-2xl"
    onOpenAutoFocus={() => {
      opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    }}
    onCloseAutoFocus={(event) => {
      event.preventDefault();
      if (opener?.isConnected) {
        opener.focus();
      }
    }}
  >
    {#if record}
      <div class="flex flex-col gap-2 pr-8 text-center sm:text-left" data-slot="dialog-header">
        <Dialog.Title class="font-sans text-base leading-none font-medium" data-slot="dialog-title">{record.title}</Dialog.Title>
        <Dialog.Description class="text-sm text-muted-foreground" data-slot="dialog-description">Notification Details</Dialog.Description>
      </div>
      <div class="grid min-h-0 gap-4 overflow-y-auto whitespace-pre-wrap [overflow-wrap:anywhere]">
        {#if record.failure}
          <ErrorCard failure={record.failure} />
        {:else if record.description}
          <p class="select-text">{record.description}</p>
        {/if}
        <dl class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-2">
          <dt class="text-muted-foreground">Source</dt>
          <dd>{record.source}</dd>
          <dt class="text-muted-foreground">Time</dt>
          <dd><time datetime={timestamp}>{new Date(record.at).toLocaleString()} ({relativeTime(timestamp)})</time></dd>
          <dt class="text-muted-foreground">Severity</dt>
          <dd class="capitalize">{record.tone}</dd>
        </dl>
        <FailureConsequences items={record.consequences ?? []} />
      </div>
      <div
        class="-mx-4 -mb-4 flex min-w-0 flex-col-reverse gap-2 rounded-b-lg border-t bg-muted/50 p-4 sm:flex-row sm:flex-wrap sm:justify-end"
        data-slot="dialog-footer"
      >
        <span role="status" class="self-center text-xs text-muted-foreground">{copyStatus}</span>
        <Button variant="outline" onclick={copy}>Copy</Button>
        <Dialog.Close class={buttonVariants({ variant: "outline" })}>Close</Dialog.Close>
      </div>
    {/if}
  </DialogContent>
</Dialog.Root>
