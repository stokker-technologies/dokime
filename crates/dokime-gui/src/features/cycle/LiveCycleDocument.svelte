<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { ConnectedSession } from "../../app/native-session/session";
  import type { ReportDefaults } from "../../plots/plot-settings";

  export type LiveCycleContext = {
    session: ConnectedSession;
    plotPalette: string[];
    reportDefaults: ReportDefaults;
    traceabilitySave?: ((json: string, filename: string) => Promise<void>) | null;
  };
</script>

<script lang="ts">
  import LiveDocumentWorkspace from "../../components/document/LiveDocumentWorkspace.svelte";
  import { documentWorkspace, type DocumentViewProps } from "./document-workspace";

  let { context, surface, plotWindow, cardFilter, cardChrome, onRetry }: DocumentViewProps & { context: LiveCycleContext } = $props();
  const workspace = $derived(documentWorkspace({ surface, plotWindow, cardFilter, cardChrome }));
</script>

{#if workspace.renderMode === "live"}
  <LiveDocumentWorkspace {...workspace} {...context} {onRetry} />
{/if}
