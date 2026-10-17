/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */
import { useNativeSessionOptional } from "@/app/native-session/context";
import { TextArtifactViewer } from "./TextArtifactViewer";
import { ArtifactTreeView, type ArtifactTreeProps } from "./ArtifactTreeView";
import { downloadNativeArtifact, nativeContentSession, openNativeArtifact } from "@/lib/api/native-content";

/* === Components === */
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
