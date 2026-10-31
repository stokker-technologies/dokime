// Dokime — SPDX-License-Identifier: Apache-2.0

/* The seam the generated client calls through. It binds to the shell's IPC
   bridge, so the WebView never opens a network connection. */

/* === External Imports === */
import type { DescMessage, MessageInitShape, MessageShape } from "@bufbuild/protobuf";

/* === Local Imports === */
import { settleUnlessStopping } from "../desktop/stopping";
import * as transport from "./ipc";
import { notificationFailure } from "../notification-details";
import { operationIds } from "./generated/operations";

/* === Types === */
export interface SessionIdentity {
  /** Decimal connection generation returned by desktop_session_select/status. */
  generation: string;
  /** Read the current connection owner; never capture a constant true value. */
  isCurrent(): boolean;
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

/* === Public Functions === */
export function unary<I extends DescMessage, O extends DescMessage>(
  path: string,
  input: I,
  output: O,
  session: SessionIdentity,
  request: MessageInitShape<I>,
  timeoutMs?: number,
): Promise<MessageShape<O>> {
  return transport
    .unary(path, input, output, session, request, timeoutMs)
    .catch((error: unknown) => settleUnlessStopping<MessageShape<O>>(operationFailure(error, path, request)));
}

export function openStream<I extends DescMessage, O extends DescMessage>(
  path: string,
  input: I,
  output: O,
  session: SessionIdentity,
  request: MessageInitShape<I>,
  envelope?: DescMessage,
  timeoutMs?: number,
): Promise<NativeStream<MessageShape<O>>> {
  return transport.openStream(path, input, output, session, request, envelope, timeoutMs).then(
    (stream) => stoppableStream(stream, path, request),
    (error: unknown) => settleUnlessStopping<NativeStream<MessageShape<O>>>(operationFailure(error, path, request)),
  );
}

export function openClientStream<I extends DescMessage, O extends DescMessage>(
  path: string,
  input: I,
  output: O,
  session: SessionIdentity,
  request: MessageInitShape<I>,
  envelope: DescMessage,
  timeoutMs: number,
): Promise<NativeClientStream<I, O>> {
  return transport.openClientStream(path, input, output, session, request, envelope, timeoutMs).then(
    (stream) => ({
      write: (message) =>
        stream.write(message).catch((error: unknown) => settleUnlessStopping<void>(operationFailure(error, path, message))),
      finish: () => stream.finish().catch((error: unknown) => settleUnlessStopping<MessageShape<O>>(operationFailure(error, path, {}))),
      cancel: () => stream.cancel(),
    }),
    (error: unknown) => settleUnlessStopping<NativeClientStream<I, O>>(operationFailure(error, path, request)),
  );
}

/* === Private Functions === */

/** A subscription whose reads stop quietly once the desktop's stop owns their failure. */
function stoppableStream<T>(stream: NativeStream<T>, path: string, request: object): NativeStream<T> {
  return {
    cancel: () => stream.cancel(),
    [Symbol.asyncIterator]: () => {
      const iterator = stream[Symbol.asyncIterator]();
      return {
        next: () =>
          iterator.next().catch((error: unknown) => settleUnlessStopping<IteratorResult<T>>(operationFailure(error, path, request))),
        return: () => iterator.return?.() ?? Promise.resolve({ done: true, value: undefined }),
      };
    },
  };
}

function operationFailure(error: unknown, path: string, request: object): unknown {
  if (!(error instanceof Error)) return error;
  const identifiers = request as { operationId?: string; requestId?: string; invocationId?: string };
  const failure = notificationFailure(error);
  Object.assign(error, {
    notificationFailure: {
      ...failure,
      operation: path,
      operationId: identifiers.operationId ?? failure?.operationId ?? operationIds[path],
      requestId: identifiers.requestId ?? failure?.requestId,
      invocationId: identifiers.invocationId ?? failure?.invocationId,
    },
  });
  return error;
}
