<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { onMount } from "svelte";
  import { releaseNativeMedia } from "../lib/api/native-delivery";
  import { type AppearanceSnapshot, organizationPack } from "../lib/brand";
  import { failureOf } from "../lib/notification-details";
  import { toast } from "../lib/notifications";
  import { loadAppearance } from "../lib/rpc/appearance";
  import DesktopPresentation from "./DesktopPresentation.svelte";
  import DesktopRoutes from "./DesktopRoutes.svelte";
  import type { ConnectedSession, NativeSession } from "./native-session/session";

  let { owner, session }: { owner: NativeSession; session: ConnectedSession } = $props();
  let appearance = $state.raw<AppearanceSnapshot | null>(null);

  onMount(() => {
    const generation = session.identity.generation;
    return () => {
      void releaseNativeMedia(generation);
    };
  });

  // The Shell's Local Pack Already Styled the Page; Only a Shell Without One Reads the Host's Appearance.
  $effect(() => {
    if (organizationPack) return;
    const current = session;
    let active = true;
    void loadAppearance(current.identity)
      .catch((error: unknown) => {
        if (active && current.identity.isCurrent() && needsAppearanceRefresh(error)) {
          return loadAppearance(current.identity);
        }
        throw error;
      })
      .then((snapshot) => {
        if (active && current.identity.isCurrent()) {
          appearance = snapshot;
        }
      })
      .catch((error: unknown) => {
        if (!active || !current.identity.isCurrent()) return;
        // The Build-Time Brand Stands In; Only a Failure That Names an Action Reaches the User.
        appearance = null;
        const failure = failureOf(error, "Host Appearance Not Applied");
        if (failure.action) toast.error(failure, { context: "background" });
      });
    return () => {
      active = false;
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
    <DesktopRoutes {owner} {session} {presentation} />
  {/snippet}
</DesktopPresentation>
