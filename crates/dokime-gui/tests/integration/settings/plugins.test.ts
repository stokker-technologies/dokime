/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import assert from "node:assert/strict";

import { beforeEach, test, vi } from "vitest";

import { type InstallPluginProgress, installNativePlugin } from "@/lib/api/native-plugins";

const fixture = vi.hoisted(() => ({
  invoke: vi.fn(),
  getInstallation: vi.fn(),
  channels: [] as { onmessage: (message: unknown) => void }[],
}));

const session = { generation: "7", isCurrent: () => true };

function installation(installId: string) {
  return {
    installId,
    sha256: "a".repeat(64),
    installedAt: { seconds: 1_760_000_000n, nanos: 0 },
    state: { HarmosVariant: { case: "Installed", value: {} } },
    active: false,
    restartRequired: true,
  };
}

// Deliver progress the way the shell does: through the channel passed with the single invoke.
function emit(...messages: InstallPluginProgress[]): void {
  const channel = fixture.channels.at(-1);
  assert.ok(channel, "the installation invoke carries a progress channel");
  for (const message of messages) channel.onmessage(message);
}

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

vi.mock("@/lib/rpc/generated/client", () => ({
  pluginsGetInstallation: fixture.getInstallation,
}));

beforeEach(() => {
  vi.clearAllMocks();
  fixture.channels.length = 0;
  fixture.getInstallation.mockImplementation((_session, request: { installId: string }) =>
    Promise.resolve(installation(request.installId)),
  );
});

test("Picked Bundle Installs In One Native Invoke", async () => {
  const progress: InstallPluginProgress[] = [];
  fixture.invoke.mockImplementation(async () => {
    emit({ phase: "reading" }, { phase: "uploading", sent: 52_500_000, total: 52_500_000 }, { phase: "installing" });
    return "picked";
  });

  const entry = await installNativePlugin(session, "pick", { onProgress: (value) => progress.push(value) });

  assert.equal(entry?.install_id, "picked");
  assert.equal(entry?.state, "installed");
  assert.equal(fixture.invoke.mock.calls.length, 1);
  const [command, args, options] = fixture.invoke.mock.calls[0];
  assert.equal(command, "desktop_plugin_install_file");
  assert.deepEqual(Object.keys(args), ["generation", "drop", "progress"]);
  assert.equal(args.generation, "7");
  assert.equal(args.drop, false);
  assert.equal(options, undefined);
  assert.deepEqual(progress, [{ phase: "reading" }, { phase: "uploading", sent: 52_500_000, total: 52_500_000 }, { phase: "installing" }]);
  assert.deepEqual(fixture.getInstallation.mock.calls, [[session, { installId: "picked" }]]);
});

test("Release Installs Natively", async () => {
  fixture.invoke.mockResolvedValue("release-install");
  const entry = await installNativePlugin(session, { release: "reviewed-candidate" });

  assert.equal(entry?.install_id, "release-install");
  assert.equal(fixture.invoke.mock.calls.length, 1);
  const [command, arguments_] = fixture.invoke.mock.calls[0];
  assert.equal(command, "desktop_plugin_release_install");
  assert.equal(arguments_.candidate, "reviewed-candidate");
  assert.equal(arguments_.generation, session.generation);
  assert.equal(fixture.getInstallation.mock.calls.length, 1);
});

test("Cancelled Picker Installs Nothing", async () => {
  fixture.invoke.mockResolvedValue(null);
  assert.equal(await installNativePlugin(session, "pick"), null);
  assert.equal(fixture.getInstallation.mock.calls.length, 0);
});

test("Connection Change Discards The Installed Installation Reply", async () => {
  const changed = { generation: "7", isCurrent: () => false };
  fixture.invoke.mockResolvedValue("old-host-installation");

  await assert.rejects(installNativePlugin(changed, "pick"), /connection changed/i);
  assert.equal(fixture.getInstallation.mock.calls.length, 0);
});

test("Native Installation Failure Preserves The Error Without Looking Up An Installation", async () => {
  const failure = new Error("The plugin bundle could not be read: permission denied");
  fixture.invoke.mockRejectedValue(failure);

  await assert.rejects(installNativePlugin(session, "drop"), (error) => error === failure);
  assert.equal(fixture.invoke.mock.calls.length, 1);
  assert.equal(fixture.getInstallation.mock.calls.length, 0);
});

test("Dropped Bundle Installs From The Shell's Recorded Drop", async () => {
  fixture.invoke.mockResolvedValue("dropped");

  const entry = await installNativePlugin(session, "drop");

  assert.equal(entry?.install_id, "dropped");
  assert.equal(fixture.invoke.mock.calls.length, 1);
  const [command, args, options] = fixture.invoke.mock.calls[0];
  assert.equal(command, "desktop_plugin_install_file");
  assert.equal(args.drop, true);
  assert.equal(options, undefined, "no bundle bytes or path cross the bridge");
});
