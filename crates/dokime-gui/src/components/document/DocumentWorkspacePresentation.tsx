/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

/* === External Imports === */

import { type ReactNode, useCallback } from "react";

/* === Local Imports === */

import { DocumentCardContentView } from "@/components/document/DocumentSurfaceView";
import { type DocumentBlockChrome, DocumentViewRenderer } from "@/components/document/DocumentViewRenderer";
import { isDocumentSurfaceCard } from "@/components/document/document-surface-policy";
import type { MediaGalleryArtifact } from "@/components/media-gallery-model";
import { TraceabilityJsonDialog } from "@/components/traceability/TraceabilityJsonDialog";
import type { ArtifactTreeInput } from "@/lib/artifact-tree";
import type { TraceabilityExportName } from "@/lib/traceability";
import type {
  DocumentViewCard,
  DocumentViewPayload,
  DocumentViewSection,
  HealthItem,
  HubRecord,
  Metadata,
  OperatorNote,
  PlotPayload,
  RecordScope,
  StepExecutionRecord,
  TelemetryStream,
  TraceabilitySnapshot,
} from "@/lib/types";
import type { LivePlotStateMap } from "@/plots/live-frames";
import type { PlotSettingsChange } from "./PlotGroupGrid";

import type { DocumentCardContentProps } from "@/components/document/DocumentSurfaceView";

/* === Constants === */

/* === Types === */

type DocumentWorkspaceViewBaseProps = {
  renderCardContent?: (props: DocumentCardContentProps) => ReactNode;
  artifacts: ArtifactTreeInput[];
  cardFilter?: (card: DocumentViewCard) => boolean;
  /** The frame each card gets; absent keeps the renderer's default. */
  cardChrome?: (card: DocumentViewCard) => DocumentBlockChrome;
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
  /** Names the traceability JSON a reader downloads from the traceability card. */
  traceabilityExportName?: TraceabilityExportName | null;
};

type LiveDocumentWorkspaceViewProps = DocumentWorkspaceViewBaseProps & {
  livePlotStates?: LivePlotStateMap;
  onPlotSettingsChange?: PlotSettingsChange;
  plotContextTelemetry: Record<string, TelemetryStream> | null;
  plotResetKey: string;
  plotWindowS?: number | null;
  renderMode: "live";
  scope: RecordScope | null;
};

type ReportDocumentWorkspaceViewProps = DocumentWorkspaceViewBaseProps & {
  renderMode: "report";
  reportPlots: Record<string, PlotPayload>;
  reportTelemetry: Record<string, TelemetryStream> | null;
};

export type DocumentWorkspaceViewProps = LiveDocumentWorkspaceViewProps | ReportDocumentWorkspaceViewProps;

/* === Components === */

export function DocumentWorkspacePresentation(props: DocumentWorkspaceViewProps) {
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
  const document = props.document;
  const livePlotStates = props.renderMode === "live" ? (props.livePlotStates ?? {}) : {};

  return (
    <DocumentViewRenderer
      cardAction={(card) =>
        card.kind === "traceability" ? (
          <TraceabilityJsonDialog exportName={props.traceabilityExportName} traceability={props.traceability} />
        ) : null
      }
      cardChrome={props.cardChrome}
      cardFilter={resolvedCardFilter}
      document={document}
      emptyMessage={props.emptyMessage}
      prependSections={props.prependSections}
      sectionChrome={props.sectionChrome}
      renderCard={(card, section) => {
        const contentProps = {
          ...documentCardContentProps(props, card, section, livePlotStates),
          ...(props.renderMode === "live" && props.onPlotSettingsChange ? { onPlotSettingsChange: props.onPlotSettingsChange } : {}),
        };
        return props.renderCardContent ? props.renderCardContent(contentProps) : <DocumentCardContentView {...contentProps} />;
      }}
    />
  );
}

/* === Private Functions === */

function documentCardContentProps(
  props: DocumentWorkspaceViewProps,
  card: DocumentViewCard,
  section: Metadata | null,
  livePlotStates: LivePlotStateMap,
): DocumentCardContentProps {
  const shared = {
    artifacts: props.artifacts,
    card,
    sectionTitle: section?.label ?? card.title,
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
      reportPlots: props.reportPlots,
      reportTelemetry: props.reportTelemetry,
    };
  }

  return {
    ...shared,
    captureScope: props.scope?.test_cycle_id && props.scope.test_execution_id ? props.scope : null,
    livePlotStates,
    noteScope: props.scope?.test_cycle_id ? props.scope : null,
    plotContextTelemetry: props.plotContextTelemetry,
    plotResetKey: props.plotResetKey,
    renderMode: "live",
  };
}

export function visibleDocumentPlots(sections: DocumentViewSection[], cardFilter: (card: DocumentViewCard) => boolean) {
  return sections.flatMap((section) =>
    section.cards.filter((card) => card.kind === "plots" && cardFilter(card)).flatMap((card) => card.plots),
  );
}

/* === Re-Exports === */

/* === Hooks === */

/* === Public Functions === */
