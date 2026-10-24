// Dokime — SPDX-License-Identifier: Apache-2.0

/* The browser-build transport: gRPC-Web straight from the page to a backend
   that serves it (`dokime server --grpc-web`). The desktop never selects it;
   its WebView reaches the backend only through the shell's IPC bridge. */

/* === External Imports === */
import { create, type DescMessage, fromBinary, type MessageInitShape, type MessageShape, toBinary } from "@bufbuild/protobuf";

/* === Local Imports === */
import type { BackendLimits } from "../desktop/connection";
import { isReportBuild } from "../ui-mode";
import { normalizeRpcError } from "./errors";
import type { NativeClientStream, NativeStream, SessionIdentity } from "./transport";

/* === Constants === */
// Keyed by connection generation: callers legitimately pass derived session
// objects that carry the same generation, so object identity is not the key.
const connections = new Map<string, Connection>();
const maxFrameBytes = 4 * 1024 * 1024;
const text = new TextDecoder("utf-8", { fatal: true });

/* === Types === */
/** Ephemeral access for one browser connection; never persist it. */
export type BrowserAccess = {
  generation: string;
  endpoint: string;
  authorization: string | null;
  backendId: string;
  epoch: string;
  limits: BackendLimits;
  methods: string[];
};

interface Connection {
  access: BrowserAccess;
  cancellation: AbortController;
}

/* === Public Functions === */
export function bindConnection(session: SessionIdentity, access: BrowserAccess): () => void {
  current(session);
  if (access.generation !== session.generation) fail("superseded");
  const cancellation = new AbortController();
  const connection = { access, cancellation };
  connections.set(session.generation, connection);
  return () => {
    cancellation.abort();
    access.authorization = null;
    if (connections.get(session.generation) === connection) {
      connections.delete(session.generation);
    }
  };
}

export async function unary<I extends DescMessage, O extends DescMessage>(
  path: string,
  input: I,
  output: O,
  session: SessionIdentity,
  request: MessageInitShape<I>,
  timeoutMs = 10_000,
): Promise<MessageShape<O>> {
  const stream = await openStream(path, input, output, session, request, undefined, timeoutMs);
  let result: MessageShape<O> | undefined;
  try {
    for await (const item of stream) {
      if (result !== undefined) throw new Error("Backend returned multiple unary replies.");
      result = item;
    }
    if (result === undefined) throw new Error("Backend returned no unary reply.");
    return result;
  } finally {
    await stream.cancel();
  }
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
  if (isReportBuild()) throw new Error("A frozen report cannot connect to a backend.");
  current(session);
  const connection = connections.get(session.generation);
  if (!connection) fail("not_connected");
  if (!/^\/[A-Za-z0-9_.]+\/[A-Za-z0-9_]+$/.test(path)) throw new Error("Invalid gRPC method path.");
  if (!connection.access.methods.includes(path)) {
    throw normalizeRpcError({ kind: "session", detail: { kind: "permission_denied" } });
  }
  const requestLimit = limit(connection.access.limits.maxRequestBytes);
  const responseLimit = limit(connection.access.limits.maxResponseBytes);
  const payload = toBinary(input, create(input, request));
  if (payload.length > requestLimit) throw normalizeRpcError({ kind: "limit_exceeded" });
  const cancellation = new AbortController();
  const signal = AbortSignal.any([connection.cancellation.signal, cancellation.signal, AbortSignal.timeout(timeoutMs)]);
  const headers: Record<string, string> = {
    "content-type": "application/grpc-web+proto",
    "x-grpc-web": "1",
    "grpc-timeout": `${timeoutMs}m`,
  };
  if (connection.access.authorization) headers.authorization = connection.access.authorization;
  let response: Response;
  try {
    response = await fetch(`${connection.access.endpoint.replace(/\/$/, "")}${path}`, {
      method: "POST",
      headers,
      body: frame(payload),
      signal,
      credentials: "omit",
      redirect: "error",
      cache: "no-store",
      referrerPolicy: "no-referrer",
    });
  } catch {
    current(session);
    throw normalizeRpcError({ kind: "session", detail: { kind: "transport" } });
  }
  if (!session.isCurrent()) {
    cancellation.abort();
    await response.body?.cancel().catch(() => undefined);
    fail("superseded");
  }
  if (!response.ok || !response.headers.get("content-type")?.startsWith("application/grpc-web")) {
    cancellation.abort();
    throw normalizeRpcError({ kind: "session", detail: { kind: "transport" } });
  }
  if (response.headers.has("grpc-status")) {
    try {
      status(response.headers);
    } catch (error) {
      cancellation.abort();
      await response.body?.cancel().catch(() => undefined);
      throw error;
    }
  }
  const body = response.body;
  if (!body) throw new Error("Backend returned no gRPC-web response body.");
  const reader = body.getReader();
  let chunk = new Uint8Array(0);
  let offset = 0;
  let ended = false;
  let reading = false;
  let finalStatus = false;
  let operationCompletion = false;
  const cancel = async () => {
    if (ended) return;
    ended = true;
    cancellation.abort();
    await reader.cancel().catch(() => undefined);
    reader.releaseLock();
  };
  async function read(size: number, eofAllowed = false): Promise<Uint8Array | undefined> {
    const bytes = new Uint8Array(size);
    let written = 0;
    while (written < size) {
      current(session);
      if (offset === chunk.length) {
        const next = await reader.read();
        if (next.done) {
          if (eofAllowed && written === 0) return undefined;
          throw new Error("Truncated gRPC-web response frame.");
        }
        chunk = next.value;
        offset = 0;
      }
      const take = Math.min(size - written, chunk.length - offset);
      bytes.set(chunk.subarray(offset, offset + take), written);
      written += take;
      offset += take;
    }
    return bytes;
  }
  const iterator: AsyncIterator<MessageShape<O>> = {
    async next() {
      if (ended) return { done: true, value: undefined };
      if (reading) throw normalizeRpcError({ kind: "busy" });
      reading = true;
      try {
        while (true) {
          const header = await read(5, true);
          if (!header) {
            if (!finalStatus) throw new Error("Backend closed without a final gRPC status.");
            if (envelope && !operationCompletion) throw new Error("Backend closed without operation completion.");
            ended = true;
            reader.releaseLock();
            return { done: true, value: undefined };
          }
          if (finalStatus) throw new Error("Backend returned data after final gRPC status.");
          const size = new DataView(header.buffer).getUint32(1, false);
          if (size > responseLimit) throw normalizeRpcError({ kind: "limit_exceeded" });
          const payload = await read(size);
          if (!payload) throw new Error("Missing gRPC-web response frame payload.");
          if (header[0] === 0x80) {
            status(
              new Headers(
                text
                  .decode(payload)
                  .trim()
                  .split("\r\n")
                  .map((line) => {
                    const colon = line.indexOf(":");
                    if (colon < 1) throw new Error("Invalid gRPC-web trailer.");
                    return [line.slice(0, colon), line.slice(colon + 1).trim()] as [string, string];
                  }),
              ),
            );
            finalStatus = true;
            continue;
          }
          if (header[0] !== 0 || operationCompletion) throw new Error("Invalid gRPC-web response frame.");
          if (!envelope) return { done: false, value: fromBinary(output, payload) };
          const decoded = fromBinary(envelope, payload) as unknown as {
            value: { case: string; value: MessageShape<O> };
          };
          if (decoded.value.case === "response") {
            operationCompletion = true;
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
  _path: string,
  _input: I,
  _output: O,
  _session: SessionIdentity,
  _request: MessageInitShape<I>,
  _envelope: DescMessage,
  _timeoutMs?: number,
): Promise<NativeClientStream<I, O>> {
  throw new Error("gRPC-Web cannot carry client streams.");
}

/* === Private Functions === */
function current(session: SessionIdentity): void {
  if (!session.isCurrent()) fail("superseded");
}
function fail(kind: "superseded" | "not_connected"): never {
  throw normalizeRpcError({ kind: "session", detail: { kind } });
}
function limit(value: string): number {
  if (!/^[1-9][0-9]*$/.test(value)) throw new Error("Invalid backend frame limit.");
  return Number(BigInt(value) < BigInt(maxFrameBytes) ? BigInt(value) : BigInt(maxFrameBytes));
}
function frame(payload: Uint8Array): Uint8Array<ArrayBuffer> {
  const bytes = new Uint8Array(5 + payload.length);
  new DataView(bytes.buffer).setUint32(1, payload.length, false);
  bytes.set(payload, 5);
  return bytes;
}
function status(headers: Headers): void {
  const value = headers.get("grpc-status");
  if (!value || !/^(?:[0-9]|1[0-6])$/.test(value)) throw new Error("Missing or invalid final gRPC status.");
  const code = Number(value);
  if (code === 0) return;
  let message = headers.get("grpc-message") ?? "";
  try {
    message = decodeURIComponent(message);
  } catch {
    /* Retain An Invalid Server Message Literally. */
  }
  const encoded = headers.get("grpc-status-details-bin");
  let details: number[] = [];
  if (encoded) {
    if (encoded.length > Math.ceil(maxFrameBytes / 3) * 4) {
      throw normalizeRpcError({ kind: "limit_exceeded" });
    }
    const bytes = atob(encoded);
    if (bytes.length > maxFrameBytes) throw normalizeRpcError({ kind: "limit_exceeded" });
    details = Array.from(bytes, (character) => character.charCodeAt(0));
  }
  throw normalizeRpcError({ kind: "grpc", detail: { code, message, details } });
}
