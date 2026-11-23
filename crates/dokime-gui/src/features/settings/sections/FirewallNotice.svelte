<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createQuery, type QueryClient } from "@tanstack/svelte-query";
  import Button from "@/components/ui/Button.svelte";
  import { commands } from "@/lib/desktop/commands";
  import SettingsCard from "../SettingsCard.svelte";

  let {
    generation,
    listenerAddress,
    enabled,
    label,
    queryClient,
  }: { generation: string; listenerAddress?: string; enabled: boolean; label: string; queryClient: QueryClient } = $props();
  let busy = $state(false);
  let message = $state<string | null>(null);
  const query = createQuery(
    () => {
      const current = generation;
      return {
        queryKey: ["settings", "firewall", current, listenerAddress, enabled],
        queryFn: () => commands.desktopFirewall(current),
        retry: false,
        refetchOnWindowFocus: true,
      };
    },
    () => queryClient,
  );
  const required = $derived(query.data?.required ?? false);
  const feedback = $derived(
    message ||
      query.data?.message ||
      (query.error
        ? "Windows Firewall access could not be checked. Ask an administrator to check inbound access on the listener's networks."
        : null),
  );

  async function allow() {
    if (busy || !required) {
      return;
    }
    busy = true;
    message = null;
    try {
      const status = await commands.desktopFirewall(generation, true);
      message =
        status.message ??
        (status.required
          ? "Windows Firewall still blocks access. Ask an administrator to check the listener's network policy."
          : "Windows Firewall allows remote access for PCs on the same subnet.");
      void query.refetch();
    } catch {
      message =
        "Windows Firewall was not changed. Administrator approval was cancelled or the change was refused. You can keep using the app.";
    } finally {
      busy = false;
    }
  }
</script>

{#if required || feedback}
  <SettingsCard title="Windows Firewall">
    <div class="grid gap-3 p-4">
      {#if required}
        <p role="status" class="text-sm text-muted-foreground">
          Allow {label} through Windows Firewall for PCs on the same subnet. This needs an administrator.
        </p>
      {/if}
      {#if feedback}
        <p role="status" class="text-sm text-muted-foreground">{feedback}</p>
      {/if}
      {#if required}
        <Button variant="outline" disabled={busy} onclick={allow}>Allow In Firewall…</Button>
      {/if}
    </div>
  </SettingsCard>
{/if}
