<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import ErrorCard from "../../components/ErrorCard.svelte";
  import { failureOf } from "../../lib/notification-details";
  import type { AutosaveState } from "./settings-autosave";

  let { onRetry, state }: { onRetry: () => void; state: AutosaveState } = $props();
</script>

{#if state.kind === "error"}
  <ErrorCard
    notificationContext="action"
    failure={failureOf(state.failure ?? state.message, "Settings Could Not Be Saved")}
    {onRetry}
    retryLabel="Save Settings"
  />
{:else}
  <span role="status" class="inline-flex min-h-5 items-center text-xs text-muted-foreground">
    {state.kind === "idle" ? "" : state.kind === "saving" ? "Saving Settings…" : "Settings Saved"}
  </span>
{/if}
