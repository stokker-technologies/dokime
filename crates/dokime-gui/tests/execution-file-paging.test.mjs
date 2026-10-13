/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import assert from "node:assert/strict";
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

/* === Constants === */

const gui = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const {
  EXECUTION_FILE_WINDOW_BYTES,
  advanceFollowCursor,
  connectingFollowCursor,
  followTailOffset,
  pageExecutionFile,
} = await import(pathToFileURL(resolve(gui, "src/lib/execution-file-paging.ts")).href);

const IDENTITY = {
  store: "store",
  epoch: "epoch",
  fileId: "file",
  executionId: "execution",
  relativePath: "mini.charger.steady/stream.log",
};

/* === Fixture === */

/** A server that refuses any window over the contract bound, as the backend does. */
function boundedServer(content, { state = "finalized", shrinkTo = Number.POSITIVE_INFINITY } = {}) {
  const requests = [];
  const read = async (offset, maxBytes) => {
    requests.push({ offset, maxBytes });
    if (maxBytes < 1 || maxBytes > EXECUTION_FILE_WINDOW_BYTES) {
      throw new Error("max_bytes must be between 0 and 65536");
    }
    const start = Math.min(offset, content.byteLength);
    const end = Math.min(content.byteLength, start + Math.min(maxBytes, shrinkTo));
    return {
      offset: start,
      observedSizeBytes: content.byteLength,
      data: content.subarray(start, end),
      state,
    };
  };
  return { read, requests };
}

function bytes(length) {
  return Uint8Array.from({ length }, (_, index) => index % 251);
}

/* === Paging === */

test("a small file loads in one request", async () => {
  const content = new TextEncoder().encode("x".repeat(314));
  const server = boundedServer(content, { state: "streaming" });
  const page = await pageExecutionFile(server.read, { budgetBytes: 256 * 1024 });
  assert.equal(server.requests.length, 1);
  assert.equal(page.stop, "complete");
  assert.equal(page.nextOffset, 314);
  assert.equal(page.state, "streaming");
  assert.deepEqual(page.data, content);
});

test("a file larger than the bound is read in bounded pages with offset continuation", async () => {
  const content = bytes(3 * EXECUTION_FILE_WINDOW_BYTES + 17);
  const server = boundedServer(content);
  const page = await pageExecutionFile(server.read, { budgetBytes: 256 * 1024 });
  assert.equal(page.stop, "complete");
  assert.equal(server.requests.length, 4);
  assert.ok(server.requests.every(({ maxBytes }) => maxBytes <= EXECUTION_FILE_WINDOW_BYTES));
  assert.deepEqual(
    server.requests.map(({ offset }) => offset),
    [0, 1, 2, 3].map((index) => index * EXECUTION_FILE_WINDOW_BYTES),
  );
  assert.deepEqual(page.data, content);
});

test("continuation follows the bytes the server actually returned", async () => {
  const content = bytes(10_000);
  const server = boundedServer(content, { shrinkTo: 4_000 });
  const page = await pageExecutionFile(server.read, { budgetBytes: 256 * 1024 });
  assert.deepEqual(
    server.requests.map(({ offset }) => offset),
    [0, 4_000, 8_000],
  );
  assert.deepEqual(page.data, content);
});

test("the budget caps retained bytes and the last request", async () => {
  const content = bytes(5 * EXECUTION_FILE_WINDOW_BYTES);
  const server = boundedServer(content);
  const page = await pageExecutionFile(server.read, { budgetBytes: 100_000 });
  assert.equal(page.stop, "budget");
  assert.equal(page.data.byteLength, 100_000);
  assert.equal(server.requests.at(-1).maxBytes, 100_000 - EXECUTION_FILE_WINDOW_BYTES);
});

test("an empty window before the observed end stops instead of spinning", async () => {
  let calls = 0;
  const page = await pageExecutionFile(
    async (offset) => {
      calls += 1;
      return { offset, observedSizeBytes: 314, data: new Uint8Array(), state: "streaming" };
    },
    { budgetBytes: 256 * 1024 },
  );
  assert.equal(calls, 1);
  assert.equal(page.stop, "stalled");
});

test("an empty file is complete after one request", async () => {
  const server = boundedServer(new Uint8Array());
  const page = await pageExecutionFile(server.read, { budgetBytes: 256 * 1024 });
  assert.equal(server.requests.length, 1);
  assert.equal(page.stop, "complete");
});

/* === Follow === */

test("a follow accepts metadata, contiguous chunks and a matching terminal", () => {
  let cursor = connectingFollowCursor();
  cursor = advanceFollowCursor(cursor, {
    kind: "metadata",
    identity: IDENTITY,
    phase: "producing",
    availableLength: 0,
    offset: 0,
  });
  assert.equal(cursor.phase, "producing");
  cursor = advanceFollowCursor(cursor, { kind: "chunk", offset: 0, data: bytes(200) });
  cursor = advanceFollowCursor(cursor, { kind: "chunk", offset: 200, data: bytes(114) });
  assert.equal(cursor.offset, 314);
  cursor = advanceFollowCursor(cursor, {
    kind: "terminal",
    phase: "finished",
    length: 314,
    sha256: "0".repeat(64),
  });
  assert.equal(cursor.phase, "finished");
});

test("a follow refuses gaps, early data, short terminals and a replaced identity", () => {
  const started = advanceFollowCursor(connectingFollowCursor(), {
    kind: "metadata",
    identity: IDENTITY,
    phase: "producing",
    availableLength: 10,
    offset: 0,
  });
  assert.throws(() =>
    advanceFollowCursor(connectingFollowCursor(), { kind: "chunk", offset: 0, data: bytes(1) }),
  );
  assert.throws(() => advanceFollowCursor(started, { kind: "chunk", offset: 5, data: bytes(1) }));
  assert.throws(() => advanceFollowCursor(started, { kind: "chunk", offset: 0, data: bytes(0) }));
  assert.throws(() =>
    advanceFollowCursor(started, { kind: "terminal", phase: "abandoned", length: 3, sha256: null }),
  );
  assert.throws(() =>
    advanceFollowCursor(connectingFollowCursor(IDENTITY, 10), {
      kind: "metadata",
      identity: { ...IDENTITY, fileId: "other" },
      phase: "producing",
      availableLength: 10,
      offset: 10,
    }),
  );
});

test("a large follow starts at the tail and a small one at zero", () => {
  assert.equal(followTailOffset(314, 64 * 1024), 0);
  assert.equal(followTailOffset(200_000, 64 * 1024), 200_000 - 64 * 1024);
});
