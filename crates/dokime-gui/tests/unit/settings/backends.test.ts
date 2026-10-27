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
import { Button } from "@/components/ui/button";
import { backendValidation } from "@/features/settings/BackendsTable";
import { beforeEach, test, vi } from "vitest";

// Local Imports
import { AppRoot } from "@/app/AppRoot";
import { NativeSession } from "@/app/native-session/session";
import type { DesktopSessionSnapshot } from "@/lib/desktop/session";
import { preferencesCommands } from "@/lib/desktop/settings";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  invoke: vi.fn(),
  identity: vi.fn(),
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
vi.mock("@/lib/rpc/generated/client", () => ({ backendGetIdentity: fixture.identity }));
vi.mock("@/app/query-client", () => ({ createDokimeQueryClient: () => ({ cancelQueries: fixture.cancel, clear: fixture.clear }) }));

beforeEach(() => {
  vi.clearAllMocks();
  fixture.effects.length = 0;
  fixture.states.length = 0;
  fixture.phase = "disconnected";
  fixture.identity.mockResolvedValue({ backendId: "fixture", epoch: "epoch" });
  fixture.invoke.mockImplementation(async (command: string) => {
    if (command === "desktop_backend_use") return connected().connection;
    if (command === "desktop_session_status") return connected();
    if (command === "desktop_session_disconnect")
      return { ...connected(), connection: { ...connected().connection, phase: "disconnected" } };
    throw new Error(`Unexpected Command: ${command}`);
  });
});

test("Profile Selection Uses The Shared Native Registry Before Negotiating Identity", async () => {
  const owner = new NativeSession();
  await owner.select({ kind: "profile", name: "lab" });
  assert.deepEqual(
    fixture.invoke.mock.calls.map(([command, args]) => [command, args]),
    [
      ["desktop_backend_use", { name: "lab" }],
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

test("Failed Selection Stays Failed Until An Explicit New Selection", async () => {
  const failure = { kind: "identity_changed" };
  fixture.invoke.mockRejectedValueOnce(failure);
  const owner = new NativeSession();
  await owner.select({ kind: "profile", name: "stale" });
  assert.deepEqual(owner.snapshot(), { phase: "failed", error: failure });
  assert.equal(fixture.invoke.mock.calls.length, 1);
  assert.equal(fixture.identity.mock.calls.length, 0);
  await owner.select({ kind: "profile", name: "lab" });
  assert.equal(owner.snapshot().phase, "connected");
});

test("Deleting The Active Backend Invalidates Its Generation And Owned Resources", async () => {
  const owner = new NativeSession();
  await owner.select({ kind: "profile", name: "lab" });
  const state = owner.snapshot();
  if (state.phase !== "connected") throw new Error("Expected Connected Session");
  const dispose = vi.fn();
  state.session.own(dispose);
  fixture.invoke.mockResolvedValueOnce({ version: 1, default: null, profiles: [] });
  assert.deepEqual(await preferencesCommands.desktopBackendRemove("lab"), { version: 1, default: null, profiles: [] });
  await owner.disconnect();
  assert.equal(state.session.identity.isCurrent(), false);
  assert.equal(dispose.mock.calls.length, 1);
  assert.equal(fixture.cancel.mock.calls.length, 1);
  assert.equal(fixture.clear.mock.calls.length, 1);
  assert.equal(owner.snapshot().phase, "disconnected");
});

test("Registry Commands Preserve Profile Fields Without Bearer Material", async () => {
  const profile = {
    name: "lab",
    endpoint: "https://lab.local",
    expected_backend_id: "fixture",
    ca_certificate_pem: "fixture-ca",
    secret: "keyring" as const,
  };
  const document = { version: 1, default: null, profiles: [profile] };
  fixture.invoke.mockResolvedValue(document);
  assert.deepEqual(await preferencesCommands.desktopBackendAdd(profile), document);
  assert.deepEqual(fixture.invoke.mock.calls.at(-1)?.slice(0, 2), ["desktop_backend_add", { profile }]);
  assert.deepEqual(await preferencesCommands.desktopBackendsList(), document);
  assert.deepEqual(fixture.invoke.mock.calls.at(-1)?.slice(0, 2), ["desktop_backends_list", undefined]);
});

test("Startup Attempts The Saved Default Once And Exposes Failure Without Fallback", async () => {
  fixture.invoke.mockImplementation(async (command: string) => {
    if (command === "desktop_session_launch") return { target: null, expectedBackendId: null, localBackendAvailable: true, error: null };
    if (command === "desktop_backends_list") return { version: 1, default: "stale", profiles: [] };
    if (command === "desktop_backend_use") return Promise.reject({ kind: "identity_changed" });
    throw new Error(`Unexpected Command: ${command}`);
  });
  AppRoot();
  assert.equal(fixture.effects.length, 1);
  const cleanup = fixture.effects[0]();
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.deepEqual(
    fixture.invoke.mock.calls.map(([command, args]) => [command, args]),
    [
      ["desktop_session_launch", undefined],
      ["desktop_backends_list", undefined],
      ["desktop_backend_use", { name: "stale" }],
    ],
  );
  assert.equal(fixture.identity.mock.calls.length, 0);
  cleanup();
});

test("Startup Without A Saved Default Leaves Selection To The Operator", async () => {
  fixture.invoke.mockImplementation(async (command: string) => {
    if (command === "desktop_session_launch") return { target: null, expectedBackendId: null, localBackendAvailable: true, error: null };
    if (command === "desktop_backends_list") return { version: 1, default: null, profiles: [] };
    throw new Error(`Unexpected Command: ${command}`);
  });
  AppRoot();
  const cleanup = fixture.effects[0]();
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.deepEqual(
    fixture.invoke.mock.calls.map(([command]) => command),
    ["desktop_session_launch", "desktop_backends_list"],
  );
  cleanup();
});

test("Backend Validation Rejects Invalid Origins And Duplicate Names", () => {
  assert.deepEqual(backendValidation(" lab ", " https://lab.local:8765 ", "", []), {});
  assert.deepEqual(Object.keys(backendValidation(" ", "not an origin", "", [])), ["name", "endpoint"]);
  assert.deepEqual(Object.keys(backendValidation("lab", "http://lab.local", "", ["lab"])), ["name"]);
  for (const endpoint of [
    "ftp://lab.local",
    "https://user:secret@lab.local",
    "https://lab.local/path",
    "https://lab.local/?key=secret",
    "https://lab.local/#key",
  ]) {
    assert.deepEqual(Object.keys(backendValidation("lab", endpoint, "", [])), ["endpoint"]);
  }
  assert.deepEqual(Object.keys(backendValidation("lab", "http://lab.local", "certificate", [])), ["endpoint"]);
  assert.deepEqual(backendValidation("lab", "https://lab.local", "certificate", []), {});
});

type TestElementProps = {
  children?: ReactNode;
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

test("Startup Failure Has One Alert With A Log Path And An Explicit Retry", async () => {
  const { ConnectionScreen } = await vi.importActual<typeof import("@/app/native-session/ConnectionScreen")>(
    "@/app/native-session/ConnectionScreen",
  );
  const diagnostics = { cause: "fixture startup failure", log: "C:/fixture/application.log" };
  const tree = ConnectionScreen({
    launch: { target: null, expectedBackendId: null, localBackendAvailable: true, error: null },
    launchError: { kind: "startup", diagnostics },
  });
  const alerts = descendants(tree).filter((element) => element.props.role === "alert");
  assert.equal(alerts.length, 1);
  const contents = descendants(alerts[0]);
  assert.equal(contents.filter((element) => element.props.value === diagnostics.log).length, 1);
  const actions = contents.filter((element) => element.type === Button);
  assert.equal(actions.length, 1);
  actions[0].props.onClick?.();
  assert.deepEqual(fixture.select.mock.calls, [[{ kind: "owned_local" }]]);
  assert.equal(descendants(tree).filter((element) => element.type === "details").length, 0);
});
