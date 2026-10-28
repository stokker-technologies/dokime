/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { useNativeSessionOptional } from "@/app/native-session/context";
import { downloadNativeArtifact, nativeContentSession, openNativeArtifact } from "@/lib/api/native-content";
import { type ArtifactTreeProps, ArtifactTreeView } from "./ArtifactTreeView";
import { TextArtifactViewer } from "./TextArtifactViewer";

export function ArtifactTree(props: ArtifactTreeProps) {
  const session = useNativeSessionOptional();
  return (
    <ArtifactTreeView
      {...props}
      actions={
        session
          ? {
              open: (artifact) => openNativeArtifact(nativeContentSession(session.identity, session.own), artifact),
              download: (artifact) => downloadNativeArtifact(nativeContentSession(session.identity, session.own), artifact),
            }
          : undefined
      }
      renderTextPreview={(artifact, onOpenChange) => (
        <TextArtifactViewer artifact={artifact} open={artifact !== null} onOpenChange={onOpenChange} />
      )}
    />
  );
}
