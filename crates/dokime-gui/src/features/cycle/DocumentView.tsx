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

// Local Imports

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";

import { DocumentWorkspaceView } from "@/components/document/DocumentWorkspaceView";

import { isDocumentSurfaceCard } from "@/components/document/document-surface-policy";

import { localArtifactHref } from "@/lib/artifact-tree";

import type {
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

type DocumentViewProps = {
  document: DocumentViewPayload | null | undefined;
  execution: TestExecution | null;
  live?: boolean;
  snapshot: HubLiveState | null;
  scope: RecordScope;
  parameters: DocumentViewOptions["parameters"];
  noteElapsedStart: string | null | undefined;
  plotWindow: PlotWindowOption;
  plotWindowOverridden: boolean;
};

/* === Components === */

export function DocumentView({
  document,
  execution,
  live = true,
  snapshot,
  scope,
  parameters,
  noteElapsedStart,
  plotWindow,
  plotWindowOverridden,
}: DocumentViewProps) {
  const view = useDocumentView({
    document: document ?? null,
    execution,
    live,
    liveSnapshot: snapshot,
    scope,
    parameters,
    noteElapsedStart,
    windowS: plotWindowOverridden ? plotWindow.value : undefined,
  });
  const {
    artifacts,
    criterionRecords,
    health,
    notes,
    records,
    reportPayloads,
    stepRecords,
    streams,
    traceability,
  } = view;
  const nativeSession = useNativeSessionOptional();
  const plotResetKey = plotWindow.label;
  const livePlotWindowS = plotWindowOverridden ? plotWindow.value : undefined;
  const localArtifacts = artifacts.map((artifact) => {
    const localHref = localArtifactHref(artifact.path);
    return {
      ...artifact,
      href: artifact.href ?? localHref,
      downloadHref: artifact.downloadHref ?? localHref,
    };
  });
  const resolvedMediaArtifacts = artifacts.flatMap((artifact) => {
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
    artifacts: localArtifacts,
    streams,
    criterionRecords,
    document,
    mediaArtifacts: resolvedMediaArtifacts,
    notes,
    plotParameterContext: view.parameterContext,
    records,
    health,
    sections: view.sections,
    stepRecords,
    traceability,
  };

  /* The frozen branch is the same surface reading embedded payloads instead of
     live telemetry. `DocumentWorkspaceView` discriminates on `renderMode`, so
     the two payload sets can never be confused for each other. */
  if (!live) {
    return (
      <>
        <DocumentWorkspaceView
          {...shared}
          renderMode="report"
          reportBoxSummaries={reportPayloads?.boxSummaries ?? {}}
          reportHistograms={reportPayloads?.histograms ?? {}}
          reportPlots={reportPayloads?.plots ?? {}}
          reportTelemetry={view.plotContextTelemetry}
        />
        <ConsoleDrawer live={false} records={records} />
      </>
    );
  }

  return (
    <DocumentWorkspaceView
      {...shared}
      boxSummarySections={view.boxSummarySections}
      histogramSections={view.histogramSections}
      plotContextTelemetry={view.plotContextTelemetry}
      plotResetKey={plotResetKey}
      plotWindowS={livePlotWindowS}
      renderMode="live"
      scope={scope}
    />
  );
}

/* === Hooks === */

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
    section.cards.some(
      (card) =>
        isDocumentSurfaceCard(card) &&
        (card.kind === "plots" || card.kind === "histograms" || card.kind === "box_summaries"),
    ),
  );
}

/* === Private Functions === */
