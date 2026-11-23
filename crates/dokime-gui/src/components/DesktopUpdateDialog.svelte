<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Dialog } from "bits-ui";
  import { buildLabel } from "../features/settings/sections/build-version";
  import type { BuildIdentity } from "../lib/contracts/connection";
  import { commands } from "../lib/desktop/commands";
  import type { UpdateCheck, UpdateMode, UpdateStatus } from "../lib/desktop/updates";
  import { failureOf } from "../lib/notification-details";
  import ErrorCard from "./ErrorCard.svelte";
  import Button from "./ui/Button.svelte";
  import DialogContent from "./ui/DialogContent.svelte";
  import SelectField from "./ui/SelectField.svelte";

  let { open, onOpenChange, build }: { open: boolean; onOpenChange: (open: boolean) => void; build: BuildIdentity } = $props();
  let status = $state.raw<UpdateStatus | null>(null);
  let update = $state.raw<UpdateCheck | null>(null);
  let busy = $state(false);
  let loading = $state(false);
  let error = $state.raw<unknown>(null);
  let actionFailure = $state(false);
  const id = $props.id();
  const modes = [
    { value: "disabled", label: "Disabled" },
    { value: "manual", label: "Manual" },
    { value: "automatic_check", label: "Automatic Check" },
  ] as const;
  const sources = $derived([
    { value: "none", label: "No Source", disabled: status?.mode !== "disabled" },
    ...(status?.sources ?? []).map((source, index) => ({ value: String(index), label: source })),
  ]);
  const source = $derived(status?.source ? String(status.sources.indexOf(status.source)) : "none");

  $effect(() => {
    if (!open) {
      return;
    }
    let active = true;
    actionFailure = false;
    loading = true;
    void commands
      .desktopUpdatesStatus()
      .then((next) => {
        if (active) {
          status = next;
          update = next.lastCheck;
          error = next.error;
        }
      })
      .catch((reason: unknown) => {
        if (active) {
          error = reason;
        }
      })
      .finally(() => {
        if (active) {
          loading = false;
        }
      });
    return () => {
      active = false;
    };
  });

  async function perform(operation: () => Promise<void>) {
    if (busy || loading) {
      return;
    }
    actionFailure = true;
    busy = true;
    error = null;
    try {
      await operation();
    } catch (reason: unknown) {
      error = reason;
    } finally {
      busy = false;
    }
  }

  async function configure(mode: UpdateMode, source: string | null) {
    await perform(async () => {
      const next = await commands.desktopUpdatesConfigure({ mode, source });
      status = next;
      update = null;
      error = next.error;
    });
  }

  function changeMode(mode: UpdateMode) {
    if (status) {
      void configure(mode, status.source);
    }
  }

  function changeSource(value: string) {
    if (status) {
      void configure(status.mode, value === "none" ? null : status.sources[Number(value)]);
    }
  }

  function checkForUpdates() {
    if (status?.source && status.mode !== "disabled") {
      void perform(async () => {
        update = await commands.desktopUpdatesCheck();
      });
    }
  }

  function installUpdate() {
    const candidate = update?.candidate;
    if (candidate) {
      void perform(async () => {
        await commands.desktopUpdatesInstall(candidate);
        update = null;
      });
    }
  }
</script>

<Dialog.Root
  {open}
  onOpenChange={(next) => {
    if (!busy) {
      onOpenChange(next);
    }
  }}
>
  <DialogContent class="sm:max-w-lg" showCloseButton={!busy}>
    <div class="flex flex-col gap-2 pr-8" data-slot="dialog-header">
      <Dialog.Title class="font-sans text-base leading-none font-medium">Desktop Updates</Dialog.Title>
      <Dialog.Description class="text-sm text-muted-foreground"
        >Choose a release source. Installation always requires your confirmation.</Dialog.Description
      >
    </div>
    {#if status}
      <div class="grid gap-4 text-sm">
        <div class="grid gap-1">
          <label for={`${id}-mode`}>Update Checks</label>
          <SelectField
            label="Update Checks"
            id={`${id}-mode`}
            class="w-full"
            disabled={busy || loading}
            value={status.mode}
            options={modes}
            onChange={changeMode}
          />
        </div>
        <div class="grid gap-1">
          <label for={`${id}-source`}>Release Source</label>
          <SelectField
            label="Release Source"
            id={`${id}-source`}
            class="w-full"
            disabled={busy || loading}
            value={source}
            options={sources}
            onChange={changeSource}
          />
        </div>
        {#if update}
          <div class="rounded border p-3">
            <div>
              {update.status === "available"
                ? `Update ${update.version} Available`
                : update.status === "disabled"
                  ? "Updates Disabled"
                  : "Up to Date"}
            </div>
            <p class="text-muted-foreground">{update.body ?? `Current Version ${buildLabel(build).text}`}</p>
          </div>
        {/if}
      </div>
    {/if}
    {#if error}
      <ErrorCard notificationContext={actionFailure ? "action" : undefined} failure={failureOf(error, "Operation Failed")} />
    {/if}
    <div
      data-slot="dialog-footer"
      class="-mx-4 -mb-4 flex flex-col-reverse gap-2 rounded-b-lg border-t bg-muted/50 p-4 sm:flex-row sm:justify-end"
    >
      <Button
        data-testid="desktop-updates-check"
        variant="outline"
        disabled={busy || loading || !status?.source || status.mode === "disabled"}
        onclick={checkForUpdates}
      >
        Check for Updates
      </Button>
      <Button data-testid="desktop-updates-install" disabled={busy || loading || !update?.candidate} onclick={installUpdate}>
        Install Update
      </Button>
    </div>
  </DialogContent>
</Dialog.Root>
