// Dokime — SPDX-License-Identifier: Apache-2.0
import {
  create,
  type DescMessage,
  fromBinary,
  type MessageInitShape,
  type MessageShape,
  toBinary,
} from "@bufbuild/protobuf";
import { invoke } from "@tauri-apps/api/core";
import { isReportBuild } from "@/lib/ui-mode";
import type { RpcError } from "../desktop/rpc";
import { normalizeRpcError } from "./errors";
import { descriptorSha256 } from "./generated/identity";

export interface SessionIdentity {
  /** Decimal connection generation returned by desktop_session_select/status. */
  generation: string;
  /** Read the current connection owner; never capture a constant true value. */
  isCurrent(): boolean;
}
const maxFrameBytes = 4 * 1024 * 1024;
const descriptor = Uint8Array.from(descriptorSha256.match(/../g) ?? [], (byte) =>
  Number.parseInt(byte, 16),
);
function fail(error: RpcError): never {
  throw normalizeRpcError(error);
}
function current(session: SessionIdentity) {
  if (!session.isCurrent()) fail({ kind: "session", detail: { kind: "superseded" } });
}
export async function invokeOwned<T>(
  session: SessionIdentity,
  command: string,
  bytes: Uint8Array,
): Promise<T> {
  /* The frozen report shares the components that reach these transports; none
     of them may address a backend from a file. Reaching the bridge here means a
     caller's own guard is missing, so it says so instead of connecting. */
  if (isReportBuild()) {
    throw new Error(`A frozen report cannot invoke the desktop bridge ("${command}").`);
  }
  try {
    return await invoke<T>(command, bytes);
  } catch (error) {
    current(session);
    throw normalizeRpcError(error);
  }
}
function integer(value: string): bigint {
  if (!/^(0|[1-9][0-9]*)$/.test(value)) fail({ kind: "invalid_request" });
  const number = BigInt(value);
  if (number > 0xffffffffffffffffn) fail({ kind: "invalid_request" });
  return number;
}
export function requestBytes(session: SessionIdentity, payload: Uint8Array): Uint8Array {
  current(session);
  if (payload.length > maxFrameBytes) fail({ kind: "limit_exceeded" });
  const result = new Uint8Array(40 + payload.length);
  new DataView(result.buffer).setBigUint64(0, integer(session.generation), true);
  result.set(descriptor, 8);
  result.set(payload, 40);
  return result;
}
export function responseBytes(
  session: SessionIdentity,
  value: ArrayBuffer,
  minimum: number,
): Uint8Array {
  current(session);
  if (
    !(value instanceof ArrayBuffer) ||
    value.byteLength < minimum ||
    value.byteLength > maxFrameBytes + 25
  )
    fail({ kind: "invalid_request" });
  if (new DataView(value).getBigUint64(0, true) !== integer(session.generation))
    fail({ kind: "session", detail: { kind: "superseded" } });
  return new Uint8Array(value);
}
export async function unary<I extends DescMessage, O extends DescMessage>(
  command: string,
  input: I,
  output: O,
  session: SessionIdentity,
  request: MessageInitShape<I>,
): Promise<MessageShape<O>> {
  const response = await invokeOwned<ArrayBuffer>(
    session,
    command,
    requestBytes(session, toBinary(input, create(input, request))),
  );
  return fromBinary(output, responseBytes(session, response, 8).subarray(8));
}

export interface NativeStream<T> extends AsyncIterable<T> {
  /** Cancel this subscription, never the backend operation producing its data. */
  cancel(): Promise<void>;
}

export interface NativeClientStream<I extends DescMessage, O extends DescMessage> {
  /** Send one bounded protobuf frame in order. */
  write(request: MessageInitShape<I>): Promise<void>;
  /** Close the input and await the typed unary response. */
  finish(): Promise<MessageShape<O>>;
  /** Cancel the backend operation and release the native stream. */
  cancel(): Promise<void>;
}

export async function openClientStream<I extends DescMessage, O extends DescMessage>(
  command: string,
  input: I,
  output: O,
  session: SessionIdentity,
  request: MessageInitShape<I>,
): Promise<NativeClientStream<I, O>> {
  const response = responseBytes(
    session,
    await invokeOwned<ArrayBuffer>(
      session,
      command,
      requestBytes(session, toBinary(input, create(input, request))),
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

  function handlePayload(withSequence: boolean): Uint8Array {
    const bytes = new Uint8Array(withSequence ? 16 : 8);
    const view = new DataView(bytes.buffer);
    view.setBigUint64(0, handle, true);
    if (withSequence) view.setBigUint64(8, sequence, true);
    return requestBytes(session, bytes);
  }
  function writePayload(message: MessageInitShape<I>): Uint8Array {
    const encoded = toBinary(input, create(input, message));
    const bytes = new Uint8Array(16 + encoded.length);
    const view = new DataView(bytes.buffer);
    view.setBigUint64(0, handle, true);
    view.setBigUint64(8, sequence, true);
    bytes.set(encoded, 16);
    return requestBytes(session, bytes);
  }
  async function cancelNative(): Promise<void> {
    if (session.isCurrent()) {
      await invokeOwned<void>(session, "client_stream_cancel", handlePayload(false));
    }
  }
  async function requestCancel(): Promise<void> {
    if (cancellationRequested) return;
    cancellationRequested = true;
    await cancelNative().catch(() => undefined);
  }
  const cancel = async () => {
    if (settled || cancellationRequested) return;
    await requestCancel();
  };
  const write = async (message: MessageInitShape<I>) => {
    if (inputClosed || settled || cancellationRequested) fail({ kind: "unknown_stream" });
    if (writing) fail({ kind: "busy" });
    writing = true;
    try {
      await invokeOwned<void>(session, "client_stream_write", writePayload(message));
      sequence += 1n;
    } catch (error) {
      inputClosed = true;
      await requestCancel();
      throw normalizeRpcError(error);
    } finally {
      writing = false;
    }
  };
  const finish = async (): Promise<MessageShape<O>> => {
    if (inputClosed || settled || cancellationRequested) fail({ kind: "unknown_stream" });
    if (writing) fail({ kind: "busy" });
    inputClosed = true;
    try {
      const result = await invokeOwned<ArrayBuffer>(
        session,
        "client_stream_finish",
        handlePayload(false),
      );
      if (cancellationRequested) fail({ kind: "cancelled" });
      const response = fromBinary(output, responseBytes(session, result, 8).subarray(8));
      settled = true;
      return response;
    } catch (error) {
      await requestCancel();
      throw normalizeRpcError(error);
    } finally {
      settled = true;
    }
  };
  return { write, finish, cancel };
}
export async function openStream<I extends DescMessage, O extends DescMessage>(
  command: string,
  input: I,
  output: O,
  session: SessionIdentity,
  request: MessageInitShape<I>,
): Promise<NativeStream<MessageShape<O>>> {
  const response = responseBytes(
    session,
    await invokeOwned<ArrayBuffer>(
      session,
      command,
      requestBytes(session, toBinary(input, create(input, request))),
    ),
    16,
  );
  if (response.byteLength !== 16) fail({ kind: "invalid_request" });
  const handle = new DataView(response.buffer).getBigUint64(8, true);
  let sequence = 0n;
  let ended = false;
  let reading = false;
  function payload(ack: boolean) {
    const bytes = new Uint8Array(ack ? 16 : 8);
    const view = new DataView(bytes.buffer);
    view.setBigUint64(0, handle, true);
    if (ack) view.setBigUint64(8, sequence, true);
    return requestBytes(session, bytes);
  }
  const cancel = async () => {
    if (ended) return;
    ended = true;
    // Session replacement already cancels native streams. Never address its successor.
    if (session.isCurrent()) await invokeOwned<void>(session, "stream_cancel", payload(false));
  };
  const iterator: AsyncIterator<MessageShape<O>> = {
    async next() {
      if (ended) return { done: true, value: undefined };
      if (reading) fail({ kind: "busy" });
      reading = true;
      try {
        while (!ended) {
          const frame = responseBytes(
            session,
            await invokeOwned<ArrayBuffer>(session, "stream_next", payload(true)),
            25,
          );
          if (ended) return { done: true, value: undefined };
          const view = new DataView(frame.buffer);
          if (view.getBigUint64(8, true) !== handle) fail({ kind: "unknown_stream" });
          const nextSequence = view.getBigUint64(16, true);
          const kind = frame[24];
          if (kind === 2 && nextSequence === sequence && frame.length === 25) continue;
          if (nextSequence !== sequence + 1n || (kind !== 0 && kind !== 1))
            fail({ kind: "invalid_ack" });
          sequence = nextSequence;
          if (kind === 1) {
            ended = true;
            if (frame.length !== 25) fail({ kind: "invalid_request" });
            return { done: true, value: undefined };
          }
          return { done: false, value: fromBinary(output, frame.subarray(25)) };
        }
        return { done: true, value: undefined };
      } catch (error) {
        await cancel().catch(() => undefined);
        throw normalizeRpcError(error);
      } finally {
        reading = false;
      }
    },
    async return() {
      await cancel();
      return { done: true, value: undefined };
    },
  };
  return { cancel, [Symbol.asyncIterator]: () => iterator };
}
