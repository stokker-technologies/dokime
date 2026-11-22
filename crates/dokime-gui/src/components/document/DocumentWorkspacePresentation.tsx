// Dokime — SPDX-License-Identifier: Apache-2.0
import { createContext, type ReactNode, useCallback, useContext, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { usePlotPalette } from "../../plots/use-plot-palette";
import { SvelteMount } from "../SvelteMount";
import { TraceabilitySaveContext } from "../traceability/TraceabilityJsonDialog";
import type { DocumentCardContentProps } from "./DocumentCardContent.svelte";
import DocumentWorkspaceBridge from "./DocumentWorkspaceBridge.svelte";
import type { DocumentWorkspaceView } from "./DocumentWorkspacePresentation.svelte";

export { visibleDocumentPlots } from "./DocumentWorkspacePresentation.svelte";

/** Retry one failed card or its enclosing structure. */
export const DocumentRetryContext = createContext<((cardId?: string) => void) | undefined>(undefined);

export type DocumentWorkspaceViewProps = DocumentWorkspaceView & {
  renderCardContent?: (props: DocumentCardContentProps) => ReactNode;
};

export function DocumentWorkspacePresentation(props: DocumentWorkspaceViewProps) {
  const onRetry = useContext(DocumentRetryContext);
  const traceabilitySave = useContext(TraceabilitySaveContext);
  const plotPalette = usePlotPalette();
  const [portals, setPortals] = useState(new Map<HTMLElement, DocumentCardContentProps>());
  const keys = useRef(new WeakMap<HTMLElement, string>());
  const nextKey = useRef(0);
  const attachPortal = useCallback((element: HTMLElement, content: DocumentCardContentProps) => {
    if (!keys.current.has(element)) {
      nextKey.current += 1;
      keys.current.set(element, String(nextKey.current));
    }
    setPortals((held) => new Map(held).set(element, content));
    return () => {
      setPortals((held) => {
        const next = new Map(held);
        next.delete(element);
        return next;
      });
    };
  }, []);
  const workspace = useMemo(() => {
    const { renderCardContent, ...view } = props;
    return { ...view, plotPalette, attachPortal, hasCardOverride: Boolean(renderCardContent), onRetry, traceabilitySave };
  }, [props, plotPalette, attachPortal, onRetry, traceabilitySave]);

  return (
    <>
      <SvelteMount component={DocumentWorkspaceBridge} props={workspace} />
      {[...portals].map(([element, content]) => createPortal(props.renderCardContent?.(content), element, keys.current.get(element)))}
    </>
  );
}
