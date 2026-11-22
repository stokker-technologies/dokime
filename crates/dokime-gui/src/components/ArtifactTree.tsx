/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useState } from "react";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/context";
import type { ArtifactTreeFile } from "@/lib/artifact-tree";
import { downloadNativeArtifact, nativeContentSession, openNativeArtifact } from "@/lib/api/native-content";
import ArtifactTreeView, { type ArtifactTreeProps } from "./ArtifactTreeView.svelte";
import { SvelteMount } from "./SvelteMount";
import { TextArtifactViewer } from "./TextArtifactViewer";

/* === Components === */

export function ArtifactTree(props: ArtifactTreeProps) {
  const session = useNativeSessionOptional();
  const [preview, setPreview] = useState<ArtifactTreeFile | null>(null);
  const content = session ? nativeContentSession(session.identity, session.own) : null;
  const actions = content
    ? {
        open: (artifact: ArtifactTreeFile) => openNativeArtifact(content, artifact),
        download: (artifact: ArtifactTreeFile) => downloadNativeArtifact(content, artifact),
      }
    : undefined;

  return (
    <>
      <SvelteMount component={ArtifactTreeView} props={{ ...props, actions, onPreview: setPreview }} />
      <TextArtifactViewer artifact={preview} open={preview !== null} onOpenChange={(open) => {
        if (!open) {
          setPreview(null);
        }
      }} />
    </>
  );
}
