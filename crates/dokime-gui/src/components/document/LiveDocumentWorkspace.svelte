<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { ComponentProps } from "svelte";
  import type { ConnectedSession } from "../../app/native-session/session";
  import { updatePlotSettings } from "../../lib/api/native-document-session";
  import { orderedDocumentSections } from "../../lib/document-layout";
  import { createLivePlotPayloads } from "../../plots/live-data.svelte";
  import type { PlotDisplaySettings, ReportDefaults } from "../../plots/plot-settings";
  import { summaryWindowStore } from "../../stores/summary-window-store";
  import MediaCaptureControls from "../MediaCaptureControls.svelte";
  import MediaGalleryView from "../MediaGalleryView.svelte";
  import NativeArtifactTree from "../NativeArtifactTree.svelte";
  import { nativeMediaTransfers } from "../native-media-transfers";
  import { isDocumentSurfaceCard } from "./document-surface-policy";
  import DocumentWorkspacePresentation, { visibleDocumentPlots } from "./DocumentWorkspacePresentation.svelte";
  import OperatorNotes from "./OperatorNotes.svelte";
  import OperatorNotesContent from "./OperatorNotesContent.svelte";

  type Workspace = Extract<ComponentProps<typeof DocumentWorkspacePresentation>, { renderMode: "live" }>;
  let { session, reportDefaults, ...workspace }: Workspace & { session: ConnectedSession | null; reportDefaults: ReportDefaults } =
    $props();
  const transfers = $derived(session ? nativeMediaTransfers(session) : undefined);
  const resolvedCardFilter = $derived(workspace.cardFilter ?? isDocumentSurfaceCard);
  const scopeKey = $derived(`${workspace.document?.document_id}/${workspace.plotResetKey}`);
  let overrides = $state.raw<{ key: string; settings: Record<string, PlotDisplaySettings> }>({ key: "", settings: {} });
  const document = $derived.by(() => {
    const source = workspace.document;
    if (!source || overrides.key !== scopeKey) {
      return source;
    }
    return {
      ...source,
      cards: source.cards.map((card) => ({
        ...card,
        plots: card.plots.map((plot) =>
          Object.hasOwn(overrides.settings, plot.plot_id) ? { ...plot, ...overrides.settings[plot.plot_id] } : plot,
        ),
      })),
    };
  });
  const sections = $derived(orderedDocumentSections(document));
  // Unconfigured Plots Use The Backend Defaults Captured In The Report.
  const plots = $derived(
    visibleDocumentPlots(sections, resolvedCardFilter).map((plot) => ({
      ...plot,
      display_points: plot.display_points ?? reportDefaults.plotPoints,
      downsample: (plot.downsample ?? "auto") === "auto" ? reportDefaults.plotDownsample : plot.downsample,
    })),
  );
  const live = createLivePlotPayloads(
    () => ({
      enabled: true,
      plots,
      scope: workspace.scope,
      windowS: workspace.plotWindowS,
      refresh: workspace.plotRefresh,
    }),
    () => session,
  );

  async function onPlotSettingsChange(plotId: string, settings: PlotDisplaySettings) {
    const current = session;
    const cycleId = workspace.scope?.cycleId;
    const executionId = workspace.scope?.executionId;
    const documentId = workspace.document?.document_id;
    const key = scopeKey;
    if (!current || !cycleId || !executionId || !documentId) {
      throw new Error("Open a Running Execution on a Connected Backend Before Saving Plot Settings");
    }
    await updatePlotSettings(current.identity, {
      cycleId,
      executionId,
      documentId,
      plotId,
      displayPoints: settings.display_points,
      downsample: settings.downsample,
    });
    if (scopeKey === key && current.identity.isCurrent()) {
      const previous = overrides.key === key ? overrides.settings : {};
      overrides = { key, settings: { ...previous, [plotId]: settings } };
    }
  }
</script>

<DocumentWorkspacePresentation
  {...workspace}
  {document}
  livePlotStates={live.states}
  measurements={summaryWindowStore.getInitialState()}
  {reportDefaults}
  onPlotSettingsChange={workspace.scope?.cycleId ? onPlotSettingsChange : undefined}
>
  {#snippet cardOverride(
    content,
  )}
    {#if content.card.kind === "media"}
      <div class="grid gap-3">
        {#if session && content.captureScope}
          <MediaCaptureControls />
        {/if}
        <MediaGalleryView artifacts={content.mediaArtifacts} emptyMessage="No media recorded." runDir={content.runDir} {transfers} />
      </div>
    {:else if content.card.kind === "artifacts"}
      <NativeArtifactTree artifacts={content.artifacts} emptyMessage="No artifacts recorded." runDir={content.runDir} {session} />
    {:else if content.card.kind === "notes"}
      {#if session && content.noteScope}
        <OperatorNotesContent notes={content.notes} scope={content.noteScope} {session} />
      {:else}
        <OperatorNotes notes={content.notes} />
      {/if}
    {/if}
  {/snippet}
</DocumentWorkspacePresentation>
