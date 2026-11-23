<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { onMount } from "svelte";
  import ErrorCard from "../components/ErrorCard.svelte";
  import Button from "../components/ui/Button.svelte";
  import { releaseNativeMedia } from "../lib/api/native-delivery";
  import type { AppearanceResource } from "../lib/native-appearance";
  import { failureOf } from "../lib/notification-details";
  import { loadAppearance } from "../lib/rpc/appearance";
  import DesktopPresentation from "./DesktopPresentation.svelte";
  import DesktopRoutes from "./DesktopRoutes.svelte";
  import type { ConnectedSession, NativeSession } from "./native-session/session";

  let { owner, session }: { owner: NativeSession; session: ConnectedSession } = $props();
  let appearance = $state.raw<AppearanceResource | null>(null);
  let error = $state.raw<unknown>();
  let attempt = $state(0);

  onMount(() => {
    const generation = session.identity.generation;
    return () => {
      void releaseNativeMedia(generation);
    };
  });

  $effect(() => {
    void attempt;
    const current = session;
    let active = true;
    let release: (() => void) | undefined;
    error = undefined;
    void loadAppearance(current.identity)
      .catch((error: unknown) => {
        if (active && current.identity.isCurrent() && needsAppearanceRefresh(error)) {
          return loadAppearance(current.identity);
        }
        throw error;
      })
      .then((resource) => {
        if (!active || !current.identity.isCurrent()) {
          resource.dispose();
          return;
        }
        release = current.own(() => resource.dispose());
        appearance = resource;
      })
      .catch((failure: unknown) => {
        if (active && current.identity.isCurrent()) {
          error = failure;
        }
      });
    return () => {
      active = false;
      release?.();
    };
  });

  function needsAppearanceRefresh(error: unknown): boolean {
    if (!error || typeof error !== "object" || !("kind" in error) || error.kind !== "grpc" || !("detail" in error)) {
      return false;
    }
    const detail = error.detail;
    return !!detail && typeof detail === "object" && "code" in detail && (detail.code === 5 || detail.code === 9);
  }
</script>

<DesktopPresentation {session} client={session.queryClient} resource={appearance}>
  {#snippet children(
    presentation,
  )}
    {#if appearance?.unavailableReason}
      <div class="border-b p-3 text-sm text-muted-foreground" role="status">{appearance.unavailableReason}</div>
    {/if}
    {#if error}
      <main class="mx-auto max-w-xl space-y-4 p-8">
        <h1 class="font-heading text-2xl">Appearance Unavailable</h1>
        <ErrorCard failure={failureOf(error, "Appearance Unavailable")} onRetry={() => (attempt += 1)} />
        <div class="flex gap-3"><Button variant="outline" onclick={() => void owner.disconnect()}>Disconnect</Button></div>
      </main>
    {:else}
      <DesktopRoutes {owner} {session} {presentation} />
    {/if}
  {/snippet}
</DesktopPresentation>
