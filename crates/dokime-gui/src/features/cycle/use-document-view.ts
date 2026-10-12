/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* One document selects its evidence through ordinary query hooks. The hooks
 * read live gRPC results or the frozen report context; presentation derivation
 * and scope filtering are shared. Only card dependencies open backend reads.
 */

/* === External Imports === */

import { useMemo } from "react";

/* === Local Imports === */

import { isDocumentSurfaceCard } from "@/components/document/document-surface-policy";
import { orderedDocumentSections } from "@/lib/document-layout";
import { scopeVisibleIn } from "@/lib/scope";
import { elapsedLabel } from "@/lib/time-format";
import type { DocumentViewPayload, HubLiveState, RecordScope, TestExecution } from "@/lib/types";
import {
  buildDocumentBoxSummarySections,
  buildDocumentHistogramSections,
  mergeParameterContexts,
  plotContextTelemetrySnapshot,
  uniqueArtifacts,
} from "./model-document";

import {
  useExecutionFilesQuery,
  useHealthQuery,
  useNotesQuery,
  useRecordsQuery,
  useTraceabilityQuery,
} from "./server-state";
import { useFrozenPlotPayloads } from "./snapshot-source";

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
  /** The operator's plot window, when they overrode the authored one. */
  windowS: number | null | undefined;
};

/* === Hooks === */

export function useDocumentView({
  document,
  liveSnapshot,
  noteElapsedStart,
  execution,
  scope,
  live,
  parameters,
  windowS,
}: DocumentViewOptions) {
  const sections = useMemo(() => orderedDocumentSections(document), [document]);
  const mainCards = useMemo(
    () => sections.flatMap((section) => section.cards).filter(isDocumentSurfaceCard),
    [sections],
  );
  const kinds = new Set(mainCards.map((card) => card.kind));
  const filesQuery = useExecutionFilesQuery(
    scope.test_execution_id ?? null,
    kinds.has("artifacts") || kinds.has("media"),
  );
  const notesQuery = useNotesQuery(kinds.has("notes"), scope);
  const healthQuery = useHealthQuery(kinds.has("health"), scope);
  // Frozen reports include the selected execution's console after its document.
  const recordsQuery = useRecordsQuery(kinds.has("records") || !live, scope);
  const traceabilityQuery = useTraceabilityQuery(kinds.has("traceability"), scope);
  const reportPayloads = useFrozenPlotPayloads(scope.test_execution_id ?? null);
  const artifacts = useMemo(() => uniqueArtifacts(filesQuery.data ?? []), [filesQuery.data]);
  const notes = useMemo(
    () => (notesQuery.data ?? []).filter((note) => scopeVisibleIn(note.scope, scope)),
    [notesQuery.data, scope],
  );
  const health = useMemo(
    () => (healthQuery.data ?? []).filter((item) => scopeVisibleIn(item.scope, scope)),
    [healthQuery.data, scope],
  );
  const records = useMemo(
    () => (recordsQuery.data ?? []).filter((record) => scopeVisibleIn(record.scope, scope)),
    [recordsQuery.data, scope],
  );
  const steps = execution?.steps;
  const stepRecords = useMemo(
    () => (steps ?? []).filter((item) => item.record_type === "step"),
    [steps],
  );
  const criterionRecords = useMemo(
    () => (steps ?? []).filter((item) => item.record_type === "criterion"),
    [steps],
  );
  const streams = useMemo(
    () => (liveSnapshot ? Object.values(liveSnapshot.telemetry) : []),
    [liveSnapshot],
  );
  const stampedNotes = useMemo(() => {
    if (!noteElapsedStart) {
      return notes;
    }
    return notes.map((note) => {
      const noteTime = Date.parse(note.timestamp);
      return {
        ...note,
        elapsed_since_start_label: Number.isFinite(noteTime)
          ? elapsedLabel(noteElapsedStart, noteTime)
          : null,
      };
    });
  }, [noteElapsedStart, notes]);
  const plots = useMemo(
    () => mainCards.filter((block) => block.kind === "plots").flatMap((block) => block.plots),
    [mainCards],
  );
  const plotContextTelemetry = useMemo(
    () => plotContextTelemetrySnapshot(liveSnapshot, plots),
    [liveSnapshot, plots],
  );
  const histogramCards = useMemo(
    () => mainCards.filter((block) => block.kind === "histograms"),
    [mainCards],
  );
  const boxSummaryCards = useMemo(
    () => mainCards.filter((block) => block.kind === "box_summaries"),
    [mainCards],
  );
  const parameterContext = useMemo(
    () => mergeParameterContexts(parameters.case, parameters.execution),
    [parameters.case, parameters.execution],
  );
  const histogramSections = useMemo(
    () =>
      buildDocumentHistogramSections({
        cards: histogramCards,
        snapshot: liveSnapshot,
        windowS,
      }),
    [histogramCards, liveSnapshot, windowS],
  );
  const boxSummarySections = useMemo(
    () =>
      buildDocumentBoxSummarySections({
        cards: boxSummaryCards,
        snapshot: liveSnapshot,
        windowS,
      }),
    [boxSummaryCards, liveSnapshot, windowS],
  );

  return {
    artifacts,
    criterionRecords,
    health,
    records,
    reportPayloads,
    stepRecords,
    streams,
    traceability: traceabilityQuery.data ?? null,
    boxSummarySections,
    histogramSections,
    notes: stampedNotes,
    parameterContext,
    plotContextTelemetry,
    sections,
  };
}
