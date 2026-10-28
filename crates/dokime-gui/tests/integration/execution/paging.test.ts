/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import assert from "node:assert/strict";

import { test } from "vitest";

import { EXECUTION_FILE_WINDOW_BYTES, pageExecutionFile } from "@/lib/execution-file-paging";
import type { ExecutionFileLifecycle, ExecutionFileReadWindow } from "@/lib/types";

function boundedServer(content: Uint8Array, { state = "finalized", shrinkTo = Number.POSITIVE_INFINITY } = {}) {
  const requests: { offset: number; maxBytes: number }[] = [];
  const read = async (offset: number, maxBytes: number): Promise<ExecutionFileReadWindow> => {
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
      state: state as ExecutionFileLifecycle,
    } as ExecutionFileReadWindow;
  };
  return { read, requests };
}

function bytes(length: number) {
  return Uint8Array.from({ length }, (_, index) => index % 251);
}

test("File Paging", async () => {
  // A Small File Loads In One Request
  const small = new TextEncoder().encode("x".repeat(314));
  const streaming = boundedServer(small, { state: "streaming" });
  const smallPage = await pageExecutionFile(streaming.read, { budgetBytes: 256 * 1024 });
  assert.equal(streaming.requests.length, 1);
  assert.equal(smallPage.stop, "complete");
  assert.equal(smallPage.nextOffset, 314);
  assert.equal(smallPage.state, "streaming");
  assert.deepEqual(smallPage.data, small);

  // A File Larger Than The Bound Is Read In Bounded Pages With Offset Continuation
  const large = bytes(3 * EXECUTION_FILE_WINDOW_BYTES + 17);
  const bounded = boundedServer(large);
  const largePage = await pageExecutionFile(bounded.read, { budgetBytes: 256 * 1024 });
  assert.equal(largePage.stop, "complete");
  assert.equal(bounded.requests.length, 4);
  assert.ok(bounded.requests.every(({ maxBytes }) => maxBytes <= EXECUTION_FILE_WINDOW_BYTES));
  assert.deepEqual(
    bounded.requests.map(({ offset }) => offset),
    [0, 1, 2, 3].map((index) => index * EXECUTION_FILE_WINDOW_BYTES),
  );
  assert.deepEqual(largePage.data, large);

  // Continuation Follows The Bytes The Server Actually Returned
  const shrinking = bytes(10_000);
  const short = boundedServer(shrinking, { shrinkTo: 4_000 });
  const shortPage = await pageExecutionFile(short.read, { budgetBytes: 256 * 1024 });
  assert.deepEqual(
    short.requests.map(({ offset }) => offset),
    [0, 4_000, 8_000],
  );
  assert.deepEqual(shortPage.data, shrinking);

  // The Budget Caps Retained Bytes And The Last Request
  const budgeted = boundedServer(bytes(5 * EXECUTION_FILE_WINDOW_BYTES));
  const budgetPage = await pageExecutionFile(budgeted.read, { budgetBytes: 100_000 });
  assert.equal(budgetPage.stop, "budget");
  assert.equal(budgetPage.data.byteLength, 100_000);
  assert.equal(budgeted.requests.at(-1)?.maxBytes, 100_000 - EXECUTION_FILE_WINDOW_BYTES);

  // An Empty Window Before The Observed End Stops Instead Of Spinning
  let calls = 0;
  const stalled = await pageExecutionFile(
    async (offset) => {
      calls += 1;
      return {
        offset,
        observedSizeBytes: 314,
        data: new Uint8Array(),
        state: "streaming",
      } as ExecutionFileReadWindow;
    },
    { budgetBytes: 256 * 1024 },
  );
  assert.equal(calls, 1);
  assert.equal(stalled.stop, "stalled");

  // An Empty File Is Complete After One Request
  const empty = boundedServer(new Uint8Array());
  const emptyPage = await pageExecutionFile(empty.read, { budgetBytes: 256 * 1024 });
  assert.equal(empty.requests.length, 1);
  assert.equal(emptyPage.stop, "complete");
});
