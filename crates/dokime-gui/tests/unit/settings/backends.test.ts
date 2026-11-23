// @vitest-environment happy-dom
/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Backend Registry Behavior
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { fireEvent, render } from "@testing-library/svelte";
import { beforeEach, test, vi } from "vitest";

// Local Imports
import { launchDesktopSession } from "@/app/native-session/launch";
import { NativeSession } from "@/app/native-session/session";
import ConnectionScreen from "@/app/native-session/ConnectionScreen.svelte";
import type { SessionOwner } from "@/app/native-session/session-state.svelte";
import BackendsTable from "@/features/settings/BackendsTable.svelte";
import { commands } from "@/lib/desktop/commands";
import type { DesktopSessionSnapshot } from "@/lib/desktop/session";
import { preferencesCommands } from "@/lib/desktop/settings";
import { toast } from "@/lib/notifications";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  invoke: vi.fn(),
  identity: vi.fn(),
  cycles: vi.fn(),
  cancel: vi.fn(),
  clear: vi.fn(),
  invalidate: vi.fn(),
  restart: vi.fn(),
  select: vi.fn(),
}));

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function connected(generation = "8"): DesktopSessionSnapshot {
  return {
    statusRevision: 1,
    ownership: "attached",
    phase: null,
    archivedBytes: null,
    archive: null,
    connection: {
      generation,
      phase: "connected",
      endpoint: "http://127.0.0.1:8765",
      remote: false,
      error: null,
      backend: {
        backendId: "fixture",
        epoch: "epoch",
        displayName: "Fixture",
        productVersion: "1",
        sourceRevision: null,
        channel: null,
        target: "fixture",
        apiVersion: "1",
        descriptorSha256: "a".repeat(64),
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
}

/* === Entry Point === */

vi.mock("@tauri-apps/api/core", () => ({ invoke: fixture.invoke }));
vi.mock("@/lib/host/client", () => ({ statusGet: fixture.identity, backendOverviewGet: fixture.cycles }));
vi.mock("@/lib/api/native-plugins", () => ({ restartNativePlugin: fixture.restart }));
vi.mock("@/app/query-client", () => ({
  createDokimeQueryClient: () => ({ cancelQueries: fixture.cancel, clear: fixture.clear, invalidateQueries: fixture.invalidate }),
}));

beforeEach(() => {
  vi.clearAllMocks();
  fixture.cycles.mockResolvedValue({ overview: { sessionId: "epoch" } });
  fixture.identity.mockResolvedValue({ status: { identity: { hostId: "fixture" } }, resolvedAt: { history: { epoch: "epoch" } } });
  fixture.invoke.mockImplementation(async (command: string) => {
    if (command === "desktop_backend_use") {
      return connected().connection;
    }
    if (command === "desktop_backends_list") {
      return { profiles: [] };
    }
    if (command === "desktop_session_status") {
      return connected();
    }
    if (command === "desktop_session_disconnect") {
      return { ...connected(), connection: { ...connected().connection, phase: "disconnected" } };
    }
    throw new Error(`Unexpected Command: ${command}`);
  });
});

test("Profile Selection Uses the Shared Native Registry Before Negotiating Identity", async () => {
  const owner = new NativeSession();
  await owner.select({ kind: "profile", identifier: "lab" });
  assert.deepEqual(
    fixture.invoke.mock.calls.map(([command, args]) => [command, args]),
    [
      ["desktop_backend_use", { identifier: "lab" }],
      ["desktop_session_status", undefined],
    ],
  );
  const state = owner.snapshot();
  assert.equal(state.phase, "connected");
  if (state.phase !== "connected") {
    throw new Error("Expected Connected Session");
  }
  assert.equal(state.session.identity.generation, "8");
  assert.equal(state.session.identity.isCurrent(), true);
  await owner.disconnect();
  assert.equal(state.session.identity.isCurrent(), false);
});

test("Failed Selection Stays Failed Until an Explicit New Selection", async () => {
  const failure = { kind: "identity_changed" };
  fixture.invoke.mockRejectedValueOnce(failure);
  const owner = new NativeSession();
  await owner.select({ kind: "profile", identifier: "stale" });
  assert.deepEqual(owner.snapshot(), { phase: "failed", error: failure });
  assert.equal(fixture.invoke.mock.calls.length, 1);
  assert.equal(fixture.identity.mock.calls.length, 0);
  await owner.select({ kind: "profile", identifier: "lab" });
  assert.equal(owner.snapshot().phase, "connected");
});

test("Plugin Recovery Uses Its Connected Session and Refreshes Settings After Success", async () => {
  const owner = new NativeSession();
  await owner.select({ kind: "profile", identifier: "lab" });
  const state = owner.snapshot();
  assert.equal(state.phase, "connected");
  if (state.phase !== "connected") {
    throw new Error("Expected Connected Session");
  }

  await state.session.restartPlugin("bench");
  assert.deepEqual(fixture.restart.mock.calls, [[state.session.identity, "bench"]]);
  assert.deepEqual(fixture.invalidate.mock.calls, [[{ queryKey: ["settings"] }]]);

  const failure = new Error("fixture-restart");
  fixture.restart.mockRejectedValueOnce(failure);
  await assert.rejects(state.session.restartPlugin("bench"), (error) => error === failure);
  assert.equal(fixture.invalidate.mock.calls.length, 1);
  await owner.disconnect();
});

test("Unavailable Host Recovery Preserves the Failure Without Announcing Success", async () => {
  const announce = vi.spyOn(toast, "success").mockReturnValue("fixture");
  const failure = { kind: "storage_reset", reason: "failed", message: "recovery unavailable" };
  fixture.invoke.mockRejectedValueOnce(failure);
  const owner = new NativeSession();
  const states: unknown[] = [];
  owner.subscribe(() => states.push(owner.snapshot()));
  try {
    await owner.backupAndResetLocalStorage("/fixture/data", "/fixture/saved.zip");
    assert.deepEqual(fixture.invoke.mock.calls[0], [
      "desktop_backup_reset_local_storage",
      { directory: "/fixture/data", archive: "/fixture/saved.zip" },
    ]);
    assert.deepEqual(states[0], { phase: "connecting", reset: "backup" });
    assert.deepEqual(owner.snapshot(), { phase: "failed", error: failure });
    assert.equal(announce.mock.calls.length, 0);
    assert.equal(fixture.identity.mock.calls.length, 0);
    await owner.disconnect();
  } finally {
    announce.mockRestore();
  }
});

test("Deleting the Active Backend Invalidates Its Generation and Owned Resources", async () => {
  const owner = new NativeSession();
  await owner.select({ kind: "profile", identifier: "lab" });
  const state = owner.snapshot();
  if (state.phase !== "connected") {
    throw new Error("Expected Connected Session");
  }
  const dispose = vi.fn();
  state.session.own(dispose);
  fixture.invoke.mockResolvedValueOnce({ profiles: [] });
  await preferencesCommands.desktopBackendRemove("lab");
  await owner.disconnect();
  assert.equal(state.session.identity.isCurrent(), false);
  assert.equal(dispose.mock.calls.length, 1);
  assert.equal(fixture.cancel.mock.calls.length, 1);
  assert.equal(fixture.clear.mock.calls.length, 1);
  assert.equal(owner.snapshot().phase, "disconnected");
});

test("Pairing Commands Pass Only Identifiers and Saved Paths Across IPC", async () => {
  const document = { profiles: [] };
  fixture.invoke.mockResolvedValueOnce(document);
  await preferencesCommands.desktopBackendImportPairing("lab");
  assert.deepEqual(fixture.invoke.mock.calls.at(-1)?.slice(0, 2), ["desktop_backend_import_pairing", { identifier: "lab" }]);

  fixture.invoke.mockResolvedValueOnce(null);
  await preferencesCommands.desktopBackendImportPairing(null);
  assert.deepEqual(fixture.invoke.mock.calls.at(-1)?.slice(0, 2), ["desktop_backend_import_pairing", { identifier: null }]);

  fixture.invoke.mockResolvedValueOnce("/private/Lab.pair");
  await commands.desktopPairingCreate();
  assert.deepEqual(fixture.invoke.mock.calls.at(-1)?.slice(0, 2), ["desktop_pairing_create", undefined]);
});

test("Startup Without a Pairing Leaves Selection to the Operator", async () => {
  fixture.invoke.mockImplementation(async (command: string) => {
    if (command === "desktop_session_launch") {
      return { target: null, expectedBackendId: null, localBackendAvailable: true, error: null };
    }
    if (command === "desktop_session_status") {
      return { ...connected(), connection: { ...connected().connection, phase: "disconnected", backend: null } };
    }
    if (command === "desktop_backends_list") {
      return { profiles: [] };
    }
    throw new Error(`Unexpected Command: ${command}`);
  });
  const owner = new NativeSession();
  const select = vi.spyOn(owner, "select");
  const published = vi.fn();
  await launchDesktopSession(owner, () => true, published);
  assert.deepEqual(
    fixture.invoke.mock.calls.map(([command]) => command),
    ["desktop_session_launch", "desktop_session_status"],
  );
  assert.equal(published.mock.calls.length, 1);
  assert.equal(select.mock.calls.length, 0);
  assert.equal(owner.snapshot().phase, "disconnected");
});

const launch = { target: null, expectedBackendId: null, localBackendAvailable: true, error: null };
const disconnected: SessionOwner = {
  snapshot: () => ({ phase: "disconnected" }),
  subscribe: () => () => {},
  select: fixture.select,
  disconnect: async () => {},
};

test("Backend Loading Stays in the Table's Status Slot", () => {
  const table = render(BackendsTable, { owner: disconnected });
  const status = table.getByRole("status");
  assert.equal(status.closest("table"), table.getByRole("table"));
  assert.equal(table.queryByRole("alert"), null);
  table.unmount();
});

test("Occupied Local Remains Usable", async () => {
  const failure = { kind: "backend_running", root: "C:/fixture/data", message: "fixture lease refusal: C:/fixture/data" };
  fixture.invoke.mockRejectedValueOnce(failure);
  const owner = new NativeSession();
  await owner.select({ kind: "owned_local" });
  assert.deepEqual(owner.snapshot(), { phase: "failed", error: failure });
  assert.equal(fixture.invoke.mock.calls.length, 1);

  const screen = render(ConnectionScreen, { owner: disconnected, launch, launchError: failure, onSettings: () => {} });
  const alert = screen.getByRole("alert");
  assert.equal(alert.closest("table"), null);
  await fireEvent.click(screen.getByRole("button", { name: "Reload" }));
  assert.deepEqual(fixture.select.mock.calls, [[{ kind: "owned_local" }]]);
  assert.ok(screen.getByRole("button", { name: "Settings" }));
  screen.unmount();

  await owner.select({ kind: "profile", identifier: "lab" });
  assert.equal(owner.snapshot().phase, "connected");
});

test("Remote Import Starts from the Backend Toolbar", async () => {
  const table = render(BackendsTable, { owner: disconnected });
  const action = table.getByRole("button", { name: "Import Remote" });
  assert.ok(action.closest("header"));
  fixture.invoke.mockResolvedValueOnce({ profiles: [] });
  await fireEvent.click(action);
  assert.deepEqual(fixture.invoke.mock.calls.at(-1)?.slice(0, 2), ["desktop_backend_import_pairing", { identifier: null }]);
  table.unmount();
});

test("Remote Import Refresh Retains the Native Connection Failure", async () => {
  const snapshot = connected();
  fixture.invoke.mockResolvedValueOnce({
    ...snapshot,
    connection: { ...snapshot.connection, phase: "failed", backend: null, error: { kind: "identity_changed" } },
  });
  const owner = new NativeSession();
  await owner.refresh();
  assert.deepEqual(owner.snapshot(), { phase: "failed", error: { kind: "identity_changed" } });
});

test("Readiness Separates Status History from the Product Execution Epoch", async () => {
  fixture.identity.mockResolvedValue({
    status: { identity: { hostId: "fixture" } },
    resolvedAt: { history: { epoch: "library-status-history" } },
  });
  const owner = new NativeSession();
  await owner.select({ kind: "profile", identifier: "lab" });
  assert.equal(owner.snapshot().phase, "connected");
  await owner.disconnect();
  fixture.cycles.mockResolvedValue({ overview: { sessionId: "different-product-epoch" } });
  await owner.select({ kind: "profile", identifier: "lab" });
  assert.equal(owner.snapshot().phase, "failed");
});

test("Failed Backend Retains Its Endpoint Beside the Failure", () => {
  const endpoint = "http://127.0.0.1:8765";
  const screen = render(ConnectionScreen, {
    owner: disconnected,
    launch: { ...launch, target: { kind: "local", endpoint } },
    launchError: new Error("fixture-connect"),
    onSettings: () => {},
  });
  const alert = screen.getByRole("alert");
  assert.equal(alert.nextElementSibling?.textContent, endpoint);
  screen.unmount();
});

test("Startup Recovery Feedback", async () => {
  const diagnostics = { cause: "fixture-startup-cause", log: "C:/fixture/startup.log" };
  const error = { kind: "startup", message: diagnostics.cause, diagnostics };
  const publish = vi.spyOn(toast, "error").mockReturnValue("fixture");
  try {
    const screen = render(ConnectionScreen, { owner: disconnected, launch, launchError: error, onSettings: () => {} });
    const alerts = screen.getAllByRole("alert");
    assert.equal(alerts.length, 1);
    assert.ok(alerts[0].textContent?.includes(diagnostics.cause));
    assert.equal(publish.mock.calls[0]?.[1]?.context, "background");
    assert.ok(screen.container.querySelector('[title="C:/fixture/startup.log"]'));
    await fireEvent.click(screen.getByRole("button", { name: "Reload" }));
    assert.deepEqual(fixture.select.mock.calls, [[{ kind: "owned_local" }]]);
    screen.unmount();
  } finally {
    publish.mockRestore();
  }
});
