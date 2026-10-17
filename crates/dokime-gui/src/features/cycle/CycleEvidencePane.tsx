/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The cycle document: what the cycle itself holds, under the cycle cockpit.
 *
 * Shown when the cycle is selected, or nothing is. Records, telemetry, notes,
 * health, artifacts and plots all arrive through `useDocumentSurface` and
 * `DocumentView`; nothing is assembled a second time here. The plot window sits
 * in the header of the plots it scales, and Focus is in the page bar. A
 * selected execution or case is the execution page (`ExecutionSections`), not
 * this pane.
 *
 * With `live={false}` the same document renders from the snapshot the hooks
 * consult: the plot window is stated as a sentence instead of a slider.
 */

/* === External Imports === */

import { useMemo } from "react";

/* === Local Imports === */

import type { HubLiveState, RecordScope, TestCyclePlan } from "@/lib/types";
import { DocumentView, useDocumentSurface } from "./DocumentView";
import { CycleEvidenceView } from "./CycleEvidenceView";
import { evidencePaneState } from "./document-query-model";
import { usePlotWindow } from "./PlotWindowSlider";
import { useCycleDocumentQuery } from "./server-state";

/* === Types === */

export type CycleEvidencePaneProps = {
  cycleId: string;
  plan: TestCyclePlan | null;
  snapshot: HubLiveState | null;
  /** False in the frozen report: no slider, no live plot window. */
  live?: boolean;
};

/* === Components === */

export function CycleEvidencePane({
  cycleId,
  live = true,
  plan,
  snapshot,
}: CycleEvidencePaneProps) {
  const plotWindow = usePlotWindow();
  const scope = useMemo<RecordScope>(() => ({ test_cycle_id: cycleId }), [cycleId]);
  const documentQuery = useCycleDocumentQuery(cycleId, true);
  const document = documentQuery.data ?? null;
  const surface = useDocumentSurface({
    document,
    execution: null,
    live,
    snapshot,
    scope,
    parameters: { case: undefined, execution: undefined },
    noteElapsedStart: plan?.test_cycle.started_at ?? null,
    plotWindow: plotWindow.option,
    plotWindowOverridden: plotWindow.overridden,
    traceabilityExportName: null,
  });
  const pane = evidencePaneState({
    hasDocument: document !== null,
    error: documentQuery.error,
    failureReason: documentQuery.failureReason,
    pending: documentQuery.isPending,
  });

  return <CycleEvidenceView live={live} surface={surface} pane={pane} plotWindow={plotWindow}
    onRetry={() => void documentQuery.refetch()} documentView={DocumentView} />;
}

/* === Private Functions === */
