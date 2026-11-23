// Dokime — SPDX-License-Identifier: Apache-2.0
import type { DocumentBlockChrome } from "../../components/document/DocumentViewRenderer.svelte";
import type { DocumentWorkspaceView } from "../../components/document/DocumentWorkspacePresentation.svelte";
import { isDocumentSurfaceCard } from "../../components/document/document-surface-policy";
import type { DocumentViewCard, DocumentViewSection } from "../../lib/types";
import { elapsedColumnUnit } from "../../plots/axis";
import type { DocumentSurface } from "./document-view-model";
import type { PlotWindow } from "./plot-window-state";

export type DocumentViewProps = {
  surface: DocumentSurface;
  cardFilter?: (card: DocumentViewCard) => boolean;
  cardChrome?: (card: DocumentViewCard) => DocumentBlockChrome;
  onRetry?: (cardId?: string) => void;
};

/** A Frozen Surface Draws Its Captured Plot Payloads. */
export function documentWorkspace({ surface, cardFilter, cardChrome }: DocumentViewProps): DocumentWorkspaceView {
  const { shared, view } = surface;
  return {
    ...shared,
    cardChrome,
    cardFilter,
    renderMode: "report",
    reportPlots: view.reportPayloads?.plots ?? {},
    reportTelemetry: view.plotContextTelemetry,
  };
}

/** The First Section With Windowed Plots Owns Their Window Control. */
export function documentSectionWindow(sections: DocumentViewSection[], window: PlotWindow) {
  const sectionId = sections.find((section) => documentHasWindowedCards([section]))?.metadata.identifier;
  return sectionId ? { sectionId, elapsedUnit: documentWindowElapsedUnit(sections), window } : undefined;
}

/** A Shared Elapsed Unit Is Available Only When Every Windowed Time Axis Agrees. */
export function documentWindowElapsedUnit(sections: DocumentViewSection[]): string | null {
  const units = new Set<string | null>();
  for (const section of sections) {
    for (const card of section.cards) {
      if (!isDocumentSurfaceCard(card) || card.kind !== "plots") {
        continue;
      }
      for (const plot of card.plots) {
        if (plot.plot_type === "time_series") {
          units.add(plot.x_format ? null : elapsedColumnUnit(plot.x));
        }
      }
    }
  }
  return units.size === 1 ? ([...units][0] ?? null) : null;
}

export function documentHasWindowedCards(sections: DocumentViewSection[]): boolean {
  return sections.some((section) => section.cards.some((card) => isDocumentSurfaceCard(card) && card.kind === "plots"));
}
