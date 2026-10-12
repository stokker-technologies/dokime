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
import type { MediaGalleryArtifact } from "@/components/MediaGallery";
import type { ArtifactTreeInput } from "@/lib/artifact-tree";
import { orderedDocumentSections } from "@/lib/document-layout";
import type { DocumentBoxSummaryPayload, DocumentHistogramPayload } from "@/lib/report-types";
import type {
  DocumentViewCard,
  DocumentViewPayload,
  DocumentViewSection,
  EvidenceAnalysisSelection,
  HealthItem,
  HubRecord,
  OperatorNote,
  PlotPayload,
  RecordScope,
  StepExecutionRecord,
  TelemetryStream,
  TraceabilitySnapshot,
} from "@/lib/types";
import { AnalysisControls } from "@/plots/AnalysisControls";
import { type LivePlotStateMap, useLivePlotPayloads } from "@/plots/live-data";

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
  const [analysisOverrides, setAnalysisOverrides] = useState<{
    key: string;
    selections: Record<string, EvidenceAnalysisSelection | null>;
  }>({ key: scopeKey, selections: {} });
  const selections = useMemo(
    () => (analysisOverrides.key === scopeKey ? analysisOverrides.selections : {}),
    [analysisOverrides, scopeKey],
  );
  const document = useMemo(
    () =>
      props.document
        ? {
            ...props.document,
            cards: props.document.cards.map((card) => ({
              ...card,
              plots: card.plots.map((plot) =>
                Object.hasOwn(selections, plot.plot_id)
                  ? { ...plot, analysis: selections[plot.plot_id] }
                  : plot,
              ),
            })),
          }
        : props.document,
    [props.document, selections],
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
      prependSections={
        <>
          {props.prependSections}
          {props.renderMode === "live" && liveDocumentPlots.length > 0 && (
            <AnalysisControls
              key={scopeKey}
              plots={liveDocumentPlots}
              onChange={(plotId, selection) =>
                setAnalysisOverrides((previous) => ({
                  key: scopeKey,
                  selections: {
                    ...(previous.key === scopeKey ? previous.selections : {}),
                    [plotId]: selection,
                  },
                }))
              }
            />
          )}
        </>
      }
      sectionChrome={props.sectionChrome}
      renderCard={(card) => (
        <DocumentCardContent {...documentCardContentProps(props, card, livePlotStates)} />
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
