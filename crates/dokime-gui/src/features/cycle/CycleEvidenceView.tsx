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
 * With `live={false}` the same document renders from captured data: the plot window is stated as a sentence instead of a slider.
 */

/* === External Imports === */

import type { ComponentType } from "react";
import { Button } from "@/components/ui/button";
import type { DocumentSurface } from "./document-view-model";
import {
  DocumentViewPresentation,
  type DocumentViewProps,
  documentWindowElapsedUnit,
  FrozenDocumentConsole,
} from "./DocumentViewPresentation";
import { BUSY_NOTICE, evidencePaneState } from "./document-query-model";
import { PlotWindowControl, usePlotWindow } from "./PlotWindowSlider";

/* === Types === */

export type CycleEvidenceViewProps = {
  live: boolean;
  surface: DocumentSurface;
  pane: ReturnType<typeof evidencePaneState>;
  plotWindow: ReturnType<typeof usePlotWindow>;
  onRetry?: () => void;
  documentView?: ComponentType<DocumentViewProps>;
};

/* === Components === */

export function CycleEvidenceView({
  live,
  surface,
  pane,
  plotWindow,
  onRetry,
  documentView: DocumentContent = DocumentViewPresentation,
}: CycleEvidenceViewProps) {
  return (
    <div className="grid min-h-0">
      <div className="min-h-0 overflow-y-auto px-4 py-4">
        {pane.kind === "failed" ? (
          <PaneNotice
            action={onRetry ? { label: "Try Again", onClick: onRetry } : undefined}
            message={pane.message}
            title="Evidence Unavailable"
          />
        ) : pane.kind === "busy" ? (
          <PaneNotice message={BUSY_NOTICE} title="Loading" />
        ) : pane.kind === "loading" ? (
          <PaneNotice message="Reading the document for this selection." title="Loading" />
        ) : (
          <div className="grid gap-6">
            {pane.staleReason ? <p className="text-xs text-muted-foreground">Showing the last evidence read. {pane.staleReason}</p> : null}
            <DocumentContent
              plotWindowControl={
                <PlotWindowControl elapsedUnit={documentWindowElapsedUnit(surface.view.sections)} live={live} window={plotWindow} />
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

function PaneNotice({ action, message, title }: { action?: { label: string; onClick: () => void }; message: string; title: string }) {
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
