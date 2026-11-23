// Dokime — SPDX-License-Identifier: Apache-2.0
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ConnectedSession } from "../app/native-session/session";
import { downloadNativeArtifact, nativeContentSession, openNativeArtifact } from "../lib/api/native-content";
import { prepareNativeMedia } from "../lib/api/native-delivery";
import { mediaArtifactSource } from "./media-gallery-model";
import type { MediaTransfers } from "./media-previews.svelte";

export function nativeMediaTransfers(session: ConnectedSession): MediaTransfers {
  const content = nativeContentSession(session.identity, session.own);
  return {
    generation: session.identity.generation,
    isCurrent: () => session.identity.isCurrent(),
    open: (artifact) => openNativeArtifact(content, artifact),
    download: (artifact) => downloadNativeArtifact(content, artifact),
    async prepare(artifact) {
      const path = await prepareNativeMedia(content, mediaArtifactSource(artifact));
      return convertFileSrc(path, "dokime-media");
    },
  };
}
