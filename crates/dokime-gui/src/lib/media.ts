/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Types === */

export type MediaArtifactKind = "image" | "video" | "audio";

export type MediaArtifactCandidate = {
  label?: string | null;
  path?: string | null;
  media_type?: string | null;
};

/* === Constants === */

const IMAGE_EXTENSIONS = new Set([".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".tif", ".tiff", ".svg"]);

const VIDEO_EXTENSIONS = new Set([".mp4", ".webm", ".mov", ".avi", ".mkv", ".m4v"]);

const AUDIO_EXTENSIONS = new Set([".wav", ".mp3", ".ogg", ".oga", ".opus", ".flac", ".m4a", ".aac"]);

/* === Public Functions === */

export function mediaArtifactKind(artifactOrPath: MediaArtifactCandidate | string, mediaType?: string | null): MediaArtifactKind | null {
  const path = typeof artifactOrPath === "string" ? artifactOrPath : artifactOrPath.path || artifactOrPath.label || "";
  const extension = artifactExtension(path);

  if (IMAGE_EXTENSIONS.has(extension)) {
    return "image";
  }
  if (VIDEO_EXTENSIONS.has(extension)) {
    return "video";
  }
  if (AUDIO_EXTENSIONS.has(extension)) {
    return "audio";
  }

  const normalizedType =
    (typeof artifactOrPath === "string" ? mediaType : (artifactOrPath.media_type ?? mediaType))?.trim().toLowerCase() ?? "";
  if (normalizedType.startsWith("image/")) {
    return "image";
  }
  if (normalizedType.startsWith("video/")) {
    return "video";
  }
  if (normalizedType.startsWith("audio/")) {
    return "audio";
  }

  return null;
}

export function isMediaArtifact(artifactOrPath: MediaArtifactCandidate | string, mediaType?: string | null): boolean {
  return mediaArtifactKind(artifactOrPath, mediaType) !== null;
}

/* === Private Functions === */

function artifactExtension(path: string): string {
  const clean = path.split(/[?#]/u)[0] ?? "";
  const filename = clean.split(/[\\/]/u).at(-1) ?? clean;
  const index = filename.lastIndexOf(".");
  return index >= 0 ? filename.slice(index).toLowerCase() : "";
}
