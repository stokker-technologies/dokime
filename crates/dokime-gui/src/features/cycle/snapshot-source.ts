/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/** Captured evidence passed directly to shared presentation by the standalone report. */

/* === Local Imports === */

import type {
  ArtifactRecord,
  DocumentViewPayload,
  HealthItem,
  HubLiveState,
  HubRecord,
  OperatorNote,
  PlotPayload,
  TestCyclePlan,
  TestExecution,
  TraceabilitySnapshot,
} from "@/lib/types";

/* === Types === */

/** The captured plot payloads a frozen document is drawn from. */
export type FrozenPlotPayloads = {
  plots: Record<string, PlotPayload>;
};

/**
 * One cycle, frozen. Every member is already a GUI projection type: the
 * snapshot boundary is crossed once, where the source is built, never inside a
 * component.
 */
export type CycleSnapshotSource = {
  /** ISO timestamp the report was generated at, for the status bar. */
  generatedAt: string | null;
  plan: TestCyclePlan | null;
  executions: TestExecution[];
  runtimeShell: HubLiveState | null;
  records: HubRecord[];
  notes: OperatorNote[];
  health: HealthItem[];
  artifacts: ArtifactRecord[];
  /** The artifacts one attempt produced, as the live execution-file read returns them. */
  executionFiles: (executionId: string | null) => ArtifactRecord[];
  /** The selected execution's document, or the cycle's own for `null`. */
  document: (executionId: string | null) => DocumentViewPayload | null;
  plotPayloads: (executionId: string | null) => FrozenPlotPayloads;
  traceability: (executionId: string | null) => TraceabilitySnapshot | null;
  /** Notices about evidence interpretation in the combined report. */
  warnings?: string[];
};
