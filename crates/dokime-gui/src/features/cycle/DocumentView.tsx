/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { useNativeSessionOptional } from "@/app/native-session/context";
import { DocumentWorkspaceView } from "@/components/document/DocumentWorkspaceView";
import { DocumentViewPresentation, type DocumentViewProps } from "./DocumentViewPresentation";
import { type DocumentSurfaceOptions, documentSurface } from "./document-view-model";
import { useDocumentView } from "./use-document-view";

export function DocumentView(props: DocumentViewProps) {
  return <DocumentViewPresentation {...props} workspace={DocumentWorkspaceView} />;
}

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
