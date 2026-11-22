// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ComponentType, ReactNode } from "react";
import type { ComponentProps } from "svelte";
import { SvelteMount, useSvelteSlots } from "../../components/SvelteMount";
import { pageTitle } from "../../shell/PageShell";
import { usePageChrome } from "../../shell/page-chrome";
import CycleEvidence, { isCycleEvidenceCard } from "./CycleEvidenceView.svelte";
import type { DocumentViewProps } from "./document-workspace";

export type CycleEvidenceViewProps = Omit<ComponentProps<typeof CycleEvidence>, "documentView" | "lead" | "promptQueue"> & {
  documentView?: ComponentType<DocumentViewProps>;
  lead?: ReactNode;
  promptQueue?: ReactNode;
  trail: string[];
  pageActions?: ReactNode;
};

export function CycleEvidenceView({
  documentView: DocumentContent,
  lead,
  pageActions,
  promptQueue,
  trail,
  ...props
}: CycleEvidenceViewProps) {
  usePageChrome({ title: pageTitle(trail), actions: pageActions });
  const documentView = DocumentContent ? (
    <DocumentContent cardFilter={isCycleEvidenceCard} onRetry={props.onRetry} plotWindow={props.plotWindow} surface={props.surface} />
  ) : null;
  const { rendered, portals } = useSvelteSlots({ documentView, lead, promptQueue });
  return (
    <>
      <SvelteMount component={CycleEvidence} props={{ ...props, ...rendered }} />
      {portals}
    </>
  );
}
