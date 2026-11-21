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
import { Children, isValidElement, type ReactElement, type ReactNode } from "react";
import { beforeEach, test, vi } from "vitest";

// Local Imports
import { AppRoot } from "@/app/AppRoot";
import { NativeSession } from "@/app/native-session/session";
import { DataTable } from "@dokime/plots/components/data-table/DataTable";
import { ErrorCard } from "@dokime/plots/components/ErrorCard";
import { PathPill } from "@dokime/document/components/PathPill";
import { Button } from "@dokime/plots/components/ui/button";
import { BackendsTable } from "@/features/settings/BackendsTable";
import { commands } from "@/lib/desktop/commands";
import type { DesktopSessionSnapshot } from "@/lib/desktop/session";
import { preferencesCommands } from "@/lib/desktop/settings";
import { toast } from "@dokime/plots/lib/notifications";

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
  states: [] as unknown[],
  phase: "disconnected",
  effects: [] as (() => () => void)[],
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

vi.mock("react", async (original) => ({
  ...(await original<typeof import("react")>()),
  useCallback: (callback: unknown) => callback,
  useMemo: (factory: () => unknown) => factory(),
  useState: (initial: unknown) => [
    fixture.states.length ? fixture.states.shift() : typeof initial === "function" ? initial() : initial,
    vi.fn(),
  ],
  useRef: (initial: unknown) => ({ current: initial }),
  useSyncExternalStore: () => ({ phase: fixture.phase }),
  useEffect: (effect: () => () => void) => {
    fixture.effects.push(effect);
  },
}));
vi.mock("@/app/native-session/context", () => ({
  useNativeSessionController: () => ({ select: fixture.select }),
  useNativeSessionOptional: () => null,
}));
vi.mock("@/app/router", () => ({ router: {} }));
vi.mock("@/app/AppProviders", () => ({ AppProviders: "fixture-providers" }));
vi.mock("@/app/native-session/ConnectionScreen", () => ({ ConnectionScreen: "fixture-connection" }));
vi.mock("@/app/native-session/NativeSessionProvider", () => ({ NativeSessionProvider: "fixture-session" }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: fixture.invoke }));
vi.mock("@/lib/host/client", () => ({ statusGet: fixture.identity, backendOverviewGet: fixture.cycles }));
vi.mock("@/lib/api/native-plugins", () => ({ restartNativePlugin: fixture.restart }));
vi.mock("@/app/query-client", () => ({
  createDokimeQueryClient: () => ({ cancelQueries: fixture.cancel, clear: fixture.clear, invalidateQueries: fixture.invalidate }),
}));

beforeEach(() => {
  vi.clearAllMocks();
  fixture.effects.length = 0;
  fixture.states.length = 0;
  fixture.phase = "disconnected";
  fixture.cycles.mockResolvedValue({ overview: { sessionId: "epoch" } });
  fixture.identity.mockResolvedValue({ status: { identity: { hostId: "fixture" } }, resolvedAt: { history: { epoch: "epoch" } } });
  fixture.invoke.mockImplementation(async (command: string) => {
    if (command === "desktop_backend_use") return connected().connection;
    if (command === "desktop_session_status") return connected();
    if (command === "desktop_session_disconnect")
      return { ...connected(), connection: { ...connected().connection, phase: "disconnected" } };
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
  if (state.phase !== "connected") throw new Error("Expected Connected Session");
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
  if (state.phase !== "connected") throw new Error("Expected Connected Session");

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
  if (state.phase !== "connected") throw new Error("Expected Connected Session");
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
    if (command === "desktop_session_launch") return { target: null, expectedBackendId: null, localBackendAvailable: true, error: null };
    if (command === "desktop_session_status")
      return { ...connected(), connection: { ...connected().connection, phase: "disconnected", backend: null } };
    if (command === "desktop_backends_list") return { profiles: [] };
    throw new Error(`Unexpected Command: ${command}`);
  });
  AppRoot();
  const cleanup = fixture.effects[0]();
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.deepEqual(
    fixture.invoke.mock.calls.map(([command]) => command),
    ["desktop_session_launch", "desktop_session_status"],
  );
  assert.equal(fixture.select.mock.calls.length, 0);
  cleanup();
});

type TestElementProps = {
  children?: ReactNode;
  actions?: ReactNode;
  failure?: unknown;
  loading?: boolean;
  onRetry?: () => void;
  notificationContext?: string;
  retry?: (profile: string | null) => void;
  tabIndex?: number;
  role?: string;
  value?: string;
  "aria-disabled"?: boolean;
  onClick?: () => void;
  onKeyDown?: (event: { key: string; target: object; currentTarget: object; preventDefault: () => void }) => void;
};

function descendants(node: unknown): ReactElement<TestElementProps>[] {
  if (!isValidElement<TestElementProps>(node)) return [];
  return [node, ...Children.toArray(node.props.children).flatMap(descendants)];
}

test("Backend Loading Stays in the Table's Status Slot", () => {
  const table = descendants(BackendsTable()).find((element) => element.type === DataTable);
  assert.ok(table);
  assert.equal(table.props.loading, true);
  assert.equal(table.props.failure, undefined);
});

test("Occupied Local Remains Usable", async () => {
  const failure = { kind: "backend_running", root: "C:/fixture/data", message: "fixture lease refusal: C:/fixture/data" };
  fixture.invoke.mockRejectedValueOnce(failure);
  const owner = new NativeSession();
  await owner.select({ kind: "owned_local" });
  assert.deepEqual(owner.snapshot(), { phase: "failed", error: failure });
  assert.equal(fixture.invoke.mock.calls.length, 1);

  const { ConnectionScreen } = await vi.importActual<typeof import("@/app/native-session/ConnectionScreen")>(
    "@/app/native-session/ConnectionScreen",
  );
  const tree = ConnectionScreen({
    launch: { target: null, expectedBackendId: null, localBackendAvailable: true, error: null },
    launchError: failure,
  });
  const contents = descendants(tree);
  const registry = contents.find((element) => element.type === BackendsTable);
  assert.ok(registry);
  assert.equal(registry.props.failure, failure);
  assert.ok(registry.props.retry);
  registry.props.retry(null);
  assert.deepEqual(fixture.select.mock.calls, [[{ kind: "owned_local" }]]);
  assert.ok(contents.some((element) => element.type === Button && element.props.children === "Settings"));

  await owner.select({ kind: "profile", identifier: "lab" });
  assert.equal(owner.snapshot().phase, "connected");
});

test("Remote Import Starts from the Backend Toolbar", async () => {
  const table = descendants(BackendsTable()).find((element) => element.props.actions);
  assert.ok(table);
  const action = descendants(table.props.actions).find((element) => element.type === Button && element.props.children === "Import Remote");
  assert.ok(action);
  fixture.invoke.mockResolvedValueOnce({ profiles: [] });
  await action.props.onClick?.();
  assert.deepEqual(fixture.invoke.mock.calls.at(-1)?.slice(0, 2), ["desktop_backend_import_pairing", { identifier: null }]);
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

test("Backend Failure Retains Its Record in the Shared Content Slot", () => {
  const error = new Error("bench_connection");
  const table = descendants(BackendsTable({ failure: error })).find((element) => element.type === DataTable);
  assert.ok(table);
  assert.equal(table.props.loading, false);
  const contents = descendants(table.props.failure);
  const cards = contents.filter((element) => element.type === ErrorCard);
  assert.equal(cards.length, 1);
  const failure = cards[0].props.failure;
  assert.ok(failure && typeof failure === "object" && "message" in failure);
  assert.equal(failure.message, error.message);
  assert.equal(contents.filter((element) => element.props.role === "status").length, 0);
});

test("Failed Backend Retains Its Endpoint Beside the Failure", () => {
  const endpoint = "http://127.0.0.1:8765";
  const table = descendants(BackendsTable({ failure: new Error("fixture-connect"), endpoint })).find(
    (element) => element.type === DataTable,
  );
  assert.ok(table);
  const contents = descendants(table.props.failure);
  assert.equal(contents.filter((element) => element.type === ErrorCard).length, 1);
  assert.ok(contents.some((element) => element.type === "p" && element.props.children === endpoint));
});

test("Startup Recovery Feedback", async () => {
  const { ConnectionScreen } = await vi.importActual<typeof import("@/app/native-session/ConnectionScreen")>(
    "@/app/native-session/ConnectionScreen",
  );
  const diagnostics = { cause: "fixture-startup-cause", log: "C:/fixture/startup.log" };
  const error = { kind: "startup", message: diagnostics.cause, diagnostics };
  const contents = descendants(
    ConnectionScreen({
      launch: {
        target: null,
        expectedBackendId: null,
        localBackendAvailable: true,
        error: null,
      },
      launchError: error,
    }),
  );
  const failures = contents.filter((element) => element.type === ErrorCard);
  assert.equal(failures.length, 1);
  assert.equal((failures[0].props.failure as { message: string }).message, diagnostics.cause);
  assert.equal(failures[0].props.notificationContext, "background");
  assert.ok(contents.some((element) => element.type === PathPill && element.props.value === diagnostics.log));
  failures[0].props.onRetry?.();
  assert.deepEqual(fixture.select.mock.calls, [[{ kind: "owned_local" }]]);
});
