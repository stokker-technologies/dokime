/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Native Response Projection
 */

/* === Imports === */

import type { Wire } from "@dokime/plots/lib/rpc/canonical";

import type { ArtifactRecord, ExecutionFileLifecycle } from "@dokime/plots/lib/types";

import * as hostModel from "@host/model";

import { nowIso, safeNumber, timestampIso } from "@dokime/plots/lib/api/native-values";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type ArtifactMetadata = Wire<hostModel.ArtifactsListResponse>["artifacts"][number];

export type ExecutionFileMetadata = Wire<hostModel.ExecutionFilesListResponse>["files"][number];

type ExecutionFileKind = ExecutionFileMetadata["kind"];

export type ExecutionFileState = ExecutionFileMetadata["state"];

type ArtifactState = ArtifactMetadata["state"];

export type NativeArtifactUploadOptions = {
  file: Blob;
  filename: string;
  label?: string | null;
  description?: string | null;
  mediaType?: string | null;
  scope?: ArtifactRecord["scope"];
  groupId?: string | null;
  groupLabel?: string | null;
};

/* === Classes === */

/* === Public Functions === */

export function mapArtifactMetadata(metadata: ArtifactMetadata, overrides?: NativeArtifactUploadOptions): ArtifactRecord {
  const scope = metadata.scope ?? null;
  const filename = metadata.filename || overrides?.filename || metadata.artifactId;
  const contentType = overrides?.mediaType ?? metadata.contentType;
  return {
    artifact_id: metadata.artifactId,
    label: overrides?.label ?? filename,
    path: filename,
    href: null,
    downloadHref: null,
    media_type: contentType ?? null,
    content_kind: mapContentKind(contentType, filename),
    stream_state: mapArtifactState(metadata.state),
    description: metadata.error ?? overrides?.description ?? null,
    size_bytes: safeNumber(metadata.sizeBytes, "Artifact Metadata size"),
    group_id: overrides?.groupId ?? "cycle-artifacts",
    group_label: overrides?.groupLabel ?? "Cycle Artifacts",
    supersedes_artifact_id: null,
    created_at: timestampIso(metadata.createdAt) ?? nowIso(),
    scope: overrides?.scope ?? scope,
  };
}

export function mapExecutionFile(executionId: string, file: ExecutionFileMetadata): ArtifactRecord {
  const createdAt = timestampIso(file.createdAt) ?? nowIso();
  return {
    artifact_id: `${executionId}:${file.relativePath}`,
    label: file.relativePath.split("/").at(-1) || file.relativePath,
    path: file.relativePath,
    href: null,
    downloadHref: null,
    media_type: file.contentType ?? null,
    content_kind: mapFileKind(file.kind),
    stream_state: mapFileState(file.state),
    description: "Execution folder file",
    size_bytes: safeNumber(file.sizeBytes, "Execution File Metadata size"),
    group_id: "execution-files",
    group_label: "Execution Files",
    created_at: createdAt,
    scope: { executionId: executionId },
  };
}

export function mapFileState(state: ExecutionFileState): ExecutionFileLifecycle {
  switch (state) {
    case hostModel.ExecutionFileState.STREAMING:
      return "streaming";
    case hostModel.ExecutionFileState.FINALIZED:
      return "finalized";
    case hostModel.ExecutionFileState.INTERRUPTED:
      return "interrupted";
    default:
      return "static";
  }
}

/* === Private Functions === */

function mapArtifactState(state: ArtifactState): "static" | "streaming" | "finalized" | "interrupted" {
  switch (state) {
    case hostModel.ArtifactState.COMPLETE:
      return "finalized";
    case hostModel.ArtifactState.CANCELLED:
    case hostModel.ArtifactState.REJECTED:
    case hostModel.ArtifactState.FAILED:
      return "interrupted";
    default:
      return "static";
  }
}

function mapContentKind(contentType: string | undefined, filename: string): "text" | "binary" | "media" | "unknown" {
  const normalized = contentType?.toLowerCase() ?? "";
  if (
    normalized.startsWith("text/") ||
    ["application/json", "application/ndjson", "application/xml"].includes(normalized) ||
    /\.(csv|jsonl?|log|md|toml|txt|xml|ya?ml)$/iu.test(filename)
  ) {
    return "text";
  }
  if (normalized.startsWith("image/") || normalized.startsWith("audio/") || normalized.startsWith("video/")) {
    return "media";
  }
  return "binary";
}

function mapFileKind(kind: ExecutionFileKind): "text" | "binary" | "media" | "unknown" {
  switch (kind) {
    case hostModel.ExecutionFileKind.TEXT:
      return "text";
    case hostModel.ExecutionFileKind.BINARY:
      return "binary";
    case hostModel.ExecutionFileKind.MEDIA:
      return "media";
    default:
      return "unknown";
  }
}

/* === Entry Point === */
