/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

import { type ReactNode, useCallback } from "react";

// Local Imports

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";

import {
  type DocumentBlockChrome,
  DocumentSectionActionContext,
} from "@/components/document/DocumentViewRenderer";
import { DocumentWorkspaceView } from "@/components/document/DocumentWorkspaceView";

import { isDocumentSurfaceCard } from "@/components/document/document-surface-policy";

import { localArtifactHref } from "@/lib/artifact-tree";

import type { TraceabilityExportName } from "@/lib/traceability";

import type {
  DocumentViewCard,
  DocumentViewPayload,
  DocumentViewSection,
  HubLiveState,
  RecordScope,
  TestExecution,
} from "@/lib/types";

import { elapsedColumnUnit } from "@/plots/axis";
import { ConsoleDrawer } from "@/shell/ConsoleDrawer";
import type { PlotWindowOption } from "./PlotWindowSlider";
import { type DocumentViewOptions, useDocumentView } from "./use-document-view";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type DocumentSurfaceOptions = {
  document: DocumentViewPayload | null | undefined;
  execution: TestExecution | null;
  live: boolean;
  snapshot: HubLiveState | null;
  scope: RecordScope;
  parameters: DocumentViewOptions["parameters"];
  noteElapsedStart: string | null | undefined;
  plotWindow: PlotWindowOption;
  plotWindowOverridden: boolean;
  traceabilityExportName?: TraceabilityExportName | null;
};

/** One selection's evidence, resolved once for every surface that shows part of it. */
export type DocumentSurface = ReturnType<typeof useDocumentSurface>;

type DocumentViewProps = {
  surface: DocumentSurface;
  /** The plot window control, placed in the header of the first plot section. */
  plotWindowControl?: ReactNode;
  /** The cards this view shows; absent shows every main-region card. */
  cardFilter?: (card: DocumentViewCard) => boolean;
  /** The frame each card gets; absent keeps the renderer's default. */
  cardChrome?: (card: DocumentViewCard) => DocumentBlockChrome;
};

/* === Components === */

export function DocumentView({
  surface,
  plotWindowControl = null,
  cardFilter,
  cardChrome,
}: DocumentViewProps) {
  const { options, shared, view } = surface;
  /* The plot window belongs to the plots it scales: the first section that
     holds a windowed card carries it in its header. */
  const windowedSectionId =
    view.sections.find((section) => documentHasWindowedCards([section]))?.metadata.identifier ??
    null;
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
        <DocumentWorkspaceView
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
      <DocumentWorkspaceView
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
  return surface.options.live ? null : (
    <ConsoleDrawer live={false} records={surface.view.records} />
  );
}

/* === Hooks === */

/**
 * Reads one selection's evidence and resolves its artifact links, so the
 * cycle document and the execution page's sections share one set of reads.
 */
export function useDocumentSurface(options: DocumentSurfaceOptions) {
  const view = useDocumentView({
    document: options.document ?? null,
    execution: options.execution,
    live: options.live,
    liveSnapshot: options.snapshot,
    scope: options.scope,
    parameters: options.parameters,
    noteElapsedStart: options.noteElapsedStart,
  });
  const nativeSession = useNativeSessionOptional();
  const artifacts = view.artifacts.map((artifact) => {
    const localHref = localArtifactHref(artifact.path);
    return {
      ...artifact,
      href: artifact.href ?? localHref,
      downloadHref: artifact.downloadHref ?? localHref,
    };
  });
  const mediaArtifacts = view.artifacts.flatMap((artifact) => {
    const href = artifact.href ?? artifact.downloadHref ?? localArtifactHref(artifact.path);
    if (!href && !nativeSession) return [];
    return [
      {
        ...artifact,
        href: href ?? null,
        downloadHref: artifact.downloadHref ?? href ?? null,
      },
    ];
  });

  const shared = {
    artifacts,
    streams: view.streams,
    criterionRecords: view.criterionRecords,
    document: options.document,
    mediaArtifacts,
    notes: view.notes,
    plotParameterContext: view.parameterContext,
    records: view.records,
    health: view.health,
    sections: view.sections,
    stepRecords: view.stepRecords,
    traceability: view.traceability,
    traceabilityExportName: options.traceabilityExportName,
  };
  return { options, shared, view };
}

/* === Public Functions === */

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
  return sections.some((section) =>
    section.cards.some((card) => isDocumentSurfaceCard(card) && card.kind === "plots"),
  );
}

/* === Private Functions === */
