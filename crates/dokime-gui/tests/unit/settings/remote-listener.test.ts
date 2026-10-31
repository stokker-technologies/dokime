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
import { SettingsRow } from "@/features/settings/SettingsRow";
import { RemoteSection } from "@/features/settings/sections/RemoteSection";
import {
  certificateIssue,
  commitListener,
  type ListenerSettings,
  type ListenerStatus,
  listenerError,
  listenerRunning,
} from "@/features/settings/sections/remote-listener";
import { commands } from "@/lib/desktop/commands";

/* === Re-Exports === */

/* === Constants === */

const notice = vi.hoisted(() => vi.fn());
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
vi.mock("@/lib/desktop-open", () => ({ isTauriDesktop: () => true }));
vi.mock("@tanstack/react-query", () => ({
  useQuery: () => fixture.query,
  useQueryClient: () => ({ invalidateQueries: vi.fn() }),
}));
vi.mock("@/lib/rpc/generated/client", () => ({}));
vi.mock("@/lib/notifications", () => ({ toast: { error: vi.fn(), success: vi.fn() } }));
vi.mock("@/components/NotificationMessage", () => ({
  NotificationMessage: (props: unknown) => {
    notice(props);
    return null;
  },
}));

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

test("Repeated Remote Settings Loads Stay Inline And Failures Still Notify", () => {
  fixture.session = {
    identity: { generation: "fixture" },
    snapshot: { connection: { backend: { methods: ["/dokime.v1.Remote/ReadListener"] } } },
  };
  for (let page = 0; page < 3; page++) {
    const html = renderToStaticMarkup(createElement(RemoteSection, {}));
    assert.match(html, /role="status"/);
    assert.equal(notice.mock.calls.length, 0);
  }
  fixture.query.isPending = false;
  fixture.query.error = new Error("fixture-failure");
  renderToStaticMarkup(createElement(RemoteSection, {}));
  assert.equal(notice.mock.calls.length, 1);
});

test("Unavailable Remote Settings Stay Inline Without Notifications", () => {
  renderToStaticMarkup(createElement(RemoteSection, {}));
  assert.equal(notice.mock.calls.length, 0);
});

test("Export Endpoint Preview", () => {
  fixture.session = {
    identity: { generation: "fixture" },
    snapshot: { connection: { backend: { methods: ["/dokime.v1.Remote/ReadListener"] } } },
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
    if (endpoint.adapterName) assert.ok(list.includes(endpoint.adapterName));
  }
  assert.equal(fixture.invoke.mock.calls.length, 0);
});

test("enabling applies the host's settings at once and shows the running listener", async () => {
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

test("a refused enable keeps the stopped state and surfaces the host error", async () => {
  const failures: string[] = [];
  const next = await commitListener({
    status: STOPPED,
    change: { enabled: true },
    send: async () => {
      throw new Error("authentication_required");
    },
    fail: (message) => failures.push(message),
  });
  assert.equal(next, STOPPED);
  assert.equal(listenerRunning(next), false);
  assert.deepEqual(failures, ["authentication_required"]);
});

test("saved intent without a bound address is not running", () => {
  assert.equal(listenerRunning({ ...STOPPED, settings: { ...SETTINGS, enabled: true }, error: "address in use" }), false);
});

test("TLS Certificate Guard", () => {
  const missing: ListenerStatus = { ...STOPPED, certificatePem: undefined };
  const issue = certificateIssue(missing);
  assert.ok(issue);
  assert.equal(certificateIssue(STOPPED), null);
  assert.equal(certificateIssue({ ...missing, settings: { ...SETTINGS, tlsEnabled: false } }), null);
  const html = renderToStaticMarkup(createElement(SettingsRow, { label: "Public Certificate", error: issue }, "Generate Certificate"));
  assert.equal(notice.mock.calls.length, 1);
  assert.doesNotMatch(html, /role="alert"/);
});

test("Export Has No WebView Credential Or Endpoint Arguments", async () => {
  fixture.invoke.mockResolvedValueOnce(null);
  await commands.desktopPairingCreate();
  assert.deepEqual(fixture.invoke.mock.calls[0], ["desktop_pairing_create", undefined]);
});

test("Export Failure Retains The Native Reason", async () => {
  const reason = "fixture-export-rejection";
  fixture.invoke.mockRejectedValueOnce({ kind: "grpc", detail: { code: 3, message: reason, details: [] } });
  await assert.rejects(commands.desktopPairingCreate(), (error: unknown) => listenerError(error) === reason);
});
