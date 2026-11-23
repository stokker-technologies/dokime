<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createQuery, type QueryClient } from "@tanstack/svelte-query";
  import type { Snippet } from "svelte";
  import type { ConnectedSession } from "@/app/native-session/session";
  import ErrorCard from "@/components/ErrorCard.svelte";
  import Button from "@/components/ui/Button.svelte";
  import { readNativeListener, regenerateNativeCertificate, updateNativeListener } from "@/lib/api/native-remote";
  import { commands } from "@/lib/desktop/commands";
  import { failureOf } from "@/lib/notification-details";
  import { toast } from "@/lib/notifications";
  import { isTauriDesktop } from "@/lib/surface-environment";
  import SettingsCard from "../SettingsCard.svelte";
  import SettingsRow from "../SettingsRow.svelte";
  import FirewallNotice from "./FirewallNotice.svelte";
  import { commitListener, listenerRunning, type ListenerStatus } from "./remote-listener";

  let {
    session,
    advanced,
    preferencesPending,
    preferences,
    label,
    queryClient,
  }: {
    session: ConnectedSession | null;
    advanced: boolean;
    preferencesPending: boolean;
    preferences?: Snippet;
    label: string;
    queryClient: QueryClient;
  } = $props();

  const canManage = $derived(
    isTauriDesktop() && session?.snapshot.connection.backend?.methods.includes("/dokime.host.v1.access.Listener/Get"),
  );
  const queryKey = $derived(["settings", "remote-listener", session?.identity.generation]);
  const query = createQuery(
    () => {
      const identity = session?.identity;
      return {
        queryKey,
        enabled: Boolean(canManage),
        queryFn: () => {
          if (!identity) {
            throw new Error("Connect to the host before reloading remote settings.");
          }
          return readNativeListener(identity);
        },
        retry: false,
      };
    },
    () => queryClient,
  );
  let busy = $state(false);
  const disabled = $derived(busy || preferencesPending);

  function publish(updated: ListenerStatus, owner: ConnectedSession, key: readonly unknown[]) {
    assertCurrent(owner);
    owner.queryClient.setQueryData(key, updated);
    return updated;
  }

  function fail(title: string, description: string, error?: unknown) {
    toast.error(failureOf(error, title), { description });
    void queryClient.invalidateQueries({ queryKey });
  }

  function assertCurrent(owner: ConnectedSession) {
    if (!owner.identity.isCurrent()) {
      throw new Error("Backend connection changed. Reconnect to check the remote settings.");
    }
  }

  async function run(work: (owner: ConnectedSession, key: readonly unknown[]) => Promise<void>) {
    if (disabled || !session) {
      return;
    }
    const owner = session;
    const key = queryKey;
    busy = true;
    try {
      await work(owner, key);
    } finally {
      busy = false;
    }
  }

  async function restart() {
    const status = query.data;
    if (!status?.settings) {
      return;
    }
    await run(async (owner, key) => {
      await commitListener({
        status,
        change: {},
        send: async (request) => publish(await updateNativeListener(owner.identity, status, request), owner, key),
        fail: (description, error) => fail("Remote Access Change Failed", description, error),
      });
    });
  }

  async function regenerate() {
    await run(async (owner, key) => {
      try {
        publish(await regenerateNativeCertificate(owner.identity), owner, key);
      } catch (error: unknown) {
        fail("Certificate Generation Failed", failureOf(error, "Settings Operation Failed").message, error);
      }
    });
  }

  async function createPairing() {
    await run(async (owner, key) => {
      try {
        const path = await commands.desktopPairingCreate();
        assertCurrent(owner);
        if (path !== null) {
          toast.success("Remote Access File Saved", { description: path });
          void owner.queryClient.invalidateQueries({ queryKey: key });
        }
      } catch (error: unknown) {
        fail("Remote Access File Not Saved", failureOf(error, "Settings Operation Failed").message, error);
      }
    });
  }
</script>

{#if !canManage || !session}
  <SettingsCard title="Remote Access">
    <SettingsRow label="Host Settings">
      <p class="text-sm text-muted-foreground">Open Remote Access in the host's connected desktop session to configure its listener.</p>
    </SettingsRow>
  </SettingsCard>
{:else}
  {@render preferences?.()}
  {#if query.isPending}
    <SettingsCard title="Remote Access">
      <SettingsRow label="Remote Settings">
        <p role="status" class="text-sm text-muted-foreground">Loading remote settings…</p>
      </SettingsRow>
    </SettingsCard>
  {:else if query.error || !query.data?.settings}
    <SettingsCard title="Remote Access">
      <SettingsRow label="Remote Settings">
        <ErrorCard
          failure={failureOf(query.error ?? "Reload to read the host's applied listener settings.", "Remote Settings Unavailable")}
          onRetry={() => void query.refetch()}
        />
      </SettingsRow>
    </SettingsCard>
  {:else}
    {@const status = query.data}
    {#if session.snapshot.ownership === "owned_local"}
      <FirewallNotice
        generation={session.identity.generation}
        listenerAddress={status.listeningAddress}
        enabled={status.settings.enabled}
        {label}
        {queryClient}
      />
    {/if}
    <SettingsCard title="Effective Listener State">
      {#snippet footer()}
        <Button variant="outline" {disabled} onclick={restart}>Restart Listener</Button>
      {/snippet}
      <SettingsRow label="Status" description="Saved preferences take effect when the listener restarts." error={status.error}>
        <span role="status">{listenerRunning(status) ? status.listeningAddress : "Stopped"}</span>
      </SettingsRow>
    </SettingsCard>
    {#if advanced}
      <SettingsCard title="Certificate">
        <SettingsRow
          label="Certificate Identity"
          description="Replacing the self-signed certificate requires exporting and importing a new pairing file."
        >
          <Button variant="outline" {disabled} onclick={regenerate}>Regenerate Certificate</Button>
        </SettingsRow>
      </SettingsCard>
    {/if}
    <SettingsCard
      title="Remote Access Files"
      description="Anyone with the file has full control. Keep pairing files private; each export creates a separate client credential."
    >
      <div class="grid gap-3 p-4">
        <p class="text-sm text-muted-foreground">
          Endpoints are detected automatically. Loopback addresses (127.0.0.1, ::1) work on the same PC only.
        </p>
        <ul aria-label="Export Endpoints" class="grid gap-1 text-sm">
          {#each status.endpoints as endpoint (endpoint.address)}
            <li>
              <span class="font-mono">{endpoint.address}</span>
              {#if endpoint.adapterName}
                <span class="text-muted-foreground"> — {endpoint.adapterName}</span>
              {/if}
            </li>
          {/each}
        </ul>
        <Button {disabled} onclick={createPairing}>Export Remote Access</Button>
      </div>
    </SettingsCard>
  {/if}
{/if}
