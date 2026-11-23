<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { QueryClient } from "@tanstack/svelte-query";
  import type { ConnectedSession } from "../../app/native-session/session";
  import type { PreferencesSection } from "./preference-fields";
  import ScopedPreferences from "./ScopedPreferences.svelte";

  let {
    queryClient,
    session = null,
    advanced = false,
    localOnly = false,
    section,
    onPendingChange,
  }: {
    queryClient: QueryClient;
    session?: ConnectedSession | null;
    advanced?: boolean;
    localOnly?: boolean;
    section?: PreferencesSection;
    onPendingChange?: (pending: boolean) => void;
  } = $props();
</script>

{#if section !== "remote"}
  <ScopedPreferences scope="local" {queryClient} {session} {advanced} {section} />
{/if}
{#if !localOnly}
  {#key session?.identity.generation}
    <ScopedPreferences scope="backend" {queryClient} {session} {advanced} {section} {onPendingChange} />
  {/key}
{/if}
