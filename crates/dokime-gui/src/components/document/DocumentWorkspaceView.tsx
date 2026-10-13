/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { type ReactNode, useCallback, useMemo, useState } from "react";

/* === Local Imports === */

import {
  type BoxSummarySectionSpec,
  DocumentCardContent,
  type DocumentCardContentProps,
  type HistogramSectionSpec,
} from "@/components/document/DocumentSurface";
import { DocumentViewRenderer } from "@/components/document/DocumentViewRenderer";
import { isDocumentSurfaceCard } from "@/components/document/document-surface-policy";
import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import type { MediaGalleryArtifact } from "@/components/MediaGallery";
import type { ArtifactTreeInput } from "@/lib/artifact-tree";
import { updatePlotSettings } from "@/lib/api/native-documents";
import { orderedDocumentSections } from "@/lib/document-layout";
import type { DocumentBoxSummaryPayload, DocumentHistogramPayload } from "@/lib/report-types";
import type {
  DocumentViewCard,
  DocumentViewPayload,
  DocumentViewSection,
  HealthItem,
  HubRecord,
  OperatorNote,
  PlotPayload,
  RecordScope,
  StepExecutionRecord,
  TelemetryStream,
  TraceabilitySnapshot,
} from "@/lib/types";
import { type LivePlotStateMap, useLivePlotPayloads } from "@/plots/live-data";
import type { PlotDisplaySettings } from "@/plots/PlotSettingsDialog";

/* === Constants === */

/* === Types === */

type DocumentWorkspaceViewBaseProps = {
  artifacts: ArtifactTreeInput[];
  cardFilter?: (card: DocumentViewCard) => boolean;
  streams: TelemetryStream[];
  criterionRecords: StepExecutionRecord[];
  document: DocumentViewPayload | null | undefined;
  emptyMessage?: ReactNode;
  records: HubRecord[];
  health: HealthItem[];
  mediaArtifacts: MediaGalleryArtifact[];
  notes: OperatorNote[];
  plotParameterContext?: Record<string, unknown> | null;
  prependSections?: ReactNode;
  runDir?: string | null;
  sectionChrome?: "plain" | "card";
  sections: DocumentViewSection[];
  stepRecords: StepExecutionRecord[];
  traceability: TraceabilitySnapshot | null | undefined;
};

type LiveDocumentWorkspaceViewProps = DocumentWorkspaceViewBaseProps & {
  boxSummarySections: BoxSummarySectionSpec[];
  histogramSections: HistogramSectionSpec[];
  plotContextTelemetry: Record<string, TelemetryStream> | null;
  plotResetKey: string;
  plotWindowS?: number | null;
  renderMode: "live";
  scope: RecordScope | null;
};

type ReportDocumentWorkspaceViewProps = DocumentWorkspaceViewBaseProps & {
  renderMode: "report";
  reportBoxSummaries: Record<string, DocumentBoxSummaryPayload>;
  reportHistograms: Record<string, DocumentHistogramPayload>;
  reportPlots: Record<string, PlotPayload>;
  reportTelemetry: Record<string, TelemetryStream> | null;
};

export type DocumentWorkspaceViewProps =
  | LiveDocumentWorkspaceViewProps
  | ReportDocumentWorkspaceViewProps;

/* === Components === */

export function DocumentWorkspaceView(props: DocumentWorkspaceViewProps) {
  const authoredFilter = props.cardFilter ?? isDocumentSurfaceCard;
  /* A frozen file cannot gain artifacts or media later, so an empty one of
     either is a card that will never fill: it is left out rather than shown
     saying nothing. Live, the same card is a promise the run may still keep. */
  const artifactCount = props.artifacts.length;
  const mediaCount = props.mediaArtifacts.length;
  const frozen = props.renderMode === "report";
  const resolvedCardFilter = useCallback(
    (card: DocumentViewCard) => {
      if (!authoredFilter(card)) return false;
      if (!frozen) return true;
      if (card.kind === "artifacts") return artifactCount > 0;
      if (card.kind === "media") return mediaCount > 0;
      return true;
    },
    [artifactCount, authoredFilter, frozen, mediaCount],
  );
  const scopeKey =
    props.renderMode === "live" ? `${props.document?.document_id}/${props.plotResetKey}` : "report";
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
  const cycleId = props.renderMode === "live" ? (props.scope?.test_cycle_id ?? null) : null;
  const onPlotSettingsChange = useCallback(
    async (plotId: string, settings: PlotDisplaySettings) => {
      if (!nativeSession || !cycleId) {
        throw new Error("A running cycle on a connected backend is required.");
      }
      await updatePlotSettings(nativeSession.identity, {
        cycleId,
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
    [cycleId, nativeSession, scopeKey],
  );
  const document = useMemo(
    () =>
      props.document
        ? {
            ...props.document,
            cards: props.document.cards.map((card) => ({
              ...card,
              plots: card.plots.map((plot) =>
                Object.hasOwn(plotSettings, plot.plot_id)
                  ? { ...plot, ...plotSettings[plot.plot_id] }
                  : plot,
              ),
            })),
          }
        : props.document,
    [props.document, plotSettings],
  );
  const sections = useMemo(() => orderedDocumentSections(document), [document]);
  const liveDocumentPlots = useMemo(
    () => (props.renderMode === "live" ? visibleDocumentPlots(sections, resolvedCardFilter) : []),
    [props.renderMode, sections, resolvedCardFilter],
  );
  const livePlotStates = useLivePlotPayloads({
    enabled: props.renderMode === "live",
    plots: liveDocumentPlots,
    scope: props.renderMode === "live" ? props.scope : null,
    windowS: props.renderMode === "live" ? props.plotWindowS : null,
  });

  return (
    <DocumentViewRenderer
      cardFilter={resolvedCardFilter}
      document={document}
      emptyMessage={props.emptyMessage}
      prependSections={props.prependSections}
      sectionChrome={props.sectionChrome}
      renderCard={(card) => (
        <DocumentCardContent
          {...documentCardContentProps(props, card, livePlotStates)}
          {...(props.renderMode === "live" && cycleId ? { onPlotSettingsChange } : {})}
        />
      )}
    />
  );
}

/* === Private Functions === */

function documentCardContentProps(
  props: DocumentWorkspaceViewProps,
  card: DocumentViewCard,
  livePlotStates: LivePlotStateMap,
): DocumentCardContentProps {
  const shared = {
    artifacts: props.artifacts,
    card,
    streams: props.streams,
    criterionRecords: props.criterionRecords,
    documentId: props.document?.document_id,
    records: props.records,
    health: props.health,
    mediaArtifacts: props.mediaArtifacts,
    notes: props.notes,
    plotParameterContext: props.plotParameterContext ?? null,
    runDir: props.runDir,
    stepRecords: props.stepRecords,
    traceability: props.traceability,
  };

  if (props.renderMode === "report") {
    return {
      ...shared,
      renderMode: "report",
      reportBoxSummaries: props.reportBoxSummaries,
      reportHistograms: props.reportHistograms,
      reportPlots: props.reportPlots,
      reportTelemetry: props.reportTelemetry,
    };
  }

  return {
    ...shared,
    boxSummarySection: props.boxSummarySections.find((item) => item.key === card.card_id),
    histogramSection: props.histogramSections.find((item) => item.key === card.card_id),
    livePlotStates,
    plotContextTelemetry: props.plotContextTelemetry,
    plotResetKey: props.plotResetKey,
    renderMode: "live",
  };
}

function visibleDocumentPlots(
  sections: DocumentViewSection[],
  cardFilter: (card: DocumentViewCard) => boolean,
) {
  return sections.flatMap((section) =>
    section.cards
      .filter((card) => card.kind === "plots" && cardFilter(card))
      .flatMap((card) => card.plots),
  );
}
