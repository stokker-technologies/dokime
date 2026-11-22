// Dokime — SPDX-License-Identifier: Apache-2.0
import { untrack } from "svelte";
import { SvelteMap } from "svelte/reactivity";
import { failureOf, type NotificationFailure } from "../lib/notification-details";
import { artifactContentKey, isNativeArtifactSource, mediaBlobType, type MediaGalleryArtifact } from "./media-gallery-model";

export type MediaTransfers = {
  generation: string;
  isCurrent(): boolean;
  open(artifact: MediaGalleryArtifact): Promise<unknown>;
  download(artifact: MediaGalleryArtifact): Promise<unknown>;
  prepare(artifact: MediaGalleryArtifact): Promise<string>;
};

type Request = { artifact: MediaGalleryArtifact; key: string; transfers: MediaTransfers };
type Preview = { url: string; failure?: never } | { url?: never; failure: NotificationFailure };

const MAX_ACTIVE_MEDIA_DOWNLOADS = 2;

export function mediaPreviews(
  getArtifacts: () => readonly MediaGalleryArtifact[],
  getRequested: () => ReadonlySet<string>,
  getTransfers: () => MediaTransfers | undefined,
) {
  const previews = new SvelteMap<string, Preview>();
  const active = new Map<string, Request>();
  let desired = new Map<string, Request>();

  function pump() {
    for (const [id, request] of desired) {
      if (active.size >= MAX_ACTIVE_MEDIA_DOWNLOADS) {
        return;
      }
      if (active.has(id) || previews.has(id) || !request.transfers.isCurrent()) {
        continue;
      }
      active.set(id, request);
      void load(id, request);
    }
  }

  async function load(id: string, request: Request) {
    let preview: Preview;
    try {
      const url = await request.transfers.prepare(request.artifact);
      preview = { url };
    } catch (error) {
      preview = { failure: failureOf(error, "Media Preview Unavailable") };
    }
    if (desired.get(id) === request && request.transfers.isCurrent()) {
      previews.set(id, preview);
    }
    active.delete(id);
    pump();
  }

  $effect(() => {
    const transfers = getTransfers();
    const artifacts = getArtifacts();
    const requested = getRequested();
    const next = new Map<string, Request>();
    if (transfers) {
      for (const artifact of artifacts) {
        const id = artifact.artifact_id;
        if (!requested.has(id) || artifact.href || artifact.downloadHref || !isNativeArtifactSource(artifact) || !mediaBlobType(artifact)) {
          continue;
        }
        const key = artifactContentKey(artifact);
        const previous = desired.get(id);
        next.set(id, previous?.key === key && previous.transfers.generation === transfers.generation ? previous : { artifact, key, transfers });
      }
    }
    untrack(() => {
      for (const [id, request] of desired) {
        if (next.get(id) !== request) {
          previews.delete(id);
        }
      }
      desired = next;
      pump();
    });
  });

  $effect(() => () => desired.clear());

  return {
    get(id: string) {
      return previews.get(id);
    },
    retry(id: string) {
      previews.delete(id);
      pump();
    },
  };
}
