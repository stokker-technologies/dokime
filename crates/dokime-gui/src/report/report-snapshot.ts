/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The embedded payload, projected into what the workspace reads.
 *
 * This is the whole of the report's data work: one function, run once at boot,
 * that turns the generated `ReportPayload` into the same projection types the
 * `lib/api` adapters produce live. Everything downstream — the workspace, the
 * sidebar, the evidence pane, the document surface — is then the application's
 * own code with `live={false}`.
 */

/* === Local Imports === */

import type { CycleSnapshotSource, FrozenPlotPayloads } from "@/features/cycle/snapshot-source";
import { mapDocumentCapture } from "@/lib/document-normalizer";
import type { ReportCaseRunPayload, ReportPayload } from "@/lib/report-types";
import type { ArtifactRecord, DocumentViewPayload, HubLiveState } from "@/lib/types";
import { executionsWithSteps } from "@/report/report-steps";

/* === Constants === */

const NO_PLOT_PAYLOADS: FrozenPlotPayloads = { boxSummaries: {}, histograms: {}, plots: {} };

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

/* === Public Functions === */

export function cycleSnapshotSource(payload: ReportPayload): CycleSnapshotSource {
  const caseRuns = new Map<string, ReportCaseRunPayload>();
  for (const caseRun of payload.case_runs) {
    if (caseRun.test_execution_id) caseRuns.set(caseRun.test_execution_id, caseRun);
  }

  const notes = latestAnnotations(
    payload.snapshot?.notes ?? [],
    (note) => note.note_id,
    (note) => note.timestamp,
  );
  const health = latestAnnotations(
    payload.snapshot?.health ?? [],
    (item) => item.health_id,
    (item) => item.created_at,
  );
  const snapshot = payload.snapshot ? { ...payload.snapshot, notes, health } : null;
  const artifacts = reportArtifacts(payload);
  const documents = new Map<string, DocumentViewPayload | null>();
  for (const [executionId, caseRun] of caseRuns) {
    documents.set(executionId, materialize(caseRun.documents.default));
  }
  const cycleDocument = materialize(payload.documents.default);

  return {
    artifacts,
    document: (executionId) =>
      executionId === null ? cycleDocument : (documents.get(executionId) ?? null),
    executionFiles: (executionId) =>
      executionId === null
        ? []
        : artifacts.filter((artifact) => artifact.scope?.test_execution_id === executionId),
    executions: executionsWithSteps(payload),
    generatedAt: payload.generated_at ?? null,
    health,
    notes,
    plan: payload.test_cycle_plan ?? null,
    plotPayloads: (executionId) => {
      if (executionId === null) {
        return {
          boxSummaries: payload.box_summaries,
          histograms: payload.histograms,
          plots: payload.plots,
        };
      }
      const caseRun = caseRuns.get(executionId);
      return caseRun
        ? {
            boxSummaries: caseRun.box_summaries,
            histograms: caseRun.histograms,
            plots: caseRun.plots,
          }
        : NO_PLOT_PAYLOADS;
    },
    records: payload.snapshot?.records ?? [],
    runtimeShell: runtimeShell(snapshot),
    traceability: (executionId) =>
      (executionId === null
        ? payload.snapshot?.traceability
        : caseRuns.get(executionId)?.traceability) ?? null,
  };
}

/* === Private Functions === */

/** Cycle annotations may occur in several execution captures. Keep each identity
    once, using its latest observation rather than execution traversal order. */
function latestAnnotations<T>(
  values: readonly T[],
  identity: (value: T) => string,
  timestamp: (value: T) => string,
): T[] {
  const selected = new Map<string, T>();
  for (const value of values) {
    const key = identity(value);
    const previous = selected.get(key);
    if (!previous || Date.parse(timestamp(value)) >= Date.parse(timestamp(previous))) {
      selected.set(key, value);
    }
  }
  return [...selected.values()];
}

function materialize(
  document: Parameters<typeof mapDocumentCapture>[0] | undefined,
): DocumentViewPayload | null {
  return document ? mapDocumentCapture(document) : null;
}

/**
 * The artifacts the file can actually reach.
 *
 * The generator writes two lists: the records the run produced and the links
 * it wrote beside them. The links carry the href, the records carry the
 * description and the grouping, so an artifact is only complete once both are
 * merged — exactly as the old snapshot workspace merged them.
 */
function reportArtifacts(payload: ReportPayload): ArtifactRecord[] {
  const records = new Map(
    (payload.snapshot?.artifacts ?? []).map((artifact) => [artifact.artifact_id, artifact]),
  );
  return payload.artifact_links.map((link) => {
    const record = records.get(link.artifact_id);
    return {
      ...record,
      ...link,
      description: link.description ?? record?.description,
      downloadHref: link.href,
      group_id: link.group_id ?? record?.group_id,
      group_label: link.group_label ?? record?.group_label,
      size_bytes: link.size_bytes ?? record?.size_bytes,
    } as ArtifactRecord;
  });
}

/** Only the telemetry matters downstream; the rest is the frame's own shape. */
function runtimeShell(snapshot: ReportPayload["snapshot"]): HubLiveState | null {
  if (!snapshot) return null;
  const { contracts: _contracts, document: _document, ...rest } = snapshot;
  return { ...rest, revisions: FROZEN_REVISIONS };
}
