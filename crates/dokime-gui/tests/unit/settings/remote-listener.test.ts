// @vitest-environment happy-dom
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
import defaults from "@generated/typescript/application-settings.json";

// Third Party Imports
import { QueryClient } from "@tanstack/svelte-query";
import { render, within } from "@testing-library/svelte";
import { beforeEach, onTestFinished, test, vi } from "vitest";

// Local Imports
import { failureOf } from "@/lib/notification-details";
import type { ConnectedSession } from "@/app/native-session/session";
import RemoteSection from "@/features/settings/sections/RemoteSection.svelte";
import { commitListener, type ListenerSettings, type ListenerStatus, listenerRunning } from "@/features/settings/sections/remote-listener";
import { commands } from "@/lib/desktop/commands";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({ read: vi.fn(), invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: fixture.invoke }));
vi.mock("@/lib/api/native-remote", async (original) => ({
  ...(await original<typeof import("@/lib/api/native-remote")>()),
  readNativeListener: fixture.read,
  withNativeListener: async (_session: unknown, snapshot: unknown) => snapshot,
}));
vi.mock("@/lib/desktop/settings", () => ({
  preferencesCommands: { desktopSettingsRead: async () => ({ revision: [7], settings: defaults }) },
}));
vi.mock("@/lib/surface-environment", () => ({ isTauriDesktop: () => true }));

function showSection() {
  const queryClient = new QueryClient();
  const session = {
    identity: { generation: "fixture", isCurrent: () => true },
    queryClient,
    snapshot: { ownership: "attached", connection: { backend: { methods: ["/dokime.host.v1.access.Listener/Get"] } } },
  } as unknown as ConnectedSession;
  const view = render(RemoteSection, { session, advanced: false, label: "Fixture", queryClient });
  onTestFinished(() => {
    view.unmount();
    queryClient.clear();
  });
  return view;
}

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
  fixture.read.mockReset();
  fixture.invoke.mockReset();
});

test("Repeated Remote Settings Loads and Failures Stay Inline", async () => {
  let refuse!: (failure: Error) => void;
  fixture.read.mockReturnValue(
    new Promise((_resolve, reject) => {
      refuse = reject;
    }),
  );
  const view = showSection();
  const card = view.getByRole("heading", { name: "Remote Access" }).closest("section");
  assert.ok(card);
  const access = within(card);
  for (const advanced of [false, true, false]) {
    await view.rerender({ advanced });
    assert.ok(access.getByRole("status"));
    assert.equal(access.queryByRole("alert"), null);
  }
  const failure = new Error("fixture-failure");
  refuse(failure);
  const alert = await view.findByRole("alert");
  assert.equal(alert.parentElement?.closest("section")?.querySelector("h2")?.textContent, "Remote Access");
  assert.ok(alert.textContent?.includes(failure.message));
});

test("Export Endpoint Preview", async () => {
  const endpoints = [
    { address: "bench-host" },
    { address: "192.168.7.4", adapterName: "Wireless" },
    { address: "169.254.23.41", adapterName: "Cable Ethernet" },
  ];
  fixture.read.mockResolvedValue({ ...STOPPED, endpoints });
  const view = showSection();
  const list = await view.findByRole("list", { name: "Export Endpoints" });
  const rows = [...list.querySelectorAll("li")];
  assert.deepEqual(
    rows.map((row) => row.querySelector(".font-mono")?.textContent),
    endpoints.map((endpoint) => endpoint.address),
  );
  for (const [index, endpoint] of endpoints.entries()) {
    if (endpoint.adapterName) {
      assert.ok(rows[index]?.textContent?.includes(endpoint.adapterName));
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
