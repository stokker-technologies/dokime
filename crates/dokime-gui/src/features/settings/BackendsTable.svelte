<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { Document } from "@generated/typescript/application-settings";
  import { Dialog, DropdownMenu } from "bits-ui";
  import { Plug } from "lucide-svelte";
  import { onMount } from "svelte";
  import { sessionState, type SessionOwner } from "../../app/native-session/session-state.svelte";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import Button from "../../components/ui/Button.svelte";
  import DialogContent from "../../components/ui/DialogContent.svelte";
  import SortableTableHeader from "../../components/ui/SortableTableHeader.svelte";
  import Table from "../../components/ui/Table.svelte";
  import { nextTableOrder, orderTableRows, type TableOrder } from "../../components/ui/table-order";
  import { activateTableRow } from "../../components/ui/table-row";
  import TableRowActions from "../../components/ui/TableRowActions.svelte";
  import { commands } from "../../lib/desktop/commands";
  import { preferencesCommands } from "../../lib/desktop/settings";
  import { failureOf } from "../../lib/notification-details";
  import { toast } from "../../lib/notifications";

  let {
    owner,
    failure,
    endpoint,
    retry,
    onConnect,
  }: {
    owner: SessionOwner;
    failure?: unknown;
    endpoint?: string | null;
    retry?: (profile: string | null) => void;
    onConnect?: () => void;
  } = $props();
  const latestConnection = sessionState(() => owner);
  let lastSelected = $state<string | null>(null);
  let importButton = $state<HTMLButtonElement>();
  let deleteButton = $state<HTMLButtonElement>();
  let registry = $state.raw<Document | null>(null);
  let error = $state.raw<unknown>();
  let deleting = $state<string | null>(null);
  let mutating = $state(false);
  const columns = [
    { key: "identifier", label: "Identifier" },
    { key: "label", label: "Label" },
    { key: "endpoint", label: "Endpoint" },
  ] as const;
  type Column = (typeof columns)[number]["key"];
  let order = $state.raw<TableOrder<Column> | undefined>();
  const profiles = $derived(orderTableRows(registry?.profiles ?? [], order));
  const deletingProfile = $derived(registry?.profiles.find((profile) => profile.identifier === deleting));
  const connection = $derived(latestConnection());
  const busy = $derived(mutating || connection.phase === "connecting");
  const connectionFailure = $derived(failure ?? (connection.phase === "failed" ? connection.error : null));
  const failedProfile = $derived(registry?.profiles.find((profile) => profile.identifier === lastSelected));
  const failedEndpoint = $derived(endpoint || failedProfile?.endpoint);
  const loading = $derived(!registry && !error && !connectionFailure);
  const cell = "px-2 first:pl-4 last:pr-4";

  onMount(() => {
    void loadRegistry();
  });

  async function loadRegistry() {
    error = undefined;
    try {
      registry = await preferencesCommands.desktopBackendsList();
    } catch (failure) {
      error = failure;
    }
  }

  function connect(identifier: string) {
    if (busy) {
      return;
    }
    onConnect?.();
    lastSelected = identifier;
    void owner.select({ kind: "profile", identifier });
  }

  function retryConnection() {
    if (!connectionFailure) {
      void loadRegistry();
    } else if (retry) {
      retry(lastSelected);
    } else if (lastSelected) {
      connect(lastSelected);
    }
  }

  async function importRemote() {
    mutating = true;
    error = undefined;
    try {
      const imported = await preferencesCommands.desktopBackendImportPairing(null);
      if (imported) {
        registry = imported;
        onConnect?.();
      }
    } catch (failure) {
      toast.error(failureOf(failure, "Backend Could Not Be Imported"));
      error = failure;
    } finally {
      mutating = false;
    }
  }

  async function remove() {
    if (!deleting) {
      return;
    }
    mutating = true;
    error = undefined;
    try {
      registry = await preferencesCommands.desktopBackendRemove(deleting);
      deleting = null;
      const status = await commands.desktopSessionStatus();
      if (status.connection.phase === "disconnected") {
        await owner.disconnect();
      }
    } catch (failure) {
      toast.error(failureOf(failure, "Backend Could Not Be Deleted"));
      error = failure;
    } finally {
      mutating = false;
    }
  }
</script>

<div
  data-slot="data-table"
  class="grid w-full min-w-0 content-start overflow-hidden rounded-lg border bg-card text-left text-card-foreground"
>
  <header class="flex flex-wrap items-center gap-2 border-b px-4 py-3">
    <div class="min-w-48 flex-1">
      <h2 class="text-sm font-semibold">Saved Backends</h2>
      <p class="mt-1 text-xs text-muted-foreground">Import a pairing file or connect to a saved host.</p>
    </div>
    <Button bind:ref={importButton} size="sm" disabled={busy} onclick={importRemote}>Import Remote</Button>
  </header>
  <Table aria-label="Saved Backends" class="table-fixed" aria-busy={loading}>
    <thead class="border-b bg-muted/50 text-left">
      <tr>
        {#each columns as column (column.key)}
          <SortableTableHeader
            class={cell}
            label={column.label}
            direction={order?.column === column.key ? order.direction : undefined}
            onclick={() => (order = nextTableOrder(order, column.key))}
          />
        {/each}
        <th scope="col" class={[cell, "h-(--list-row-height) font-medium"]}>Actions</th>
      </tr>
    </thead>
    <tbody class="divide-y">
      {#if connectionFailure || (error && !deleting)}
        <tr>
          <td colspan={4} class="px-4 py-3">
            <div class="grid gap-3 text-sm">
              <ErrorCard
                notificationContext={connectionFailure && lastSelected ? "action" : undefined}
                failure={failureOf(connectionFailure ?? error, connectionFailure ? "Backend Connection Failed" : "Backend Registry Failed")}
                onRetry={retryConnection}
                retryDisabled={busy}
              />
              {#if connectionFailure && failedEndpoint}
                <p class="break-all font-mono text-xs">{failedEndpoint}</p>
              {/if}
            </div>
          </td>
        </tr>
      {/if}
      {#each profiles as profile (profile.identifier)}
        <tr
          data-row-id={profile.identifier}
          class="h-(--list-row-height) cursor-pointer hover:bg-muted/50"
          {@attach (row) => activateTableRow(row, () => connect(profile.identifier))}
        >
          <td class={cell}>
            <Button
              variant="plain"
              size="inline"
              class="block w-full min-w-0 truncate text-left"
              disabled={busy}
              onclick={() => connect(profile.identifier)}
            >
              {profile.identifier}
            </Button>
          </td>
          <td class={[cell, "truncate"]}>{profile.label}</td>
          <td class={[cell, "truncate"]} title={profile.endpoint}>{profile.endpoint}</td>
          <td class={cell}>
            <TableRowActions primary="Connect" label={profile.identifier} disabled={busy} onSelect={() => connect(profile.identifier)}>
              {#snippet icon()}
                <Plug />
              {/snippet}
              <DropdownMenu.Item
                class="cursor-default rounded-lg px-2 py-1 text-sm text-destructive outline-none data-highlighted:bg-destructive/10 data-disabled:opacity-50"
                disabled={busy}
                onSelect={() => (deleting = profile.identifier)}
              >
                Delete
              </DropdownMenu.Item>
            </TableRowActions>
          </td>
        </tr>
      {:else}
        {#if !connectionFailure && !error}
          <tr>
            <td colspan={4} class="h-24 px-4 text-center text-muted-foreground">
              <span role={loading ? "status" : undefined}>{loading ? "Loading Rows…" : "No Saved Backends"}</span>
            </td>
          </tr>
        {/if}
      {/each}
    </tbody>
  </Table>
</div>

<Dialog.Root
  open={deleting !== null}
  onOpenChange={(open) => {
    if (!open && !busy) {
      deleting = null;
    }
  }}
>
  <DialogContent
    onOpenAutoFocus={(event) => {
      event.preventDefault();
      deleteButton?.focus();
    }}
    onCloseAutoFocus={(event) => {
      event.preventDefault();
      importButton?.focus();
    }}
  >
    <div class="flex flex-col gap-2 pr-8 text-center sm:text-left" data-slot="dialog-header">
      <Dialog.Title class="font-sans text-base leading-none font-medium">Delete Backend</Dialog.Title>
      <Dialog.Description class="text-sm text-muted-foreground">
        Delete {deletingProfile?.label || deleting}? An active connection will disconnect.
        {#if deletingProfile}
          <span class="mt-2 block break-all font-mono text-xs">{deletingProfile.identifier}</span>
        {/if}
      </Dialog.Description>
    </div>
    <div class="flex justify-end gap-2">
      <Button bind:ref={deleteButton} disabled={busy} variant="outline" onclick={() => (deleting = null)}>Cancel</Button>
      <Button disabled={busy} variant="destructive" onclick={remove}>Confirm Delete</Button>
    </div>
    {#if error}
      <ErrorCard failure={failureOf(error, "Backend Could Not Be Deleted")} />
    {/if}
  </DialogContent>
</Dialog.Root>
