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

import * as protocolV1Evidence from "@host/dokime/v1/evidence_pb.js";
import * as protocolClientV1ContentExecutionFiles from "@host/dokime/client/v1/content/execution_files_pb.js";
import type { ArtifactSource } from "@/lib/desktop/delivery";
import { EXECUTION_FILE_WINDOW_BYTES } from "@/lib/execution-file-paging";
import { openNativeArtifactUpload } from "@/lib/rpc/artifact-upload";
import { required, cutKey } from "@/lib/rpc/canonical";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/host/client";
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
type ArtifactMetadata = Wire<Awaited<ReturnType<typeof rpc.contentArtifactsList>>>["artifacts"][number];
type ExecutionFileMetadata = Wire<Awaited<ReturnType<typeof rpc.contentExecutionFilesList>>>["files"][number];
type ExecutionFilesCursor = NonNullable<Wire<Awaited<ReturnType<typeof rpc.contentExecutionFilesList>>>["resolvedAt"]>;
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
  let pageToken = "";
  let at: ExecutionFilesCursor | undefined;
  const tokens = new Set<string>();
  for (let index = 0; index < NATIVE_MAX_PAGES; index++) {
    const reply = await rpc.contentExecutionFilesList(session, {
      parent: executionId,
      at,
      page: { pageSize: GUI_SMALL_PAGE_ITEMS, pageToken },
    });
    const cut = required(reply.resolvedAt, "List Execution Files Cut");
    if (at && cutKey(at) !== cutKey(cut)) throw new Error("List Execution Files: Retained Cut Changed");
    at = cut;
    files.push(...reply.files);
    if (!reply.nextPageToken) return files.map((file) => mapExecutionFile(executionId, file));
    if (tokens.has(reply.nextPageToken)) throw new Error("List Execution Files: Repeated Page Token");
    pageToken = reply.nextPageToken;
    tokens.add(pageToken);
  }
  throw new NativeApiLimitError("ExecutionFiles.List", "Page Budget Reached");
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
    throw new NativeApiLimitError("ExecutionFiles.Get", `max_bytes must be between 1 and ${EXECUTION_FILE_WINDOW_BYTES}`);
  }
  if (!Number.isSafeInteger(offset) || offset < 0) {
    throw new NativeApiLimitError("ExecutionFiles.Get", "offset is invalid");
  }
  const item = await rpc.contentExecutionFilesGet(session, { parent: executionId, name: path });
  const file = required(item.file, "Read Execution File");
  const metadata = required(item.metadata, "Read Execution File Metadata");
  const window = await rpc.filesRead(session, { file, offset: BigInt(offset), maxBytes: BigInt(maxBytes) });
  if (window.totalLength !== file.length || cutKey(window.sha256) !== cutKey(file.sha256) || window.data.length > maxBytes)
    throw new Error("Read Execution File: Invalid Range or Digest");
  return {
    offset,
    observedSizeBytes: safeNumber(window.totalLength, "Read File Length"),
    data: window.data,
    state: mapFileState(metadata.state),
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
    throw new NativeApiLimitError("ExecutionFiles.Get", "a resume offset requires the original identity");
  }
  const item = await rpc.contentExecutionFilesGet(session, { parent: executionId, name: path });
  const file = required(item.file, "Follow Execution File");
  const history = required(file.position?.history, "Follow Execution File History");
  const id = required(file.metadata, "Follow Execution File Metadata").id;
  if (identity && (identity.store !== history.store || identity.epoch !== history.epoch || identity.fileId !== id))
    throw new Error("Follow Execution File: Original Identity Changed");
  const stream = await rpc.filesWatch(session, {
    file,
    offset: BigInt(offset),
  });
  return {
    cancel: () => stream.cancel(),
    async *[Symbol.asyncIterator]() {
      let length = offset;
      let completed = false;
      const state = required(item.metadata, "Follow Execution File State").state;
      yield {
        kind: "metadata",
        identity: { store: history.store, epoch: history.epoch, fileId: id, executionId, relativePath: path },
        phase:
          state === protocolClientV1ContentExecutionFiles.ExecutionFileState.STREAMING
            ? "producing"
            : state === protocolClientV1ContentExecutionFiles.ExecutionFileState.INTERRUPTED
              ? "abandoned"
              : "finished",
        availableLength: safeNumber(file.length, "Follow File Length"),
        offset,
      } as ExecutionFileFollowEvent;
      for await (const frame of stream) {
        const data = frame.data;
        if (data) {
          const position = safeNumber(data.offset, "Follow File Offset");
          if (position !== length) throw new Error("Follow Execution File: Noncontiguous Range");
          yield { kind: "chunk", offset: position, data: data.bytes } as ExecutionFileFollowEvent;
          length += data.bytes.length;
        }
        if (frame.completion) completed = true;
        if (frame.completion && frame.completion.bytes !== BigInt(length))
          throw new Error("Follow Execution File: Invalid Terminal Length");
      }
      if (!completed) throw new Error("Follow Execution File: Stream Closed Before Completion");
      const finalFile = required((await rpc.filesGet(session, { file })).file, "Follow Finalized File");
      yield {
        kind: "terminal",
        phase: state === protocolClientV1ContentExecutionFiles.ExecutionFileState.INTERRUPTED ? "abandoned" : "finished",
        length,
        sha256: [...finalFile.sha256].map((byte) => byte.toString(16).padStart(2, "0")).join(""),
      } as ExecutionFileFollowEvent;
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
  const reply = await rpc.contentArtifactsGet(session, { name: artifactId });
  const reference = required(reply.file, "Read Artifact File");
  return readFileReference(session, reference, signal);
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
    throw new NativeApiLimitError("Artifacts.Create", `payload exceeds ${NATIVE_CONTENT_BYTES} bytes`);
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
      requestId: crypto.randomUUID(),
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

/** Maps an artifact record scope to the native upload scope; refuses records without a cycle and execution. */
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
    description: metadata.error ?? overrides?.description ?? null,
    size_bytes: safeNumber(metadata.sizeBytes, "Artifact Metadata size"),
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
  switch (state) {
    case protocolV1Evidence.ArtifactState.COMPLETE:
      return "finalized";
    case protocolV1Evidence.ArtifactState.CANCELLED:
    case protocolV1Evidence.ArtifactState.REJECTED:
    case protocolV1Evidence.ArtifactState.FAILED:
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
    size_bytes: safeNumber(file.sizeBytes, "Execution File Metadata size"),
    group_id: "execution-files",
    group_label: "Execution Files",
    created_at: createdAt,
    scope: { test_execution_id: executionId },
  };
}

function mapFileKind(kind: ExecutionFileKind): "text" | "binary" | "media" | "unknown" {
  switch (kind) {
    case protocolClientV1ContentExecutionFiles.ExecutionFileKind.TEXT:
      return "text";
    case protocolClientV1ContentExecutionFiles.ExecutionFileKind.BINARY:
      return "binary";
    case protocolClientV1ContentExecutionFiles.ExecutionFileKind.MEDIA:
      return "media";
    default:
      return "unknown";
  }
}

function mapFileState(state: ExecutionFileState): ExecutionFileLifecycle {
  switch (state) {
    case protocolClientV1ContentExecutionFiles.ExecutionFileState.STREAMING:
      return "streaming";
    case protocolClientV1ContentExecutionFiles.ExecutionFileState.FINALIZED:
      return "finalized";
    case protocolClientV1ContentExecutionFiles.ExecutionFileState.INTERRUPTED:
      return "interrupted";
    default:
      return "static";
  }
}

/** Bounded Library Range Reads, Pinned to One Returned Cut and Verified Digest. */
export async function readFileReference(
  session: SessionIdentity,
  file: NonNullable<Parameters<typeof rpc.filesRead>[1]["file"]>,
  signal?: AbortSignal,
  maxBytes = NATIVE_CONTENT_BYTES,
): Promise<Uint8Array> {
  const length = safeNumber(required(file.length, "Read File Length"), "Read File Length");
  if (length > maxBytes || required(file.sha256, "Read File Digest").length !== 32)
    throw new Error("Read File: Content Exceeds Its Budget or Has No Digest");
  const selected = await rpc.filesGet(session, { file });
  let at = required(selected.resolvedAt, "Read File Cut");
  const chunkBytes = safeNumber(selected.chunkBytes, "Read File Chunk Bound");
  const result = new Uint8Array(length);
  for (let offset = 0; offset < length; ) {
    if (!session.isCurrent() || signal?.aborted) throw new Error("Read File: Transfer Cancelled");
    const reply = await rpc.filesRead(session, {
      file,
      offset: BigInt(offset),
      maxBytes: BigInt(Math.min(EXECUTION_FILE_WINDOW_BYTES, chunkBytes, length - offset)),
      at,
    });
    const cut = required(reply.resolvedAt, "Read File Cut");
    if (
      (at && cutKey(at) !== cutKey(cut)) ||
      reply.totalLength !== BigInt(length) ||
      cutKey(reply.sha256) !== cutKey(file.sha256) ||
      !reply.data.length ||
      reply.data.length > Math.min(EXECUTION_FILE_WINDOW_BYTES, chunkBytes, length - offset)
    )
      throw new Error("Read File: Invalid Range, Digest or Retained Cut");
    at = cut;
    result.set(reply.data, offset);
    offset += reply.data.length;
  }
  if (
    (await sha256Hex(result)) !== [...required(file.sha256, "Read File Digest")].map((byte) => byte.toString(16).padStart(2, "0")).join("")
  )
    throw new Error("Read File: SHA-256 Mismatch");
  return result;
}
