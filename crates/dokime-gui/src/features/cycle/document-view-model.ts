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
import type { EvidenceScope } from "@generated/host/model";
import type { Wire } from "@/lib/rpc/canonical";
import { isDocumentSurfaceCard } from "../../components/document/document-surface-policy";
import { localArtifactHref } from "../../lib/artifact-tree";
import { orderedDocumentSections } from "../../lib/document-layout";
import { scopeVisibleIn } from "../../lib/scope";
import { elapsedLabel } from "../../lib/time-format";
import type { TraceabilityExportName } from "../../lib/traceability";
import type {
  ArtifactRecord,
  DocumentViewCard,
  DocumentViewPayload,
  DocumentViewSection,
  HealthItem,
  HubLiveState,
  HubRecord,
  OperatorNote,
  TestExecution,
  TraceabilitySnapshot,
} from "@/lib/types";
import { mergeParameterContexts, plotContextTelemetrySnapshot, uniqueArtifacts } from "./model-document";
import type { PlotWindow } from "./plot-window-state";
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
  scope: Wire<EvidenceScope>;
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
  scope: Wire<EvidenceScope>;
  parameters: DocumentViewOptions["parameters"];
  noteElapsedStart: string | null | undefined;
  /** The Live Plot Window; A Frozen Report Has None. */
  plotWindow?: PlotWindow;
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
  return {
    artifacts: uniqueArtifacts(evidence.artifacts),
    criterionRecords: executionRecords(execution, "criterion"),
    health: evidence.health.filter((item) => scopeVisibleIn(item.scope, scope)),
    records: evidence.records.filter((record) => scopeVisibleIn(record.scope, scope)),
    reportPayloads: evidence.reportPayloads,
    stepRecords: executionRecords(execution, "step"),
    streams: liveSnapshot ? Object.values(liveSnapshot.telemetry) : [],
    traceability: evidence.traceability,
    notes: scopedNotes(evidence.notes, scope, noteElapsedStart),
    parameterContext: mergeParameterContexts(parameters.case, parameters.execution),
    plotContextTelemetry: plotContextTelemetrySnapshot(liveSnapshot, surfacePlots(surfaceCards(sections))),
    sections,
  };
}

export function documentSurface(options: DocumentSurfaceOptions, view: ReturnType<typeof documentView>, nativeAvailable: boolean) {
  const shared = {
    ...surfaceArtifacts(view.artifacts, nativeAvailable),
    streams: view.streams,
    criterionRecords: view.criterionRecords,
    document: options.document,
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

/** The Main Cards A Surface Renders, In Section Order. */
export function surfaceCards(sections: DocumentViewSection[]): DocumentViewCard[] {
  return sections.flatMap((section) => section.cards).filter(isDocumentSurfaceCard);
}

export function surfacePlots(cards: DocumentViewCard[]) {
  return cards.filter((card) => card.kind === "plots").flatMap((card) => card.plots);
}

export function executionRecords(execution: TestExecution | null, type: "step" | "criterion") {
  return (execution?.steps ?? []).filter((item) => item.record_type === type);
}

/** Notes Visible In Scope, Labelled With Their Elapsed Time Since The Run Started. */
export function scopedNotes(notes: OperatorNote[], scope: Wire<EvidenceScope>, start: string | null | undefined): OperatorNote[] {
  const visible = notes.filter((note) => scopeVisibleIn(note.scope, scope));
  if (!start) {
    return visible;
  }
  return visible.map((note) => {
    const noteTime = Date.parse(note.timestamp);
    return { ...note, elapsed_since_start_label: Number.isFinite(noteTime) ? elapsedLabel(start, noteTime) : null };
  });
}

/** Linked Artifacts And The Media A Surface Can Open. */
export function surfaceArtifacts(records: ArtifactRecord[], nativeAvailable: boolean) {
  const artifacts = records.map((artifact) => {
    const localHref = localArtifactHref(artifact.path);
    return {
      ...artifact,
      href: artifact.href ?? localHref,
      downloadHref: artifact.downloadHref ?? localHref,
    };
  });
  const mediaArtifacts = records.flatMap((artifact) => {
    const href = artifact.href ?? artifact.downloadHref ?? localArtifactHref(artifact.path);
    if (!href && !nativeAvailable) {
      return [];
    }
    return [
      {
        ...artifact,
        href: href ?? null,
        downloadHref: artifact.downloadHref ?? href ?? null,
      },
    ];
  });
  return { artifacts, mediaArtifacts };
}

/* === Private Functions === */

/* === Entry Point === */
