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
import { FailureSchema, PluginState } from "@host/model";
import { create, toBinary } from "@bufbuild/protobuf";
import { beforeEach, test, vi } from "vitest";

// Local Imports
import { type InstallPluginProgress, installNativePlugin, getNativePlugin } from "@/lib/api/native-plugins";
import { getInstalledPluginDetail } from "@/features/settings/installed-plugins/installed-plugin-data-source";
import { failureOf } from "@/lib/notification-details";
import { commands } from "@/lib/desktop/commands";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  invoke: vi.fn(),
  getPlugin: vi.fn(),
  listPlugins: vi.fn(),
  listPluginsUnused: vi.fn(),
  getSettings: vi.fn(),
  channels: [] as { onmessage: (message: unknown) => void }[],
}));

const session = { generation: "7", isCurrent: () => true };

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function managedPlugin(name: string) {
  return {
    name,
    revision: new Uint8Array([3]),
    activation: 1n,
    state: PluginState.CONNECTING,
    value: { enabled: true },
    placement: { kind: { case: "managed", value: {} } },
  };
}

// Deliver progress the way the shell does: through the channel passed with the single invoke.
function emit(...messages: InstallPluginProgress[]): void {
  const channel = fixture.channels.at(-1);
  assert.ok(channel, "the installation invoke carries a progress channel");
  for (const message of messages) channel.onmessage(message);
}

/* === Entry Point === */

vi.mock("@tauri-apps/api/core", () => ({
  Channel: class {
    id = fixture.channels.length;
    onmessage: (message: unknown) => void;
    constructor(onmessage: (message: unknown) => void) {
      this.onmessage = onmessage;
      fixture.channels.push(this);
    }
    toJSON() {
      return `__CHANNEL__:${this.id}`;
    }
  },
  invoke: fixture.invoke,
}));

vi.mock("@/lib/host/client", () => ({
  pluginsGet: fixture.getPlugin,
  pluginsList: fixture.listPlugins,
}));

vi.mock("@/lib/api/native-settings", () => ({ getNativePluginSettings: fixture.getSettings }));

beforeEach(() => {
  vi.clearAllMocks();
  fixture.channels.length = 0;
  fixture.getPlugin.mockImplementation((_session, request: { name: string }) => Promise.resolve({ entry: managedPlugin(request.name) }));
});

test("Picked Bundle Installs in One Native Invoke", async () => {
  const progress: InstallPluginProgress[] = [];
  fixture.invoke.mockImplementation(async () => {
    emit({ phase: "reading" }, { phase: "uploading", sent: 52_500_000, total: 52_500_000 }, { phase: "installing" });
    return "picked";
  });

  const entry = await installNativePlugin(session, "pick", { onProgress: (value) => progress.push(value) });

  assert.equal(entry?.name, "picked");
  assert.equal(entry?.state, PluginState.CONNECTING);
  assert.equal(fixture.invoke.mock.calls.length, 1);
  const [command, args, options] = fixture.invoke.mock.calls[0];
  assert.equal(command, "desktop_plugin_install_file");
  assert.deepEqual(Object.keys(args), ["generation", "drop", "progress"]);
  assert.equal(args.generation, "7");
  assert.equal(args.drop, false);
  assert.equal(options, undefined);
  assert.deepEqual(progress, [{ phase: "reading" }, { phase: "uploading", sent: 52_500_000, total: 52_500_000 }, { phase: "installing" }]);
  assert.deepEqual(fixture.getPlugin.mock.calls, [[session, { name: "picked" }]]);
});

test("Release Installs Natively", async () => {
  const releases = ["first", "second"].map((name) => ({
    candidate: `${name}-candidate`,
    package: name,
    version: "1.2.3",
    target: "native",
    requiresDokime: "^0.1",
  }));
  fixture.invoke.mockResolvedValue(releases);
  assert.deepEqual(await commands.desktopPluginReleaseCheck(session.generation), releases);
  fixture.invoke.mockClear();
  fixture.invoke.mockResolvedValue("release-install");
  const entry = await installNativePlugin(session, { release: releases[1].candidate });

  assert.equal(entry?.name, "release-install");
  assert.equal(fixture.invoke.mock.calls.length, 1);
  const [command, arguments_] = fixture.invoke.mock.calls[0];
  assert.equal(command, "desktop_plugin_release_install");
  assert.equal(arguments_.candidate, "second-candidate");
  assert.equal(arguments_.generation, session.generation);
  assert.equal(fixture.getPlugin.mock.calls.length, 1);
});

test("Cancelled Picker Installs Nothing", async () => {
  fixture.invoke.mockResolvedValue(null);
  assert.equal(await installNativePlugin(session, "pick"), null);
  assert.equal(fixture.getPlugin.mock.calls.length, 0);
});

test("Connection Change Discards the Installed Installation Reply", async () => {
  const changed = { generation: "7", isCurrent: () => false };
  fixture.invoke.mockResolvedValue("old-host-installation");

  await assert.rejects(installNativePlugin(changed, "pick"), /connection changed/i);
  assert.equal(fixture.getPlugin.mock.calls.length, 0);
});

test("Native Installation Failure Preserves the Error Without Looking Up an Installation", async () => {
  const failure = new Error("The plugin bundle could not be read: permission denied");
  fixture.invoke.mockRejectedValue(failure);

  await assert.rejects(installNativePlugin(session, "drop"), (error) => error === failure);
  assert.equal(fixture.invoke.mock.calls.length, 1);
  assert.equal(fixture.getPlugin.mock.calls.length, 0);
});

test("Dropped Bundle Installs from the Shell's Recorded Drop", async () => {
  fixture.invoke.mockResolvedValue("dropped");

  const entry = await installNativePlugin(session, "drop");

  assert.equal(entry?.name, "dropped");
  assert.equal(fixture.invoke.mock.calls.length, 1);
  const [command, args, options] = fixture.invoke.mock.calls[0];
  assert.equal(command, "desktop_plugin_install_file");
  assert.equal(args.drop, true);
  assert.equal(options, undefined, "no bundle bytes or path cross the bridge");
});

test("Installation Preserves the Native Structured Failure Across IPC", async () => {
  const refusal = create(FailureSchema, {
    code: "bench_activation_failed",
    message: "bench_process_exited",
    action: "Restart Plugin",
    plugin: "bench",
    cause: { message: "bench_startup_cause" },
  });
  fixture.invoke.mockRejectedValueOnce({
    kind: "grpc",
    detail: { code: 9, message: "operation request failed", details: Array.from(toBinary(FailureSchema, refusal)) },
  });
  await assert.rejects(installNativePlugin(session, "pick"), (error) => {
    const failure = failureOf(error, "bench_install");
    assert.equal(failure.reason, refusal.code);
    assert.equal(failure.message, refusal.message);
    assert.equal(failure.action, refusal.action);
    assert.equal(failure.plugin, refusal.plugin);
    assert.equal(failure.operation, "desktop_plugin_install_file");
    assert.deepEqual(failure.causes, [refusal.cause?.message]);
    return true;
  });
});

test("Failed Entry Retains Its Identity and Secret Availability", async () => {
  const failed = { ...managedPlugin("example"), failure: { message: "activation-refused" } };
  fixture.listPlugins.mockResolvedValue({ entries: [failed] });
  fixture.getSettings.mockResolvedValue({
    settings: { contributions: [] },
    secrets: [{ identifier: "api_token", available: false }],
    values: {},
    discovered: {},
  });
  fixture.getPlugin.mockResolvedValueOnce({ entry: failed });
  assert.equal(await getNativePlugin(session, "example"), failed);
  const detail = await getInstalledPluginDetail(session, "example");
  assert.equal(detail?.status, "failed");
  assert.equal(detail?.plugin_id, failed.name);
  assert.equal(detail?.error, failed.failure.message);
  assert.equal(detail?.owner?.kind, "bundle");
  assert.deepEqual(detail?.secrets, [{ identifier: "api_token", available: false }]);
  assert.deepEqual(fixture.getSettings.mock.calls, [[session, "example"]]);
});
