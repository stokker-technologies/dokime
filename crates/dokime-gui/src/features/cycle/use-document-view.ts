/* ***
 * Dokime
 * SPDX-License-Identifier: Apache-2.0
 * ***
 */
import { useMemo } from "react";
import { isDocumentSurfaceCard } from "@/components/document/document-surface-policy";
import { orderedDocumentSections } from "@/lib/document-layout";
import { scopeVisibleIn } from "@/lib/scope";
import type { HubRecord } from "@/lib/types";
import { type DocumentViewOptions, documentView } from "./document-view-model";
import { useExecutionFilesQuery, useHealthQuery, useNotesQuery, useRecordsQuery, useTraceabilityQuery } from "./server-state";
import { useFrozenPlotPayloads } from "./snapshot-source";

const EMPTY_RECORDS: HubRecord[] = [];

export function useDocumentView(options: DocumentViewOptions) {
  const { document, liveSnapshot, noteElapsedStart, execution, scope, live, parameters } = options;
  const caseParameters = parameters.case;
  const executionParameters = parameters.execution;
  const sections = useMemo(() => orderedDocumentSections(document), [document]);
  const kinds = new Set(
    sections
      .flatMap((section) => section.cards)
      .filter(isDocumentSurfaceCard)
      .map((card) => card.kind),
  );
  const filesQuery = useExecutionFilesQuery(scope.test_execution_id ?? null, kinds.has("artifacts") || kinds.has("media"));
  const notesQuery = useNotesQuery(kinds.has("notes"), scope);
  const healthQuery = useHealthQuery(kinds.has("health"), scope);
  /* Plots mark prompts and other records on their axes, so a document with
     plots reads its records too. */
  const recordsQuery = useRecordsQuery(kinds.has("records") || kinds.has("plots") || !live, scope);
  const traceabilityQuery = useTraceabilityQuery(kinds.has("traceability"), scope);
  const reportPayloads = useFrozenPlotPayloads(scope.test_execution_id ?? null);
  const view = useMemo(
    () =>
      documentView(
        {
          document,
          liveSnapshot,
          noteElapsedStart,
          execution,
          scope,
          live,
          parameters: { case: caseParameters, execution: executionParameters },
        },
        {
          artifacts: filesQuery.data ?? [],
          notes: notesQuery.data ?? [],
          health: healthQuery.data ?? [],
          records: EMPTY_RECORDS,
          traceability: traceabilityQuery.data ?? null,
          reportPayloads,
        },
      ),
    [
      document,
      liveSnapshot,
      noteElapsedStart,
      execution,
      scope,
      live,
      caseParameters,
      executionParameters,
      filesQuery.data,
      notesQuery.data,
      healthQuery.data,
      traceabilityQuery.data,
      reportPayloads,
    ],
  );
  const records = useMemo(
    () => (recordsQuery.data ?? EMPTY_RECORDS).filter((record) => scopeVisibleIn(record.scope, scope)),
    [recordsQuery.data, scope],
  );
  return useMemo(() => ({ ...view, records }), [view, records]);
}
