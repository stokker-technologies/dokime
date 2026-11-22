// Dokime — SPDX-License-Identifier: Apache-2.0
import { type ComponentType, type ReactNode, useContext } from "react";
import type { ComponentProps } from "svelte";
import { DocumentRetryContext } from "../../components/document/DocumentWorkspacePresentation";
import { SvelteMount, useSvelteSlots } from "../../components/SvelteMount";
import { TraceabilitySaveContext } from "../../components/traceability/TraceabilityJsonDialog";
import { pageTitle } from "../../shell/PageShell";
import { usePageChrome } from "../../shell/page-chrome";
import type { DocumentViewProps } from "./document-workspace";
import ExecutionSections, { executionReportCardChrome, isExecutionReportCard } from "./ExecutionSectionsView.svelte";

export type ExecutionSectionsViewProps = Omit<
  ComponentProps<typeof ExecutionSections>,
  "control" | "promptQueue" | "notesContent" | "mediaContent" | "artifactsContent" | "documentView" | "traceabilitySave"
> & {
  control?: ReactNode;
  promptQueue?: ReactNode;
  notesContent?: ReactNode;
  mediaContent?: ReactNode;
  artifactsContent?: ReactNode;
  documentView?: ComponentType<DocumentViewProps>;
  trail: string[];
  pageActions?: ReactNode;
};

export function ExecutionSectionsView({
  documentView: DocumentContent,
  control,
  promptQueue,
  notesContent,
  mediaContent,
  artifactsContent,
  trail,
  pageActions,
  ...props
}: ExecutionSectionsViewProps) {
  usePageChrome({
    title: pageTitle([...trail.slice(0, -1), `${props.titlePrefix}${props.label}`]),
    actions: pageActions,
  });
  const traceabilitySave = useContext(TraceabilitySaveContext);
  const documentView = DocumentContent ? (
    <DocumentRetryContext.Provider value={props.onRetry}>
      <DocumentContent cardChrome={executionReportCardChrome} cardFilter={isExecutionReportCard} surface={props.surface} />
    </DocumentRetryContext.Provider>
  ) : null;
  const { rendered, portals } = useSvelteSlots({ documentView, control, promptQueue, notesContent, mediaContent, artifactsContent });
  return (
    <>
      <SvelteMount component={ExecutionSections} props={{ ...props, ...rendered, traceabilitySave }} />
      {portals}
    </>
  );
}
