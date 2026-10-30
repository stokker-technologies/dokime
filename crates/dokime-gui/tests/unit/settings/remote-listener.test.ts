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
import { Children, createElement, isValidElement, type ReactElement, type ReactNode } from "react";
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
  pairingEndpoint,
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

const STOPPED: ListenerStatus = { settings: SETTINGS, keyConfigured: true, certificatePem: "-----BEGIN CERTIFICATE-----" };

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

test.each([
  "invalid pairing endpoint",
  "pairing endpoint is too large",
  "pairing endpoint must be an HTTP(S) origin with an explicit port",
  "pairing endpoint requires a port",
  "invalid pairing endpoint port",
  "IPv6 pairing endpoints require brackets",
  "bracketed pairing hosts must be IPv6 addresses",
  "invalid pairing endpoint host",
  "pairing endpoint must be a concrete host",
  "HTTPS requires a CA certificate; HTTP must not include one",
  "pairing endpoint differs from the active listener",
  "pairing endpoint is not covered by the active TLS certificate",
  "pairing endpoint resolution timed out",
  "pairing endpoint cannot be resolved",
  "pairing endpoint does not resolve to the active host listener",
])("Pairing Reason Reaches The Field: %s", async (reason) => {
  fixture.invoke.mockRejectedValueOnce({ kind: "grpc", detail: { code: 3, message: reason, details: [] } });
  let failure: unknown;
  try {
    await commands.desktopPairingCreate("Python", "https://127.0.0.01");
    assert.fail("Invalid endpoint should be rejected");
  } catch (error) {
    failure = error;
  }
  assert.equal(listenerError(failure), reason);
  fixture.session = {
    identity: { generation: "fixture" },
    snapshot: { connection: { backend: { methods: ["/dokime.v1.Remote/ReadListener"] } } },
  };
  fixture.query.isPending = false;
  fixture.query.data = STOPPED;
  fixture.states = [false, false, "Python", "https://127.0.0.01", 0, listenerError(failure)];
  const html = renderToStaticMarkup(createElement(RemoteSection));
  assert.match(html, /aria-invalid="true"/);
  assert.ok(html.includes(`role="alert" class="text-sm text-destructive">${reason}</p>`));
});

test("Missing Pairing Reasons Have Visible Fallback Text", async () => {
  fixture.invoke.mockRejectedValueOnce({ kind: "grpc", detail: { code: 3, message: "", details: [] } });
  await assert.rejects(commands.desktopPairingCreate("Python", "https://host"), (error: unknown) => Boolean(listenerError(error).trim()));
  assert.ok(listenerError(new Error(" ")).trim());
});

test.each([
  ["127.0.0.1:9012", true, [], "https://127.0.0.1:9012"],
  ["192.168.1.20:9012", false, [], "http://192.168.1.20:9012"],
  ["[::1]:9012", true, [], "https://[::1]:9012"],
  ["0.0.0.0:9012", true, ["lab-host", "192.168.1.20"], "https://lab-host:9012"],
  ["0.0.0.0:9012", false, ["192.168.1.20"], "http://192.168.1.20:9012"],
  ["[::]:9012", true, ["2001:db8::5"], "https://[2001:db8::5]:9012"],
  ["[::]:9012", true, ["::", "lab-host"], "https://lab-host:9012"],
  ["[0:0:0:0:0:0:0:0]:9012", true, ["0:0:0:0:0:0:0:0", "lab-host"], "https://lab-host:9012"],
  ["0.0.0.0:9012", false, [], "http://127.0.0.1:9012"],
  ["[::]:9012", false, [], "http://[::1]:9012"],
  ["0.0.0.0:9012", true, [], ""],
] as const)("Pairing Prefill Uses Saved Listener %s With TLS %s", (address, tlsEnabled, certificateNames, endpoint) => {
  const settings = { ...SETTINGS, address, tlsEnabled, certificateNames: [...certificateNames] };
  assert.deepEqual(pairingEndpoint(settings), { endpoint, port: "9012" });
  fixture.session = {
    identity: { generation: "fixture" },
    snapshot: { connection: { backend: { methods: ["/dokime.v1.Remote/ReadListener"] } } },
  };
  fixture.query.isPending = false;
  fixture.query.data = { ...STOPPED, settings, listeningAddress: "0.0.0.0:8765" };
  const html = renderToStaticMarkup(createElement(RemoteSection));
  assert.ok(html.includes(`value="${endpoint}"`));
  const hint = html.match(/<p id="fixture-field-endpoint-hint"[^>]*>(.*?)<\/p>/)?.[1];
  assert.ok(hint?.includes("9012"));
  assert.doesNotMatch(html, /aria-invalid="true"/);
});

test("Pairing Submission Keeps The Native Failure Beside The Endpoint", async () => {
  const failure = new Error("fixture-rejection");
  fixture.invoke.mockRejectedValueOnce({ kind: "grpc", detail: { code: 3, message: failure.message, details: [] } });
  fixture.session = {
    identity: { generation: "fixture" },
    snapshot: { connection: { backend: { methods: ["/dokime.v1.Remote/ReadListener"] } } },
  };
  fixture.query.isPending = false;
  fixture.query.data = STOPPED;
  const root = RemoteSection({});
  const connected = Children.toArray(root.props.children).at(-1);
  assert.ok(isValidElement(connected));
  const listener = (connected.type as (props: unknown) => ReactElement)(connected.props);
  const controls = (listener.type as (props: unknown) => ReactElement)(listener.props);
  function findForm(node: ReactNode): ReactElement<{ onSubmit: (event: { preventDefault: () => void }) => void }> | undefined {
    if (!isValidElement<{ children?: ReactNode }>(node)) return undefined;
    if (node.type === "form") return node as ReturnType<typeof findForm>;
    return Children.toArray(node.props.children).map(findForm).find(Boolean);
  }
  const form = findForm(controls);
  assert.ok(form);
  assert.equal((form.props as { noValidate?: boolean }).noValidate, true);
  form.props.onSubmit({ preventDefault() {} });
  await vi.waitFor(() => assert.ok(fixture.updates.includes(failure.message)));
});
