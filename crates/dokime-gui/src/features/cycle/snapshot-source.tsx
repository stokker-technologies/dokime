/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Captured evidence for shared cycle presentation and recorded-data adapters.
 * The standalone report passes this source directly into presentation views;
 * application query adapters can also use it when displaying captured data.
 */

/* === External Imports === */

import type { UseQueryResult } from "@tanstack/react-query";
import { createContext, type ReactNode, useContext } from "react";

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
  /** What the report left out or cut to fit its budget, and why. */
};

/* === Constants === */

const CycleSnapshotContext = createContext<CycleSnapshotSource | null>(null);

/* === Components === */

/**
 * Puts one frozen cycle in reach of the workspace's hooks. Present exactly
 * when the surface renders with `live={false}`.
 */
export function CycleSnapshotProvider({ children, source }: { children: ReactNode; source: CycleSnapshotSource }) {
  return <CycleSnapshotContext.Provider value={source}>{children}</CycleSnapshotContext.Provider>;
}

/* === Hooks === */

/** The frozen cycle, or null in the live application. */
export function useCycleSnapshotSource(): CycleSnapshotSource | null {
  return useContext(CycleSnapshotContext);
}

/** The captured plot payloads a frozen document is drawn from. */
export function useFrozenPlotPayloads(executionId: string | null): FrozenPlotPayloads | null {
  const snapshot = useContext(CycleSnapshotContext);
  return snapshot ? snapshot.plotPayloads(executionId) : null;
}

/* === Public Functions === */

/**
 * A settled query result carrying a value that was never fetched.
 *
 * The cast is deliberate and lives here alone: `UseQueryResult` has two dozen
 * status members that no consumer of these hooks reads, and widening every
 * hook's declared return type to some narrower structural type would change
 * the live contract to describe the frozen one. The shape below is the
 * success shape of a resolved query.
 */
export function frozenQuery<T>(data: T): UseQueryResult<T> {
  return {
    data,
    error: null,
    failureCount: 0,
    failureReason: null,
    fetchStatus: "idle",
    isError: false,
    isFetched: true,
    isFetching: false,
    isLoading: false,
    isPending: false,
    isRefetching: false,
    isStale: false,
    isSuccess: true,
    refetch: () => Promise.resolve(undefined as never),
    status: "success",
  } as unknown as UseQueryResult<T>;
}
