/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */
import { convertFileSrc } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";

/* === Local Imports === */
import { useNativeSessionOptional } from "@/app/native-session/context";
import { downloadNativeArtifact, nativeContentSession, openNativeArtifact } from "@/lib/api/native-content";
import { prepareNativeMedia } from "@/lib/api/native-delivery";
import { isReportBuild } from "@/lib/ui-mode";
import {
  type MediaArtifactCandidate, type MediaPreviewState, type MediaGalleryProps,
  artifactContentKey, mediaBlobType, isNativeArtifactSource, mediaArtifactSource,
} from "./media-gallery-model";
import { MediaGallerySurface, useMediaGallery } from "./MediaGalleryView";

/* === Types === */
type MediaUrlEntry = {
  key: string;
  generation: string;
  url: string;
};

type MediaErrorEntry = {
  generation: string;
  key: string;
  message: string;
};

type MediaDownloadRequest = {
  artifact: MediaArtifactCandidate;
  controller: AbortController;
  key: string;
  generation: string;
  cancelled: boolean;
};

/* === Constants === */
const MAX_ACTIVE_MEDIA_DOWNLOADS = 2;

/* === Components === */
export function MediaGallery({ artifacts, emptyMessage, runDir }: MediaGalleryProps) {
  const session = useNativeSessionOptional();
  const model = useMediaGallery(artifacts, runDir);
  const preview = useNativeMediaUrls(model.media, model.requestedArtifactIds);
  return (
    <MediaGallerySurface
      model={model}
      emptyMessage={emptyMessage}
      preview={preview}
      actions={session ? {
        open: (artifact) => openNativeArtifact(nativeContentSession(session.identity, session.own), artifact),
        download: (artifact) => downloadNativeArtifact(nativeContentSession(session.identity, session.own), artifact),
      } : undefined}
    />
  );
}

/* === Hooks === */
function useNativeMediaUrls(
  artifacts: MediaArtifactCandidate[],
  requestedIds: ReadonlySet<string>,
): MediaPreviewState {
  const session = useNativeSessionOptional();
  const [, redraw] = useState(0);
  const entriesRef = useRef(new Map<string, MediaUrlEntry>());
  const errorsRef = useRef(new Map<string, MediaErrorEntry>());
  const requestsRef = useRef(new Map<string, MediaDownloadRequest>());
  const queueRef = useRef<MediaDownloadRequest[]>([]);
  const activeRef = useRef(0);
  const artifactsRef = useRef(new Map<string, MediaArtifactCandidate>());
  const requestedRef = useRef<ReadonlySet<string>>(requestedIds);
  const sessionRef = useRef(session);
  const pumpRef = useRef<() => void>(() => undefined);
  const mountedRef = useRef(true);

  artifactsRef.current = new Map(artifacts.map((artifact) => [artifact.artifact_id, artifact]));
  requestedRef.current = requestedIds;
  sessionRef.current = session;

  const revoke = useCallback((artifactId: string) => {
    // The URL is a `dokime-media://` path through the native scheme handler,
    // not a Blob URL: there is nothing to revoke, only the local cache entry.
    return entriesRef.current.delete(artifactId);
  }, []);

  const pump = useCallback(() => {
    /* A frozen report has no preview cache to materialize into and no custom
       scheme to read it back through: `convertFileSrc` would mint a
       `dokime-media://` URL that the WebView cannot resolve from a file. The
       report's media comes from the payload's own hrefs instead. */
    if (isReportBuild()) return;
    while (activeRef.current < MAX_ACTIVE_MEDIA_DOWNLOADS) {
      const request = queueRef.current.shift();
      if (!request) return;
      const currentSession = sessionRef.current;
      const currentArtifact = artifactsRef.current.get(request.artifact.artifact_id);
      if (
        request.cancelled ||
        !currentSession ||
        request.generation !== currentSession.identity.generation ||
        !requestedRef.current.has(request.artifact.artifact_id) ||
        !currentArtifact ||
        artifactContentKey(currentArtifact) !== request.key ||
        currentArtifact.href ||
        currentArtifact.downloadHref ||
        !isNativeArtifactSource(currentArtifact) ||
        !mediaBlobType(currentArtifact)
      ) {
        continue;
      }
      const existingEntry = entriesRef.current.get(request.artifact.artifact_id);
      if (
        existingEntry &&
        existingEntry.key === request.key &&
        existingEntry.generation === request.generation
      ) {
        continue;
      }
      if (requestsRef.current.has(request.artifact.artifact_id)) continue;
      requestsRef.current.set(request.artifact.artifact_id, request);
      activeRef.current += 1;
      void (async () => {
        let completed = false;
        try {
          const path = await prepareNativeMedia(
            nativeContentSession(currentSession.identity, currentSession.own),
            mediaArtifactSource(request.artifact),
          );
          const latestSession = sessionRef.current;
          const latestArtifact = artifactsRef.current.get(request.artifact.artifact_id);
          const mimeType = latestArtifact ? mediaBlobType(latestArtifact) : null;
          if (
            request.cancelled ||
            !latestSession ||
            latestSession.identity.generation !== request.generation ||
            !latestSession.identity.isCurrent() ||
            !latestArtifact ||
            artifactContentKey(latestArtifact) !== request.key ||
            !requestedRef.current.has(request.artifact.artifact_id) ||
            !mimeType
          ) {
            return;
          }
          revoke(request.artifact.artifact_id);
          const url = convertFileSrc(path, "dokime-media");
          entriesRef.current.set(request.artifact.artifact_id, {
            generation: request.generation,
            key: request.key,
            url,
          });
          errorsRef.current.delete(request.artifact.artifact_id);
          completed = true;
          redraw((value) => value + 1);
        } catch (error) {
          const latestSession = sessionRef.current;
          if (
            !request.cancelled &&
            !request.controller.signal.aborted &&
            mountedRef.current &&
            latestSession?.identity.generation === request.generation &&
            latestSession.identity.isCurrent()
          ) {
            errorsRef.current.set(request.artifact.artifact_id, {
              generation: request.generation,
              key: request.key,
              message:
                error instanceof Error ? error.message : "Unable to load this media preview.",
            });
            redraw((value) => value + 1);
          }
        } finally {
          requestsRef.current.delete(request.artifact.artifact_id);
          activeRef.current -= 1;
          const latestSession = sessionRef.current;
          const latestArtifact = artifactsRef.current.get(request.artifact.artifact_id);
          if (
            !completed &&
            request.cancelled &&
            mountedRef.current &&
            latestSession &&
            latestArtifact &&
            requestedRef.current.has(request.artifact.artifact_id) &&
            !entriesRef.current.has(request.artifact.artifact_id) &&
            !queueRef.current.some(
              (queued) =>
                queued.artifact.artifact_id === request.artifact.artifact_id &&
                queued.key === artifactContentKey(latestArtifact) &&
                queued.generation === latestSession.identity.generation,
            )
          ) {
            queueRef.current.push({
              artifact: latestArtifact,
              controller: new AbortController(),
              key: artifactContentKey(latestArtifact),
              generation: latestSession.identity.generation,
              cancelled: false,
            });
          }
          pumpRef.current();
        }
      })();
    }
  }, [revoke]);

  pumpRef.current = pump;

  useEffect(() => {
    mountedRef.current = true;
    const generation = session?.identity.generation;
    let changed = false;
    const currentArtifacts = artifactsRef.current;
    for (const [artifactId, entry] of entriesRef.current) {
      const artifact = currentArtifacts.get(artifactId);
      if (
        !session ||
        entry.generation !== generation ||
        entry.key !== (artifact ? artifactContentKey(artifact) : "") ||
        !requestedIds.has(artifactId)
      ) {
        changed = revoke(artifactId) || changed;
      }
    }
    for (const [artifactId, error] of errorsRef.current) {
      const artifact = currentArtifacts.get(artifactId);
      if (
        !session ||
        error.generation !== generation ||
        !artifact ||
        error.key !== artifactContentKey(artifact) ||
        !requestedIds.has(artifactId)
      ) {
        errorsRef.current.delete(artifactId);
        changed = true;
      }
    }
    for (const request of requestsRef.current.values()) {
      const artifact = currentArtifacts.get(request.artifact.artifact_id);
      if (
        !session ||
        request.generation !== generation ||
        !artifact ||
        artifactContentKey(artifact) !== request.key ||
        !requestedIds.has(request.artifact.artifact_id)
      ) {
        request.cancelled = true;
        request.controller.abort();
      }
    }
    const queuedKeys = new Set<string>();
    queueRef.current = queueRef.current.filter((request) => {
      const artifact = currentArtifacts.get(request.artifact.artifact_id);
      const queueKey = `${request.artifact.artifact_id}\u0000${request.generation}\u0000${request.key}`;
      const keep = Boolean(
        session &&
          request.generation === generation &&
          artifact &&
          artifactContentKey(artifact) === request.key &&
          requestedIds.has(request.artifact.artifact_id),
      );
      if (!keep || queuedKeys.has(queueKey)) {
        changed = true;
        return false;
      }
      queuedKeys.add(queueKey);
      return true;
    });
    if (session) {
      for (const artifact of artifacts) {
        if (
          !requestedIds.has(artifact.artifact_id) ||
          artifact.href ||
          artifact.downloadHref ||
          !isNativeArtifactSource(artifact) ||
          !mediaBlobType(artifact) ||
          entriesRef.current.has(artifact.artifact_id) ||
          requestsRef.current.has(artifact.artifact_id) ||
          queueRef.current.some(
            (request) =>
              request.artifact.artifact_id === artifact.artifact_id &&
              request.generation === session.identity.generation &&
              request.key === artifactContentKey(artifact),
          )
        ) {
          continue;
        }
        queueRef.current.push({
          artifact,
          controller: new AbortController(),
          key: artifactContentKey(artifact),
          generation: session.identity.generation,
          cancelled: false,
        });
      }
      pump();
    }
    if (changed) redraw((value) => value + 1);
  }, [artifacts, pump, requestedIds, revoke, session]);

  const retry = useCallback(
    (artifactId: string) => {
      errorsRef.current.delete(artifactId);
      revoke(artifactId);
      const currentSession = sessionRef.current;
      const artifact = artifactsRef.current.get(artifactId);
      if (
        !currentSession ||
        !artifact ||
        !requestedRef.current.has(artifactId) ||
        artifact.href ||
        artifact.downloadHref ||
        !isNativeArtifactSource(artifact) ||
        !mediaBlobType(artifact) ||
        requestsRef.current.has(artifactId) ||
        queueRef.current.some(
          (request) =>
            request.artifact.artifact_id === artifactId &&
            request.generation === currentSession.identity.generation &&
            request.key === artifactContentKey(artifact),
        )
      ) {
        redraw((value) => value + 1);
        return;
      }
      queueRef.current.push({
        artifact,
        controller: new AbortController(),
        key: artifactContentKey(artifact),
        generation: currentSession.identity.generation,
        cancelled: false,
      });
      redraw((value) => value + 1);
      pumpRef.current();
    },
    [revoke],
  );

  useEffect(
    () => () => {
      mountedRef.current = false;
      entriesRef.current.clear();
      errorsRef.current.clear();
      for (const request of requestsRef.current.values()) {
        request.cancelled = true;
        request.controller.abort();
      }
      requestsRef.current.clear();
      queueRef.current = [];
    },
    [],
  );

  return {
    errors: Object.fromEntries(
      [...errorsRef.current].map(([artifactId, error]) => [artifactId, error.message]),
    ),
    retry,
    urls: Object.fromEntries(
      [...entriesRef.current].map(([artifactId, entry]) => [artifactId, entry.url]),
    ),
  };
}

