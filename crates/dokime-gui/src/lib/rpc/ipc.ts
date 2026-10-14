// Dokime — SPDX-License-Identifier: Apache-2.0

/* The desktop transport: opaque protobuf bytes over the shell's IPC bridge.

   Every request is fenced by the connection generation and the descriptor
   digest; every reply and channel frame carries the generation back. A unary
   call is one `rpc` command. A stream is one `rpc_stream_open` whose frames
   arrive on a Tauri channel, released in batches of acknowledged credit. */

/* === External Imports === */
import {
  create,
  type DescMessage,
  fromBinary,
  type MessageInitShape,
  type MessageShape,
  toBinary,
} from "@bufbuild/protobuf";
import { Channel, invoke } from "@tauri-apps/api/core";

/* === Local Imports === */
import type { RpcError } from "../desktop/rpc";
import { isReportBuild } from "../ui-mode";
import { normalizeRpcError } from "./errors";
import { descriptorSha256 } from "./generated/identity";
import type { NativeClientStream, NativeStream, SessionIdentity } from "./transport";

/* === Types === */
interface Call {
  /** The next message, or undefined once the call ended with an OK status. */
  next(): Promise<Uint8Array | undefined>;
  send(payload: Uint8Array): Promise<void>;
  finish(): Promise<void>;
  cancel(): Promise<void>;
}

/* === Constants === */
const maxFrameBytes = 4 * 1024 * 1024;
/** Mirrors `STREAM_CREDIT` in `src-tauri/src/rpc/calls.rs`. */
const streamCredit = 32;
const acknowledgeEvery = streamCredit / 2;
const frameMessage = 0;
const frameEnd = 1;
const frameError = 2;
const headers = {
  path: "dokime-rpc-path",
  deadline: "dokime-rpc-deadline-ms",
  channel: "dokime-rpc-channel",
  input: "dokime-rpc-input",
  stream: "dokime-rpc-stream",
  credit: "dokime-rpc-credit",
} as const;
const descriptor = Uint8Array.from(descriptorSha256.match(/../g) ?? [], (byte) =>
  Number.parseInt(byte, 16),
);
const text = new TextDecoder("utf-8", { fatal: true });

/* === Public Functions === */
export async function invokeOwned<T>(
  session: SessionIdentity,
  command: string,
  bytes: Uint8Array,
  options?: Record<string, string>,
): Promise<T> {
  /* The frozen report shares the components that reach these transports; none
     of them may address a backend from a file. Reaching the bridge here means a
     caller's own guard is missing, so it says so instead of connecting. */
  if (isReportBuild()) {
    throw new Error(`A frozen report cannot invoke the desktop bridge ("${command}").`);
  }
  try {
    return await invoke<T>(command, bytes, options ? { headers: options } : undefined);
  } catch (error) {
    current(session);
    throw normalizeRpcError(error);
  }
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
  path: string,
  input: I,
  output: O,
  session: SessionIdentity,
  request: MessageInitShape<I>,
  timeoutMs = 10_000,
): Promise<MessageShape<O>> {
  const reply = await invokeOwned<ArrayBuffer>(
    session,
    "rpc",
    requestBytes(session, toBinary(input, create(input, request))),
    { [headers.path]: path, [headers.deadline]: String(timeoutMs) },
  );
  return fromBinary(output, responseBytes(session, reply, 8).subarray(8));
}

export async function openStream<I extends DescMessage, O extends DescMessage>(
  path: string,
  input: I,
  output: O,
  session: SessionIdentity,
  request: MessageInitShape<I>,
  envelope?: DescMessage,
  timeoutMs = 86_400_000,
): Promise<NativeStream<MessageShape<O>>> {
  const call = await openCall(
    path,
    session,
    toBinary(input, create(input, request)),
    false,
    timeoutMs,
  );
  let ended = false;
  let reading = false;
  let completion = false;
  const cancel = async () => {
    if (ended) return;
    ended = true;
    await call.cancel();
  };
  const iterator: AsyncIterator<MessageShape<O>> = {
    async next() {
      if (ended) return { done: true, value: undefined };
      if (reading) throw normalizeRpcError({ kind: "busy" });
      reading = true;
      try {
        while (true) {
          const payload = await call.next();
          if (payload === undefined) {
            if (envelope && !completion)
              throw new Error("Backend closed without operation completion.");
            ended = true;
            return { done: true, value: undefined };
          }
          if (completion) throw new Error("Backend returned data after operation completion.");
          if (!envelope) return { done: false, value: fromBinary(output, payload) };
          const decoded = fromBinary(envelope, payload) as unknown as {
            value: { case: string; value: MessageShape<O> };
          };
          if (decoded.value.case === "response") {
            completion = true;
            continue;
          }
          if (decoded.value.case !== "item") throw new Error("Invalid operation stream envelope.");
          return { done: false, value: decoded.value.value };
        }
      } catch (error) {
        await cancel();
        current(session);
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

export async function openClientStream<I extends DescMessage, O extends DescMessage>(
  path: string,
  input: I,
  output: O,
  session: SessionIdentity,
  request: MessageInitShape<I>,
  envelope: DescMessage,
  timeoutMs = 300_000,
): Promise<NativeClientStream<I, O>> {
  /* A client-streaming operation opens with its request, then carries items;
     the generated client sends a default request and hands the first item
     here, so callers write items only. */
  const frame = (value: { case: "request" | "item"; value: unknown }) =>
    toBinary(envelope, create(envelope, { value } as never));
  const call = await openCall(
    path,
    session,
    frame({ case: "request", value: {} }),
    true,
    timeoutMs,
  );
  let inputClosed = false;
  let settled = false;
  let writing = false;
  const cancel = async () => {
    if (settled) return;
    settled = true;
    await call.cancel();
  };
  const write = async (message: MessageInitShape<I>) => {
    if (inputClosed || settled) fail({ kind: "unknown_stream" });
    if (writing) fail({ kind: "busy" });
    writing = true;
    try {
      await call.send(frame({ case: "item", value: create(input, message) }));
    } catch (error) {
      inputClosed = true;
      throw normalizeRpcError(await failure(error));
    } finally {
      writing = false;
    }
  };
  /* A stream the backend already ended refuses further input; its own
     terminal status, already on the channel, says why. */
  const failure = async (error: unknown): Promise<unknown> => {
    if ((error as { kind?: string } | null)?.kind !== "unknown_stream") {
      await cancel();
      return error;
    }
    settled = true;
    try {
      await call.next();
    } catch (terminal) {
      return terminal;
    }
    return error;
  };
  const finish = async (): Promise<MessageShape<O>> => {
    if (inputClosed || settled) fail({ kind: "unknown_stream" });
    if (writing) fail({ kind: "busy" });
    inputClosed = true;
    try {
      await call.finish();
      const payload = await call.next();
      if (payload === undefined) throw new Error("Backend returned no client-stream reply.");
      const response = fromBinary(output, payload);
      if ((await call.next()) !== undefined)
        throw new Error("Backend returned multiple client-stream replies.");
      settled = true;
      return response;
    } catch (error) {
      await cancel();
      throw normalizeRpcError(error);
    }
  };
  try {
    await write(request);
  } catch (error) {
    await cancel();
    throw error;
  }
  return { write, finish, cancel };
}

/* === Private Functions === */
async function openCall(
  path: string,
  session: SessionIdentity,
  opening: Uint8Array,
  keepInput: boolean,
  timeoutMs: number,
): Promise<Call> {
  const generation = integer(session.generation);
  const frames: Uint8Array[] = [];
  let wake: (() => void) | undefined;
  let finished = false;
  let cancelled = false;
  let unacknowledged = 0;
  const channel = new Channel<ArrayBuffer>((message) => {
    frames.push(new Uint8Array(message));
    wake?.();
  });
  const reply = await invokeOwned<ArrayBuffer>(
    session,
    "rpc_stream_open",
    requestBytes(session, opening),
    {
      [headers.path]: path,
      [headers.deadline]: String(timeoutMs),
      [headers.channel]: channel.toJSON(),
      [headers.input]: keepInput ? "open" : "closed",
    },
  );
  const opened = responseBytes(session, reply, 16);
  if (opened.byteLength !== 16) fail({ kind: "invalid_request" });
  const stream = String(new DataView(opened.buffer).getBigUint64(8, true));
  const control = (command: string, extra: Record<string, string> = {}) =>
    invokeOwned<void>(session, command, requestBytes(session, new Uint8Array(0)), {
      [headers.stream]: stream,
      ...extra,
    });
  const cancel = async () => {
    if (finished || cancelled) return;
    cancelled = true;
    wake?.();
    if (session.isCurrent()) await control("rpc_stream_cancel").catch(() => undefined);
  };
  const next = async (): Promise<Uint8Array | undefined> => {
    while (true) {
      if (cancelled) fail({ kind: "cancelled" });
      if (finished) return undefined;
      current(session);
      const frame = frames.shift();
      if (!frame) {
        await new Promise<void>((resolve) => {
          wake = resolve;
        });
        wake = undefined;
        continue;
      }
      if (frame.byteLength < 9) fail({ kind: "invalid_request" });
      if (new DataView(frame.buffer, frame.byteOffset).getBigUint64(0, true) !== generation) {
        await cancel();
        fail({ kind: "session", detail: { kind: "superseded" } });
      }
      const payload = frame.subarray(9);
      switch (frame[8]) {
        case frameMessage:
          unacknowledged += 1;
          if (unacknowledged >= acknowledgeEvery) {
            const credit = String(unacknowledged);
            unacknowledged = 0;
            void control("rpc_stream_ack", { [headers.credit]: credit }).catch(() => undefined);
          }
          return payload;
        case frameEnd:
          finished = true;
          return undefined;
        case frameError:
          finished = true;
          throw normalizeRpcError(JSON.parse(text.decode(payload)));
        default:
          fail({ kind: "invalid_request" });
      }
    }
  };
  return {
    next,
    cancel,
    send: (payload) =>
      invokeOwned<void>(session, "rpc_stream_send", requestBytes(session, payload), {
        [headers.stream]: stream,
      }),
    finish: () => control("rpc_stream_finish"),
  };
}

function current(session: SessionIdentity): void {
  if (!session.isCurrent()) fail({ kind: "session", detail: { kind: "superseded" } });
}

function fail(error: RpcError): never {
  throw normalizeRpcError(error);
}

function integer(value: string): bigint {
  if (!/^(0|[1-9][0-9]*)$/.test(value)) fail({ kind: "invalid_request" });
  const number = BigInt(value);
  if (number > 0xffffffffffffffffn) fail({ kind: "invalid_request" });
  return number;
}
