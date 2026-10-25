/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { test, vi } from "vitest";

// Local Imports
import { NativeSession, type SessionControls } from "@/app/native-session/session";
import type { DesktopSessionSnapshot } from "@/lib/desktop/session";

/* === Constants === */

const disconnected: DesktopSessionSnapshot = {
  ownership: "none",
  selection: { kind: "profile", name: "laboratory" },
  phase: null,
  archivedBytes: null,
  connection: {
    generation: "1",
    phase: "disconnected",
    endpoint: null,
    remote: false,
    backend: null,
    error: null,
  },
};

vi.mock("@/lib/rpc/generated/client", () => ({
  backendGetIdentity: async () => ({ backendId: "backend", epoch: "epoch" }),
}));

/* === Tests === */

test("Failed Remote Selection", async () => {
  const select = vi.fn(async () => {
    throw new Error("Unavailable");
  });
  const controls: SessionControls = {
    select,
    status: async () => disconnected,
    disconnect: async () => disconnected,
    resetLocalStorage: async () => disconnected,
    backupResetLocalStorage: async () => disconnected,
  };
  const owner = new NativeSession(controls);
  await owner.select({ kind: "profile", name: "laboratory" });
  assert.equal(owner.snapshot().phase, "failed");
  assert.deepEqual(owner.snapshot().selection, { kind: "profile", name: "laboratory" });
  assert.deepEqual(select.mock.calls, [[{ kind: "profile", name: "laboratory" }]]);
  await owner.disconnect();
  assert.equal(owner.snapshot().phase, "disconnected");
  assert.deepEqual(owner.snapshot().selection, { kind: "profile", name: "laboratory" });
});

test("Transient Attach Privacy", async () => {
  const controls: SessionControls = {
    select: async () => {
      throw new Error("Unavailable");
    },
    status: async () => ({ ...disconnected, selection: null }),
    disconnect: async () => disconnected,
    resetLocalStorage: async () => disconnected,
    backupResetLocalStorage: async () => disconnected,
  };
  const owner = new NativeSession(controls);
  await owner.select({
    kind: "attach",
    request: {
      target: { kind: "remote", endpoint: "https://backend.example:50051", caCertificatePem: null },
      secrets: { kind: "bearer", token: "private-test-key", remember: false },
      expectedBackendId: null,
    },
  });
  assert.equal(owner.snapshot().phase, "failed");
  assert.equal(owner.snapshot().selection, null);
  assert.equal(JSON.stringify(owner.snapshot()).includes("private-test-key"), false);
});

test("Rejected Profile Preserves Connection", async () => {
  const connected: DesktopSessionSnapshot = {
    ...disconnected,
    ownership: "owned_local",
    selection: { kind: "owned_local" },
    connection: {
      ...disconnected.connection,
      phase: "connected",
      backend: {
        backendId: "backend",
        epoch: "epoch",
        displayName: "Fixture",
        productVersion: "0.1.0",
        target: "fixture",
        apiVersion: "v1",
        descriptorSha256: "fixture",
        methods: [],
        limits: {
          maxRequestBytes: "1024",
          maxResponseBytes: "1024",
          maxContentBytes: null,
          maxEvidenceBytes: null,
          maxChunkBytes: null,
          maxPageItems: null,
          maxStreamBufferItems: null,
        },
      },
    },
  };
  const refusal = new Error("Unknown backend profile: missing");
  const controls: SessionControls = {
    select: async (selection) => {
      if (selection.kind === "profile") throw refusal;
      return connected;
    },
    status: async () => connected,
    disconnect: async () => disconnected,
    resetLocalStorage: async () => disconnected,
    backupResetLocalStorage: async () => disconnected,
  };
  const owner = new NativeSession(controls);
  await owner.select({ kind: "owned_local" });
  await owner.select({ kind: "profile", name: "missing" });
  const state = owner.snapshot();
  assert.ok(state.phase === "connected");
  assert.deepEqual(state.selection, { kind: "owned_local" });
  assert.equal(state.session.snapshot.connection.backend?.backendId, "backend");
  assert.equal(state.session.identity.isCurrent(), true);
  assert.equal(state.selectionError, refusal);
});
