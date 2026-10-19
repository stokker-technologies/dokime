/* ***
 * Dokime
 * SPDX-License-Identifier: Apache-2.0
 * ***
 */
import { useMemo } from "react";
import { isDocumentSurfaceCard } from "@/components/document/document-surface-policy";
import { orderedDocumentSections } from "@/lib/document-layout";
import { type DocumentViewOptions, documentView } from "./document-view-model";
import { useExecutionFilesQuery, useHealthQuery, useNotesQuery, useRecordsQuery, useTraceabilityQuery } from "./server-state";
import { useFrozenPlotPayloads } from "./snapshot-source";

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
  const recordsQuery = useRecordsQuery(kinds.has("records") || !live, scope);
  const traceabilityQuery = useTraceabilityQuery(kinds.has("traceability"), scope);
  const reportPayloads = useFrozenPlotPayloads(scope.test_execution_id ?? null);
  return useMemo(
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
          records: recordsQuery.data ?? [],
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
      recordsQuery.data,
      traceabilityQuery.data,
      reportPayloads,
    ],
  );
}
