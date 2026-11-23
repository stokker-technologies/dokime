// @vitest-environment happy-dom
/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Installed Plugin Settings Access
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { beforeEach, onTestFinished, test, vi } from "vitest";
import { type ComponentProps, tick } from "svelte";
import { create } from "@bufbuild/protobuf";
import { QueryClient } from "@tanstack/svelte-query";
import { fireEvent, render, within } from "@testing-library/svelte";
import type { ConnectedSession } from "@/app/native-session/session";
import InstalledPluginDetailView from "@/features/settings/installed-plugins/InstalledPluginDetail.svelte";
import type { InstalledPluginDetail } from "@/features/settings/installed-plugins/installed-plugin-contracts";

// Local Imports
import { getInstalledPluginDetail } from "@/features/settings/installed-plugins/installed-plugin-data-source";
import PluginSettingsSurface from "@/features/settings/PluginSettingsSurface.svelte";
import { statusPresentation } from "@/features/settings/plugins/plugin-rows";
import PluginTable from "@/features/settings/plugins/PluginTable.svelte";
import { bundleSettingsPage, programRows } from "@/features/settings/plugins/plugin-programs";
import type { RuntimeInstalledPluginRecord } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { pluginQueryKeys, startNotification } from "@/features/settings/plugins/plugin-queries";
import SecretsSection from "@/features/settings/sections/SecretsSection.svelte";
import type { ManagedPlugin } from "@/lib/api/native-plugins";
import { FailureSchema, PluginEntrySchema, PluginState, SettingDeclarationSchema } from "@generated/host/model";
import { formatPluginSetting, parsePluginSetting, readPluginSetting } from "@/lib/api/native-settings";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  admitted: vi.fn(),
  installed: vi.fn(),
  settings: vi.fn(),
  save: vi.fn(),
  detail: vi.fn(),
  installations: vi.fn(),
  readSecrets: vi.fn(),
  reload: vi.fn(),
}));
const session = { generation: "1", isCurrent: () => true };
const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
const connected = { identity: session, queryClient } as unknown as ConnectedSession;
const entry: ManagedPlugin = create(PluginEntrySchema, {
  name: "bench",
  revision: new Uint8Array([1]),
  activation: 1n,
  state: PluginState.UNAVAILABLE,
  value: { enabled: true },
  failure: create(FailureSchema, { message: "activation-refused" }),
});

vi.mock("@/lib/api/native-plugins", () => ({ listNativeAdmittedPlugins: fixture.admitted, listNativePlugins: fixture.installed }));
vi.mock("@/lib/api/native-settings", async (original) => ({
  ...(await original<typeof import("@/lib/api/native-settings")>()),
  getNativePluginSettings: fixture.settings,
}));
vi.mock("@/lib/api/native-secrets", () => ({
  readSecrets: fixture.readSecrets,
  putSecret: vi.fn(),
  removeSecret: vi.fn(),
  exportSecrets: vi.fn(),
  importSecrets: vi.fn(),
  pickSecretArchive: vi.fn(),
  previewSecrets: vi.fn(),
}));
vi.mock("@/features/settings/installed-plugins/installed-plugin-queries", async (original) => ({
  ...(await original<typeof import("@/features/settings/installed-plugins/installed-plugin-queries")>()),
  installedPluginDetailsOptions: (_session: ConnectedSession, pluginId: string) => ({
    enabled: Boolean(pluginId),
    queryKey: ["settings", "installed-plugins", "detail", session.generation, pluginId],
    queryFn: async () => {
      const result = fixture.detail(pluginId);
      if (result.isError) {
        throw result.error;
      }
      return result.data;
    },
  }),
  installedPluginSettingsOptions: () => ({ mutationFn: (input: unknown) => fixture.save(input) }),
}));
vi.mock("@/features/settings/settings-queries", async (original) => ({
  ...(await original<typeof import("@/features/settings/settings-queries")>()),
  settingsFrameQueryOptions: () => ({ queryKey: ["settings", "frame"], queryFn: async () => ({ sections: [], cards: [] }) }),
}));
vi.mock("@/features/settings/plugins/plugin-queries", async (original) => ({
  ...(await original<typeof import("@/features/settings/plugins/plugin-queries")>()),
  pluginsListQueryOptions: () => ({
    queryKey: ["settings", "plugins", "list", session.generation],
    queryFn: async () => fixture.installations().data,
  }),
  reloadPluginOptions: () => ({ mutationFn: fixture.reload }),
}));
vi.mock("@/lib/surface-environment", () => ({ isTauriDesktop: () => true }));

beforeEach(() => {
  vi.clearAllMocks();
  queryClient.clear();
  fixture.readSecrets.mockResolvedValue({ entries: [], generation: 1n, hostLabel: "Fixture", hostId: "fixture" });
  fixture.save.mockResolvedValue(undefined);
  fixture.admitted.mockResolvedValue([]);
  fixture.installed.mockResolvedValue([entry]);
  fixture.installations.mockReturnValue({ data: [] });
  fixture.detail.mockImplementation((pluginId: string) => ({
    data: {
      plugin_id: pluginId,
      label: pluginId,
      version: "1.0.0",
      runtime: "code",
      status: "failed",
      error: null,
      settings: {
        contributions: [
          {
            kind: "setting",
            declaration: create(SettingDeclarationSchema, {
              name: "address",
              title: "Address",
              default: { value: { case: "text", value: "" } },
            }),
          },
        ],
      },
      values: { address: "corrected" },
      secrets: [],
    },
  }));
  fixture.settings.mockResolvedValue({
    settings: { contributions: [] },
    values: { address: "corrected" },
    secrets: [],
  });
});

function showPluginSettings() {
  const view = render(PluginSettingsSurface, { pluginId: entry.name, session: connected, label: "Fixture" });
  onTestFinished(() => view.unmount());
  return view;
}

test("Inactive Entry Keeps Its Settings Navigation", () => {
  assert.equal(bundleSettingsPage(entry, [], null), entry.name);
  assert.equal(bundleSettingsPage({ ...entry, value: { enabled: false } }, [], null), entry.name);
});

test("Entry Settings Remain Available After Activation Failure", async () => {
  const plugin = { plugin_id: entry.name, label: entry.name, status: "failed", error: entry.failure?.message };
  fixture.admitted.mockResolvedValue([plugin]);
  const detail = await getInstalledPluginDetail(session, entry.name);
  assert.equal(detail?.plugin_id, entry.name);
  assert.equal(detail?.error, entry.failure?.message);
  assert.deepEqual(detail?.values, { address: "corrected" });
  assert.deepEqual(fixture.settings.mock.calls, [[session, entry.name]]);
  assert.equal(await getInstalledPluginDetail(session, "unknown"), null);
});

test("Failed Forms Keep Their Controls Editable", async () => {
  fixture.installations.mockReturnValue({ data: [entry] });
  const view = showPluginSettings();
  await view.findByRole("textbox", { name: "Address" });
  assert.deepEqual(
    fixture.detail.mock.calls.map(([id]) => id),
    [entry.name],
  );
  assert.equal(view.container.querySelectorAll("input").length, 1);
  assert.equal(view.container.querySelector("[disabled]"), null);
});

test("Entry States Follow Current Activation and Enablement", () => {
  assert.equal(statusPresentation({ ...entry, failure: undefined, state: PluginState.READY }).variant, "success");
  assert.equal(statusPresentation(entry).reason, entry.failure?.message);
  assert.equal(statusPresentation({ ...entry, failure: undefined, value: { enabled: false } }).variant, "outline");
  assert.equal(startNotification({ ...entry, failure: undefined }), null);
  assert.notEqual(startNotification(entry), null);
});

function showPluginEntries(entries: ManagedPlugin[], props: Partial<ComponentProps<typeof PluginTable>> = {}) {
  const view = render(PluginTable, {
    entries,
    generation: session.generation,
    reloading: false,
    removingId: null,
    changingEnabled: false,
    onReload: fixture.reload,
    onRemove: vi.fn(),
    onSetEnabled: vi.fn(),
    openSettings: vi.fn(),
    settingsSectionOf: () => null,
    ...props,
  });
  onTestFinished(() => view.unmount());
  return view;
}

test("Unmanaged Offline Entry Lists Once With Its Placement and Status", () => {
  const managed = create(PluginEntrySchema, { ...entry, name: "equipment" });
  const registered: ManagedPlugin = create(PluginEntrySchema, {
    name: "bench",
    state: PluginState.OFFLINE,
    value: { enabled: true },
    placement: { kind: { case: "remote", value: { url: "http://127.0.0.1:1" } } },
  });
  const view = showPluginEntries([managed, registered]);
  assert.deepEqual(
    [...view.container.querySelectorAll<HTMLTableRowElement>("tr[data-row-id]")].map((row) => [
      row.dataset.rowId,
      row.cells[2].textContent?.trim(),
      row.cells[1].textContent?.trim(),
    ]),
    [
      ["equipment", "Managed", "Failed"],
      ["bench", "Unmanaged", "Offline"],
    ],
  );
  const program = { plugin_id: "bench", owner: { kind: "remote", url: "http://127.0.0.1:1" } } as RuntimeInstalledPluginRecord;
  assert.deepEqual(programRows([program], [registered], null), []);
});

test("Failed Entry Actions Address Its Persisted Name", async () => {
  const remove = vi.fn();
  const enabled = vi.fn();
  const view = showPluginEntries([entry], { onRemove: remove, onSetEnabled: enabled, settingsSectionOf: () => entry.name });
  await fireEvent.click(view.getByRole("button", { name: `Delete ${entry.name}` }));
  await fireEvent.click(view.getByRole("button", { name: `Reload ${entry.name}` }));
  await fireEvent.click(view.getByRole("switch", { name: `Disable ${entry.name}` }));
  assert.deepEqual(remove.mock.calls, [[entry]]);
  assert.deepEqual(fixture.reload.mock.calls, [[entry.name]]);
  assert.deepEqual(enabled.mock.calls, [[entry, false]]);
});

test("Failed Entry Retains Its Failure Without a Root Message", () => {
  const failed = { ...entry, failure: create(FailureSchema, { code: "fixture_refused", action: "fixture_action" }) };
  const view = showPluginEntries([failed], { settingsSectionOf: () => failed.name });
  const alert = view.getByRole("alert");
  assert.equal(view.getAllByRole("alert").length, 1);
  const technical = alert.querySelector("details pre")?.textContent;
  assert.ok(technical?.includes(`Plugin: ${failed.name}`));
  assert.ok(technical?.includes(`Code: ${failed.failure.code}`));
});

test("Declared Secret Add", async () => {
  const detail: InstalledPluginDetail = {
    plugin_id: "equipment.main",
    owner: null,
    label: "Equipment",
    version: "1.0.0",
    enabled: true,
    runtime: "code",
    status: "failed",
    error: null,
    values: {},
    settings: {
      contributions: [
        { kind: "secret", id: "api_token", title: "API Token", description: null, required: true },
        { kind: "secret", id: "optional", title: "Optional", description: null, required: false },
      ],
    },
    secrets: [
      { identifier: "api_token", available: false },
      { identifier: "optional", available: true },
    ],
  };
  const view = render(InstalledPluginDetailView, { detail, session: connected, brandLabel: "Fixture", onSaveSettings: async () => {} });
  onTestFinished(() => view.unmount());
  assert.ok(view.getByText("api_token", { exact: true }));
  assert.equal(view.container.querySelector('input[type="password"]'), null);
  const add = view.getAllByRole("button", { name: "Add in Secret Storage" });
  assert.equal(add.length, 1);
  await fireEvent.click(add[0]);
  const dialog = within(await view.findByRole("dialog", { name: "Add Secret" }));
  const identifier = await dialog.findByLabelText("Identifier");
  assert.ok(identifier instanceof HTMLInputElement);
  assert.equal(identifier.value, "api_token");
  assert.equal(dialog.getByLabelText("Value").getAttribute("type"), "password");
});

test("Missing Secret Action", async () => {
  const clearHash = vi.fn();
  const view = render(SecretsSection, { session: connected, hash: "add-secret=api_token", clearHash });
  onTestFinished(() => view.unmount());
  const dialog = within(await view.findByRole("dialog", { name: "Add Secret" }));
  const identifier = dialog.getByLabelText("Identifier");
  assert.ok(identifier instanceof HTMLInputElement);
  assert.equal(identifier.value, "api_token");
  assert.equal(dialog.getByLabelText("Value").getAttribute("type"), "password");
  assert.equal(clearHash.mock.calls.length, 1);
  await view.rerender({ hash: "" });
  await fireEvent.click(dialog.getByRole("button", { name: "Cancel" }));
  assert.equal(view.queryByRole("dialog"), null);
});

test("Undescribed Plugin Waits Without Requesting Settings", async () => {
  const installation = { ...entry, failure: undefined, state: PluginState.CONNECTING };
  queryClient.setQueryData(pluginQueryKeys.list(session.generation), [installation]);
  queryClient.setQueryData(["settings", "frame"], { sections: [], cards: [] });
  fixture.installations.mockReturnValue({ data: [installation] });
  const view = showPluginSettings();
  assert.ok(view.getByRole("status"));
  assert.equal(view.queryByRole("alert"), null);
  assert.equal(view.queryByRole("button", { name: "Reload" }), null);
  assert.equal(fixture.detail.mock.calls.length, 0);
});

test("Independent Failed Section Occupies Its Content Slot", async () => {
  fixture.installations.mockReturnValue({
    data: [{ ...entry, failure: undefined, description: { id: "bench" }, state: PluginState.READY }],
  });
  const refusal = new Error("fixture-section");
  fixture.detail.mockReturnValue({ isError: true, error: refusal });
  const view = showPluginSettings();
  const alert = await view.findByRole("alert");
  assert.equal(view.getAllByRole("alert").length, 1);
  assert.ok(alert.textContent?.includes(refusal.message));
});

test("Failed Plugin Settings Save the Corrected Value", async () => {
  fixture.installations.mockReturnValue({ data: [entry] });
  const view = showPluginSettings();
  const input = await view.findByRole("textbox", { name: "Address" });
  assert.ok(input instanceof HTMLInputElement);
  assert.equal(input.disabled, false);
  vi.useFakeTimers();
  try {
    await fireEvent.input(input, { target: { value: "repaired-address" } });
    await vi.advanceTimersByTimeAsync(100);
    await tick();
    assert.deepEqual(fixture.save.mock.calls, [[{ pluginId: entry.name, values: { address: "repaired-address" } }]]);
  } finally {
    vi.useRealTimers();
  }
});

test("Durations Format Compact Units and Parse Them Without Losing Nanoseconds", () => {
  const declaration = create(SettingDeclarationSchema, {
    name: "timeout",
    title: "Timeout",
    default: { value: { case: "duration", value: { seconds: 5n } } },
  });
  for (const [input, seconds, nanos, display] of [
    ["5.000000000", 5n, 0, "5 s"],
    ["1800.000000000", 1800n, 0, "30 min"],
    ["1.5 s", 1n, 500000000, "1.5 s"],
    ["1500 ms", 1n, 500000000, "1.5 s"],
    ["0.000001 ms", 0n, 1, "1 ns"],
    ["-0.5 s", 0n, -500000000, "-500 ms"],
    ["0 ns", 0n, 0, "0 s"],
    ["2 h", 7200n, 0, "2 h"],
    ["1 d", 86400n, 0, "1 d"],
    ["42 us", 0n, 42000, "42 us"],
  ] as const) {
    const parsed = parsePluginSetting(declaration, input);
    assert.deepEqual(parsed.value, { case: "duration", value: { seconds, nanos } });
    assert.equal(formatPluginSetting(parsed), display);
    assert.deepEqual(parsePluginSetting(declaration, display), parsed);
  }
  for (const input of ["", "1,5 s", "1 000 s", "0.1 ns", "1.0000000001 s", "315576000001 s", "5 seconds"]) {
    assert.throws(() => parsePluginSetting(declaration, input));
  }
});

test("Typed Bounds Reject Invalid Drafts With Exact Integer and Duration Comparisons", () => {
  const integer = create(SettingDeclarationSchema, {
    name: "count",
    title: "Count",
    default: { value: { case: "integer", value: 9007199254740993n } },
    minimum: { value: { case: "integer", value: 9007199254740992n } },
    maximum: { value: { case: "integer", value: 9007199254740994n } },
  });
  assert.equal(readPluginSetting(parsePluginSetting(integer, "9007199254740993")), "9007199254740993");
  for (const value of ["9007199254740991", "9007199254740995", "2.5", "1e3", "1,000"]) {
    assert.throws(() => parsePluginSetting(integer, value));
  }
  const duration = create(SettingDeclarationSchema, {
    name: "timeout",
    title: "Timeout",
    default: { value: { case: "duration", value: { seconds: 5n } } },
    minimum: { value: { case: "duration", value: { seconds: 5n } } },
    maximum: { value: { case: "duration", value: { seconds: 1800n } } },
  });
  assert.equal(readPluginSetting(parsePluginSetting(duration, "30 min")), "1800");
  for (const value of ["4.999999999 s", "1800.000000001 s"]) {
    assert.throws(() => parsePluginSetting(duration, value));
  }
  const number = create(SettingDeclarationSchema, {
    name: "gain",
    title: "Gain",
    default: { value: { case: "number", value: 2.5 } },
    minimum: { value: { case: "number", value: 0.5 } },
    maximum: { value: { case: "number", value: 10 } },
  });
  assert.equal(formatPluginSetting(parsePluginSetting(number, "2.5000")), "2.5");
  for (const value of ["", "2,5", "0.49", "10.01", "Infinity", "NaN", " ", "0x10"]) {
    assert.throws(() => parsePluginSetting(number, value));
  }
});
