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

// Third Party Imports
import { FailureSchema, PluginState, SettingDeclarationSchema } from "@generated/host/model";
import "fake-indexeddb/auto";
import { create, toBinary } from "@bufbuild/protobuf";
import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import { beforeEach, onTestFinished, test, vi } from "vitest";

// Local Imports
import { type InstallPluginProgress, installNativePlugin, getNativePlugin } from "@/lib/api/native-plugins";
import { getInstalledPluginDetail } from "@/features/settings/installed-plugins/installed-plugin-data-source";
import { failureOf } from "@/lib/notification-details";
import { InstalledPluginSettingsForm } from "@/features/settings/installed-plugins/InstalledPluginSettingsForm";
import { mapNativeRuntimePluginSettings, updateNativePluginSettings } from "@/lib/api/native-settings";
import type { SettingValues } from "@/features/settings/installed-plugins/installed-plugin-contracts";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  invoke: vi.fn(),
  getPlugin: vi.fn(),
  listPlugins: vi.fn(),
  getSettings: vi.fn(),
  settingsList: vi.fn(),
  settingsSet: vi.fn(),
  secretsList: vi.fn(),
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
  for (const message of messages) {
    channel.onmessage(message);
  }
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
  pluginsSettingsList: fixture.settingsList,
  pluginsSettingsSet: fixture.settingsSet,
  pluginsSecretsList: fixture.secretsList,
}));

vi.mock("@/lib/api/native-settings", async (original) => ({
  ...(await original<typeof import("@/lib/api/native-settings")>()),
  getNativePluginSettings: fixture.getSettings,
}));

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
  fixture.invoke.mockResolvedValue("release-install");
  const entry = await installNativePlugin(session, { release: "second-candidate" });

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

test("Every Setting Kind Saves Through the Form and Reads Back Without Losing Other Edits", async () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  vi.useFakeTimers();
  const declarations = [
    create(SettingDeclarationSchema, { name: "enabled", title: "Enabled", default: { value: { case: "flag", value: false } } }),
    create(SettingDeclarationSchema, {
      name: "count",
      title: "Count",
      default: { value: { case: "integer", value: 2n } },
      minimum: { value: { case: "integer", value: 1n } },
      maximum: { value: { case: "integer", value: 9223372036854775807n } },
    }),
    create(SettingDeclarationSchema, {
      name: "gain",
      title: "Gain",
      default: { value: { case: "number", value: 1.5 } },
      minimum: { value: { case: "number", value: 0.5 } },
      maximum: { value: { case: "number", value: 10 } },
    }),
    create(SettingDeclarationSchema, { name: "address", title: "Address", default: { value: { case: "text", value: "initial" } } }),
    create(SettingDeclarationSchema, {
      name: "timeout",
      title: "Timeout",
      default: { value: { case: "duration", value: { seconds: 5n } } },
      minimum: { value: { case: "duration", value: { seconds: 1n } } },
      maximum: { value: { case: "duration", value: { seconds: 1800n } } },
    }),
  ];
  let revision = 1;
  let stored = Object.fromEntries(
    declarations.map((declaration) => {
      assert.ok(declaration.default);
      return [declaration.name, declaration.default] as const;
    }),
  );
  let releaseFirstWrite: () => void = () => undefined;
  const firstWrite = new Promise<void>((resolve) => {
    releaseFirstWrite = resolve;
  });
  fixture.settingsList.mockImplementation(async () => ({
    declarations,
    revision: new Uint8Array([revision]),
    value: { values: revision === 1 ? {} : stored },
  }));
  fixture.secretsList.mockResolvedValue({ secrets: [] });
  fixture.settingsSet.mockImplementation(async (identity, request) => {
    if (fixture.settingsSet.mock.calls.length === 1) {
      await firstWrite;
    }
    assert.equal(identity, session);
    assert.deepEqual(request.expectedRevision, new Uint8Array([revision]));
    assert.ok(request.requestId);
    assert.deepEqual(Object.keys(request.value.values).sort(), Object.keys(stored).sort());
    for (const declaration of declarations) {
      const value = request.value.values[declaration.name];
      assert.equal(value.value.case, declaration.default?.value.case);
    }
    stored = request.value.values;
    revision++;
    return { value: { values: stored }, revision: new Uint8Array([revision]) };
  });
  const initial = mapNativeRuntimePluginSettings(await fixture.settingsList(), { secrets: [] });
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  const saved: SettingValues[] = [];
  onTestFinished(async () => {
    releaseFirstWrite();
    await act(async () => root.unmount());
    host.remove();
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });
  await act(async () =>
    root.render(
      createElement(InstalledPluginSettingsForm, {
        schema: initial.settings,
        values: initial.values,
        onSubmit: async (values) => {
          const result = await updateNativePluginSettings(session, "example", values);
          saved.push(result.values);
        },
      }),
    ),
  );
  const edit = async (title: string, value: string) => {
    const label = [...host.querySelectorAll("label")].find(
      (element) => element.textContent === title || element.textContent === `${title} [s]`,
    );
    assert.ok(label);
    const control = host.querySelector(`#${CSS.escape(label.htmlFor)}`);
    assert.ok(control instanceof HTMLInputElement);
    const prototype = HTMLInputElement.prototype;
    await act(async () => {
      Object.getOwnPropertyDescriptor(prototype, "value")?.set?.call(control, value);
      control.dispatchEvent(new Event("input", { bubbles: true }));
    });
  };
  // Reject Bounds Before Any Host Mutation.
  await edit("Gain", "11");
  await act(async () => vi.runOnlyPendingTimersAsync());
  assert.equal(fixture.settingsSet.mock.calls.length, 0);
  assert.ok(host.querySelector('[aria-invalid="true"]'));

  // Edit All Rows Before Their Debounces Drain.
  const flag = host.querySelector('[role="switch"]');
  assert.ok(flag instanceof HTMLButtonElement);
  await act(async () => flag.click());
  await edit("Count", "9007199254740993");
  await edit("Gain", "2.75");
  await edit("Address", "changed");
  await edit("Timeout", "30 min");
  await act(async () => vi.runOnlyPendingTimersAsync());
  assert.equal(fixture.settingsSet.mock.calls.length, 1);
  await act(async () => releaseFirstWrite());
  assert.equal(saved.length, 5);
  assert.equal(host.querySelectorAll("textarea").length, 0);
  assert.equal(host.querySelectorAll('input[type="number"]').length, 2);
  const durationLabel = [...host.querySelectorAll("label")].find((label) => label.textContent === "Timeout [s]");
  assert.ok(durationLabel);
  const durationInput = host.querySelector(`#${CSS.escape(durationLabel.htmlFor)}`);
  assert.ok(durationInput instanceof HTMLInputElement);
  assert.equal(durationInput.value, "30 min");
  assert.deepEqual(saved.at(-1), { enabled: true, count: "9007199254740993", gain: 2.75, address: "changed", timeout: "1800" });
  assert.equal(stored.count.value.case, "integer");
  assert.deepEqual(stored.count.value, { case: "integer", value: 9007199254740993n });
  assert.equal(new Set(fixture.settingsSet.mock.calls.map(([, request]) => request.requestId)).size, 5);

  // A Refusal Keeps the Recovery Action and a Corrected Edit Can Save.
  const refusal = { code: "revision_conflict", message: "fixture_conflict", action: "fixture_reload" };
  fixture.settingsSet.mockRejectedValueOnce(refusal);
  await edit("Address", "refused");
  await act(async () => vi.runOnlyPendingTimersAsync());
  assert.equal(stored.address.value.value, "changed");
  assert.ok(host.querySelector('[role="alert"]'));
  await edit("Address", "corrected");
  await act(async () => vi.runOnlyPendingTimersAsync());
  assert.equal(stored.address.value.value, "corrected");
  assert.equal(saved.at(-1)?.address, "corrected");
});

test("Bounds and Typed Values Are Validated Before Set and Stale Generations Never Write", async () => {
  const declaration = create(SettingDeclarationSchema, {
    name: "count",
    title: "Count",
    default: { value: { case: "integer", value: 2n } },
    minimum: { value: { case: "integer", value: 1n } },
    maximum: { value: { case: "integer", value: 3n } },
  });
  fixture.settingsList.mockResolvedValue({
    declarations: [declaration],
    revision: new Uint8Array([1]),
    value: { values: { count: declaration.default } },
  });
  for (const value of ["0", "4", "1.5", "9223372036854775808"]) {
    await assert.rejects(updateNativePluginSettings(session, "example", { count: value }));
  }
  assert.equal(fixture.settingsSet.mock.calls.length, 0);
  let current = true;
  fixture.settingsList.mockImplementationOnce(async () => {
    current = false;
    return { declarations: [declaration], revision: new Uint8Array([1]) };
  });
  await assert.rejects(updateNativePluginSettings({ generation: session.generation, isCurrent: () => current }, "example", { count: "2" }));
  assert.equal(fixture.settingsSet.mock.calls.length, 0);
  current = true;
  fixture.settingsList.mockResolvedValue({
    declarations: [declaration],
    revision: new Uint8Array([1]),
    value: { values: { count: declaration.default } },
  });
  const refusal = { code: "revision_conflict", message: "fixture_conflict", action: "fixture_reload" };
  fixture.settingsSet.mockRejectedValueOnce(refusal);
  await assert.rejects(updateNativePluginSettings(session, "example", { count: "2" }), (error: unknown) => {
    const failure = failureOf(error, "fixture");
    assert.equal(failure.action, refusal.action);
    assert.equal(failure.message, refusal.message);
    return true;
  });
});
