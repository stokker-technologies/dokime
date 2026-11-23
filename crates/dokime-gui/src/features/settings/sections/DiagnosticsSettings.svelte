<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { Tooltip } from "bits-ui";
  import { Download, TriangleAlert } from "lucide-svelte";
  import type { ConnectedSession } from "../../../app/native-session/session";
  import ErrorCard from "../../../components/ErrorCard.svelte";
  import Button from "../../../components/ui/Button.svelte";
  import { collectDiagnosticsBundle, diagnosticsBundleSupported } from "../../../lib/api/native-diagnostics";
  import { saveNativeDiagnosticsBundle } from "../../../lib/api/native-document-delivery";
  import { openLocalPath } from "../../../lib/desktop-open";
  import { failureOf } from "../../../lib/notification-details";
  import { toast } from "../../../lib/notifications";
  import { isTauriDesktop } from "../../../lib/surface-environment";
  import SettingsCard from "../SettingsCard.svelte";
  import SettingsPathField from "../SettingsPathField.svelte";
  import SettingsRow from "../SettingsRow.svelte";
  import StorageSourcesCard from "../StorageSourcesCard.svelte";
  import { settingsFrameQueryOptions } from "../settings-queries";
  import { storageResetAvailable } from "../storage-reset-model";

  let { session }: { session: ConnectedSession } = $props();
  const query = createQuery(
    () => settingsFrameQueryOptions(session.identity),
    () => session.queryClient,
  );
  const paths = $derived(query.data?.bootstrapPaths);
  const pathFields = [
    { key: "application", label: "Application Directory" },
    { key: "storage", label: "Data Directory" },
    { key: "plugins", label: "Plugins Directory" },
    { key: "logs", label: "Logs Directory" },
  ] as const;
  const local = $derived(session.snapshot.ownership === "owned_local");
  const resettable = $derived(
    storageResetAvailable({
      development: import.meta.env.DEV,
      desktop: isTauriDesktop(),
      ownership: session.snapshot.ownership,
      directory: paths?.storage,
    }),
  );
  const supported = diagnosticsBundleSupported();
  let busy = $state(false);

  async function download() {
    if (!supported || busy) {
      return;
    }
    const { identity } = session;
    busy = true;
    try {
      const bundle = await collectDiagnosticsBundle(identity);
      const path = await saveNativeDiagnosticsBundle(identity, { bytes: bundle.bytes, filename: bundle.filename });
      toast.success("Diagnostics Bundle Saved", {
        action: { label: "Open Folder", onClick: () => void openLocalPath(path) },
      });
    } catch (failure: unknown) {
      toast.error(failureOf(failure, "The Diagnostics Bundle Could Not Be Saved"));
    } finally {
      busy = false;
    }
  }
</script>

{#snippet bundleButton()}
  <Button disabled={!supported || busy} onclick={download} size="sm" variant="outline">
    <Download data-icon="inline-start" />
    {busy ? "Collecting…" : "Download Bundle"}
  </Button>
{/snippet}

<SettingsCard title="Bootstrap Paths" description="Paths are selected at startup.">
  {#if query.isPending}
    <p role="status" class="text-sm text-muted-foreground">Loading Paths…</p>
  {/if}
  {#if query.error}
    <ErrorCard failure={failureOf(query.error, "Data Unavailable")} />
  {/if}
  {#if paths}
    {#each pathFields as field (field.key)}
      <SettingsRow label={field.label}>
        <SettingsPathField value={paths[field.key]} openable={local} />
      </SettingsRow>
    {/each}
  {/if}
</SettingsCard>

<SettingsCard title="Support">
  <SettingsRow
    label="Diagnostics Bundle"
    description="Share this archive with the developers when reporting a problem. Secrets are redacted."
  >
    {#if supported}
      {@render bundleButton()}
    {:else}
      <Tooltip.Provider>
        <Tooltip.Root>
          <Tooltip.Trigger>
            {#snippet child({
              props,
            })}
              <span class="inline-flex" {...props}>{@render bundleButton()}</span>
            {/snippet}
          </Tooltip.Trigger>
          <Tooltip.Portal>
            <Tooltip.Content class="z-50 rounded-lg bg-popover px-3 py-2 text-xs text-popover-foreground ring-1 ring-border">
              Not Available On This Backend
            </Tooltip.Content>
          </Tooltip.Portal>
        </Tooltip.Root>
      </Tooltip.Provider>
    {/if}
  </SettingsRow>
</SettingsCard>

{#if resettable && paths?.storage}
  <SettingsCard description="Shown only in development builds that own their local backend." title="Development">
    <SettingsRow
      label="Reset Local Storage"
      description="Recovery is unavailable until the host can preserve plugins, settings, identity and secrets. Preserve this folder and select a fresh storage root."
    >
      <code class="text-xs break-all">{paths.storage}</code>
      <Button disabled size="sm" variant="destructive"><TriangleAlert data-icon="inline-start" /> Reset Local Storage</Button>
    </SettingsRow>
  </SettingsCard>
{/if}

<StorageSourcesCard {session} />
