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
import { beforeEach, test, vi } from "vitest";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

// Local Imports
import { getInstalledPluginDetail } from "@/features/settings/installed-plugins/installed-plugin-data-source";
import { PluginSettingsSurface } from "@/features/settings/PluginSettingsSurface";
import { bundleSettingsPage } from "@/features/settings/plugins/PluginPrograms";
import type { PluginEntry } from "@/lib/types/view-contracts";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({ admitted: vi.fn(), installed: vi.fn(), settings: vi.fn(), detail: vi.fn(), installations: vi.fn() }));
const session = { generation: "1", isCurrent: () => true };
const entry: PluginEntry = {
  install_id: "installation",
  sha256: "a".repeat(64),
  installed_at: "2026-10-30T00:00:00Z",
  state: "installed",
  last_boot: null,
  metadata: {
    identity: { name: "equipment", version: "1.0.0", description: "" },
    owners: ["equipment.main", "equipment.extra"],
    size_bytes: 1,
  },
  active: false,
  restart_required: false,
  boot_error: "startup-refused",
  error: null,
};

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

vi.mock("@/lib/api/native-plugins", () => ({ listNativeAdmittedPlugins: fixture.admitted, listNativePlugins: fixture.installed }));
vi.mock("@/lib/api/native-settings", () => ({ getNativePluginSettings: fixture.settings }));
vi.mock("@/features/settings/installed-plugins/installed-plugin-queries", () => ({
  useInstalledPluginDetailQuery: fixture.detail,
  useInstalledPluginSettingsMutation: () => ({ mutateAsync: vi.fn() }),
  useInstalledPluginSecretMutation: () => ({ mutateAsync: vi.fn(), reset: vi.fn(), isPending: false }),
}));
vi.mock("@/features/settings/plugins/plugin-queries", () => ({
  usePluginsQuery: fixture.installations,
  useRestartAppMutation: () => ({ mutate: vi.fn(), isPending: false, isError: false }),
}));
vi.mock("@/features/settings/use-settings-sections", () => ({ useSettingsSections: () => ({ frame: { sections: [], cards: [] } }) }));
vi.mock("@/features/settings/SettingsSurface", () => ({ SettingsAdvancedSwitch: () => null }));
vi.mock("@/features/settings/PluginSettingsCards", () => ({ PluginSettingsCards: () => null }));
vi.mock("@/shell/page-chrome", () => ({ usePageChrome: vi.fn() }));
vi.mock("@/lib/brand", () => ({ useBrand: () => ({ label: "Fixture" }) }));
vi.mock("@/lib/desktop-open", () => ({ isTauriDesktop: () => true }));

beforeEach(() => {
  vi.clearAllMocks();
  fixture.admitted.mockResolvedValue([]);
  fixture.installed.mockResolvedValue([entry]);
  fixture.settings.mockResolvedValue({
    settings: { contributions: [] },
    values: { address: "corrected" },
    discovered: {},
    secret_bindings: { bindings: {}, missing_required: [] },
  });
});

test("Inactive Bundle Navigation", () => {
  assert.equal(bundleSettingsPage(entry, [], null), entry.install_id);
  assert.equal(bundleSettingsPage({ ...entry, state: "disabled" }, [], null), entry.install_id);
  assert.equal(bundleSettingsPage({ ...entry, state: "removal_pending" }, [], null), null);
  assert.equal(bundleSettingsPage({ ...entry, metadata: null }, [], null), null);
});

test("Failed Owner Settings", async () => {
  assert.ok(entry.metadata);
  for (const owner of entry.metadata.owners) {
    const detail = await getInstalledPluginDetail(session, owner);
    assert.equal(detail?.plugin_id, owner);
    assert.equal(detail?.error, entry.boot_error);
    assert.equal(detail?.status, "failed");
    assert.deepEqual(detail?.values, { address: "corrected" });
  }
  assert.deepEqual(
    fixture.settings.mock.calls,
    entry.metadata.owners.map((owner) => [session, owner]),
  );
  assert.equal(await getInstalledPluginDetail(session, "unknown"), null);
  fixture.installed.mockResolvedValue([{ ...entry, state: "removal_pending" }]);
  assert.equal(await getInstalledPluginDetail(session, "equipment.main"), null);
});

test("Running Settings Preserved", async () => {
  const plugin = { plugin_id: "equipment.main", label: "Equipment", status: "ready", error: null };
  fixture.admitted.mockResolvedValue([plugin]);
  const detail = await getInstalledPluginDetail(session, plugin.plugin_id);
  assert.equal(detail?.label, plugin.label);
  assert.equal(detail?.status, "ready");
  assert.equal(fixture.installed.mock.calls.length, 0);
  const program = {
    ...plugin,
    owner: { kind: "bundle" as const, package: "equipment", path: null, installation: entry.install_id },
    version: "1.0.0",
    enabled: true,
    runtime: "code",
  };
  const page = bundleSettingsPage({ ...entry, state: "removal_pending", active: true }, [program], {
    bootstrap_paths: null,
    cards: [],
    sections: [
      { section_id: "equipment.settings", label: "Equipment", owner: { kind: "installed_plugin", plugin_id: program.plugin_id }, order: 0 },
    ],
  });
  assert.equal(page, "equipment.settings");
});

test("Failed Forms Editable", () => {
  fixture.installations.mockReturnValue({ data: [entry] });
  fixture.detail.mockImplementation((pluginId: string) => ({
    data: {
      plugin_id: pluginId,
      label: pluginId,
      version: "1.0.0",
      runtime: "code",
      status: "failed",
      error: entry.boot_error,
      settings: {
        contributions: [
          { kind: "text_area", id: "address", title: "Address", default: "", required: false },
          { kind: "secret_reference", id: "token", title: "Token", default: null, required: true },
        ],
      },
      values: { address: "corrected" },
      discovered: {},
      secret_bindings: { bindings: {}, missing_required: ["token"] },
    },
  }));
  const html = renderToStaticMarkup(createElement(PluginSettingsSurface, { pluginId: entry.install_id }));
  assert.equal(fixture.detail.mock.calls.length, 2);
  assert.deepEqual(
    fixture.detail.mock.calls.map(([id]) => id),
    entry.metadata?.owners,
  );
  assert.equal((html.match(/<textarea/g) ?? []).length, 2);
  assert.equal((html.match(/type="password"/g) ?? []).length, 2);
  assert.equal((html.match(/<button/g) ?? []).length, 1);
  assert.equal(html.includes("disabled="), false);
  assert.equal((html.match(/>corrected<\/textarea>/g) ?? []).length, 2);
});
