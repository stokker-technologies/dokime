/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { type ComponentType, type ReactNode, useCallback } from "react";

import { type DocumentBlockChrome, DocumentSectionActionContext } from "@/components/document/DocumentViewRenderer";
import { DocumentWorkspacePresentation, type DocumentWorkspaceViewProps } from "@/components/document/DocumentWorkspacePresentation";

import { isDocumentSurfaceCard } from "@/components/document/document-surface-policy";

import type { DocumentViewCard, DocumentViewSection } from "@/lib/types";

import { elapsedColumnUnit } from "@/plots/axis";
import { FrozenConsole } from "@/shell/ConsoleDrawerView";
import type { DocumentSurface } from "./document-view-model";

export type DocumentViewProps = {
  workspace?: ComponentType<DocumentWorkspaceViewProps>;
  surface: DocumentSurface;
  /** The plot window control, placed in the header of the first plot section. */
  plotWindowControl?: ReactNode;
  /** The cards this view shows; absent shows every main-region card. */
  cardFilter?: (card: DocumentViewCard) => boolean;
  /** The frame each card gets; absent keeps the renderer's default. */
  cardChrome?: (card: DocumentViewCard) => DocumentBlockChrome;
};

export function DocumentViewPresentation({
  workspace: Workspace = DocumentWorkspacePresentation,
  surface,
  plotWindowControl = null,
  cardFilter,
  cardChrome,
}: DocumentViewProps) {
  const { options, shared, view } = surface;
  /* The plot window belongs to the plots it scales: the first section that
     holds a windowed card carries it in its header. */
  const windowedSectionId = view.sections.find((section) => documentHasWindowedCards([section]))?.metadata.identifier ?? null;
  const sectionAction = useCallback(
    (sectionId: string) => (sectionId === windowedSectionId ? plotWindowControl : null),
    [plotWindowControl, windowedSectionId],
  );
  const shown = { ...shared, cardChrome, cardFilter };

  /* The frozen branch is the same surface reading embedded payloads instead of
     live telemetry. `DocumentWorkspaceView` discriminates on `renderMode`, so
     the two payload sets can never be confused for each other. */
  if (!options.live) {
    return (
      <DocumentSectionActionContext.Provider value={sectionAction}>
        <Workspace
          {...shown}
          renderMode="report"
          reportPlots={view.reportPayloads?.plots ?? {}}
          reportTelemetry={view.plotContextTelemetry}
        />
      </DocumentSectionActionContext.Provider>
    );
  }

  return (
    <DocumentSectionActionContext.Provider value={sectionAction}>
      <Workspace
        {...shown}
        plotContextTelemetry={view.plotContextTelemetry}
        plotResetKey={options.plotWindow.label}
        plotWindowS={options.plotWindowOverridden ? options.plotWindow.value : undefined}
        renderMode="live"
        scope={options.scope}
      />
    </DocumentSectionActionContext.Provider>
  );
}

/** The frozen console a report prints after the document it belongs to. */
export function FrozenDocumentConsole({ surface }: { surface: DocumentSurface }) {
  return surface.options.live ? null : <FrozenConsole records={surface.view.records} />;
}

/**
 * The one elapsed unit every windowed time-series card in `sections` shares,
 * or `null` when the window is wall time.
 *
 * One toolbar carries one window control, so the control can only say
 * "elapsed" when every plot under it agrees — both that its axis is elapsed
 * and about the unit it counts in. A document that mixes an elapsed trace with
 * a wall-time one keeps the plain "Plot Window", which is true of both.
 */
export function documentWindowElapsedUnit(sections: DocumentViewSection[]): string | null {
  const units = new Set<string | null>();
  for (const section of sections) {
    for (const card of section.cards) {
      if (!isDocumentSurfaceCard(card) || card.kind !== "plots") continue;
      for (const plot of card.plots) {
        if (plot.plot_type !== "time_series") continue;
        units.add(plot.x_format ? null : elapsedColumnUnit(plot.x));
      }
    }
  }
  if (units.size !== 1) return null;
  return [...units][0] ?? null;
}

/** Does any visible card in `sections` honour the live plot window? */
export function documentHasWindowedCards(sections: DocumentViewSection[]): boolean {
  return sections.some((section) => section.cards.some((card) => isDocumentSurfaceCard(card) && card.kind === "plots"));
}
