/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Pure Document Evidence Selection And Shared Surface Projection.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import { isDocumentSurfaceCard } from "@/components/document/document-surface-policy";
import { localArtifactHref } from "@/lib/artifact-tree";
import { orderedDocumentSections } from "@/lib/document-layout";
import { scopeVisibleIn } from "@/lib/scope";
import { elapsedLabel } from "@/lib/time-format";
import type { TraceabilityExportName } from "@/lib/traceability";
import type {
  ArtifactRecord,
  DocumentViewPayload,
  HealthItem,
  HubLiveState,
  HubRecord,
  OperatorNote,
  RecordScope,
  TestExecution,
  TraceabilitySnapshot,
} from "@/lib/types";
import { mergeParameterContexts, plotContextTelemetrySnapshot, uniqueArtifacts } from "./model-document";
import type { PlotWindowOption } from "./PlotWindowSlider";
import type { FrozenPlotPayloads } from "./snapshot-source";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */
export type DocumentViewOptions = {
  document: DocumentViewPayload | null;
  /** The live runtime, or `null` while a recorded cycle is on screen. */
  liveSnapshot: HubLiveState | null;
  /** Start of the run a note's elapsed label counts from. */
  noteElapsedStart: string | null | undefined;
  execution: TestExecution | null;
  scope: RecordScope;
  live: boolean;
  /** The parameter scopes a plot resolves an expression against. */
  parameters: {
    case: Record<string, unknown> | null | undefined;
    execution: Record<string, unknown> | null | undefined;
  };
};

export type DocumentSurfaceOptions = {
  document: DocumentViewPayload | null | undefined;
  execution: TestExecution | null;
  live: boolean;
  snapshot: HubLiveState | null;
  scope: RecordScope;
  parameters: DocumentViewOptions["parameters"];
  noteElapsedStart: string | null | undefined;
  plotWindow: PlotWindowOption;
  traceabilityExportName?: TraceabilityExportName | null;
};

export type DocumentEvidence = {
  artifacts: ArtifactRecord[];
  notes: OperatorNote[];
  health: HealthItem[];
  records: HubRecord[];
  traceability: TraceabilitySnapshot | null;
  reportPayloads: FrozenPlotPayloads | null;
};

export type DocumentSurface = ReturnType<typeof documentSurface>;

/* === Classes === */

/* === Public Functions === */
export function documentView(options: DocumentViewOptions, evidence: DocumentEvidence) {
  const { document, liveSnapshot, noteElapsedStart, execution, scope, parameters } = options;
  const sections = orderedDocumentSections(document);
  const mainCards = sections.flatMap((section) => section.cards).filter(isDocumentSurfaceCard);
  const artifacts = uniqueArtifacts(evidence.artifacts);
  const notes = evidence.notes.filter((note) => scopeVisibleIn(note.scope, scope));
  const health = evidence.health.filter((item) => scopeVisibleIn(item.scope, scope));
  const records = evidence.records.filter((record) => scopeVisibleIn(record.scope, scope));
  const steps = execution?.steps ?? [];
  const stepRecords = steps.filter((item) => item.record_type === "step");
  const criterionRecords = steps.filter((item) => item.record_type === "criterion");
  const streams = liveSnapshot ? Object.values(liveSnapshot.telemetry) : [];
  const stampedNotes = noteElapsedStart
    ? notes.map((note) => {
        const noteTime = Date.parse(note.timestamp);
        return {
          ...note,
          elapsed_since_start_label: Number.isFinite(noteTime) ? elapsedLabel(noteElapsedStart, noteTime) : null,
        };
      })
    : notes;
  const plots = mainCards.filter((block) => block.kind === "plots").flatMap((block) => block.plots);
  return {
    artifacts,
    criterionRecords,
    health,
    records,
    reportPayloads: evidence.reportPayloads,
    stepRecords,
    streams,
    traceability: evidence.traceability,
    notes: stampedNotes,
    parameterContext: mergeParameterContexts(parameters.case, parameters.execution),
    plotContextTelemetry: plotContextTelemetrySnapshot(liveSnapshot, plots),
    sections,
  };
}

export function documentSurface(options: DocumentSurfaceOptions, view: ReturnType<typeof documentView>, nativeAvailable: boolean) {
  const artifacts = view.artifacts.map((artifact) => {
    const localHref = localArtifactHref(artifact.path);
    return {
      ...artifact,
      href: artifact.href ?? localHref,
      downloadHref: artifact.downloadHref ?? localHref,
    };
  });
  const mediaArtifacts = view.artifacts.flatMap((artifact) => {
    const href = artifact.href ?? artifact.downloadHref ?? localArtifactHref(artifact.path);
    if (!href && !nativeAvailable) return [];
    return [
      {
        ...artifact,
        href: href ?? null,
        downloadHref: artifact.downloadHref ?? href ?? null,
      },
    ];
  });

  const shared = {
    artifacts,
    streams: view.streams,
    criterionRecords: view.criterionRecords,
    document: options.document,
    mediaArtifacts,
    notes: view.notes,
    plotParameterContext: view.parameterContext,
    records: view.records,
    health: view.health,
    sections: view.sections,
    stepRecords: view.stepRecords,
    traceability: view.traceability,
    traceabilityExportName: options.traceabilityExportName,
  };
  return { options, shared, view };
}

/* === Private Functions === */

/* === Entry Point === */
