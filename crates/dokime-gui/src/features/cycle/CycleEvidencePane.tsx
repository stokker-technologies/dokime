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

import { Button } from "@/components/ui/button";
import type { HubLiveState, RecordScope, TestCyclePlan } from "@/lib/types";
import { DocumentView, useDocumentSurface } from "./DocumentView";
import { documentWindowElapsedUnit, FrozenDocumentConsole } from "./DocumentViewPresentation";
import { BUSY_NOTICE, evidencePaneState } from "./document-query-model";
import { PlotWindowControl, usePlotWindow } from "./PlotWindowSlider";
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

  return (
    <div className="grid min-h-0">
      <div className="min-h-0 overflow-y-auto px-4 py-4">
        {pane.kind === "failed" ? (
          <PaneNotice
            action={{ label: "Try Again", onClick: () => void documentQuery.refetch() }}
            message={pane.message}
            title="Evidence Unavailable"
          />
        ) : pane.kind === "busy" ? (
          <PaneNotice message={BUSY_NOTICE} title="Loading" />
        ) : pane.kind === "loading" ? (
          <PaneNotice message="Reading the document for this selection." title="Loading" />
        ) : (
          <div className="grid gap-6">
            {pane.staleReason ? (
              <p className="text-xs text-muted-foreground">
                Showing the last evidence read. {pane.staleReason}
              </p>
            ) : null}
            <DocumentView
              plotWindowControl={
                <PlotWindowControl
                  elapsedUnit={documentWindowElapsedUnit(surface.view.sections)}
                  live={live}
                  window={plotWindow}
                />
              }
              surface={surface}
            />
            <FrozenDocumentConsole surface={surface} />
          </div>
        )}
      </div>
    </div>
  );
}

/* === Private Components === */

function PaneNotice({
  action,
  message,
  title,
}: {
  action?: { label: string; onClick: () => void };
  message: string;
  title: string;
}) {
  return (
    <div className="grid justify-items-start gap-2 py-8">
      <p className="font-heading text-sm font-semibold">{title}</p>
      <p className="max-w-prose text-sm text-muted-foreground">{message}</p>
      {action ? (
        <Button onClick={action.onClick} size="sm" variant="outline">
          {action.label}
        </Button>
      ) : null}
    </div>
  );
}

/* === Private Functions === */
