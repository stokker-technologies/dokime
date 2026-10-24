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
import { PluginDropZone } from "@/features/settings/plugins/PluginDropZone";
import { consentNativePlugin, type StagePluginProgress, stageNativePlugin } from "@/lib/api/native-plugins";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  invoke: vi.fn(),
  getInstallation: vi.fn(),
  consent: vi.fn(),
  channels: [] as { onmessage: (message: unknown) => void }[],
}));

const session = { generation: "7", isCurrent: () => true };

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function installation(installId: string) {
  return {
    installId,
    sha256: "a".repeat(64),
    stagedAt: { seconds: 1_760_000_000n, nanos: 0 },
    state: { HarmosVariant: { case: "Staged", value: {} } },
    active: false,
    restartRequired: false,
  };
}

// Deliver progress the way the shell does: through the channel passed with the single invoke.
function emit(...messages: StagePluginProgress[]): void {
  const channel = fixture.channels.at(-1);
  assert.ok(channel, "the staging invoke carries a progress channel");
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

vi.mock("@/lib/rpc/generated/client", () => ({
  pluginsGetInstallation: fixture.getInstallation,
  pluginsConsent: fixture.consent,
}));

beforeEach(() => {
  vi.clearAllMocks();
  fixture.channels.length = 0;
  fixture.getInstallation.mockImplementation((_session, request: { installId: string }) =>
    Promise.resolve(installation(request.installId)),
  );
  fixture.consent.mockImplementation((_session, request: { installId: string }) => Promise.resolve(installation(request.installId)));
});

test("Consent Binds The Digest The Operator Reviewed", async () => {
  const reviewed = { install_id: "reviewed", sha256: "c".repeat(64) };
  fixture.getInstallation.mockResolvedValue({ ...installation("reviewed"), sha256: "b".repeat(64) });
  fixture.consent.mockResolvedValue({ ...installation("reviewed"), sha256: reviewed.sha256 });

  const entry = await consentNativePlugin(session, reviewed);

  assert.equal(entry.sha256, reviewed.sha256);
  assert.equal(fixture.getInstallation.mock.calls.length, 0);
  assert.deepEqual(fixture.consent.mock.calls, [[session, { installId: reviewed.install_id, expectedSha256: reviewed.sha256 }]]);
});

test("Picked Bundle Stages In One Native Invoke", async () => {
  const progress: StagePluginProgress[] = [];
  fixture.invoke.mockImplementation(async () => {
    emit({ phase: "reading" }, { phase: "uploading", sent: 52_500_000, total: 52_500_000 }, { phase: "inspecting" });
    return "picked";
  });

  const entry = await stageNativePlugin(session, "pick", { onProgress: (value) => progress.push(value) });

  assert.equal(entry?.install_id, "picked");
  assert.equal(entry?.state, "staged");
  assert.equal(fixture.invoke.mock.calls.length, 1);
  const [command, args, options] = fixture.invoke.mock.calls[0];
  assert.equal(command, "desktop_plugin_stage_file");
  assert.deepEqual(Object.keys(args), ["generation", "drop", "progress"]);
  assert.equal(args.generation, "7");
  assert.equal(args.drop, false);
  assert.equal(options, undefined);
  assert.deepEqual(progress, [{ phase: "reading" }, { phase: "uploading", sent: 52_500_000, total: 52_500_000 }, { phase: "inspecting" }]);
  assert.deepEqual(fixture.getInstallation.mock.calls, [[session, { installId: "picked" }]]);
});

test("Reviewed Release Reuses Native Staging And Leaves Consent Explicit", async () => {
  fixture.invoke.mockResolvedValue("release-install");
  const entry = await stageNativePlugin(session, { release: "reviewed-candidate" });

  assert.equal(entry?.install_id, "release-install");
  assert.equal(fixture.invoke.mock.calls.length, 1);
  const [command, arguments_] = fixture.invoke.mock.calls[0];
  assert.equal(command, "desktop_plugin_release_stage");
  assert.equal(arguments_.candidate, "reviewed-candidate");
  assert.equal(arguments_.generation, session.generation);
  assert.equal(fixture.getInstallation.mock.calls.length, 1);
});

test("Cancelled Picker Stages Nothing", async () => {
  fixture.invoke.mockResolvedValue(null);
  assert.equal(await stageNativePlugin(session, "pick"), null);
  assert.equal(fixture.getInstallation.mock.calls.length, 0);
});

test("Connection Change Discards The Staged Installation Reply", async () => {
  const changed = { generation: "7", isCurrent: () => false };
  fixture.invoke.mockResolvedValue("old-host-installation");

  await assert.rejects(stageNativePlugin(changed, "pick"), /connection changed/i);
  assert.equal(fixture.getInstallation.mock.calls.length, 0);
});

test("Native Staging Failure Preserves The Error Without Looking Up An Installation", async () => {
  const failure = new Error("The plugin bundle could not be read: permission denied");
  fixture.invoke.mockRejectedValue(failure);

  await assert.rejects(stageNativePlugin(session, "drop"), (error) => error === failure);
  assert.equal(fixture.invoke.mock.calls.length, 1);
  assert.equal(fixture.getInstallation.mock.calls.length, 0);
});

test("Dropped Bundle Stages From The Shell's Recorded Drop", async () => {
  fixture.invoke.mockResolvedValue("dropped");

  const entry = await stageNativePlugin(session, "drop");

  assert.equal(entry?.install_id, "dropped");
  assert.equal(fixture.invoke.mock.calls.length, 1);
  const [command, args, options] = fixture.invoke.mock.calls[0];
  assert.equal(command, "desktop_plugin_stage_file");
  assert.equal(args.drop, true);
  assert.equal(options, undefined, "no bundle bytes or path cross the bridge");
});

test("Drop Zone Reports Each Staging Phase", () => {
  const render = (progress: StagePluginProgress | null) =>
    renderToStaticMarkup(
      createElement(PluginDropZone, { disabled: true, error: null, filename: "obc-plugin.dokime", onDrop() {}, onPick() {}, progress }),
    );
  assert.match(render(null), /Inspecting obc-plugin\.dokime…/);
  assert.match(render({ phase: "reading" }), /Reading file…/);
  assert.match(render({ phase: "uploading", sent: 26_250_000, total: 52_500_000 }), /Uploading 26\.3 MB of 52\.5 MB/);
  assert.match(render({ phase: "inspecting" }), /Inspecting on the host…/);
});
