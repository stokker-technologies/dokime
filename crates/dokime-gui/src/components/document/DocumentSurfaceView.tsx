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

import type { EvidenceScope } from "@generated/host/model";
import type { Wire } from "@/lib/rpc/canonical";
import { type ReactNode, useContext } from "react";

/* === Local Imports === */

import { ErrorCard } from "@/components/ErrorCard";
import { DocumentRetryContext } from "./DocumentViewRenderer";
import ArtifactTreeView from "../ArtifactTreeView.svelte";
import DocumentMetricFlow from "./overview/DocumentMetricFlow.svelte";
import { metricFlowStorageKey } from "./overview/metric-model";
import DocumentStatus from "./DocumentStatus.svelte";
import DocumentTable from "./DocumentTable.svelte";
import DocumentTraceability from "./DocumentTraceability.svelte";
import { SvelteMount } from "../SvelteMount";
import OperatorNotes from "./OperatorNotes.svelte";
import { LivePlotGroupGrid, type PlotSettingsChange } from "@/components/document/PlotGroupGrid";
import { RecordTimeline } from "./RecordTimeline";
import StepRecordList from "./StepRecordList.svelte";
import MediaGalleryView from "../MediaGalleryView.svelte";
import type { MediaGalleryArtifact } from "../media-gallery-model";
import type { ArtifactTreeInput } from "../../lib/artifact-tree";
import type {
  DocumentViewCard,
  DocumentViewPlot,
  HealthItem,
  HubRecord,
  OperatorNote,
  PlotPayload,
  StepExecutionRecord,
  TelemetryStream,
  TraceabilitySnapshot,
} from "@/lib/types";
import { emptyLivePlotState, type LivePlotStateMap } from "@/plots/live-frames";

/* === Types === */

type DocumentSurfaceSharedProps = {
  captureControls?: ReactNode;
  mediaContent?: ReactNode;
  artifactsContent?: ReactNode;
  notesContent?: ReactNode;
  artifacts: ArtifactTreeInput[];
  card: DocumentViewCard;
  streams: TelemetryStream[];
  criterionRecords: StepExecutionRecord[];
  documentId?: string;
  records: HubRecord[];
  health: HealthItem[];
  mediaArtifacts: MediaGalleryArtifact[];
  notes: OperatorNote[];
  plotParameterContext?: Record<string, unknown> | null;
  runDir?: string | null;
  /** The enclosing section's label, or the card's own title at the top level. */
  sectionTitle: string;
  stepRecords: StepExecutionRecord[];
  traceability: TraceabilitySnapshot | null | undefined;
};

type LiveDocumentSurfaceProps = DocumentSurfaceSharedProps & {
  /** The execution scope host captures attach to; absent hides the controls. */
  captureScope?: Wire<EvidenceScope> | null;
  /** The unsealed cycle or execution a new operator note is filed against. */
  noteScope?: Wire<EvidenceScope> | null;
  livePlotStates: LivePlotStateMap;
  onPlotSettingsChange?: PlotSettingsChange;
  plotContextTelemetry: Record<string, TelemetryStream> | null;
  plotResetKey: string;
  renderMode?: "live";
};

type ReportDocumentSurfaceProps = DocumentSurfaceSharedProps & {
  renderMode: "report";
  reportPlots: Record<string, PlotPayload>;
  reportTelemetry: Record<string, TelemetryStream> | null;
};

export type DocumentCardContentProps = LiveDocumentSurfaceProps | ReportDocumentSurfaceProps;

/* === Constants === */

/** A recorded number in fixed notation, or the absent-value dash. */

/* === Public Functions === */

export function DocumentCardContentView(props: DocumentCardContentProps) {
  const retry = useContext(DocumentRetryContext);
  const reportMode = isReportDocumentSurface(props);
  if (props.card.failure)
    return <ErrorCard failure={props.card.failure} onRetry={reportMode || !retry ? undefined : () => retry(props.card.card_id)} />;
  if (props.card.loading)
    return (
      <p role="status" className="text-sm text-muted-foreground">
        Loading…
      </p>
    );

  if (props.card.kind === "overview") {
    const groups = props.card.groups.map((group) => ({
      groupId: group.group_id,
      title: group.title,
      metrics: group.metrics.map((metric) => ({
        metricId: metric.metric_id,
        detail: metric.detail ?? undefined,
        label: metric.label,
        rawValue: metric.value,
        presentation: metric.presentation,
        state: metric.state,
        value: metric.formatted,
        exactValue: metric.exact ?? undefined,
      })),
    }));
    const storageKey = metricFlowStorageKey({
      blockId: props.card.card_id,
      documentId: props.documentId,
    });
    return <SvelteMount component={DocumentMetricFlow} props={{ groups, storageKey }} />;
  }

  if (props.card.kind === "table") {
    return <SvelteMount component={DocumentTable} props={{ rows: props.card.rows }} />;
  }

  if (props.card.kind === "plots") {
    return (
      <div className="plot-grid grid gap-3">
        <LivePlotGroupGrid
          records={props.records}
          documentPlots={props.card.plots}
          liveStates={reportMode ? reportPlotStates(props.card.plots, props.reportPlots) : props.livePlotStates}
          onPlotSettingsChange={reportMode ? undefined : props.onPlotSettingsChange}
          parameters={props.plotParameterContext}
          resetKey={reportMode ? "All" : props.plotResetKey}
          showGroupHeaders={false}
          telemetry={reportMode ? props.reportTelemetry : props.plotContextTelemetry}
          title={props.sectionTitle}
          viewportMode={reportMode ? "fit" : "live"}
          zoom={reportMode}
        />
      </div>
    );
  }

  if (props.card.kind === "media") {
    return (
      <div className="grid gap-3">
        {reportMode ? null : props.captureControls}
        {props.mediaContent ?? (
          <SvelteMount component={MediaGalleryView} props={{ artifacts: props.mediaArtifacts, emptyMessage: "No media recorded.", runDir: props.runDir }} />
        )}
      </div>
    );
  }

  if (props.card.kind === "artifacts") {
    return (
      props.artifactsContent ?? <SvelteMount component={ArtifactTreeView} props={{ artifacts: props.artifacts, emptyMessage: "No artifacts recorded.", runDir: props.runDir }} />
    );
  }

  if (props.card.kind === "traceability") {
    return <SvelteMount component={DocumentTraceability} props={{ traceability: props.traceability }} />;
  }

  if (props.card.kind === "telemetry_context") {
    return <SvelteMount component={DocumentStatus} props={{ streams: props.streams }} />;
  }

  if (props.card.kind === "notes") {
    return props.notesContent ?? <SvelteMount component={OperatorNotes} props={{ notes: props.notes }} />;
  }

  if (props.card.kind === "steps") {
    return (
      <SvelteMount component={StepRecordList} props={{
        records: props.stepRecords,
        emptyMessage: reportMode ? "No step records for the selected case run." : "No step records for the selected execution yet.",
      }} />
    );
  }

  if (props.card.kind === "criteria") {
    return (
      <SvelteMount component={StepRecordList} props={{
        records: props.criterionRecords,
        emptyMessage: reportMode ? "No criteria records for the selected case run." : "No criteria records for the selected execution yet.",
      }} />
    );
  }

  if (props.card.kind === "health") {
    return <SvelteMount component={DocumentStatus} props={{ health: props.health }} />;
  }

  if (props.card.kind === "records") {
    return <RecordListContent live={!reportMode} records={props.records} />;
  }

  return null;
}

export function RecordListContent({ live = true, records }: { live?: boolean; records: HubRecord[] }) {
  return <RecordTimeline live={live} records={records} />;
}

/* === Private Functions === */

function reportPlotStates(documentPlots: DocumentViewPlot[], reportPlots: Record<string, PlotPayload>): LivePlotStateMap {
  return Object.fromEntries(
    documentPlots.flatMap((plot) => {
      const payload = reportPlots[plot.plot_id];
      if (!payload) {
        return [];
      }
      return [
        [
          plot.plot_id,
          {
            ...emptyLivePlotState(),
            payload,
            status: "live" as const,
          },
        ],
      ];
    }),
  );
}

function isReportDocumentSurface(props: DocumentCardContentProps): props is ReportDocumentSurfaceProps {
  return props.renderMode === "report";
}

/* === Re-Exports === */

/* === Components === */

/* === Hooks === */

/* === Private Functions === */
