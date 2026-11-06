/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Frozen And Live Gallery Presentation; Media Resources Are Supplied.
 */

/* === Imports === */

// Standard Library Imports

import { ErrorCard } from "@/components/ErrorCard";
import type { NotificationFailure } from "@/lib/notification-details";
import { failureOf } from "@/lib/notification-details";
import { AudioLines, Download, ExternalLink, Image as ImageIcon, Search, Video } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
// Third Party Imports
import { NotificationMessage } from "@/components/NotificationMessage";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { artifactDisplayPath, formatArtifactSize } from "@/lib/artifact-tree";
import { artifactLabel } from "@/lib/dashboard";
import { type MediaArtifactKind, mediaArtifactKind } from "@/lib/media";

import {
  isNativeArtifactSource,
  type MediaArtifactCandidate,
  type MediaGalleryArtifact,
  type MediaGalleryProps,
  type MediaPreviewState,
  mediaBlobType,
  type ResolvedMediaGalleryArtifact,
} from "./media-gallery-model";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */
export type MediaActions = {
  open: (artifact: MediaGalleryArtifact) => Promise<unknown>;
  download: (artifact: MediaGalleryArtifact) => Promise<unknown>;
};
export type MediaGalleryModel = ReturnType<typeof useMediaGallery>;

/* === Components === */
export function MediaGalleryView({ artifacts, emptyMessage, runDir }: MediaGalleryProps) {
  const model = useMediaGallery(artifacts, runDir);
  return <MediaGallerySurface model={model} emptyMessage={emptyMessage} />;
}

export function MediaGallerySurface({
  model,
  emptyMessage,
  actions,
  preview,
}: {
  model: MediaGalleryModel;
  emptyMessage: string;
  actions?: MediaActions;
  preview?: MediaPreviewState;
}) {
  const { media, selectedArtifactId, setSelectedArtifactId, markVisible } = model;
  const [nativeDownloadArtifactId, setNativeDownloadArtifactId] = useState<string | null>(null);
  const [nativeDownloadError, setNativeDownloadError] = useState<{
    artifactId: string;
    message: string;
    failure: NotificationFailure;
  } | null>(null);
  const [nativeOpenArtifactId, setNativeOpenArtifactId] = useState<string | null>(null);
  const [nativeOpenError, setNativeOpenError] = useState<{
    artifactId: string;
    message: string;
    failure: NotificationFailure;
  } | null>(null);
  const openArtifact = useCallback(
    async (artifact: MediaGalleryArtifact) => {
      if (!actions || nativeOpenArtifactId || nativeDownloadArtifactId) return;
      setNativeOpenArtifactId(artifact.artifact_id);
      setNativeOpenError(null);
      try {
        await actions.open(artifact);
      } catch (error) {
        setNativeOpenError({
          artifactId: artifact.artifact_id,
          message: failureOf(error, "Artifact Action Failed").message,
          failure: failureOf(error, "Artifact Action Failed"),
        });
      } finally {
        setNativeOpenArtifactId(null);
      }
    },
    [nativeDownloadArtifactId, nativeOpenArtifactId, actions],
  );
  const downloadArtifact = useCallback(
    async (artifact: MediaGalleryArtifact) => {
      if (!actions || nativeDownloadArtifactId) return;
      setNativeDownloadArtifactId(artifact.artifact_id);
      setNativeDownloadError(null);
      try {
        await actions.download(artifact);
      } catch (error) {
        setNativeDownloadError({
          artifactId: artifact.artifact_id,
          message: failureOf(error, "Artifact Action Failed").message,
          failure: failureOf(error, "Artifact Action Failed"),
        });
      } finally {
        setNativeDownloadArtifactId(null);
      }
    },
    [nativeDownloadArtifactId, actions],
  );
  const resolvedMedia = useMemo(
    () =>
      media.map((artifact) => {
        const nativeTransfer = actions !== undefined && !artifact.href && !artifact.downloadHref && isNativeArtifactSource(artifact);
        return {
          ...artifact,
          href: artifact.href ?? artifact.downloadHref ?? preview?.urls[artifact.artifact_id] ?? null,
          downloadHref: artifact.downloadHref ?? artifact.href ?? null,
          nativeTransfer,
        };
      }),
    [media, preview?.urls, actions],
  );
  const selectedArtifact = resolvedMedia.find((artifact) => artifact.artifact_id === selectedArtifactId) ?? null;

  if (!media.length) {
    return <p className="text-sm text-muted-foreground">{emptyMessage}</p>;
  }

  return (
    <>
      <div className="grid grid-cols-[repeat(auto-fill,minmax(min(100%,--spacing(36)),1fr))] gap-2" data-testid="media-gallery">
        {resolvedMedia.map((artifact) => (
          <MediaGalleryCard
            artifact={artifact}
            key={artifact.artifact_id}
            onInspect={() => setSelectedArtifactId(artifact.artifact_id)}
            onNativeOpen={() => void openArtifact(artifact)}
            onNativeDownload={() => void downloadArtifact(artifact)}
            nativeOpenBusy={nativeOpenArtifactId === artifact.artifact_id}
            nativeOpenError={nativeOpenError?.artifactId === artifact.artifact_id ? nativeOpenError.message : null}
            nativeDownloadBusy={nativeDownloadArtifactId === artifact.artifact_id}
            nativeDownloadError={nativeDownloadError?.artifactId === artifact.artifact_id ? nativeDownloadError.message : null}
            onRetry={() => preview?.retry(artifact.artifact_id)}
            previewError={preview?.errors[artifact.artifact_id] ?? null}
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
        <DialogContent className="sm:max-w-5xl">
          {selectedArtifact ? (
            <div className="grid gap-4">
              <DialogHeader>
                <DialogTitle className="truncate" title={mediaArtifactTitle(selectedArtifact)}>
                  {mediaArtifactTitle(selectedArtifact)}
                </DialogTitle>
                <DialogDescription className="truncate" title={selectedArtifact.displayPath}>
                  {selectedArtifact.displayPath}
                </DialogDescription>
              </DialogHeader>
              <div className="overflow-hidden rounded-lg border bg-muted/20 p-2">
                {selectedArtifact.href ? (
                  selectedArtifact.kind === "image" ? (
                    <img
                      alt={mediaArtifactTitle(selectedArtifact)}
                      className="max-h-[70vh] w-full object-contain"
                      src={selectedArtifact.href}
                    />
                  ) : selectedArtifact.kind === "audio" ? (
                    // biome-ignore lint/a11y/useMediaCaption: operator audio recordings have no caption sidecars.
                    <audio className="w-full" controls preload="metadata" src={selectedArtifact.href} />
                  ) : (
                    // biome-ignore lint/a11y/useMediaCaption: artifact videos do not have caption sidecars in the current media contract.
                    <video autoPlay className="max-h-[70vh] w-full" controls preload="metadata" src={selectedArtifact.href} />
                  )
                ) : selectedArtifact.nativeTransfer && mediaBlobType(selectedArtifact) ? (
                  <div className="grid justify-items-center gap-3 p-8 text-center text-sm text-muted-foreground">
                    {preview?.errors[selectedArtifact.artifact_id] ? (
                      <ErrorCard failure={failureOf(preview?.failures?.[selectedArtifact.artifact_id], "Media Preview Unavailable")} />
                    ) : (
                      <p>Loading media preview…</p>
                    )}
                    {preview?.errors[selectedArtifact.artifact_id] ? (
                      <Button size="sm" type="button" variant="outline" onClick={() => preview?.retry(selectedArtifact.artifact_id)}>
                        Retry Preview
                      </Button>
                    ) : null}
                  </div>
                ) : (
                  <p className="p-8 text-center text-sm text-muted-foreground">Preview unavailable for this media artifact.</p>
                )}
              </div>
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div className="text-sm text-muted-foreground">
                  {selectedArtifact.group_label ? artifactLabel(selectedArtifact.group_label) : "Media Artifact"}
                  {selectedArtifact.size_bytes ? ` • ${formatArtifactSize(selectedArtifact.size_bytes)}` : ""}
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
                          nativeOpenArtifactId === selectedArtifact.artifact_id || nativeDownloadArtifactId === selectedArtifact.artifact_id
                        }
                        size="sm"
                        type="button"
                        variant="outline"
                        onClick={() => void openArtifact(selectedArtifact)}
                      >
                        <ExternalLink data-icon="inline-start" size={14} />
                        {nativeOpenArtifactId === selectedArtifact.artifact_id ? "Opening…" : "Open"}
                      </Button>
                      <Button
                        disabled={
                          nativeOpenArtifactId === selectedArtifact.artifact_id || nativeDownloadArtifactId === selectedArtifact.artifact_id
                        }
                        size="sm"
                        type="button"
                        variant="outline"
                        onClick={() => void downloadArtifact(selectedArtifact)}
                      >
                        <Download data-icon="inline-start" size={14} />
                        {nativeDownloadArtifactId === selectedArtifact.artifact_id ? "Saving…" : "Download"}
                      </Button>
                    </>
                  ) : null}
                  {nativeOpenError?.artifactId === selectedArtifact.artifact_id ? (
                    <NotificationMessage failure={nativeOpenError.failure}>{nativeOpenError.message}</NotificationMessage>
                  ) : null}
                  {nativeDownloadError?.artifactId === selectedArtifact.artifact_id ? (
                    <NotificationMessage failure={nativeDownloadError.failure}>{nativeDownloadError.message}</NotificationMessage>
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
  const [failedImageHref, setFailedImageHref] = useState<string | null>(null);
  const cardRef = useRef<HTMLElement | null>(null);
  const title = mediaArtifactTitle(artifact);
  const meta = artifact.group_label ? artifactLabel(artifact.group_label) : formatArtifactSize(artifact.size_bytes);
  const showImagePreview = artifact.kind === "image" && Boolean(artifact.href) && artifact.href !== failedImageHref;

  useEffect(() => {
    const element = cardRef.current;
    if (!element) return;
    if (typeof IntersectionObserver === "undefined") {
      onVisibilityChange(artifact.artifact_id, true);
      return;
    }
    const observer = new IntersectionObserver(([entry]) => onVisibilityChange(artifact.artifact_id, entry?.isIntersecting ?? false), {
      rootMargin: "160px",
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, [artifact.artifact_id, onVisibilityChange]);

  return (
    <article className="grid min-w-0 overflow-hidden rounded-lg border bg-background" ref={cardRef}>
      <div className="flex aspect-video min-w-0 flex-col overflow-hidden bg-muted/45">
        <div className="relative flex min-h-0 min-w-0 flex-1 items-center justify-center overflow-hidden">
          {showImagePreview ? (
            <button aria-label={title} className="absolute inset-0" type="button" onClick={onInspect}>
              <img
                alt={title}
                className="h-full w-full object-cover"
                decoding="async"
                loading="lazy"
                src={artifact.href ?? undefined}
                onError={() => setFailedImageHref(artifact.href)}
              />
            </button>
          ) : artifact.href && artifact.kind === "audio" ? (
            // biome-ignore lint/a11y/useMediaCaption: operator recordings have no caption sidecars.
            <audio aria-label={title} className="w-full min-w-0 px-2" controls preload="metadata" src={artifact.href} />
          ) : artifact.href && artifact.kind === "video" ? (
            // biome-ignore lint/a11y/useMediaCaption: captured videos have no caption sidecars.
            <video aria-label={title} className="h-full w-full object-contain" controls preload="metadata" src={artifact.href} />
          ) : (
            <span className="flex min-w-0 items-center justify-center p-3">
              <MediaKindIcon className="text-muted-foreground" kind={artifact.kind} size={18} />
            </span>
          )}
        </div>
        <button
          className="flex shrink-0 items-center justify-end gap-1 px-2 py-1 text-xs font-semibold"
          title={artifact.displayPath}
          type="button"
          onClick={onInspect}
        >
          <Search size={12} />
          Inspect
        </button>
      </div>
      <div className="grid min-w-0 gap-1 px-2 py-2">
        <div className="flex min-w-0 items-start gap-2">
          <MediaKindIcon className="mt-1 shrink-0" kind={artifact.kind} size={14} />
          <div className="min-w-0 flex-1">
            <h3 className="truncate text-xs font-semibold" title={title}>
              {title}
            </h3>
            <p className="truncate text-xs text-muted-foreground" title={artifact.displayPath}>
              {artifact.displayPath}
            </p>
          </div>
        </div>
        <div className="flex min-w-0 items-center justify-between gap-2">
          {meta ? <span className="truncate text-xs text-muted-foreground">{meta}</span> : <span />}
          {previewError ? (
            <Button className="h-auto px-1 py-0 text-xs" size="sm" type="button" variant="ghost" onClick={onRetry}>
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
                  aria-label={nativeOpenError ? `Retry Open ${title}` : `Open ${title}`}
                  disabled={nativeOpenBusy || nativeDownloadBusy}
                  size="icon-xs"
                  title={nativeOpenError ?? "Open Artifact"}
                  type="button"
                  variant="ghost"
                  onClick={onNativeOpen}
                >
                  <ExternalLink size={13} />
                </Button>
                <Button
                  aria-label={nativeDownloadError ? `Retry Download ${title}` : `Download ${title}`}
                  disabled={nativeOpenBusy || nativeDownloadBusy}
                  size="icon-xs"
                  title={nativeDownloadError ?? "Download Artifact"}
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

function MediaKindIcon({ className, kind, size }: { className: string; kind: MediaArtifactKind; size: number }) {
  if (kind === "image") return <ImageIcon className={className} size={size} />;
  if (kind === "audio") return <AudioLines className={className} size={size} />;
  return <Video className={className} size={size} />;
}

function mediaArtifactTitle(artifact: Pick<ResolvedMediaGalleryArtifact, "displayPath" | "label">): string {
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

/* === Hooks === */
export function useMediaGallery(artifacts: MediaGalleryArtifact[], runDir?: string | null) {
  const [selectedArtifactId, setSelectedArtifactId] = useState<string | null>(null);
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
      media.filter((artifact) => visibleArtifactIds.has(artifact.artifact_id)).map((artifact) => artifact.artifact_id),
    );
    if (selectedArtifactId) requested.add(selectedArtifactId);
    return requested;
  }, [media, selectedArtifactId, visibleArtifactIds]);
  return { media, selectedArtifactId, setSelectedArtifactId, markVisible, requestedArtifactIds };
}

/* === Public Functions === */

/* === Private Functions === */
