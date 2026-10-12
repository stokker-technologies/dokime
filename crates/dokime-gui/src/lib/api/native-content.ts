/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native content adapters for the original routed UI. */

/* === External Imports === */

import { create } from "@bufbuild/protobuf";

/* === Local Imports === */

import type { ArtifactSource } from "@/lib/desktop/delivery";
import { openNativeArtifactUpload } from "@/lib/rpc/artifact-upload";
import * as rpc from "@/lib/rpc/generated/client";
import {
  type ArtifactCursor,
  type ArtifactFrame,
  type ArtifactMetadata,
  type ArtifactScope,
  ArtifactScopeSchema,
  ArtifactState,
  type ExecutionFileDownloadFrame,
  type ExecutionFileKind,
  ExecutionFileKindSchema,
  type ExecutionFileMetadata,
  ExecutionFileState,
  type ExecutionFilesCursor,
} from "@/lib/rpc/generated/dokime/v1/content_pb";
import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import type { ArtifactRecord, ExecutionFileTextRead } from "@/lib/types";
import { deliverNativeArtifact } from "./native-delivery";
import {
  enumTail,
  NATIVE_MAX_PAGES,
  NativeApiLimitError,
  nowIso,
  safeNumber,
  sha256Hex,
  timestampIso,
} from "./native-values";

const NATIVE_FILE_WINDOW_BYTES = 64 * 1024;
const NATIVE_CONTENT_BYTES = 2 * 1024 * 1024 * 1024;
const NATIVE_CONTENT_CHUNK_BYTES = 1024 * 1024;
const NATIVE_UPLOAD_SLICE_BYTES = 1024 * 1024;

export interface NativeContentSession extends SessionIdentity {
  own(dispose: () => void): () => void;
}

/** Add generation ownership to a connected session for a content transfer. */
export function nativeContentSession(
  session: SessionIdentity,
  own: NativeContentSession["own"],
): NativeContentSession {
  return { ...session, own };
}

export async function fetchNativeExecutionFiles(
  session: SessionIdentity,
  executionId: string,
): Promise<ArtifactRecord[]> {
  const files: ExecutionFileMetadata[] = [];
  let afterPath = "";
  let cursor: ExecutionFilesCursor | undefined;
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.contentListExecutionFiles(session, {
      executionId,
      limit: 100,
      afterPath,
      cursor,
    });
    files.push(...reply.files);
    const next = reply.nextAfterPath;
    if (!next) return files.map((file) => mapExecutionFile(executionId, file));
    if (next === afterPath) {
      throw new NativeApiLimitError(
        "Content.ListExecutionFiles",
        `cursor did not advance at ${next}`,
      );
    }
    afterPath = next;
    cursor = reply.cursor;
  }
  throw new NativeApiLimitError(
    "Content.ListExecutionFiles",
    `more than ${NATIVE_MAX_PAGES} pages`,
  );
}

export async function fetchNativeExecutionFileText(
  session: SessionIdentity,
  executionId: string,
  path: string,
  options: { offset?: number; tailBytes?: number; maxBytes?: number } = {},
): Promise<ExecutionFileTextRead> {
  const maxBytes = options.maxBytes ?? NATIVE_FILE_WINDOW_BYTES;
  if (!Number.isSafeInteger(maxBytes) || maxBytes < 0 || maxBytes > NATIVE_FILE_WINDOW_BYTES) {
    throw new NativeApiLimitError(
      "Content.ReadExecutionFile",
      `max_bytes must be between 0 and ${NATIVE_FILE_WINDOW_BYTES}`,
    );
  }
  let offset = options.offset ?? 0;
  if (options.tailBytes !== undefined) {
    if (!Number.isSafeInteger(options.tailBytes) || options.tailBytes < 0) {
      throw new NativeApiLimitError("Content.ReadExecutionFile", "tail_bytes is invalid");
    }
    const files = await fetchNativeExecutionFiles(session, executionId);
    const file = files.find((item) => item.path === path);
    if (!file) throw new Error(`Execution file ${path} was not found.`);
    offset = Math.max(0, (file.size_bytes ?? 0) - options.tailBytes);
  }
  if (!Number.isSafeInteger(offset) || offset < 0) {
    throw new NativeApiLimitError("Content.ReadExecutionFile", "offset is invalid");
  }
  if (maxBytes === 0) {
    return {
      text: "",
      offset,
      sizeBytes: offset,
      nextOffset: offset,
      truncated: false,
      rotated: false,
    };
  }
  const window = await rpc.contentReadExecutionFile(session, {
    executionId,
    relativePath: path,
    offset: BigInt(offset),
    maxBytes,
  });
  const observedSize = safeNumber(window.observedSizeBytes, "Content.ReadExecutionFile size");
  const actualOffset = safeNumber(window.offset, "Content.ReadExecutionFile offset");
  const text = new TextDecoder().decode(window.data);
  const byteLength = window.data.byteLength;
  const nextOffset = actualOffset + byteLength;
  return {
    text,
    offset: actualOffset,
    sizeBytes: observedSize,
    nextOffset,
    truncated: nextOffset < observedSize,
    rotated: options.offset !== undefined && actualOffset < options.offset,
  };
}

/**
 * Read a static artifact through the native server stream. The original UI
 * uses this for report and operator artifacts that are identified by an
 * admitted artifact id; no public URL is constructed.
 */
export async function fetchNativeArtifactText(
  session: NativeContentSession,
  artifactId: string,
  signal?: AbortSignal,
): Promise<string> {
  const bytes = await fetchNativeArtifactBytes(session, artifactId, signal);
  return new TextDecoder().decode(bytes);
}

/** Download and verify one static artifact without routing bytes through HTTP. */
export async function fetchNativeArtifactBytes(
  session: NativeContentSession,
  artifactId: string,
  signal?: AbortSignal,
): Promise<Uint8Array> {
  const id = artifactId.trim();
  if (!id) throw new Error("Content.DownloadArtifact requires an artifact ID.");
  let reader: NativeStream<ArtifactFrame> | undefined;
  let release: (() => void) | undefined;
  let cancelPromise: Promise<void> | undefined;
  const cancel = async (): Promise<void> => {
    if (!reader) return;
    cancelPromise ??= reader.cancel();
    await cancelPromise;
  };
  if (signal?.aborted || !session.isCurrent()) {
    throw new Error("Artifact transfer cancelled.");
  }
  const abort = () => void cancel().catch(() => undefined);
  signal?.addEventListener("abort", abort, { once: true });
  try {
    reader = await rpc.contentDownloadArtifact(session, { artifactId: id });
    if (signal?.aborted || !session.isCurrent()) {
      await cancel();
      throw new Error("Artifact transfer cancelled.");
    }
    release = session.own(() => void cancel().catch(() => undefined));
    return await assembleArtifactBytes(reader, id, () => !signal?.aborted && session.isCurrent());
  } finally {
    signal?.removeEventListener("abort", abort);
    await cancel().catch(() => undefined);
    release?.();
  }
}

/** Download and verify one immutable execution-file snapshot through native IPC. */
export async function fetchNativeExecutionFileBytes(
  session: NativeContentSession,
  executionId: string,
  path: string,
  signal?: AbortSignal,
): Promise<Uint8Array> {
  const id = executionId.trim();
  const relativePath = path;
  if (!id) throw new Error("Content.DownloadExecutionFile requires an execution ID.");
  if (!relativePath.trim())
    throw new Error("Content.DownloadExecutionFile requires a relative path.");
  let reader: NativeStream<ExecutionFileDownloadFrame> | undefined;
  let release: (() => void) | undefined;
  let cancelPromise: Promise<void> | undefined;
  const cancel = async (): Promise<void> => {
    if (!reader) return;
    cancelPromise ??= reader.cancel();
    await cancelPromise;
  };
  if (signal?.aborted || !session.isCurrent()) {
    throw new Error("Execution-file transfer cancelled.");
  }
  const abort = () => void cancel().catch(() => undefined);
  signal?.addEventListener("abort", abort, { once: true });
  try {
    reader = await rpc.contentDownloadExecutionFile(session, {
      executionId: id,
      relativePath,
    });
    if (signal?.aborted || !session.isCurrent()) {
      await cancel();
      throw new Error("Execution-file transfer cancelled.");
    }
    release = session.own(() => void cancel().catch(() => undefined));
    return await assembleExecutionFileBytes(
      reader,
      relativePath,
      () => !signal?.aborted && session.isCurrent(),
    );
  } finally {
    signal?.removeEventListener("abort", abort);
    await cancel().catch(() => undefined);
    release?.();
  }
}

type NativeContentArtifact = Pick<
  ArtifactRecord,
  "artifact_id" | "group_id" | "label" | "path" | "scope"
>;

/** Route registry artifacts and execution-file snapshots to their distinct native RPCs. */
export async function fetchNativeContentBytes(
  session: NativeContentSession,
  artifact: Pick<ArtifactRecord, "artifact_id" | "group_id" | "path" | "scope">,
  signal?: AbortSignal,
): Promise<Uint8Array> {
  if (artifact.group_id === "execution-files") {
    const reference = nativeExecutionFileReference(artifact);
    return fetchNativeExecutionFileBytes(
      session,
      reference.executionId,
      reference.relativePath,
      signal,
    );
  }
  return fetchNativeArtifactBytes(session, artifact.artifact_id, signal);
}

/**
 * Save one verified native content item through the native file picker.
 *
 * The content never crosses the webview: `desktop_artifact_deliver` pulls it
 * straight from the server into a verified cache file and hands that path to
 * the save dialog, so this scales past the 32 MiB `saveNativeDocument` bound.
 */
export async function downloadNativeArtifact(
  session: NativeContentSession,
  artifact: NativeContentArtifact,
  signal?: AbortSignal,
): Promise<boolean> {
  if (signal?.aborted) throw new Error("Artifact transfer cancelled.");
  return deliverNativeArtifact(session, {
    source: nativeArtifactSource(artifact),
    kind: "artifact",
    filename: nativeArtifactFilename(artifact),
    action: "save",
  });
}

/** Open one verified native content item through the native cache and OS opener. */
export async function openNativeArtifact(
  session: NativeContentSession,
  artifact: NativeContentArtifact,
  signal?: AbortSignal,
): Promise<boolean> {
  if (signal?.aborted) throw new Error("Artifact transfer cancelled.");
  return deliverNativeArtifact(session, {
    source: nativeArtifactSource(artifact),
    kind: "artifact",
    filename: nativeArtifactFilename(artifact),
    action: "open",
  });
}

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

export async function uploadNativeArtifact(
  session: NativeContentSession,
  options: NativeArtifactUploadOptions,
): Promise<ArtifactRecord> {
  const scope = uploadScope(options.scope);
  const filename = options.filename.trim();
  if (!filename || new TextEncoder().encode(filename).byteLength > 255) {
    throw new Error("Native artifact filename must contain 1–255 bytes.");
  }
  const contentType = options.mediaType?.trim() || options.file.type || undefined;
  if (contentType && new TextEncoder().encode(contentType).byteLength > 255) {
    throw new Error("Native artifact content type must not exceed 255 bytes.");
  }
  const file = options.file;
  if (!Number.isSafeInteger(file.size) || file.size < 0 || file.size > NATIVE_CONTENT_BYTES) {
    throw new NativeApiLimitError(
      "Content.UploadArtifact",
      `payload exceeds ${NATIVE_CONTENT_BYTES} bytes`,
    );
  }
  let upload: Awaited<ReturnType<typeof openNativeArtifactUpload>> | undefined;
  let release: (() => void) | undefined;
  let cancelPromise: Promise<void> | undefined;
  const cancel = async (): Promise<void> => {
    if (!upload) return;
    cancelPromise ??= upload.cancel();
    await cancelPromise;
  };
  try {
    // The webview never hashes or buffers the whole file: it slices the Blob
    // and streams raw bytes through the native sink, which hashes
    // incrementally and hands the verified digest back in a trailing
    // completion frame. Peak webview heap is therefore one 1 MiB slice, not
    // the whole (up to 2 GiB) upload.
    upload = await openNativeArtifactUpload(session, {
      scope,
      filename,
      ...(contentType ? { contentType } : {}),
      sizeBytes: BigInt(file.size),
    });
    release = session.own(() => void cancel().catch(() => undefined));
    for (let offset = 0; offset < file.size; offset += NATIVE_UPLOAD_SLICE_BYTES) {
      const slice = new Uint8Array(
        await file.slice(offset, offset + NATIVE_UPLOAD_SLICE_BYTES).arrayBuffer(),
      );
      await upload.write(slice);
    }
    return mapArtifactMetadata(await upload.finish(), options);
  } finally {
    await cancel().catch(() => undefined);
    release?.();
  }
}

export async function fetchNativeArtifacts(session: SessionIdentity): Promise<ArtifactRecord[]> {
  const artifacts: ArtifactMetadata[] = [];
  let afterArtifactId = "";
  let cursor: ArtifactCursor | undefined;
  for (let page = 0; page < NATIVE_MAX_PAGES; page += 1) {
    const response = await rpc.contentListArtifacts(session, {
      limit: 100,
      afterArtifactId,
      cursor,
    });
    artifacts.push(...response.artifacts);
    const next = response.nextAfterArtifactId;
    if (!next) return artifacts.map((artifact) => mapArtifactMetadata(artifact));
    if (next === afterArtifactId) {
      throw new NativeApiLimitError("Content.ListArtifacts", "cursor did not advance");
    }
    if (!response.cursor) {
      throw new Error("Content.ListArtifacts returned a continuation without a cursor.");
    }
    afterArtifactId = next;
    cursor = response.cursor;
  }
  throw new NativeApiLimitError("Content.ListArtifacts", `more than ${NATIVE_MAX_PAGES} pages`);
}

/**
 * Save a generated cycle ZIP through the native file picker.
 *
 * Unlike the deleted `fetchNativeCycleBundle`, the bundle's bytes never
 * cross the webview: `desktop_artifact_deliver` captures the cycle export
 * and streams it straight to the chosen destination, scaling past the
 * 32 MiB base64 JSON bound this used to share with HTML reports.
 */
export async function downloadNativeCycleBundle(
  session: NativeContentSession,
  cycleId: string,
  signal?: AbortSignal,
): Promise<void> {
  const id = cycleId.trim();
  if (!id) throw new Error("Content.DownloadCycleBundle requires a cycle ID.");
  if (signal?.aborted) throw new Error("Content.DownloadCycleBundle was cancelled.");
  await deliverNativeArtifact(session, {
    source: { kind: "cycle_bundle", cycleId: id },
    kind: "cycle_bundle",
    filename: `test-cycle-${id}.zip`,
    action: "save",
  });
}

export async function fetchNativeCycleFolderPath(
  session: SessionIdentity,
  cycleId: string,
): Promise<string> {
  const id = cycleId.trim();
  if (!id) throw new Error("Content.GetCycleFolderPath requires a cycle ID.");
  const response = await rpc.contentGetCycleFolderPath(session, { cycleId: id });
  const path = response.path.trim();
  if (!path) throw new Error("Content.GetCycleFolderPath returned an empty path.");
  return path;
}

/** Shared with `ArtifactUploader.tsx`'s native recording sink, which builds an
 * `UploadArtifactHeader` directly rather than through `uploadNativeArtifact`. */
export function uploadScope(scope: ArtifactRecord["scope"]): ArtifactScope {
  const cycleId = scope?.test_cycle_id?.trim();
  const executionId = scope?.test_execution_id?.trim();
  if (!cycleId || !executionId) {
    throw new Error("Native artifact upload requires a cycle and execution scope.");
  }
  const caseId = scope?.test_case_id?.trim();
  return create(ArtifactScopeSchema, {
    cycleId,
    ...(caseId ? { caseId } : {}),
    executionId,
  });
}

function mapArtifactMetadata(
  metadata: ArtifactMetadata,
  overrides?: NativeArtifactUploadOptions,
): ArtifactRecord {
  const scope = mapArtifactScope(metadata.scope);
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
    description: overrides?.description ?? null,
    size_bytes: safeNumber(metadata.sizeBytes, "Content.ArtifactMetadata size"),
    group_id: overrides?.groupId ?? "cycle-artifacts",
    group_label: overrides?.groupLabel ?? "Cycle Artifacts",
    supersedes_artifact_id: null,
    created_at: timestampIso(metadata.createdAt) ?? nowIso(),
    scope: overrides?.scope ?? scope,
  };
}

function nativeArtifactFilename(
  artifact: Pick<ArtifactRecord, "artifact_id" | "label" | "path">,
): string {
  const path = artifact.path.trim().replaceAll("\\", "/");
  const filename = path.split("/").filter(Boolean).at(-1)?.trim();
  return filename || artifact.label.trim() || artifact.artifact_id.trim();
}

function nativeExecutionFileReference(artifact: Pick<ArtifactRecord, "path" | "scope">): {
  executionId: string;
  relativePath: string;
} {
  const executionId = artifact.scope?.test_execution_id?.trim();
  const relativePath = artifact.path;
  if (!executionId || !relativePath.trim()) {
    throw new Error("Execution-file content requires a test execution scope and relative path.");
  }
  return { executionId, relativePath };
}

/** Route registry artifacts and execution-file snapshots to their distinct verified pull source. */
function nativeArtifactSource(
  artifact: Pick<ArtifactRecord, "artifact_id" | "group_id" | "path" | "scope">,
): ArtifactSource {
  if (artifact.group_id === "execution-files") {
    const reference = nativeExecutionFileReference(artifact);
    return {
      kind: "execution_file",
      executionId: reference.executionId,
      relativePath: reference.relativePath,
    };
  }
  return { kind: "artifact", artifactId: artifact.artifact_id };
}

function mapArtifactScope(scope?: ArtifactScope): ArtifactRecord["scope"] {
  if (!scope) return null;
  const caseId = scope.caseId ?? null;
  return {
    test_cycle_id: scope.cycleId ?? null,
    test_execution_id: scope.executionId ?? null,
    test_case_id: caseId,
  };
}

function mapArtifactState(
  state: ArtifactState,
): "static" | "streaming" | "finalized" | "interrupted" {
  switch (state) {
    case ArtifactState.COMPLETE:
      return "finalized";
    case ArtifactState.CANCELLED:
    case ArtifactState.REJECTED:
      return "interrupted";
    default:
      return "static";
  }
}

function mapContentKind(
  contentType: string | undefined,
  filename: string,
): "text" | "binary" | "media" | "unknown" {
  const normalized = contentType?.toLowerCase() ?? "";
  if (
    normalized.startsWith("text/") ||
    ["application/json", "application/ndjson", "application/xml"].includes(normalized) ||
    /\.(csv|jsonl?|log|md|toml|txt|xml|ya?ml)$/iu.test(filename)
  ) {
    return "text";
  }
  if (
    normalized.startsWith("image/") ||
    normalized.startsWith("audio/") ||
    normalized.startsWith("video/")
  ) {
    return "media";
  }
  return "binary";
}

function mapExecutionFile(executionId: string, file: ExecutionFileMetadata): ArtifactRecord {
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
    size_bytes: safeNumber(file.sizeBytes, "Content.ExecutionFileMetadata size"),
    group_id: "execution-files",
    group_label: "Execution Files",
    created_at: createdAt,
    scope: { test_execution_id: executionId },
  };
}

function mapFileKind(kind: ExecutionFileKind): "text" | "binary" | "media" | "unknown" {
  return enumTail(ExecutionFileKindSchema, kind, "unknown");
}

function mapFileState(
  state: ExecutionFileState,
): "static" | "streaming" | "finalized" | "interrupted" {
  switch (state) {
    case ExecutionFileState.STREAMING:
      return "streaming";
    case ExecutionFileState.FINALIZED:
      return "finalized";
    case ExecutionFileState.INTERRUPTED:
      return "interrupted";
    default:
      return "static";
  }
}

async function assembleArtifactBytes(
  reader: NativeStream<ArtifactFrame>,
  artifactId: string,
  active: () => boolean,
): Promise<Uint8Array> {
  let metadataSize: number | undefined;
  let expectedSha: string | undefined;
  let expectedOffset = 0;
  let completeSize: number | undefined;
  let completeSha: string | undefined;
  const chunks: Uint8Array[] = [];
  let total = 0;
  let completed = false;
  for await (const frame of reader) {
    if (!active()) throw new Error("Artifact transfer cancelled.");
    if (completed) throw new Error("Artifact transfer continued after completion.");
    switch (frame.payload.case) {
      case "metadata": {
        const metadata = frame.payload.value;
        if (metadata.artifactId !== artifactId || metadataSize !== undefined) {
          throw new Error("Artifact metadata identity is invalid.");
        }
        metadataSize = safeNumber(metadata.sizeBytes, "Content.ArtifactMetadata size");
        expectedSha = metadata.sha256.toLowerCase();
        if (!/^[0-9a-f]{64}$/u.test(expectedSha)) {
          throw new Error("Artifact metadata digest is invalid.");
        }
        if (metadataSize > NATIVE_CONTENT_BYTES)
          throw new NativeApiLimitError("Content.DownloadArtifact", "artifact is too large");
        break;
      }
      case "chunk": {
        if (metadataSize === undefined) {
          throw new Error("Artifact data arrived before metadata.");
        }
        const chunk = frame.payload.value;
        if (chunk.data.byteLength === 0 || chunk.data.byteLength > NATIVE_CONTENT_CHUNK_BYTES) {
          throw new Error("Artifact chunk size is invalid.");
        }
        const offset = safeNumber(chunk.offset, "Content.ArtifactChunk offset");
        if (offset !== expectedOffset) throw new Error("Artifact chunks are not contiguous.");
        total += chunk.data.byteLength;
        if (total > NATIVE_CONTENT_BYTES)
          throw new NativeApiLimitError("Content.DownloadArtifact", "artifact is too large");
        chunks.push(chunk.data);
        expectedOffset += chunk.data.byteLength;
        break;
      }
      case "complete": {
        if (metadataSize === undefined || completeSize !== undefined) {
          throw new Error("Artifact completed before metadata or twice.");
        }
        const completion = frame.payload.value;
        completeSize = safeNumber(completion.sizeBytes, "Content.ArtifactCompletion size");
        completeSha = completion.sha256.toLowerCase();
        if (!/^[0-9a-f]{64}$/u.test(completeSha)) {
          throw new Error("Artifact completion digest is invalid.");
        }
        completed = true;
        break;
      }
      default:
        throw new Error("Artifact transfer contained an unknown frame.");
    }
  }
  if (metadataSize === undefined || completeSize === undefined || !completed)
    throw new Error("Artifact transfer ended before metadata and completion.");
  if (metadataSize !== total || completeSize !== total || completeSha !== expectedSha)
    throw new Error("Artifact transfer size or digest does not match its metadata.");
  const bytes = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  const actualSha = await sha256Hex(bytes);
  if (!active()) throw new Error("Artifact transfer cancelled.");
  if (actualSha !== completeSha) throw new Error("Artifact digest verification failed.");
  return bytes;
}

async function assembleExecutionFileBytes(
  reader: NativeStream<ExecutionFileDownloadFrame>,
  relativePath: string,
  active: () => boolean,
): Promise<Uint8Array> {
  let metadataPath: string | undefined;
  let metadataSize: number | undefined;
  let expectedOffset = 0;
  let completeSize: number | undefined;
  let completeSha: string | undefined;
  let completed = false;
  const chunks: Uint8Array[] = [];
  let total = 0;
  for await (const frame of reader) {
    if (!active()) throw new Error("Content.DownloadExecutionFile was cancelled.");
    if (completed) throw new Error("Content.DownloadExecutionFile continued after completion.");
    switch (frame.payload.case) {
      case "metadata": {
        const metadata = frame.payload.value;
        if (metadataPath !== undefined || metadata.relativePath !== relativePath) {
          throw new Error("Content.DownloadExecutionFile metadata identity is invalid.");
        }
        metadataPath = metadata.relativePath;
        metadataSize = safeNumber(
          metadata.observedSizeBytes,
          "Content.DownloadExecutionFile observed size",
        );
        if (metadataSize > NATIVE_CONTENT_BYTES) {
          throw new NativeApiLimitError(
            "Content.DownloadExecutionFile",
            "execution file is too large",
          );
        }
        break;
      }
      case "chunk": {
        if (metadataSize === undefined) {
          throw new Error("Content.DownloadExecutionFile sent data before metadata.");
        }
        const chunk = frame.payload.value;
        if (chunk.data.byteLength === 0 || chunk.data.byteLength > NATIVE_CONTENT_CHUNK_BYTES) {
          throw new Error("Content.DownloadExecutionFile chunk size is invalid.");
        }
        const offset = safeNumber(chunk.offset, "Content.DownloadExecutionFile chunk offset");
        if (offset !== expectedOffset) {
          throw new Error("Content.DownloadExecutionFile chunks are not contiguous.");
        }
        total += chunk.data.byteLength;
        if (total > metadataSize || total > NATIVE_CONTENT_BYTES) {
          throw new NativeApiLimitError(
            "Content.DownloadExecutionFile",
            "execution file is too large",
          );
        }
        chunks.push(chunk.data);
        expectedOffset = total;
        break;
      }
      case "complete": {
        if (metadataSize === undefined || completeSize !== undefined) {
          throw new Error("Content.DownloadExecutionFile completed before metadata or twice.");
        }
        const completion = frame.payload.value;
        completeSize = safeNumber(completion.sizeBytes, "Content.DownloadExecutionFile size");
        if (completeSize > NATIVE_CONTENT_BYTES) {
          throw new NativeApiLimitError(
            "Content.DownloadExecutionFile",
            "execution file is too large",
          );
        }
        completeSha = completion.sha256.toLowerCase();
        if (!/^[0-9a-f]{64}$/u.test(completeSha)) {
          throw new Error("Content.DownloadExecutionFile completion digest is invalid.");
        }
        completed = true;
        break;
      }
      default:
        throw new Error("Content.DownloadExecutionFile contained an unknown frame.");
    }
  }
  if (metadataSize === undefined || completeSize === undefined || !completeSha || !completed) {
    throw new Error("Content.DownloadExecutionFile ended before metadata and completion.");
  }
  if (metadataSize !== total || completeSize !== total) {
    throw new Error("Content.DownloadExecutionFile size does not match its metadata.");
  }
  const bytes = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  const actualSha = await sha256Hex(bytes);
  if (!active()) throw new Error("Content.DownloadExecutionFile was cancelled.");
  if (actualSha !== completeSha) {
    throw new Error("Content.DownloadExecutionFile digest verification failed.");
  }
  return bytes;
}
