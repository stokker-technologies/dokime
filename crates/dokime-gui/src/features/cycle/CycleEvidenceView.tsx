/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The cycle viewer's column: the lead blocks the viewer composes — the shared
 * header and the Control card — then the cycle's own prompt queue, the cycle
 * document, and the shared Records section, in the order the execution viewer
 * uses.
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

import type { ComponentType, ReactNode } from "react";
import { isDocumentSurfaceCard } from "@/components/document/document-surface-policy";
import { Button } from "@/components/ui/button";
import type { DocumentViewCard, PromptRequest } from "@/lib/types";
import {
  DocumentViewPresentation,
  type DocumentViewProps,
  documentWindowElapsedUnit,
  FrozenDocumentConsole,
} from "./DocumentViewPresentation";
import { BUSY_NOTICE, type evidencePaneState } from "./document-query-model";
import type { DocumentSurface } from "./document-view-model";
import { PlotWindowControl, type usePlotWindow } from "./PlotWindowSlider";
import { PromptQueue } from "./PromptQueue";
import { RecordsSection } from "./RecordsSection";

/* === Types === */

export type CycleEvidenceViewProps = {
  live: boolean;
  surface: DocumentSurface;
  pane: ReturnType<typeof evidencePaneState>;
  plotWindow: ReturnType<typeof usePlotWindow>;
  onRetry?: () => void;
  documentView?: ComponentType<DocumentViewProps>;
  /** The header and Control card the viewer composes above the document. */
  lead?: ReactNode;
  /** The prompts the cycle itself asked, which no execution owns; none in a file. */
  prompts?: PromptRequest[];
};

/* === Components === */

export function CycleEvidenceView({
  live,
  surface,
  pane,
  plotWindow,
  onRetry,
  documentView: DocumentContent = DocumentViewPresentation,
  lead,
  prompts = [],
}: CycleEvidenceViewProps) {
  const cycleRecords = surface.view.records.filter((record) => !record.scope?.test_execution_id);
  return (
    <div className="grid min-h-0">
      <div className="grid min-h-0 content-start gap-4 overflow-y-auto px-[var(--shell-inset)] py-4">
        {lead}
        <PromptQueue prompts={prompts} records={cycleRecords} />
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
              cardFilter={isDocumentCard}
              plotWindowControl={
                <PlotWindowControl elapsedUnit={documentWindowElapsedUnit(surface.view.sections)} live={live} window={plotWindow} />
              }
              surface={surface}
            />
            <RecordsSection live={live} records={surface.view.records} />
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

/** The document's cards, Records excepted: the shared Records section closes the column. */
function isDocumentCard(card: DocumentViewCard): boolean {
  return isDocumentSurfaceCard(card) && card.kind !== "records";
}
