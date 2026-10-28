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

import {
  currentCursor,
  firstPage,
  formatDuration,
  newerPage,
  olderPage,
  type RemoteSession,
  sessionRow,
} from "@/features/settings/sections/remote-sessions";

test("a rejected session row carries its outcome, reason and closed duration", () => {
  const session = {
    id: "7",
    peerAddress: "192.168.1.20:51000",
    principal: "anonymous",
    connectedAt: 1_000n,
    lastSeenAt: 61_000n,
    disconnectedAt: 61_000n,
    calls: 3n,
    outcome: { HarmosVariant: { case: "Rejected", value: {} } },
    rejection: "invalid secrets",
  } as RemoteSession;
  const row = sessionRow(session, (millis) => `t${millis}`);
  assert.deepEqual(row, {
    key: "7",
    peer: "192.168.1.20:51000",
    principal: "Anonymous",
    connected: "t1000",
    lastSeen: "t61000",
    duration: "1 min 0 s",
    calls: "3",
    outcome: "Rejected",
    rejection: "invalid secrets",
    open: false,
  });
  const open = sessionRow({ ...session, principal: "access-key", disconnectedAt: undefined }, String);
  assert.equal(open.open, true);
  assert.equal(open.principal, "Access Key");
});

test("durations read in seconds, minutes and hours", () => {
  assert.equal(formatDuration(-5), "0 s");
  assert.equal(formatDuration(45_000), "45 s");
  assert.equal(formatDuration(192_000), "3 min 12 s");
  assert.equal(formatDuration(7_500_000), "2 h 05 min");
});

test("paging walks older cursors and back to the newest page", () => {
  assert.equal(currentCursor(firstPage), undefined);
  const second = olderPage(firstPage, "481");
  const third = olderPage(second, "461");
  assert.equal(currentCursor(third), "461");
  assert.equal(currentCursor(newerPage(third)), "481");
  assert.deepEqual(newerPage(newerPage(newerPage(third))), firstPage);
});
