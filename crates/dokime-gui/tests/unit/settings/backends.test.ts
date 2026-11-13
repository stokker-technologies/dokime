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
import { NotificationMessage } from "@/components/NotificationMessage";
import { Button } from "@/components/ui/button";
import { BackendsTable } from "@/features/settings/BackendsTable";
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
vi.mock("@/app/query-client", () => ({ createDokimeQueryClient: () => ({ cancelQueries: fixture.cancel, clear: fixture.clear }) }));

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
  assert.deepEqual(await preferencesCommands.desktopBackendRemove("lab"), { profiles: [] });
  await owner.disconnect();
  assert.equal(state.session.identity.isCurrent(), false);
  assert.equal(dispose.mock.calls.length, 1);
  assert.equal(fixture.cancel.mock.calls.length, 1);
  assert.equal(fixture.clear.mock.calls.length, 1);
  assert.equal(owner.snapshot().phase, "disconnected");
});

test("Registry Commands Preserve Profile Fields Without Bearer Material", async () => {
  const profile = {
    identifier: "lab",
    label: "Lab",
    endpoint: "https://lab.local",
    endpoints: ["https://lab.local"],
    expected_backend_id: "fixture",
    certificate_pem: "fixture-ca",
    pairing_file: "/fixture/lab.pair",
  };
  const document = { profiles: [profile] };
  fixture.invoke.mockResolvedValue(document);
  assert.deepEqual(await preferencesCommands.desktopBackendsList(), document);
  assert.deepEqual(fixture.invoke.mock.calls.at(-1)?.slice(0, 2), ["desktop_backends_list", undefined]);
});

test("Pairing Commands Pass Only Identifiers and Saved Paths Across IPC", async () => {
  const document = { profiles: [] };
  fixture.invoke.mockResolvedValueOnce(document);
  assert.deepEqual(await preferencesCommands.desktopBackendImportPairing("lab"), document);
  assert.deepEqual(fixture.invoke.mock.calls.at(-1)?.slice(0, 2), ["desktop_backend_import_pairing", { identifier: "lab" }]);

  fixture.invoke.mockResolvedValueOnce(null);
  assert.equal(await preferencesCommands.desktopBackendImportPairing(null), null);
  assert.deepEqual(fixture.invoke.mock.calls.at(-1)?.slice(0, 2), ["desktop_backend_import_pairing", { identifier: null }]);

  fixture.invoke.mockResolvedValueOnce("/private/Lab.pair");
  assert.equal(await commands.desktopPairingCreate(), "/private/Lab.pair");
  assert.deepEqual(fixture.invoke.mock.calls.at(-1)?.slice(0, 2), ["desktop_pairing_create", undefined]);
  fixture.invoke.mockResolvedValueOnce(null);
  assert.equal(await commands.desktopPairingCreate(), null);
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
  failure?: ReactNode;
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

test("Startup Recovery Feedback", async () => {
  const { ConnectionScreen } = await vi.importActual<typeof import("@/app/native-session/ConnectionScreen")>(
    "@/app/native-session/ConnectionScreen",
  );
  const diagnostics = { cause: "fixture startup failure", log: "C:/fixture/application.log" };
  const tree = ConnectionScreen({
    launch: { target: null, expectedBackendId: null, localBackendAvailable: true, error: null },
    launchError: { kind: "startup", diagnostics },
  });
  const notices = descendants(tree).filter((element) => element.type === NotificationMessage);
  assert.equal(notices.length, 1);
  assert.equal(notices[0].props.children, diagnostics.cause);
  const recovery = descendants(tree).find(
    (element) =>
      element.type === "div" &&
      Children.toArray(element.props.children).some((child) => isValidElement(child) && child.type === NotificationMessage),
  );
  assert.ok(recovery);
  const contents = descendants(recovery);
  assert.equal(contents.filter((element) => element.props.value === diagnostics.log).length, 1);
  const actions = contents.filter((element) => element.type === Button);
  assert.equal(actions.length, 1);
  actions[0].props.onClick?.();
  assert.deepEqual(fixture.select.mock.calls, [[{ kind: "owned_local" }]]);
  assert.equal(descendants(tree).filter((element) => element.type === "details").length, 0);
});

test("Backend Loading Stays in the Status Indicator", () => {
  const contents = descendants(BackendsTable());
  assert.equal(contents.filter((element) => element.props.role === "status").length, 1);
  assert.equal(contents.filter((element) => element.type === NotificationMessage).length, 0);
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
  assert.ok(isValidElement<{ children: string }>(registry.props.failure));
  assert.equal(registry.props.failure.props.children, failure.message);
  assert.ok(registry.props.retry);
  registry.props.retry(null);
  assert.deepEqual(fixture.select.mock.calls, [[{ kind: "owned_local" }]]);
  assert.ok(contents.some((element) => element.type === Button && element.props.children === "Settings"));

  await owner.select({ kind: "profile", identifier: "lab" });
  assert.equal(owner.snapshot().phase, "connected");
});

test("Backend Failure Replaces Loading with Notification Feedback", () => {
  const contents = descendants(BackendsTable({ failure: "fixture-failure" }));
  const notices = contents.filter((element) => element.type === NotificationMessage);
  assert.equal(notices.length, 1);
  assert.equal(contents.filter((element) => element.props.role === "status").length, 0);
  assert.ok(Children.toArray(notices[0].props.children).includes("fixture-failure"));
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
