<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createMutation } from "@tanstack/svelte-query";
  import { Dialog } from "bits-ui";
  import type { ConnectedSession } from "../../app/native-session/session";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import Button from "../../components/ui/Button.svelte";
  import DialogContent from "../../components/ui/DialogContent.svelte";
  import { createNativeTestCycle, type NewCycleRequest } from "../../lib/api/native-planning";
  import { failureOf } from "../../lib/notification-details";
  import { invalidateWorkflowResourceQueries } from "../cycle/server-state";
  import SettingsFields from "../cycle/SettingsFields.svelte";
  import { cyclesHomeQueryKeys } from "./cycles-query-options";
  import { initialNewCycleForm, newCycleFormError, newCycleRequest, type NewCycleForm } from "./new-cycle-form";

  let {
    onCreated,
    onOpenChange,
    open,
    session,
  }: {
    onCreated: (cycleId: string) => void;
    onOpenChange: (open: boolean) => void;
    open: boolean;
    session: ConnectedSession;
  } = $props();
  let form = $state.raw(initialNewCycleForm(new Date()));
  let error = $state.raw<unknown>(null);
  const create = createMutation(
    () => ({
      mutationFn: (request: NewCycleRequest) => createNativeTestCycle(session.identity, request),
      onSuccess: async () => {
        const client = session.queryClient;
        await Promise.all([invalidateWorkflowResourceQueries(client), client.invalidateQueries({ queryKey: cyclesHomeQueryKeys.all })]);
      },
    }),
    () => session.queryClient,
  );

  $effect(() => {
    if (open) {
      form = initialNewCycleForm(new Date());
      error = null;
    }
  });

  function change(patch: Partial<NewCycleForm>) {
    error = null;
    form = { ...form, ...patch };
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (create.isPending) {
      return;
    }
    const refusal = newCycleFormError(form);
    if (refusal) {
      error = refusal;
      return;
    }
    try {
      const { cycleId } = await create.mutateAsync(newCycleRequest(form));
      onOpenChange(false);
      onCreated(cycleId);
    } catch (failure: unknown) {
      error = failure;
    }
  }
</script>

<Dialog.Root {open} onOpenChange={(next) => !create.isPending && onOpenChange(next)}>
  <DialogContent>
    <form class="grid gap-4" onsubmit={submit}>
      <div class="flex flex-col gap-2 pr-8 text-center sm:text-left" data-slot="dialog-header">
        <Dialog.Title class="font-sans text-base leading-none font-medium">New Cycle</Dialog.Title>
        <Dialog.Description class="text-sm text-muted-foreground"
          >The cycle starts empty. Add cases to it from the Library.</Dialog.Description
        >
      </div>
      <SettingsFields idPrefix="new-cycle" onChange={change} titleInvalid={error !== null && !form.title.trim()} values={form} />
      {#if error}
        <ErrorCard notificationContext="action" failure={failureOf(error, "Operation Failed")} />
      {/if}
      <div
        class="-mx-4 -mb-4 flex flex-col-reverse gap-2 rounded-b-lg border-t bg-muted/50 p-4 sm:flex-row sm:justify-end"
        data-slot="dialog-footer"
      >
        <Button disabled={create.isPending} onclick={() => onOpenChange(false)} variant="outline">Cancel</Button>
        <Button disabled={create.isPending} type="submit">Create Cycle</Button>
      </div>
    </form>
  </DialogContent>
</Dialog.Root>
