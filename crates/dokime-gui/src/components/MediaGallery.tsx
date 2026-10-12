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
import { Download, ExternalLink, Image as ImageIcon, Search, Video } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  downloadNativeArtifact,
  nativeContentSession,
  openNativeArtifact,
} from "@/lib/api/native-content";
import { prepareNativeMedia } from "@/lib/api/native-delivery";
import { artifactDisplayPath, formatArtifactSize } from "@/lib/artifact-tree";
import { artifactLabel } from "@/lib/dashboard";
import type { ArtifactSource } from "@/lib/desktop/delivery";
import { type MediaArtifactKind, mediaArtifactKind } from "@/lib/media";
import type { RecordScope } from "@/lib/types";
import { isReportBuild } from "@/lib/ui-mode";

/* === Types === */

export type MediaGalleryArtifact = {
  artifact_id: string;
  label: string;
  path: string;
  href?: string | null;
  downloadHref?: string | null;
  media_type?: string | null;
  description?: string | null;
  size_bytes?: number | null;
  group_id?: string | null;
  group_label?: string | null;
  scope?: RecordScope | null;
};

type MediaGalleryProps = {
  artifacts: MediaGalleryArtifact[];
  emptyMessage: string;
  runDir?: string | null;
};

type ResolvedMediaGalleryArtifact = MediaGalleryArtifact & {
  displayPath: string;
  kind: MediaArtifactKind;
  href: string | null;
  nativeTransfer: boolean;
};

type MediaArtifactCandidate = MediaGalleryArtifact & {
  displayPath: string;
  kind: MediaArtifactKind;
};

type MediaUrlEntry = {
  key: string;
  generation: string;
  url: string;
};

type NativeMediaState = {
  errors: Record<string, string>;
  retry(artifactId: string): void;
  urls: Record<string, string>;
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

const MAX_ACTIVE_MEDIA_DOWNLOADS = 2;
const SAFE_IMAGE_TYPES = new Set([
  "image/avif",
  "image/bmp",
  "image/gif",
  "image/jpeg",
  "image/png",
  "image/webp",
]);
const SAFE_VIDEO_TYPES = new Set(["video/mp4", "video/mpeg", "video/ogg", "video/webm"]);

function useNativeMediaUrls(
  artifacts: MediaArtifactCandidate[],
  requestedIds: ReadonlySet<string>,
): NativeMediaState {
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

function artifactContentKey(artifact: MediaGalleryArtifact): string {
  return [
    artifact.artifact_id,
    artifact.group_id ?? "",
    artifact.path,
    artifact.media_type ?? "",
    artifact.size_bytes ?? "",
    artifact.scope?.test_execution_id ?? "",
  ].join("\u0000");
}

function mediaBlobType(artifact: MediaGalleryArtifact): string | null {
  const explicit = artifact.media_type?.trim().toLowerCase();
  if (explicit) return safeMediaType(explicit) ? explicit : null;
  const path = artifact.path.toLowerCase().split(/[?#]/u)[0] ?? "";
  const extension = path.slice(path.lastIndexOf("."));
  if (extension === ".png") return "image/png";
  if (extension === ".jpg" || extension === ".jpeg") return "image/jpeg";
  if (extension === ".gif") return "image/gif";
  if (extension === ".webp") return "image/webp";
  if (extension === ".bmp") return "image/bmp";
  if (extension === ".avif") return "image/avif";
  if (extension === ".mp4" || extension === ".m4v") return "video/mp4";
  if (extension === ".webm") return "video/webm";
  if (extension === ".ogg") return "video/ogg";
  if (extension === ".mpeg" || extension === ".mpg") return "video/mpeg";
  return null;
}

function safeMediaType(value: string): boolean {
  return SAFE_IMAGE_TYPES.has(value) || SAFE_VIDEO_TYPES.has(value);
}

function isNativeArtifactSource(artifact: MediaGalleryArtifact): boolean {
  if (artifact.group_id === "execution-files") {
    return Boolean(artifact.scope?.test_execution_id?.trim() && artifact.path.trim());
  }
  return true;
}

/** Route registry artifacts and execution-file snapshots to their distinct verified pull source. */
function mediaArtifactSource(artifact: MediaGalleryArtifact): ArtifactSource {
  if (artifact.group_id === "execution-files") {
    const executionId = artifact.scope?.test_execution_id?.trim();
    if (!executionId) {
      throw new Error("Execution media requires a test execution scope.");
    }
    return { kind: "execution_file", executionId, relativePath: artifact.path };
  }
  return { kind: "artifact", artifactId: artifact.artifact_id };
}

/* === Components === */

export function MediaGallery({ artifacts, emptyMessage, runDir }: MediaGalleryProps) {
  const nativeSession = useNativeSessionOptional();
  const [selectedArtifactId, setSelectedArtifactId] = useState<string | null>(null);
  const [nativeDownloadArtifactId, setNativeDownloadArtifactId] = useState<string | null>(null);
  const [nativeDownloadError, setNativeDownloadError] = useState<{
    artifactId: string;
    message: string;
  } | null>(null);
  const [nativeOpenArtifactId, setNativeOpenArtifactId] = useState<string | null>(null);
  const [nativeOpenError, setNativeOpenError] = useState<{
    artifactId: string;
    message: string;
  } | null>(null);
  const media = useMemo(
    () =>
      artifacts
        .map((artifact) => ({
          ...artifact,
          displayPath: artifactDisplayPath(artifact, runDir),
          kind: mediaArtifactKind(artifact),
        }))
        .filter(isResolvedMediaArtifact),
    [artifacts, runDir],
  );
  const [visibleArtifactIds, setVisibleArtifactIds] = useState<Set<string>>(() => new Set());
  const markVisible = useCallback((artifactId: string, visible: boolean) => {
    setVisibleArtifactIds((current) => {
      if (visible === current.has(artifactId)) return current;
      const next = new Set(current);
      if (visible) next.add(artifactId);
      else next.delete(artifactId);
      return next;
    });
  }, []);
  const requestedArtifactIds = useMemo(() => {
    const requested = new Set(
      media
        .filter((artifact) => visibleArtifactIds.has(artifact.artifact_id))
        .map((artifact) => artifact.artifact_id),
    );
    if (selectedArtifactId) requested.add(selectedArtifactId);
    return requested;
  }, [media, selectedArtifactId, visibleArtifactIds]);
  const nativeMedia = useNativeMediaUrls(media, requestedArtifactIds);
  const openArtifact = useCallback(
    async (artifact: MediaGalleryArtifact) => {
      if (!nativeSession || nativeOpenArtifactId || nativeDownloadArtifactId) return;
      setNativeOpenArtifactId(artifact.artifact_id);
      setNativeOpenError(null);
      try {
        await openNativeArtifact(
          nativeContentSession(nativeSession.identity, nativeSession.own),
          artifact,
        );
      } catch (error) {
        setNativeOpenError({
          artifactId: artifact.artifact_id,
          message: error instanceof Error ? error.message : "Unable to open this artifact.",
        });
      } finally {
        setNativeOpenArtifactId(null);
      }
    },
    [nativeDownloadArtifactId, nativeOpenArtifactId, nativeSession],
  );
  const downloadArtifact = useCallback(
    async (artifact: MediaGalleryArtifact) => {
      if (!nativeSession || nativeDownloadArtifactId) return;
      setNativeDownloadArtifactId(artifact.artifact_id);
      setNativeDownloadError(null);
      try {
        await downloadNativeArtifact(
          nativeContentSession(nativeSession.identity, nativeSession.own),
          artifact,
        );
      } catch (error) {
        setNativeDownloadError({
          artifactId: artifact.artifact_id,
          message: error instanceof Error ? error.message : "Unable to download this artifact.",
        });
      } finally {
        setNativeDownloadArtifactId(null);
      }
    },
    [nativeDownloadArtifactId, nativeSession],
  );
  const resolvedMedia = useMemo(
    () =>
      media.map((artifact) => {
        const nativeTransfer =
          nativeSession !== null &&
          !artifact.href &&
          !artifact.downloadHref &&
          isNativeArtifactSource(artifact);
        return {
          ...artifact,
          href:
            artifact.href ??
            artifact.downloadHref ??
            nativeMedia.urls[artifact.artifact_id] ??
            null,
          downloadHref: artifact.downloadHref ?? artifact.href ?? null,
          nativeTransfer,
        };
      }),
    [media, nativeMedia.urls, nativeSession],
  );
  const selectedArtifact =
    resolvedMedia.find((artifact) => artifact.artifact_id === selectedArtifactId) ?? null;

  if (!media.length) {
    return <p className="text-sm text-muted-foreground">{emptyMessage}</p>;
  }

  return (
    <>
      <div
        className="grid grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-2"
        data-testid="media-gallery"
      >
        {resolvedMedia.map((artifact) => (
          <MediaGalleryCard
            artifact={artifact}
            key={artifact.artifact_id}
            onInspect={() => setSelectedArtifactId(artifact.artifact_id)}
            onNativeOpen={() => void openArtifact(artifact)}
            onNativeDownload={() => void downloadArtifact(artifact)}
            nativeOpenBusy={nativeOpenArtifactId === artifact.artifact_id}
            nativeOpenError={
              nativeOpenError?.artifactId === artifact.artifact_id ? nativeOpenError.message : null
            }
            nativeDownloadBusy={nativeDownloadArtifactId === artifact.artifact_id}
            nativeDownloadError={
              nativeDownloadError?.artifactId === artifact.artifact_id
                ? nativeDownloadError.message
                : null
            }
            onRetry={() => nativeMedia.retry(artifact.artifact_id)}
            previewError={nativeMedia.errors[artifact.artifact_id] ?? null}
            onVisibilityChange={markVisible}
          />
        ))}
      </div>
      <Dialog
        open={selectedArtifact !== null}
        onOpenChange={(open) => {
          if (!open) {
            setSelectedArtifactId(null);
          }
        }}
      >
        <DialogContent className="max-w-5xl">
          {selectedArtifact ? (
            <div className="grid gap-4">
              <DialogHeader>
                <DialogTitle>{mediaArtifactTitle(selectedArtifact)}</DialogTitle>
                <DialogDescription>{selectedArtifact.displayPath}</DialogDescription>
              </DialogHeader>
              <div className="overflow-hidden rounded-xl border bg-muted/20 p-2">
                {selectedArtifact.href ? (
                  selectedArtifact.kind === "image" ? (
                    <img
                      alt={mediaArtifactTitle(selectedArtifact)}
                      className="max-h-[70vh] w-full object-contain"
                      src={selectedArtifact.href}
                    />
                  ) : (
                    // biome-ignore lint/a11y/useMediaCaption: artifact videos do not have caption sidecars in the current media contract.
                    <video
                      autoPlay
                      className="max-h-[70vh] w-full"
                      controls
                      preload="metadata"
                      src={selectedArtifact.href}
                    />
                  )
                ) : selectedArtifact.nativeTransfer && mediaBlobType(selectedArtifact) ? (
                  <div className="grid justify-items-center gap-3 p-8 text-center text-sm text-muted-foreground">
                    {nativeMedia.errors[selectedArtifact.artifact_id] ? (
                      <p role="alert">{nativeMedia.errors[selectedArtifact.artifact_id]}</p>
                    ) : (
                      <p>Loading media preview…</p>
                    )}
                    {nativeMedia.errors[selectedArtifact.artifact_id] ? (
                      <Button
                        size="sm"
                        type="button"
                        variant="outline"
                        onClick={() => nativeMedia.retry(selectedArtifact.artifact_id)}
                      >
                        Retry preview
                      </Button>
                    ) : null}
                  </div>
                ) : (
                  <p className="p-8 text-center text-sm text-muted-foreground">
                    Preview unavailable for this media artifact.
                  </p>
                )}
              </div>
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div className="text-sm text-muted-foreground">
                  {selectedArtifact.group_label
                    ? artifactLabel(selectedArtifact.group_label)
                    : "Media artifact"}
                  {selectedArtifact.size_bytes
                    ? ` • ${formatArtifactSize(selectedArtifact.size_bytes)}`
                    : ""}
                </div>
                <div className="flex flex-wrap items-center gap-2">
                  {selectedArtifact.href && !selectedArtifact.nativeTransfer ? (
                    <Button asChild size="sm" variant="outline">
                      <a href={selectedArtifact.href} rel="noreferrer" target="_blank">
                        <ExternalLink data-icon="inline-start" size={14} />
                        Open
                      </a>
                    </Button>
                  ) : null}
                  {selectedArtifact.downloadHref && !selectedArtifact.nativeTransfer ? (
                    <Button asChild size="sm" variant="outline">
                      <a href={selectedArtifact.downloadHref}>
                        <Download data-icon="inline-start" size={14} />
                        Download
                      </a>
                    </Button>
                  ) : null}
                  {selectedArtifact.nativeTransfer ? (
                    <>
                      <Button
                        disabled={
                          nativeOpenArtifactId === selectedArtifact.artifact_id ||
                          nativeDownloadArtifactId === selectedArtifact.artifact_id
                        }
                        size="sm"
                        type="button"
                        variant="outline"
                        onClick={() => void openArtifact(selectedArtifact)}
                      >
                        <ExternalLink data-icon="inline-start" size={14} />
                        {nativeOpenArtifactId === selectedArtifact.artifact_id
                          ? "Opening…"
                          : "Open"}
                      </Button>
                      <Button
                        disabled={
                          nativeOpenArtifactId === selectedArtifact.artifact_id ||
                          nativeDownloadArtifactId === selectedArtifact.artifact_id
                        }
                        size="sm"
                        type="button"
                        variant="outline"
                        onClick={() => void downloadArtifact(selectedArtifact)}
                      >
                        <Download data-icon="inline-start" size={14} />
                        {nativeDownloadArtifactId === selectedArtifact.artifact_id
                          ? "Saving…"
                          : "Download"}
                      </Button>
                    </>
                  ) : null}
                  {nativeOpenError?.artifactId === selectedArtifact.artifact_id ? (
                    <p className="basis-full text-sm text-destructive" role="alert">
                      {nativeOpenError.message}
                    </p>
                  ) : null}
                  {nativeDownloadError?.artifactId === selectedArtifact.artifact_id ? (
                    <p className="basis-full text-sm text-destructive" role="alert">
                      {nativeDownloadError.message}
                    </p>
                  ) : null}
                </div>
              </div>
            </div>
          ) : null}
        </DialogContent>
      </Dialog>
    </>
  );
}

function MediaGalleryCard({
  artifact,
  nativeOpenBusy,
  nativeOpenError,
  nativeDownloadBusy,
  nativeDownloadError,
  onInspect,
  onNativeOpen,
  onNativeDownload,
  onRetry,
  onVisibilityChange,
  previewError,
}: {
  artifact: ResolvedMediaGalleryArtifact;
  nativeOpenBusy: boolean;
  nativeOpenError: string | null;
  nativeDownloadBusy: boolean;
  nativeDownloadError: string | null;
  onInspect: () => void;
  onNativeOpen: () => void;
  onNativeDownload: () => void;
  onRetry: () => void;
  onVisibilityChange: (artifactId: string, visible: boolean) => void;
  previewError: string | null;
}) {
  const [imagePreviewFailed, setImagePreviewFailed] = useState(false);
  const cardRef = useRef<HTMLElement | null>(null);
  const title = mediaArtifactTitle(artifact);
  const meta = artifact.group_label
    ? artifactLabel(artifact.group_label)
    : formatArtifactSize(artifact.size_bytes);
  const showImagePreview =
    artifact.kind === "image" && Boolean(artifact.href) && !imagePreviewFailed;

  useEffect(() => {
    const element = cardRef.current;
    if (!element || typeof IntersectionObserver === "undefined") return;
    const observer = new IntersectionObserver(
      ([entry]) => onVisibilityChange(artifact.artifact_id, entry?.isIntersecting ?? false),
      { rootMargin: "160px" },
    );
    observer.observe(element);
    return () => observer.disconnect();
  }, [artifact.artifact_id, onVisibilityChange]);

  return (
    <article className="grid min-w-0 overflow-hidden rounded-md border bg-background" ref={cardRef}>
      <button
        className="group/media relative grid aspect-video place-items-center overflow-hidden bg-muted/45 text-left"
        title={artifact.displayPath}
        type="button"
        onClick={onInspect}
      >
        {showImagePreview ? (
          <img
            alt={title}
            className="h-full w-full object-cover transition-transform group-hover/media:scale-[1.02]"
            loading="lazy"
            src={artifact.href ?? undefined}
            onError={() => setImagePreviewFailed(true)}
          />
        ) : null}
        <span className="pointer-events-none absolute inset-0 bg-gradient-to-b from-transparent via-transparent to-background/50" />
        {!showImagePreview ? (
          <span className="relative z-10 flex max-w-full flex-col items-center gap-2 px-3 text-center">
            {artifact.kind === "image" ? (
              <ImageIcon className="text-muted-foreground" size={18} />
            ) : (
              <Video className="text-muted-foreground" size={18} />
            )}
            <span className="line-clamp-2 text-xs font-semibold text-foreground">{title}</span>
          </span>
        ) : null}
        <span className="absolute inset-x-2 bottom-2 flex justify-end">
          <span className="inline-flex items-center gap-1 rounded-md bg-background/85 px-2 py-1 text-[0.68rem] font-semibold text-foreground shadow-sm">
            <Search size={12} />
            Inspect
          </span>
        </span>
      </button>
      <div className="grid min-w-0 gap-1 px-2 py-2">
        <div className="flex min-w-0 items-start gap-1.5">
          {artifact.kind === "image" ? (
            <ImageIcon className="mt-0.5 shrink-0" size={14} />
          ) : (
            <Video className="mt-0.5 shrink-0" size={14} />
          )}
          <div className="min-w-0">
            <h3 className="line-clamp-2 break-all text-xs font-semibold" title={title}>
              {title}
            </h3>
            <p
              className="truncate text-[0.7rem] text-muted-foreground"
              title={artifact.displayPath}
            >
              {artifact.displayPath}
            </p>
          </div>
        </div>
        <div className="flex min-w-0 items-center justify-between gap-2">
          {meta ? (
            <span className="truncate text-[0.7rem] text-muted-foreground">{meta}</span>
          ) : (
            <span />
          )}
          {previewError ? (
            <Button
              className="h-auto px-1 py-0 text-[0.68rem]"
              size="sm"
              type="button"
              variant="ghost"
              onClick={onRetry}
            >
              Retry
            </Button>
          ) : null}
          <span className="flex shrink-0 items-center gap-1">
            {artifact.href && !artifact.nativeTransfer ? (
              <Button asChild size="icon-xs" variant="ghost">
                <a aria-label={`Open ${title}`} href={artifact.href}>
                  <ExternalLink size={13} />
                </a>
              </Button>
            ) : null}
            {artifact.downloadHref && !artifact.nativeTransfer ? (
              <Button asChild size="icon-xs" variant="ghost">
                <a aria-label={`Download ${title}`} href={artifact.downloadHref}>
                  <Download size={13} />
                </a>
              </Button>
            ) : null}
            {artifact.nativeTransfer ? (
              <>
                <Button
                  aria-label={nativeOpenError ? `Retry open ${title}` : `Open ${title}`}
                  disabled={nativeOpenBusy || nativeDownloadBusy}
                  size="icon-xs"
                  title={nativeOpenError ?? "Open artifact"}
                  type="button"
                  variant="ghost"
                  onClick={onNativeOpen}
                >
                  <ExternalLink size={13} />
                </Button>
                <Button
                  aria-label={nativeDownloadError ? `Retry download ${title}` : `Download ${title}`}
                  disabled={nativeOpenBusy || nativeDownloadBusy}
                  size="icon-xs"
                  title={nativeDownloadError ?? "Download artifact"}
                  type="button"
                  variant="ghost"
                  onClick={onNativeDownload}
                >
                  <Download size={13} />
                </Button>
              </>
            ) : null}
          </span>
        </div>
      </div>
    </article>
  );
}

function mediaArtifactTitle(
  artifact: Pick<ResolvedMediaGalleryArtifact, "displayPath" | "label">,
): string {
  return artifact.displayPath.split("/").filter(Boolean).at(-1) ?? artifactLabel(artifact.label);
}

function isResolvedMediaArtifact(
  artifact: MediaGalleryArtifact & {
    displayPath: string;
    kind: MediaArtifactKind | null;
  },
): artifact is MediaArtifactCandidate {
  return artifact.kind !== null;
}
