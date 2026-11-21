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

import type { ArtifactSource } from "@dokime/document/lib/contracts/delivery";
import { EXECUTION_FILE_WINDOW_BYTES } from "@/lib/execution-file-paging";
import * as rpc from "@/lib/host/client";
import { openNativeArtifactUpload } from "@/lib/rpc/artifact-upload";
import type { Wire } from "@dokime/plots/lib/rpc/canonical";
import { cutKey, required } from "@dokime/plots/lib/rpc/canonical";
import type { NativeStream, SessionIdentity } from "@/lib/rpc/transport";
import type { ArtifactRecord, ExecutionFileFollowEvent, ExecutionFileIdentity, ExecutionFileReadWindow } from "@dokime/plots/lib/types";
import * as desktopProtocol from "@generated/desktop-protocol";
import * as hostModel from "@host/model";
import {
  type ArtifactMetadata,
  type ExecutionFileMetadata,
  type NativeArtifactUploadOptions,
  mapArtifactMetadata,
  mapExecutionFile,
  mapFileState,
} from "@dokime/document/lib/api/native-content-projection";
import { deliverNativeArtifact } from "./native-delivery";
import { NATIVE_MAX_PAGES, NativeApiLimitError, safeNumber, sha256Hex } from "@dokime/plots/lib/api/native-values";

const GUI_CONTENT_BYTES = 2_147_483_648;
const GUI_SMALL_PAGE_ITEMS = 100;

/* === Canonical Types === */

type ArtifactScope = NonNullable<ArtifactMetadata["scope"]>;

type ExecutionFilesCursor = NonNullable<Wire<Awaited<ReturnType<typeof rpc.contentExecutionFilesList>>>["resolvedAt"]>;

const NATIVE_CONTENT_BYTES = GUI_CONTENT_BYTES;
const NATIVE_UPLOAD_SLICE_BYTES = desktopProtocol.UPLOAD_SLICE_BYTES;

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
    target: { case: "file", value: file },
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
          state === hostModel.ExecutionFileState.STREAMING
            ? "producing"
            : state === hostModel.ExecutionFileState.INTERRUPTED
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
        phase: state === hostModel.ExecutionFileState.INTERRUPTED ? "abandoned" : "finished",
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
  const cycleId = scope?.cycleId?.trim();
  const executionId = scope?.executionId?.trim();
  if (!cycleId || !executionId) {
    throw new Error("Native artifact upload requires a cycle and execution scope.");
  }
  const caseId = scope?.caseId?.trim();
  return {
    cycleId,
    ...(caseId ? { caseId } : {}),
    executionId,
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
  const executionId = artifact.scope?.executionId?.trim();
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
