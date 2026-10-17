/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Bounded paging and follow bookkeeping for registered execution files.

   This module owns the published read bound and stays free of runtime imports:
   the transport is injected, so the same logic runs under the node test runner
   and in the viewer. Type-only imports are erased before either runs. */

/* === Local Imports === */

import type { ExecutionFileFollowEvent, ExecutionFileIdentity, ExecutionFileLifecycle, ExecutionFileReadWindow } from "@/lib/types";

/* === Constants === */

/** The contract's bound on one `ReadExecutionFile` window and one follow chunk. */
export const EXECUTION_FILE_WINDOW_BYTES = 64 * 1024;

/* === Types === */

export type ReadExecutionFileWindow = (offset: number, maxBytes: number) => Promise<ExecutionFileReadWindow>;

/** The contiguous bytes one paged read retained, and why it stopped. */
export type ExecutionFilePage = {
  startOffset: number;
  nextOffset: number;
  observedSizeBytes: number;
  data: Uint8Array;
  state: ExecutionFileLifecycle;
  stop: "complete" | "budget" | "stalled";
  requests: number;
};

/** Where a follow stands after the frames it has accepted so far. */
export type ExecutionFileFollowCursor = {
  identity: ExecutionFileIdentity | null;
  offset: number;
  availableLength: number;
  phase: "connecting" | "producing" | "finished" | "abandoned";
};

/* === Public Functions === */

/**
 * Read `[offset, offset + budgetBytes)` in windows no larger than the contract
 * bound, continuing from the bytes each window actually returned. An empty
 * window before the observed end stops the read as `stalled` rather than
 * asking again, so a server that cannot make progress is never polled in a loop.
 */
export async function pageExecutionFile(
  read: ReadExecutionFileWindow,
  { offset = 0, budgetBytes }: { offset?: number; budgetBytes: number },
): Promise<ExecutionFilePage> {
  if (!Number.isSafeInteger(offset) || offset < 0) {
    throw new RangeError("execution file offset is invalid");
  }
  if (!Number.isSafeInteger(budgetBytes) || budgetBytes < 1) {
    throw new RangeError("execution file read budget is invalid");
  }

  // Phase: First Window Establishes Where The Server Actually Started
  const chunks: Uint8Array[] = [];
  let requests = 0;
  let retained = 0;
  let startOffset = offset;
  let nextOffset = offset;
  let observedSizeBytes = 0;
  let state: ExecutionFileLifecycle = "static";
  let stop: ExecutionFilePage["stop"] = "budget";

  // Phase: Bounded Continuation From The Bytes Actually Returned
  while (retained < budgetBytes) {
    const maxBytes = Math.min(EXECUTION_FILE_WINDOW_BYTES, budgetBytes - retained);
    const window = await read(nextOffset, maxBytes);
    requests += 1;
    if (window.data.byteLength > maxBytes) {
      throw new RangeError("execution file window exceeds the requested bound");
    }
    if (requests === 1) {
      // The server clamps an offset past the end to that end.
      startOffset = window.offset;
      nextOffset = window.offset;
    } else if (window.offset !== nextOffset) {
      throw new Error("execution file changed while it was being paged");
    }
    observedSizeBytes = window.observedSizeBytes;
    state = window.state;
    if (window.data.byteLength > 0) {
      chunks.push(window.data);
      retained += window.data.byteLength;
      nextOffset += window.data.byteLength;
    }
    if (nextOffset >= observedSizeBytes) {
      stop = "complete";
      break;
    }
    if (window.data.byteLength === 0) {
      stop = "stalled";
      break;
    }
  }

  return {
    startOffset,
    nextOffset,
    observedSizeBytes,
    data: concatenate(chunks, retained),
    state,
    stop,
    requests,
  };
}

/** A cursor that has accepted nothing yet; a resume carries its identity. */
export function connectingFollowCursor(identity: ExecutionFileIdentity | null = null, offset = 0): ExecutionFileFollowCursor {
  return { identity, offset, availableLength: offset, phase: "connecting" };
}

/**
 * Accept one follow frame, or refuse it. A refusal is a protocol failure the
 * viewer reports; it is never retried silently.
 */
export function advanceFollowCursor(cursor: ExecutionFileFollowCursor, event: ExecutionFileFollowEvent): ExecutionFileFollowCursor {
  switch (event.kind) {
    case "metadata": {
      if (cursor.phase !== "connecting") {
        throw new Error("follow sent a second metadata frame");
      }
      if (event.offset !== cursor.offset || event.availableLength < event.offset) {
        throw new Error("follow metadata does not start at the requested offset");
      }
      if (cursor.identity && !sameIdentity(cursor.identity, event.identity)) {
        throw new Error("followed file was replaced; refresh to read it again");
      }
      return {
        identity: event.identity,
        offset: event.offset,
        availableLength: event.availableLength,
        phase: event.phase,
      };
    }
    case "chunk": {
      if (cursor.phase === "connecting") {
        throw new Error("follow sent data before its metadata");
      }
      if (event.offset !== cursor.offset || event.data.byteLength === 0 || event.data.byteLength > EXECUTION_FILE_WINDOW_BYTES) {
        throw new Error("follow chunks are empty, oversized or not contiguous");
      }
      const offset = cursor.offset + event.data.byteLength;
      return {
        ...cursor,
        offset,
        availableLength: Math.max(cursor.availableLength, offset),
      };
    }
    case "terminal": {
      if (cursor.phase === "connecting") {
        throw new Error("follow ended before its metadata");
      }
      if (event.length !== cursor.offset) {
        throw new Error("follow terminal length differs from the bytes delivered");
      }
      if (event.phase === "finished" && event.sha256 === null) {
        throw new Error("finished follow carries no sealed digest");
      }
      return { ...cursor, availableLength: event.length, phase: event.phase };
    }
  }
}

/** The offset a follow should resume from to show at most `tailBytes`. */
export function followTailOffset(availableLength: number, tailBytes: number): number {
  return Math.max(0, availableLength - tailBytes);
}

/* === Private Functions === */

function sameIdentity(left: ExecutionFileIdentity, right: ExecutionFileIdentity): boolean {
  return (
    left.store === right.store &&
    left.epoch === right.epoch &&
    left.fileId === right.fileId &&
    left.executionId === right.executionId &&
    left.relativePath === right.relativePath
  );
}

function concatenate(chunks: Uint8Array[], length: number): Uint8Array {
  if (chunks.length === 1) {
    return chunks[0];
  }
  const bytes = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return bytes;
}
