// Dokime — SPDX-License-Identifier: Apache-2.0
import { useContext } from "react";
import { useNativeSessionOptional } from "../../app/native-session/context";
import { useReportDefaults } from "../../lib/application-settings";
import { usePlotPalette } from "../../plots/use-plot-palette";
import { SvelteMount } from "../SvelteMount";
import { TraceabilitySaveContext } from "../traceability/TraceabilityJsonDialog";
import DocumentWorkspacePresentation, { type DocumentWorkspaceView } from "./DocumentWorkspacePresentation.svelte";
import LiveDocumentWorkspace from "./LiveDocumentWorkspace.svelte";

type Props = DocumentWorkspaceView & { onRetry?: (cardId?: string) => void };

export function DocumentWorkspace(props: Props) {
  const plotPalette = usePlotPalette();
  const traceabilitySave = useContext(TraceabilitySaveContext);
  const view = { ...props, plotPalette, traceabilitySave };

  if (view.renderMode === "report") {
    return <SvelteMount component={DocumentWorkspacePresentation} props={view} />;
  }
  return <ConnectedDocumentWorkspace {...view} />;
}

function ConnectedDocumentWorkspace(
  props: Extract<Props, { renderMode: "live" }> & {
    traceabilitySave?: ((json: string, filename: string) => Promise<void>) | null;
  },
) {
  const session = useNativeSessionOptional();
  const reportDefaults = useReportDefaults();
  return <SvelteMount component={LiveDocumentWorkspace} props={{ ...props, reportDefaults, session }} />;
}
