// Dokime — SPDX-License-Identifier: Apache-2.0

import { create, fromBinary, type MessageInitShape, toBinary } from "@bufbuild/protobuf";

import {
  type Row2 as ArtifactMetadata,
  Row2Schema as ArtifactMetadataSchema,
  Row15Schema as UploadArtifactHeaderSchema,
} from "@generated/dokime/v1/ContentMessages/UploadArtifact/messages_pb";
import type { RpcError } from "../desktop/rpc";
import type { Wire } from "./canonical";
import { normalizeRpcError } from "./errors";
import { invokeOwned, requestBytes, responseBytes } from "./ipc";
import type { SessionIdentity } from "./transport";

export interface NativeArtifactUpload {
  /** Send one bounded raw slice in order. */
  write(slice: Uint8Array): Promise<void>;
  /** Close the upload and await its verified, server-issued metadata. */
  finish(): Promise<Wire<ArtifactMetadata>>;
  /** Cancel the upload and release the native sink. */
  cancel(): Promise<void>;
}

/**
 * Open a native artifact upload sink.
 *
 * Unlike `openClientStream`, the caller never builds or encodes an
 * `UploadArtifactFrame` chunk or completion frame itself: only the initial
 * header crosses as a typed protobuf message. Rust hashes every later raw
 * slice incrementally and synthesizes the chunk/completion frames from it,
 * so `write` here takes plain bytes rather than a protobuf message.
 */
export async function openNativeArtifactUpload(
  session: SessionIdentity,
  header: MessageInitShape<typeof UploadArtifactHeaderSchema>,
): Promise<NativeArtifactUpload> {
  const response = responseBytes(
    session,
    await invokeOwned<ArrayBuffer>(
      session,
      "artifact_upload_open",
      requestBytes(session, toBinary(UploadArtifactHeaderSchema, create(UploadArtifactHeaderSchema, header))),
    ),
    16,
  );
  if (response.byteLength !== 16) fail({ kind: "invalid_request" });
  const handle = new DataView(response.buffer).getBigUint64(8, true);
  let sequence = 1n;
  let inputClosed = false;
  let settled = false;
  let cancellationRequested = false;
  let writing = false;

  function handlePayload(): Uint8Array {
    const bytes = new Uint8Array(8);
    new DataView(bytes.buffer).setBigUint64(0, handle, true);
    return requestBytes(session, bytes);
  }
  async function cancelNative(): Promise<void> {
    if (session.isCurrent()) {
      await invokeOwned<void>(session, "artifact_upload_cancel", handlePayload());
    }
  }
  async function requestCancel(): Promise<void> {
    if (cancellationRequested) return;
    cancellationRequested = true;
    await cancelNative().catch(() => undefined);
  }
  const cancel = async (): Promise<void> => {
    if (settled || cancellationRequested) return;
    await requestCancel();
  };
  const write = async (slice: Uint8Array): Promise<void> => {
    if (inputClosed || settled || cancellationRequested) fail({ kind: "unknown_stream" });
    if (writing) fail({ kind: "busy" });
    writing = true;
    try {
      const bytes = new Uint8Array(16 + slice.byteLength);
      const view = new DataView(bytes.buffer);
      view.setBigUint64(0, handle, true);
      view.setBigUint64(8, sequence, true);
      bytes.set(slice, 16);
      await invokeOwned<void>(session, "artifact_upload_write", requestBytes(session, bytes));
      sequence += 1n;
    } catch (error) {
      inputClosed = true;
      await requestCancel();
      throw normalizeRpcError(error);
    } finally {
      writing = false;
    }
  };
  const finish = async (): Promise<Wire<ArtifactMetadata>> => {
    if (inputClosed || settled || cancellationRequested) fail({ kind: "unknown_stream" });
    if (writing) fail({ kind: "busy" });
    inputClosed = true;
    try {
      const result = await invokeOwned<ArrayBuffer>(session, "artifact_upload_finish", handlePayload());
      if (cancellationRequested) fail({ kind: "cancelled" });
      const metadata = fromBinary(ArtifactMetadataSchema, responseBytes(session, result, 8).subarray(8));
      settled = true;
      return metadata;
    } catch (error) {
      await requestCancel();
      throw normalizeRpcError(error);
    } finally {
      settled = true;
    }
  };
  return { write, finish, cancel };
}

export interface NativeRecordingSink {
  /** Append one MediaRecorder timeslice, in arrival order. */
  write(slice: Uint8Array): Promise<void>;
  /** Drop the sink and delete its temp file. Safe to call more than once. */
  cancel(): Promise<void>;
}

/**
 * Open a native recording sink: a local temp file the raw MediaRecorder
 * timeslices are appended to, not a direct upload stream. Timeslices are
 * container-level fragments, not independently valid frames, so they cannot
 * be hashed and forwarded incrementally the way an upload's slices are;
 * `desktop_recording_finish` (a JSON `bindings/mod.rs` command, called
 * separately once the recorder stops) patches an MP4's duration in place on
 * that file and only then hashes and uploads it.
 *
 * Unlike `openNativeArtifactUpload`, there is no `finish` here and no chunk
 * sequence number either (the caller already serializes writes one in
 * flight, and a MediaRecorder always delivers its timeslices strictly in
 * order) — `desktop_recording_finish` settles the sink instead. `cancel`
 * exists for every abandonment path (a recorder error, a failed write, an
 * unmount mid-recording, a failed `finish`) so a recording never lingers
 * past the point the caller gave up on it.
 */
export async function openNativeRecording(
  session: SessionIdentity,
  header: MessageInitShape<typeof UploadArtifactHeaderSchema>,
): Promise<NativeRecordingSink> {
  const response = await invokeOwned<ArrayBuffer>(
    session,
    "recording_open",
    requestBytes(session, toBinary(UploadArtifactHeaderSchema, create(UploadArtifactHeaderSchema, header))),
  );
  responseBytes(session, response, 8);
  let writing = false;
  let cancelled = false;
  const write = async (slice: Uint8Array): Promise<void> => {
    if (cancelled) fail({ kind: "unknown_stream" });
    if (writing) fail({ kind: "busy" });
    writing = true;
    try {
      await invokeOwned<void>(session, "recording_write", requestBytes(session, slice));
    } catch (error) {
      throw normalizeRpcError(error);
    } finally {
      writing = false;
    }
  };
  const cancel = async (): Promise<void> => {
    if (cancelled) return;
    cancelled = true;
    if (session.isCurrent()) {
      await invokeOwned<void>(session, "recording_cancel", requestBytes(session, new Uint8Array(0))).catch(() => undefined);
    }
  };
  return { write, cancel };
}

function fail(error: RpcError): never {
  throw normalizeRpcError(error);
}
