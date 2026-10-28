/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { ReactNode } from "react";

import { ArtifactTreeView } from "@/components/ArtifactTreeView";
import { DataTable } from "@/components/data-table/DataTable";
import { dataTableColumns } from "@/components/data-table/data-table-model";
import { DocumentEmbeddedBlock, DocumentEmbeddedEmptyState } from "@/components/document/DocumentEmbeddedContent";
import { DocumentMetricFlow, metricFlowStorageKey } from "@/components/document/DocumentOverviewCards";
import { FrozenOperatorNotes } from "@/components/document/OperatorNotesView";
import { LivePlotGroupGrid, type PlotSettingsChange } from "@/components/document/PlotGroupGrid";
import { RecordTimeline } from "@/components/document/RecordTimeline";
import { StepRecordList } from "@/components/document/StepRecordsCard";
import { MediaGalleryView } from "@/components/MediaGalleryView";
import type { MediaGalleryArtifact } from "@/components/media-gallery-model";
import { CopyableTraceabilityValue } from "@/components/traceability/CopyableTraceabilityValue";
import type { ArtifactTreeInput } from "@/lib/artifact-tree";
import { formatValue } from "@/lib/dashboard";
import { isStatedGroup } from "@/lib/traceability";
import type {
  DocumentViewCard,
  DocumentViewPlot,
  DocumentViewTableRow,
  HealthItem,
  HubRecord,
  OperatorNote,
  PlotPayload,
  RecordScope,
  StepExecutionRecord,
  TelemetryStream,
  TraceabilitySnapshot,
} from "@/lib/types";
import { PlotInteractionProvider } from "@/plots/interaction";
import { emptyLivePlotState, type LivePlotStateMap } from "@/plots/live-frames";

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
  captureScope?: RecordScope | null;
  /** The unsealed cycle or execution a new operator note is filed against. */
  noteScope?: RecordScope | null;
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

export function DocumentCardContentView(props: DocumentCardContentProps) {
  const reportMode = isReportDocumentSurface(props);

  if (props.card.kind === "overview") {
    return (
      <DocumentMetricFlow
        groups={props.card.groups.map((group) => ({
          groupId: group.group_id,
          title: group.title,
          metrics: group.metrics.map((metric) => ({
            detail: metric.detail ?? undefined,
            label: metric.label,
            rawValue: metric.value,
            presentation: metric.presentation,
            state: metric.state,
            value: metric.formatted,
          })),
        }))}
        storageKey={metricFlowStorageKey({
          blockId: props.card.card_id,
          documentId: props.documentId,
        })}
      />
    );
  }

  if (props.card.kind === "table") {
    return <DocumentTableContent rows={props.card.rows} />;
  }

  if (props.card.kind === "plots") {
    return reportMode ? (
      <div className="plot-grid grid gap-3">
        <PlotInteractionProvider zoom>
          <LivePlotGroupGrid
            {...plotMarkers(props.records)}
            documentPlots={props.card.plots}
            liveStates={reportPlotStates(props.card.plots, props.reportPlots)}
            parameters={props.plotParameterContext}
            resetKey="All"
            showGroupHeaders={false}
            telemetry={props.reportTelemetry}
            title={props.sectionTitle}
            viewportMode="fit"
          />
        </PlotInteractionProvider>
      </div>
    ) : (
      <div className="plot-grid grid gap-3">
        <LivePlotGroupGrid
          {...plotMarkers(props.records)}
          documentPlots={props.card.plots}
          liveStates={props.livePlotStates}
          onPlotSettingsChange={props.onPlotSettingsChange}
          parameters={props.plotParameterContext}
          resetKey={props.plotResetKey}
          showGroupHeaders={false}
          telemetry={props.plotContextTelemetry}
          title={props.sectionTitle}
        />
      </div>
    );
  }

  if (props.card.kind === "media") {
    return (
      <div className="grid gap-3">
        {reportMode ? null : props.captureControls}
        {props.mediaContent ?? (
          <MediaGalleryView artifacts={props.mediaArtifacts} emptyMessage="No media recorded." runDir={props.runDir} />
        )}
      </div>
    );
  }

  if (props.card.kind === "artifacts") {
    return (
      props.artifactsContent ?? <ArtifactTreeView artifacts={props.artifacts} emptyMessage="No artifacts recorded." runDir={props.runDir} />
    );
  }

  if (props.card.kind === "traceability") {
    return <TraceabilityDocumentContent traceability={props.traceability} />;
  }

  if (props.card.kind === "telemetry_context") {
    return <TelemetryContextDocumentContent streams={props.streams} />;
  }

  if (props.card.kind === "notes") {
    return props.notesContent ?? <FrozenOperatorNotes notes={props.notes} />;
  }

  if (props.card.kind === "steps") {
    return (
      <StepRecordList
        records={props.stepRecords}
        emptyMessage={reportMode ? "No step records for the selected case run." : "No step records for the selected execution yet."}
      />
    );
  }

  if (props.card.kind === "criteria") {
    return (
      <StepRecordList
        records={props.criterionRecords}
        emptyMessage={reportMode ? "No criteria records for the selected case run." : "No criteria records for the selected execution yet."}
      />
    );
  }

  if (props.card.kind === "health") {
    return <HealthDocumentContent health={props.health} />;
  }

  if (props.card.kind === "records") {
    return <RecordListContent live={!reportMode} records={props.records} />;
  }

  return null;
}

/** A long table pages inside its own card: the document stays one ordered
    surface, so nothing here touches routing or hides rows behind a tab. */
export function DocumentTableContent({ rows }: { rows: DocumentViewTableRow[] }) {
  const column = dataTableColumns<DocumentViewTableRow>();
  const names = tableColumns(rows);
  const columns = (names.length ? names : ["Value"]).map((name) =>
    column.accessor((row) => row.cells[name] ?? "-", { id: name, header: name, meta: { filter: "text" } }),
  );
  return <DataTable columns={columns} data={rows} getRowId={(row) => row.row_id} emptyText="No table rows recorded." />;
}

export function TraceabilityDocumentContent({ traceability }: { traceability: TraceabilitySnapshot | null | undefined }) {
  const groups = traceability?.groups ?? [];
  return groups.length ? (
    <div className="min-w-0 columns-1 gap-4 md:columns-2 xl:columns-3">
      {groups.map((group) => (
        <DocumentEmbeddedBlock className="mb-4 break-inside-avoid" key={group.group_id}>
          <div className="min-w-0">
            <strong className="block truncate" title={group.title}>
              {group.title}
            </strong>
            {group.summary ? <p className="text-xs text-muted-foreground [overflow-wrap:anywhere]">{group.summary}</p> : null}
          </div>
          <dl className="grid min-w-0 gap-2">
            {/* A stated section is shown whole: the author chose its rows, so
                cutting it at an arbitrary count would drop evidence on
                purpose. A derived group is a projection and stays capped. */}
            {(isStatedGroup(group) ? group.items : group.items.slice(0, 8)).map((item) => (
              <div className="grid min-w-0 grid-cols-2 gap-2" key={String(item.details?.identifier ?? item.label)}>
                <dt className="truncate text-xs text-muted-foreground" title={item.label}>
                  {item.label}
                </dt>
                <dd className="min-w-0 text-right">
                  <CopyableTraceabilityValue value={formatValue(item.value, item.unit)} />
                </dd>
              </div>
            ))}
          </dl>
        </DocumentEmbeddedBlock>
      ))}
      {traceability?.warnings.length ? (
        <DocumentEmbeddedBlock as="ul" className="mb-4 break-inside-avoid gap-1 text-sm text-muted-foreground" tone="warning">
          {traceability.warnings.slice(0, 6).map((warning) => (
            <li key={warning}>{warning}</li>
          ))}
        </DocumentEmbeddedBlock>
      ) : null}
    </div>
  ) : (
    <DocumentEmbeddedEmptyState>No traceability information available.</DocumentEmbeddedEmptyState>
  );
}

export function TelemetryContextDocumentContent({ streams }: { streams: TelemetryStream[] }) {
  return streams.length ? (
    <div className="grid gap-2 md:grid-cols-2 xl:grid-cols-3">
      {streams.map((stream) => (
        <DocumentEmbeddedBlock className="gap-1" key={stream.stream}>
          <div className="min-w-0">
            <strong className="block truncate" title={stream.spec?.label ?? stream.stream}>
              {stream.spec?.label ?? stream.stream}
            </strong>
            <span className="text-xs text-muted-foreground">{stream.stream}</span>
          </div>
          <p className="text-xs text-muted-foreground">
            {stream.total_samples} samples
            {stream.dropped_samples ? `, ${stream.dropped_samples} dropped` : ""}
          </p>
        </DocumentEmbeddedBlock>
      ))}
    </div>
  ) : (
    <DocumentEmbeddedEmptyState>No telemetry streams recorded.</DocumentEmbeddedEmptyState>
  );
}

export function HealthDocumentContent({ health }: { health: HealthItem[] }) {
  return health.length ? (
    <div className="grid gap-2">
      {health.map((item) => (
        <DocumentEmbeddedBlock as="article" className="gap-1" key={item.health_id}>
          <div className="flex items-center justify-between gap-3">
            <strong className="[overflow-wrap:anywhere]">{item.title}</strong>
            <span className="rounded-lg border bg-muted/40 px-2 py-1 text-xs capitalize text-muted-foreground">{item.level}</span>
          </div>
          {item.message ? <p className="text-sm text-muted-foreground [overflow-wrap:anywhere]">{item.message}</p> : null}
        </DocumentEmbeddedBlock>
      ))}
    </div>
  ) : (
    <DocumentEmbeddedEmptyState>No health records for this scope.</DocumentEmbeddedEmptyState>
  );
}

export function RecordListContent({ live = true, records }: { live?: boolean; records: HubRecord[] }) {
  return <RecordTimeline live={live} records={records} />;
}

/**
 * The records the plots mark, prompts among them. Spread rather than named,
 * so this view passes them whether or not the grid it is built against
 * declares the prop yet.
 */
function plotMarkers(records: HubRecord[]): { records: HubRecord[] } {
  return { records };
}

/** Columns are the union of the row keys in first-appearance order, so a row
    that omits a field does not reorder or drop a column. */
function tableColumns(rows: DocumentViewTableRow[]): string[] {
  const columns: string[] = [];
  const seen = new Set<string>();
  for (const row of rows) {
    for (const column of Object.keys(row.cells)) {
      if (seen.has(column)) continue;
      seen.add(column);
      columns.push(column);
    }
  }
  return columns;
}

function reportPlotStates(documentPlots: DocumentViewPlot[], reportPlots: Record<string, PlotPayload>): LivePlotStateMap {
  return Object.fromEntries(
    documentPlots.flatMap((plot) => {
      const payload = reportPlots[plot.plot_id];
      if (!payload) return [];
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
