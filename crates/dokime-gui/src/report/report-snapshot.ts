/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The embedded payload, read as what the workspace reads.
 *
 * The generator already normalised the payload: notes and health once per
 * subject, artifacts with their download target, steps under their execution.
 * This only indexes it by execution and maps the captured documents, so the
 * workspace, the sidebar, the evidence pane and the document surface run as the
 * application's own code with `live={false}`.
 */

import type { CycleSnapshotSource, FrozenPlotPayloads } from "@/features/cycle/snapshot-source";
import { mapDocumentCapture } from "@/lib/document-normalizer";
import type { ReportCaseRunPayload, ReportPayload } from "@/lib/report-types";
import type { DocumentViewPayload, HubLiveState } from "@/lib/types";

const NO_PLOT_PAYLOADS: FrozenPlotPayloads = { plots: {} };

/** Revisions describe what has changed since a frame; nothing has. */
const FROZEN_REVISIONS = {
  artifacts: 0,
  contracts: 0,
  cycles: 0,
  document: 0,
  executionFiles: 0,
  health: 0,
  notes: 0,
  prompts: 0,
  records: 0,
  result_summary: 0,
  run: 0,
  telemetry: {},
  traceability: 0,
};

export function cycleSnapshotSource(payload: ReportPayload): CycleSnapshotSource {
  const caseRuns = new Map<string, ReportCaseRunPayload>();
  for (const caseRun of payload.case_runs) {
    if (caseRun.test_execution_id) caseRuns.set(caseRun.test_execution_id, caseRun);
  }

  const snapshot = payload.snapshot ?? null;
  const artifacts = snapshot?.artifacts ?? [];
  const documents = new Map<string, DocumentViewPayload | null>();
  for (const [executionId, caseRun] of caseRuns) {
    documents.set(executionId, materialize(caseRun.documents.default));
  }
  const cycleDocument = materialize(payload.documents.default);

  return {
    artifacts,
    document: (executionId) => (executionId === null ? cycleDocument : (documents.get(executionId) ?? null)),
    executionFiles: (executionId) =>
      executionId === null ? [] : artifacts.filter((artifact) => artifact.scope?.test_execution_id === executionId),
    executions: payload.test_executions,
    generatedAt: payload.generated_at ?? null,
    health: snapshot?.health ?? [],
    notes: snapshot?.notes ?? [],
    plan: payload.test_cycle_plan ?? null,
    plotPayloads: (executionId) => {
      if (executionId === null) {
        return { plots: payload.plots };
      }
      const caseRun = caseRuns.get(executionId);
      return caseRun ? { plots: caseRun.plots } : NO_PLOT_PAYLOADS;
    },
    records: snapshot?.records ?? [],
    runtimeShell: runtimeShell(snapshot),
    traceability: (executionId) =>
      (executionId === null ? payload.snapshot?.traceability : caseRuns.get(executionId)?.traceability) ?? null,
  };
}

function materialize(document: Parameters<typeof mapDocumentCapture>[0] | undefined): DocumentViewPayload | null {
  return document ? mapDocumentCapture(document) : null;
}

/** Only the telemetry matters downstream; the rest is the frame's own shape. */
function runtimeShell(snapshot: ReportPayload["snapshot"]): HubLiveState | null {
  if (!snapshot) return null;
  const { contracts: _contracts, document: _document, ...rest } = snapshot;
  return { ...rest, revisions: FROZEN_REVISIONS };
}
