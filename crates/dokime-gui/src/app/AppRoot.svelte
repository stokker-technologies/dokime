<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { onMount } from "svelte";
  import DisconnectedSettings from "../features/settings/DisconnectedSettings.svelte";
  import { watchRemoteImports } from "../lib/desktop/remote-import";
  import type { DesktopLaunch } from "../lib/desktop/session";
  import { watchStopping } from "../lib/desktop/stopping";
  import DesktopPresentation from "./DesktopPresentation.svelte";
  import DesktopSession from "./DesktopSession.svelte";
  import ConnectionScreen from "./native-session/ConnectionScreen.svelte";
  import { launchDesktopSession } from "./native-session/launch";
  import { NativeSession } from "./native-session/session";
  import { createDokimeQueryClient } from "./query-client";

  const owner = new NativeSession();
  const offline = createDokimeQueryClient();
  let sessionState = $state.raw(owner.snapshot());
  let settings = $state(false);
  let launch = $state.raw<DesktopLaunch | null>(null);
  let launchError = $state.raw<unknown>();

  onMount(() => {
    const unsubscribe = owner.subscribe(() => {
      sessionState = owner.snapshot();
      if (sessionState.phase === "connected") {
        settings = false;
      }
    });
    const stopWatching = watchStopping();
    const stopImports = watchRemoteImports(owner);
    let active = true;
    void launchDesktopSession(
      owner,
      () => active,
      (configuration) => (launch = configuration),
    ).catch((error: unknown) => {
      if (active) {
        launchError = error;
      }
    });
    return () => {
      active = false;
      unsubscribe();
      stopWatching();
      stopImports();
      offline.clear();
      void owner.disconnect();
    };
  });
</script>

{#if sessionState.phase === "connected"}
  {#key sessionState.session.identity.generation}
    <DesktopSession {owner} session={sessionState.session} />
  {/key}
{:else}
  <DesktopPresentation session={null} client={offline} resource={null}>
    {#snippet children(
      presentation,
    )}
      {#if settings}
        <DisconnectedSettings {owner} queryClient={offline} close={() => (settings = false)} />
      {:else}
        <ConnectionScreen {owner} brand={presentation.brand} {launch} {launchError} onSettings={() => (settings = true)} />
      {/if}
    {/snippet}
  </DesktopPresentation>
{/if}
