<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { type ComponentProps, untrack } from "svelte";
  import type { ConnectedSession } from "../../app/native-session/session";
  import type { PlotWindow } from "../../features/cycle/plot-window-state";
  import { savedPlotPoints, updatePlotSettings } from "../../lib/api/native-document-session";
  import { orderedDocumentSections } from "../../lib/document-layout";
  import { failureOf } from "../../lib/notification-details";
  import { toast } from "../../lib/notifications";
  import { livePlotDefinition, widthBudgeted } from "../../plots/live-data";
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

  type Workspace = Omit<
    Extract<ComponentProps<typeof DocumentWorkspacePresentation>, { renderMode: "live" }>,
    "renderMode" | "livePlotStates" | "plotResetKey" | "measurements"
  >;
  let {
    session,
    reportDefaults,
    plotWindow,
    complete,
    ...workspace
  }: Workspace & { session: ConnectedSession | null; reportDefaults: ReportDefaults; plotWindow?: PlotWindow; complete?: boolean } =
    $props();
  const transfers = $derived(session ? nativeMediaTransfers(session) : undefined);
  const resolvedCardFilter = $derived(workspace.cardFilter ?? isDocumentSurfaceCard);
  const plotResetKey = $derived(plotWindow?.option().label ?? "");
  const scopeKey = $derived(`${workspace.document?.document_id}/${plotResetKey}`);
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
  // The Host Merges A Saved Count Into The Definition, So Only A Plot Carrying A Count Can Have One.
  const pointsKey = $derived(`${workspace.scope?.executionId}/${workspace.document?.document_id}`);
  const countedPlotIds = $derived(
    visibleDocumentPlots(orderedDocumentSections(workspace.document), resolvedCardFilter)
      .filter((plot) => widthBudgeted(plot) && plot.display_points != null)
      .map((plot) => plot.plot_id)
      .join(";"),
  );
  let savedPoints = $state.raw<{ key: string; points: Record<string, number | null> }>({ key: "", points: {} });
  const saved = $derived(savedPoints.key === pointsKey ? savedPoints.points : {});
  // Unconfigured Plots Use The Backend Defaults Captured In The Report; Cartesian Plots Size From Their Width.
  const plots = $derived(
    visibleDocumentPlots(sections, resolvedCardFilter).map((plot) => livePlotDefinition(plot, saved[plot.plot_id], reportDefaults)),
  );
  const live = createLivePlotPayloads(
    () => ({ complete, enabled: true, plots, scope: workspace.scope, window: plotWindow }),
    () => session,
  );

  // Keyed By Execution And Document, So A Refreshed Scope Or Frame Does Not Read Again.
  $effect(() => {
    const current = session;
    const key = pointsKey;
    const plotIds = countedPlotIds ? countedPlotIds.split(";") : [];
    const scope = untrack(() => workspace.scope);
    const executionId = scope?.executionId;
    const documentId = untrack(() => workspace.document?.document_id);
    if (!current || !executionId || !documentId || plotIds.length === 0) {
      return;
    }
    let active = true;
    savedPlotPoints(current.identity, { cycleId: scope?.cycleId ?? "", executionId, documentId }, plotIds).then(
      (points) => {
        if (active && current.identity.isCurrent()) {
          // A Count Saved In This Session Stays Over The Read.
          savedPoints = { key, points: { ...points, ...(savedPoints.key === key ? savedPoints.points : {}) } };
        }
      },
      (error: unknown) => {
        if (active && current.identity.isCurrent()) {
          toast.error(failureOf(error, "Read Plot Settings Failed"), { context: "background" });
        }
      },
    );
    return () => {
      active = false;
    };
  });

  async function onPlotSettingsChange(plotId: string, settings: PlotDisplaySettings) {
    const current = session;
    const cycleId = workspace.scope?.cycleId;
    const executionId = workspace.scope?.executionId;
    const documentId = workspace.document?.document_id;
    const key = scopeKey;
    const savedKey = pointsKey;
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
    if (!current.identity.isCurrent()) {
      return;
    }
    if (scopeKey === key) {
      const previous = overrides.key === key ? overrides.settings : {};
      overrides = { key, settings: { ...previous, [plotId]: settings } };
    }
    if (pointsKey === savedKey) {
      const previous = savedPoints.key === savedKey ? savedPoints.points : {};
      savedPoints = { key: savedKey, points: { ...previous, [plotId]: settings.display_points } };
    }
  }
</script>

<DocumentWorkspacePresentation
  {...workspace}
  {document}
  renderMode="live"
  {plotResetKey}
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
