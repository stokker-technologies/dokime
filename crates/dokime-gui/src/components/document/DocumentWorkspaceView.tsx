/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */
import { useCallback, useMemo, useState } from "react";

/* === Local Imports === */
import { useNativeSessionOptional } from "@/app/native-session/context";
import { updatePlotSettings } from "@/lib/api/native-documents";
import { useReportDefaults } from "@/lib/application-settings";
import { orderedDocumentSections } from "@/lib/document-layout";
import { useLivePlotPayloads } from "@/plots/live-data";
import { PlotSettingsButton } from "@/plots/PlotSettingsDialog";
import { PlotSettingsEditorContext, type PlotDisplaySettings } from "@/plots/plot-settings";
import { DocumentCardContent } from "./DocumentSurface";
import { isDocumentSurfaceCard } from "./document-surface-policy";
import { DocumentWorkspacePresentation, type DocumentWorkspaceViewProps, visibleDocumentPlots } from "./DocumentWorkspacePresentation";

/* === Components === */
export function DocumentWorkspaceView(props: DocumentWorkspaceViewProps) {
  return props.renderMode === "report" ? (
    <DocumentWorkspacePresentation {...props} />
  ) : (
    <PlotSettingsEditorContext.Provider value={PlotSettingsButton}>
      <ConnectedDocumentWorkspace {...props} />
    </PlotSettingsEditorContext.Provider>
  );
}

/* === Private Components === */
function ConnectedDocumentWorkspace(props: Extract<DocumentWorkspaceViewProps, { renderMode: "live" }>) {
  const resolvedCardFilter = props.cardFilter ?? isDocumentSurfaceCard;
  const scopeKey = `${props.document?.document_id}/${props.plotResetKey}`;
  /* A persisted plot setting is applied to the open document as soon as the
     host accepted it, so the watch reopens with the new budget without waiting
     for the frozen document to be captured again. */
  const [plotSettingsOverrides, setPlotSettingsOverrides] = useState<{
    key: string;
    settings: Record<string, PlotDisplaySettings>;
  }>({ key: scopeKey, settings: {} });
  const plotSettings = useMemo(
    () => (plotSettingsOverrides.key === scopeKey ? plotSettingsOverrides.settings : {}),
    [plotSettingsOverrides, scopeKey],
  );
  const nativeSession = useNativeSessionOptional();
  const cycleId = props.scope?.test_cycle_id ?? null;
  const executionId = props.scope?.test_execution_id ?? null;
  const documentId = props.document?.document_id;
  const onPlotSettingsChange = useCallback(
    async (plotId: string, settings: PlotDisplaySettings) => {
      if (!nativeSession || !cycleId || !executionId || !documentId) {
        throw new Error("A running cycle on a connected backend is required.");
      }
      await updatePlotSettings(nativeSession.identity, {
        cycleId,
        executionId,
        documentId,
        plotId,
        displayPoints: settings.display_points,
        downsample: settings.downsample,
      });
      setPlotSettingsOverrides((previous) => ({
        key: scopeKey,
        settings: {
          ...(previous.key === scopeKey ? previous.settings : {}),
          [plotId]: settings,
        },
      }));
    },
    [cycleId, documentId, executionId, nativeSession, scopeKey],
  );
  const document = useMemo(
    () =>
      props.document
        ? {
            ...props.document,
            cards: props.document.cards.map((card) => ({
              ...card,
              plots: card.plots.map((plot) =>
                Object.hasOwn(plotSettings, plot.plot_id) ? { ...plot, ...plotSettings[plot.plot_id] } : plot,
              ),
            })),
          }
        : props.document,
    [props.document, plotSettings],
  );
  const sections = useMemo(() => orderedDocumentSections(document), [document]);
  const reportDefaults = useReportDefaults();
  /* A plot nobody configured streams with the application defaults, which the
     backend also freezes into every report of this run. */
  const liveDocumentPlots = useMemo(
    () =>
      visibleDocumentPlots(sections, resolvedCardFilter).map((plot) => ({
        ...plot,
        display_points: plot.display_points ?? reportDefaults.plotPoints,
        downsample: (plot.downsample ?? "auto") === "auto" ? reportDefaults.plotDownsample : plot.downsample,
      })),
    [sections, resolvedCardFilter, reportDefaults],
  );
  const livePlotStates = useLivePlotPayloads({
    enabled: true,
    plots: liveDocumentPlots,
    scope: props.scope,
    windowS: props.plotWindowS,
  });

  return (
    <DocumentWorkspacePresentation
      {...props}
      document={document}
      livePlotStates={livePlotStates}
      onPlotSettingsChange={cycleId ? onPlotSettingsChange : undefined}
      renderCardContent={(cardProps) => <DocumentCardContent {...cardProps} />}
    />
  );
}
