// Dokime — SPDX-License-Identifier: Apache-2.0

/* The seam the generated client calls through. The desktop binds it to the
   shell's IPC bridge, so the WebView never opens a network connection; a
   browser build selects gRPC-Web at build time instead. */

/* === External Imports === */
import type { DescMessage, MessageInitShape, MessageShape } from "@bufbuild/protobuf";

/* === Local Imports === */
import { settleUnlessStopping } from "../desktop/stopping";
import * as grpcWeb from "./grpc-web";
import * as ipc from "./ipc";

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

/* === Constants === */
const transport = __DOKIME_TRANSPORT__ === "grpc-web" ? grpcWeb : ipc;

/* === Public Functions === */
export function unary<I extends DescMessage, O extends DescMessage>(
  path: string,
  input: I,
  output: O,
  session: SessionIdentity,
  request: MessageInitShape<I>,
  timeoutMs?: number,
): Promise<MessageShape<O>> {
  return transport.unary(path, input, output, session, request, timeoutMs).catch(settleUnlessStopping<MessageShape<O>>);
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
  return transport
    .openStream(path, input, output, session, request, envelope, timeoutMs)
    .then(stoppableStream, settleUnlessStopping<NativeStream<MessageShape<O>>>);
}

export function openClientStream<I extends DescMessage, O extends DescMessage>(
  path: string,
  input: I,
  output: O,
  session: SessionIdentity,
  request: MessageInitShape<I>,
  envelope: DescMessage,
  timeoutMs?: number,
): Promise<NativeClientStream<I, O>> {
  return transport.openClientStream(path, input, output, session, request, envelope, timeoutMs).then(
    (stream) => ({
      write: (message) => stream.write(message).catch(settleUnlessStopping<void>),
      finish: () => stream.finish().catch(settleUnlessStopping<MessageShape<O>>),
      cancel: () => stream.cancel(),
    }),
    settleUnlessStopping<NativeClientStream<I, O>>,
  );
}

/* === Private Functions === */

/** A subscription whose reads stop quietly once the desktop's stop owns their failure. */
function stoppableStream<T>(stream: NativeStream<T>): NativeStream<T> {
  return {
    cancel: () => stream.cancel(),
    [Symbol.asyncIterator]: () => {
      const iterator = stream[Symbol.asyncIterator]();
      return {
        next: () => iterator.next().catch(settleUnlessStopping<IteratorResult<T>>),
        return: () => iterator.return?.() ?? Promise.resolve({ done: true, value: undefined }),
      };
    },
  };
}
