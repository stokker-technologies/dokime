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
  import { documentSectionWindow, type DocumentViewProps } from "./document-workspace";

  let {
    context,
    surface,
    windowControl = false,
    cardFilter,
    cardChrome,
    onRetry,
  }: DocumentViewProps & { context: LiveCycleContext; windowControl?: boolean } = $props();
  const { options, shared, view } = $derived(surface);
  const plotWindow = $derived(options.plotWindow);
  const sectionWindow = $derived(windowControl && plotWindow ? documentSectionWindow(view.sections, plotWindow) : undefined);
</script>

<!-- Each Field Of The Surface Reaches Only Its Own Readers. -->
<LiveDocumentWorkspace
  {...shared}
  {...context}
  {cardChrome}
  {cardFilter}
  {onRetry}
  plotContextTelemetry={view.plotContextTelemetry}
  {plotWindow}
  scope={options.scope}
  {sectionWindow}
/>
