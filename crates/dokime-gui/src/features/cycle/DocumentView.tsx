/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */
import { useNativeSessionOptional } from "@/app/native-session/context";
import { DocumentWorkspaceView } from "@/components/document/DocumentWorkspaceView";
import { type DocumentSurfaceOptions, documentSurface } from "@/features/cycle/document-view-model";
import { documentWorkspace, type DocumentViewProps } from "./document-workspace";
import { useDocumentView } from "./use-document-view";

/* === Components === */
export function DocumentView(props: DocumentViewProps) {
  return <DocumentWorkspaceView {...documentWorkspace(props)} />;
}

/* === Hooks === */
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
  const session = useNativeSessionOptional();
  return documentSurface(options, view, session !== null);
}
