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

/* === Local Imports === */

import { UPLOAD_SLICE_BYTES as DESKTOP_UPLOAD_SLICE_BYTES } from "@generated/desktop-protocol";

import type { Row3 as ArtifactDownloadFrame } from "@generated/dokime/v1/ContentMessages/DownloadArtifact/messages_pb";
import type { Row3 as FileFollowFrame } from "@generated/dokime/v1/ContentMessages/FollowExecutionFile/messages_pb";
import type { ArtifactSource } from "@/lib/desktop/delivery";
import { EXECUTION_FILE_WINDOW_BYTES } from "@/lib/execution-file-paging";
import { openNativeArtifactUpload } from "@/lib/rpc/artifact-upload";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";
import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import type {
  ArtifactRecord,
  ExecutionFileFollowEvent,
  ExecutionFileIdentity,
  ExecutionFileLifecycle,
  ExecutionFileReadWindow,
} from "@/lib/types";
import { deliverNativeArtifact } from "./native-delivery";
import { NATIVE_MAX_PAGES, NativeApiLimitError, nowIso, safeNumber, sha256Hex, timestampIso } from "./native-values";

const GUI_CONTENT_BYTES = 2_147_483_648;
const GUI_SMALL_PAGE_ITEMS = 100;

/* === Canonical Types === */

type ArtifactScope = NonNullable<ArtifactMetadata["scope"]>;
type ArtifactFrame = Wire<ArtifactDownloadFrame>;
type ExecutionFileFollowFrame = Wire<FileFollowFrame>;
type ArtifactMetadata = Wire<Awaited<ReturnType<typeof rpc.contentListArtifacts>>>["artifacts"][number];
type ArtifactCursor = NonNullable<Wire<Awaited<ReturnType<typeof rpc.contentListArtifacts>>>["cursor"]>;
type ExecutionFileMetadata = Wire<Awaited<ReturnType<typeof rpc.contentListExecutionFiles>>>["files"][number];
type ExecutionFilesCursor = NonNullable<Wire<Awaited<ReturnType<typeof rpc.contentListExecutionFiles>>>["cursor"]>;
type ExecutionFileKind = ExecutionFileMetadata["kind"];
type ExecutionFileState = ExecutionFileMetadata["state"];
type ArtifactState = ArtifactMetadata["state"];

const NATIVE_CONTENT_BYTES = GUI_CONTENT_BYTES;
const NATIVE_UPLOAD_SLICE_BYTES = DESKTOP_UPLOAD_SLICE_BYTES;

export interface NativeContentSession extends SessionIdentity {
  own(dispose: () => void): () => void;
}

/** Add generation ownership to a connected session for a content transfer. */
export function nativeContentSession(session: SessionIdentity, own: NativeContentSession["own"]): NativeContentSession {
  return { ...session, own };
}

export async function fetchNativeExecutionFiles(session: SessionIdentity, executionId: string): Promise<ArtifactRecord[]> {
  const files: ExecutionFileMetadata[] = [];
  let afterPath = "";
  let cursor: ExecutionFilesCursor | undefined;
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.contentListExecutionFiles(session, {
      executionId,
      limit: GUI_SMALL_PAGE_ITEMS,
      afterPath,
      cursor,
    });
    files.push(...reply.files);
    const next = reply.nextAfterPath;
    if (!next) return files.map((file) => mapExecutionFile(executionId, file));
    if (next === afterPath) {
      throw new NativeApiLimitError("Content.ListExecutionFiles", `cursor did not advance at ${next}`);
    }
    afterPath = next;
    cursor = reply.cursor;
  }
  throw new NativeApiLimitError("Content.ListExecutionFiles", `more than ${NATIVE_MAX_PAGES} pages`);
}

/**
 * Read one bounded window of a registered execution file. Callers page with
 * `pageExecutionFile`, which keeps every request within the contract bound.
 */
export async function readNativeExecutionFileWindow(
  session: SessionIdentity,
  executionId: string,
  path: string,
  offset: number,
  maxBytes: number,
): Promise<ExecutionFileReadWindow> {
  if (!Number.isSafeInteger(maxBytes) || maxBytes < 1 || maxBytes > EXECUTION_FILE_WINDOW_BYTES) {
    throw new NativeApiLimitError("Content.ReadExecutionFile", `max_bytes must be between 1 and ${EXECUTION_FILE_WINDOW_BYTES}`);
  }
  if (!Number.isSafeInteger(offset) || offset < 0) {
    throw new NativeApiLimitError("Content.ReadExecutionFile", "offset is invalid");
  }
  const window = await rpc.contentReadExecutionFile(session, {
    executionId,
    relativePath: path,
    offset: BigInt(offset),
    maxBytes,
  });
  return {
    offset: safeNumber(window.offset, "Content.ReadExecutionFile offset"),
    observedSizeBytes: safeNumber(window.observedSizeBytes, "Content.ReadExecutionFile size"),
    data: window.data,
    state: mapFileState(window.state),
  };
}

/**
 * Follow a registered execution file from `offset`. A nonzero offset resumes
 * the retained content `identity` names; the first event is always metadata.
 * Cancelling the returned stream ends this subscription, never the producer.
 */
export async function followNativeExecutionFile(
  session: SessionIdentity,
  executionId: string,
  path: string,
  { identity, offset = 0 }: { identity?: ExecutionFileIdentity | null; offset?: number } = {},
): Promise<NativeStream<ExecutionFileFollowEvent>> {
  if (!Number.isSafeInteger(offset) || offset < 0 || (offset > 0 && !identity)) {
    throw new NativeApiLimitError("Content.FollowExecutionFile", "a resume offset requires the original identity");
  }
  const stream = await rpc.contentFollowExecutionFile(session, {
    executionId,
    relativePath: path,
    identity: identity
      ? {
          history: { store: identity.store, epoch: identity.epoch },
          fileId: identity.fileId,
          executionId: identity.executionId,
          relativePath: identity.relativePath,
        }
      : undefined,
    offset: BigInt(offset),
    maxBytes: EXECUTION_FILE_WINDOW_BYTES,
  });
  return {
    cancel: () => stream.cancel(),
    async *[Symbol.asyncIterator]() {
      for await (const frame of stream) {
        yield mapFollowFrame(frame);
      }
    },
  };
}

/**
 * Read a static artifact through the native server stream. The original UI
 * uses this for report and operator artifacts that are identified by an
 * admitted artifact id; no public URL is constructed.
 */
export async function fetchNativeArtifactText(session: NativeContentSession, artifactId: string, signal?: AbortSignal): Promise<string> {
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

type NativeContentArtifact = Pick<ArtifactRecord, "artifact_id" | "group_id" | "label" | "path" | "scope">;

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

export async function uploadNativeArtifact(session: NativeContentSession, options: NativeArtifactUploadOptions): Promise<ArtifactRecord> {
  const scope = uploadScope(options.scope);
  const filename = operatorUploadName(options.filename);
  if (!filename || new TextEncoder().encode(filename).byteLength > 255) {
    throw new Error("Native artifact filename must contain 1–255 bytes.");
  }
  const contentType = options.mediaType?.trim() || options.file.type || undefined;
  if (contentType && new TextEncoder().encode(contentType).byteLength > 255) {
    throw new Error("Native artifact content type must not exceed 255 bytes.");
  }
  const file = options.file;
  if (!Number.isSafeInteger(file.size) || file.size < 0 || file.size > NATIVE_CONTENT_BYTES) {
    throw new NativeApiLimitError("Content.UploadArtifact", `payload exceeds ${NATIVE_CONTENT_BYTES} bytes`);
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
      const slice = new Uint8Array(await file.slice(offset, offset + NATIVE_UPLOAD_SLICE_BYTES).arrayBuffer());
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
      limit: GUI_SMALL_PAGE_ITEMS,
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

/** Shared with `ArtifactUploader.tsx`'s native recording sink, which builds an
 * `UploadArtifactHeader` directly rather than through `uploadNativeArtifact`. */
export function uploadScope(scope: ArtifactRecord["scope"]): ArtifactScope {
  const cycleId = scope?.test_cycle_id?.trim();
  const executionId = scope?.test_execution_id?.trim();
  if (!cycleId || !executionId) {
    throw new Error("Native artifact upload requires a cycle and execution scope.");
  }
  const caseId = scope?.test_case_id?.trim();
  return {
    cycleId,
    ...(caseId ? { caseId } : {}),
    executionId,
  };
}

function mapArtifactMetadata(metadata: ArtifactMetadata, overrides?: NativeArtifactUploadOptions): ArtifactRecord {
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

/** A bare operator filename lands in the `upload/` folder; a relative path keeps its own. */
function operatorUploadName(filename: string): string {
  const name = filename.trim();
  return name.includes("/") ? name : `upload/${name}`;
}

function nativeArtifactFilename(artifact: Pick<ArtifactRecord, "artifact_id" | "label" | "path">): string {
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
function nativeArtifactSource(artifact: Pick<ArtifactRecord, "artifact_id" | "group_id" | "path" | "scope">): ArtifactSource {
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

function mapArtifactState(state: ArtifactState): "static" | "streaming" | "finalized" | "interrupted" {
  switch (state?.HarmosVariant.case) {
    case "Complete":
      return "finalized";
    case "Cancelled":
    case "Rejected":
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
  switch (kind?.HarmosVariant.case) {
    case "Text":
      return "text";
    case "Binary":
      return "binary";
    case "Media":
      return "media";
    default:
      return "unknown";
  }
}

function mapFileState(state: ExecutionFileState): ExecutionFileLifecycle {
  switch (state?.HarmosVariant.case) {
    case "Streaming":
      return "streaming";
    case "Finalized":
      return "finalized";
    case "Interrupted":
      return "interrupted";
    default:
      return "static";
  }
}

function mapFollowFrame(frame: ExecutionFileFollowFrame): ExecutionFileFollowEvent {
  const payload = frame.payload?.HarmosVariant;
  switch (payload?.case) {
    case "Metadata": {
      const metadata = payload.value.field0;
      const identity = metadata?.identity;
      if (!metadata || !identity?.history || !identity.fileId) {
        throw new Error("Content.FollowExecutionFile metadata has no original identity.");
      }
      return {
        kind: "metadata",
        identity: {
          store: identity.history.store,
          epoch: identity.history.epoch,
          fileId: identity.fileId,
          executionId: identity.executionId,
          relativePath: identity.relativePath,
        },
        phase: mapFollowPhase(metadata.phase, ["Producing", "Finished", "Abandoned"]),
        availableLength: safeNumber(metadata.availableLength, "Content.FollowExecutionFile available length"),
        offset: safeNumber(metadata.offset, "Content.FollowExecutionFile offset"),
      };
    }
    case "Chunk": {
      const chunk = payload.value.field0;
      if (!chunk) throw new Error("Content.FollowExecutionFile chunk is missing.");
      return {
        kind: "chunk",
        offset: safeNumber(chunk.offset, "Content.FollowExecutionFile chunk offset"),
        data: chunk.data,
      };
    }
    case "Terminal": {
      const terminal = payload.value.field0;
      if (!terminal) throw new Error("Content.FollowExecutionFile terminal is missing.");
      const sha256 = terminal.sealed?.sha256.toLowerCase() ?? null;
      if (sha256 !== null && !/^[0-9a-f]{64}$/u.test(sha256)) {
        throw new Error("Content.FollowExecutionFile seal digest is invalid.");
      }
      return {
        kind: "terminal",
        phase: mapFollowPhase(terminal.phase, ["Finished", "Abandoned"]),
        length: safeNumber(terminal.length, "Content.FollowExecutionFile length"),
        sha256,
      };
    }
    default:
      throw new Error("Content.FollowExecutionFile sent an unknown frame.");
  }
}

function mapFollowPhase<const Allowed extends readonly ("Producing" | "Finished" | "Abandoned")[]>(
  phase: { HarmosVariant: { case: string | undefined } } | undefined,
  allowed: Allowed,
): Lowercase<Allowed[number]> {
  const name = phase?.HarmosVariant.case;
  const match = allowed.find((candidate) => candidate === name);
  if (!match) throw new Error(`Content.FollowExecutionFile phase ${name ?? "unset"} is invalid here.`);
  return match.toLowerCase() as Lowercase<Allowed[number]>;
}

async function assembleArtifactBytes(reader: NativeStream<ArtifactFrame>, artifactId: string, active: () => boolean): Promise<Uint8Array> {
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
    const payload = frame.payload?.HarmosVariant;
    if (!payload?.case || !payload.value.field0) throw new Error("Content transfer contained an incomplete frame.");
    switch (payload.case) {
      case "Metadata": {
        if (!payload.value.field0) throw new Error("Transfer frame payload is missing.");
        const metadata = payload.value.field0;
        if (metadata.artifactId !== artifactId || metadataSize !== undefined) {
          throw new Error("Artifact metadata identity is invalid.");
        }
        metadataSize = safeNumber(metadata.sizeBytes, "Content.ArtifactMetadata size");
        expectedSha = metadata.sha256.toLowerCase();
        if (!/^[0-9a-f]{64}$/u.test(expectedSha)) {
          throw new Error("Artifact metadata digest is invalid.");
        }
        if (metadataSize > NATIVE_CONTENT_BYTES) throw new NativeApiLimitError("Content.DownloadArtifact", "artifact is too large");
        break;
      }
      case "Chunk": {
        if (!payload.value.field0) throw new Error("Transfer frame payload is missing.");
        if (metadataSize === undefined) {
          throw new Error("Artifact data arrived before metadata.");
        }
        const chunk = payload.value.field0;
        if (chunk.data.byteLength === 0) {
          throw new Error("Artifact chunk size is invalid.");
        }
        const offset = safeNumber(chunk.offset, "Content.ArtifactChunk offset");
        if (offset !== expectedOffset) throw new Error("Artifact chunks are not contiguous.");
        total += chunk.data.byteLength;
        if (total > NATIVE_CONTENT_BYTES) throw new NativeApiLimitError("Content.DownloadArtifact", "artifact is too large");
        chunks.push(chunk.data);
        expectedOffset += chunk.data.byteLength;
        break;
      }
      case "Complete": {
        if (!payload.value.field0) throw new Error("Transfer frame payload is missing.");
        if (metadataSize === undefined || completeSize !== undefined) {
          throw new Error("Artifact completed before metadata or twice.");
        }
        const completion = payload.value.field0;
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
