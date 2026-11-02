/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Portable Media Gallery Values And Pure Artifact Computations.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

import type { ArtifactSource } from "@/lib/desktop/delivery";
// Local Imports
import type { MediaArtifactKind } from "@/lib/media";
import type { RecordScope } from "@/lib/types";

/* === Re-Exports === */

/* === Constants === */
const SAFE_IMAGE_TYPES = new Set(["image/avif", "image/bmp", "image/gif", "image/jpeg", "image/png", "image/webp"]);
const SAFE_VIDEO_TYPES = new Set(["video/mp4", "video/mpeg", "video/ogg", "video/webm"]);
const SAFE_AUDIO_TYPES = new Set(["audio/flac", "audio/mp4", "audio/mpeg", "audio/ogg", "audio/wav", "audio/webm"]);

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

export type MediaGalleryProps = {
  artifacts: MediaGalleryArtifact[];
  emptyMessage: string;
  runDir?: string | null;
};

export type ResolvedMediaGalleryArtifact = MediaGalleryArtifact & {
  displayPath: string;
  kind: MediaArtifactKind;
  href: string | null;
  nativeTransfer: boolean;
};

export type MediaArtifactCandidate = MediaGalleryArtifact & {
  displayPath: string;
  kind: MediaArtifactKind;
};

export type MediaPreviewState = {
  errors: Record<string, string>;
  failures?: Record<string, unknown>;
  retry(artifactId: string): void;
  urls: Record<string, string>;
};

/* === Classes === */

/* === Public Functions === */
export function artifactContentKey(artifact: MediaGalleryArtifact): string {
  return [
    artifact.artifact_id,
    artifact.group_id ?? "",
    artifact.path,
    artifact.media_type ?? "",
    artifact.size_bytes ?? "",
    artifact.scope?.test_execution_id ?? "",
  ].join("\u0000");
}

export function mediaBlobType(artifact: MediaGalleryArtifact): string | null {
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
  if (extension === ".wav") return "audio/wav";
  if (extension === ".mp3") return "audio/mpeg";
  if (extension === ".flac") return "audio/flac";
  if (extension === ".m4a") return "audio/mp4";
  return null;
}

function safeMediaType(value: string): boolean {
  return SAFE_IMAGE_TYPES.has(value) || SAFE_VIDEO_TYPES.has(value) || SAFE_AUDIO_TYPES.has(value);
}

export function isNativeArtifactSource(artifact: MediaGalleryArtifact): boolean {
  if (artifact.group_id === "execution-files") {
    return Boolean(artifact.scope?.test_execution_id?.trim() && artifact.path.trim());
  }
  return true;
}

/** Route registry artifacts and execution-file snapshots to their distinct verified pull source. */
export function mediaArtifactSource(artifact: MediaGalleryArtifact): ArtifactSource {
  if (artifact.group_id === "execution-files") {
    const executionId = artifact.scope?.test_execution_id?.trim();
    if (!executionId) {
      throw new Error("Execution media requires a test execution scope.");
    }
    return { kind: "execution_file", executionId, relativePath: artifact.path };
  }
  return { kind: "artifact", artifactId: artifact.artifact_id };
}

/* === Private Functions === */

/* === Entry Point === */
