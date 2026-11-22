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
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, test, vi } from "vitest";

// Local Imports
import { failureOf } from "@/lib/notification-details";
import { RemoteSection } from "@/features/settings/sections/RemoteSection";
import { commitListener, type ListenerSettings, type ListenerStatus, listenerRunning } from "@/features/settings/sections/remote-listener";
import { commands } from "@/lib/desktop/commands";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  session: null as { identity: { generation: string }; snapshot: { connection: { backend: { methods: string[] } } } } | null,
  query: { isPending: true, error: null as Error | null, data: undefined as ListenerStatus | undefined, refetch: vi.fn() },
  states: [] as unknown[],
  updates: [] as unknown[],
  invoke: vi.fn(),
}));
vi.mock("react", async (original) => ({
  ...(await original<typeof import("react")>()),
  useState: (initial: unknown) => [
    fixture.states.length ? fixture.states.shift() : initial,
    (value: unknown) => fixture.updates.push(value),
  ],
  useId: () => "fixture-field",
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => fixture.invoke(...args) }));
vi.mock("@/app/native-session/context", () => ({ useNativeSessionOptional: () => fixture.session }));
vi.mock("@/features/settings/ApplicationPreferences", () => ({ ApplicationPreferences: () => null }));
vi.mock("@/lib/surface-environment", () => ({ isTauriDesktop: () => true }));
vi.mock("@tanstack/react-query", () => ({
  useQuery: () => fixture.query,
  useQueryClient: () => ({ invalidateQueries: vi.fn() }),
}));
vi.mock("@/lib/host/client", () => ({}));
vi.mock("@/lib/notifications", () => ({ toast: { error: vi.fn(), success: vi.fn() } }));

const SETTINGS: ListenerSettings = {
  enabled: false,
  address: "0.0.0.0:8765",
  tlsEnabled: true,
  authenticationEnabled: true,
  certificateNames: ["lab-host", "192.168.1.20"],
};

const STOPPED: ListenerStatus = { settings: SETTINGS, keyConfigured: true, certificatePem: "-----BEGIN CERTIFICATE-----", endpoints: [] };

/* === Types === */

/* === Classes === */

/* === Tests === */

beforeEach(() => {
  vi.clearAllMocks();
  fixture.session = null;
  fixture.query.isPending = true;
  fixture.query.error = null;
  fixture.query.data = undefined;
  fixture.states = [];
  fixture.updates = [];
});

test("Repeated Remote Settings Loads and Failures Stay Inline", () => {
  fixture.session = {
    identity: { generation: "fixture" },
    snapshot: { connection: { backend: { methods: ["/dokime.host.v1.access.Listener/Get"] } } },
  };
  for (let page = 0; page < 3; page++) {
    const html = renderToStaticMarkup(createElement(RemoteSection, {}));
    assert.match(html, /role="status"/);
  }
  fixture.query.isPending = false;
  fixture.query.error = new Error("fixture-failure");
  const html = renderToStaticMarkup(createElement(RemoteSection, {}));
  assert.match(html, /role="alert"/);
  assert.ok(html.includes(fixture.query.error.message));
});

test("Export Endpoint Preview", () => {
  fixture.session = {
    identity: { generation: "fixture" },
    snapshot: { connection: { backend: { methods: ["/dokime.host.v1.access.Listener/Get"] } } },
  };
  const endpoints = [
    { address: "bench-host" },
    { address: "192.168.7.4", adapterName: "Wireless" },
    { address: "169.254.23.41", adapterName: "Cable Ethernet" },
  ];
  fixture.query.isPending = false;
  fixture.query.data = { ...STOPPED, endpoints };
  const html = renderToStaticMarkup(createElement(RemoteSection, {}));
  const list = html.match(/<ul aria-label="Export Endpoints"[^>]*>(.*?)<\/ul>/)?.[1];
  assert.ok(list);
  let previous = -1;
  for (const endpoint of endpoints) {
    const position = list.indexOf(endpoint.address);
    assert.ok(position > previous);
    previous = position;
    if (endpoint.adapterName) {
      assert.ok(list.includes(endpoint.adapterName));
    }
  }
  assert.equal(fixture.invoke.mock.calls.length, 0);
});

test("Enabling Applies the Host's Settings at Once and Shows the Running Listener", async () => {
  const sent: ListenerSettings[] = [];
  const failures: string[] = [];
  assert.equal(listenerRunning(STOPPED), false);
  const next = await commitListener({
    status: STOPPED,
    change: { enabled: true },
    send: async (settings) => {
      sent.push(settings);
      return { ...STOPPED, settings, listeningAddress: "0.0.0.0:8765" };
    },
    fail: (message) => failures.push(message),
  });
  assert.deepEqual(sent, [{ ...SETTINGS, enabled: true }]);
  assert.equal(listenerRunning(next), true);
  assert.deepEqual(failures, []);
});

test("A Refused Enable Keeps the Stopped State and Surfaces the Host Error", async () => {
  const failures: string[] = [];
  const reason = new Error("authentication_required");
  const details: unknown[] = [];
  const next = await commitListener({
    status: STOPPED,
    change: { enabled: true },
    send: async () => {
      throw reason;
    },
    fail: (message, error) => {
      failures.push(message);
      details.push(error);
    },
  });
  assert.equal(next, STOPPED);
  assert.equal(listenerRunning(next), false);
  assert.deepEqual(failures, ["authentication_required"]);
  assert.deepEqual(details, [reason]);
});

test("Saved Intent Without a Bound Address Is Not Running", () => {
  assert.equal(listenerRunning({ ...STOPPED, settings: { ...SETTINGS, enabled: true }, error: "address in use" }), false);
});

test("Export Failure Retains the Native Reason", async () => {
  const reason = "fixture-export-rejection";
  fixture.invoke.mockRejectedValueOnce({ kind: "grpc", detail: { code: 3, message: reason, details: [] } });
  await assert.rejects(commands.desktopPairingCreate(), (error: unknown) => failureOf(error, "fixture").message === reason);
});
