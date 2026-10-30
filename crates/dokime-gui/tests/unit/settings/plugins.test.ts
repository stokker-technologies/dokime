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
import { createElement, type ComponentProps, type ReactNode } from "react";
import { InstalledPluginDetail } from "@/features/settings/installed-plugins/InstalledPluginDetail";
import type { InstalledPluginDetail as PluginDetail } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { renderToStaticMarkup } from "react-dom/server";

// Local Imports
import { getInstalledPluginDetail } from "@/features/settings/installed-plugins/installed-plugin-data-source";
import { PluginSettingsSurface } from "@/features/settings/PluginSettingsSurface";
import { bundleSettingsPage } from "@/features/settings/plugins/PluginPrograms";
import { SecretsSection } from "@/features/settings/sections/SecretsSection";
import type { PluginEntry } from "@/lib/types/view-contracts";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  admitted: vi.fn(),
  installed: vi.fn(),
  settings: vi.fn(),
  detail: vi.fn(),
  installations: vi.fn(),
  captureAdding: false,
  capturedAdding: false,
  adding: null as string | null,
  buttons: [] as (() => void)[],
  hash: "",
  navigate: vi.fn(),
}));
const session = { generation: "1", isCurrent: () => true };
const entry: PluginEntry = {
  install_id: "installation",
  sha256: "a".repeat(64),
  installed_at: "2026-10-30T00:00:00Z",
  state: "installed",
  last_boot: {
    state: "failed",
    at: "2026-10-30T00:00:00Z",
    failure: {
      plugin_id: "equipment.main",
      plugin_name: "Equipment",
      message: "startup-refused",
      code: "10001",
      missing_secret: "token",
      consequences: ["dependent workflow requires equipment"],
    },
  },
  metadata: {
    identity: { name: "equipment", version: "1.0.0", description: "" },
    owners: ["equipment.main", "equipment.extra"],
    size_bytes: 1,
  },
  active: false,
  restart_required: false,
  boot_failed: true,
  error: null,
};

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

vi.mock("react", async (original) => {
  const react = await original<typeof import("react")>();
  return {
    ...react,
    useState: (initial: unknown) => {
      const state = react.useState(initial);
      if (fixture.captureAdding && initial === null && !fixture.capturedAdding) {
        fixture.capturedAdding = true;
        return [
          fixture.adding,
          (value: string | null) => {
            fixture.adding = value;
          },
        ];
      }
      return state;
    },
  };
});
vi.mock("@/components/ui/button", () => ({
  Button: ({ onClick, ...props }: ComponentProps<"button">) => {
    if (onClick) fixture.buttons.push(() => onClick({} as never));
    return createElement("button", { type: "button", ...props });
  },
}));
vi.mock("@/lib/api/native-plugins", () => ({ listNativeAdmittedPlugins: fixture.admitted, listNativePlugins: fixture.installed }));
vi.mock("@/lib/api/native-settings", () => ({ getNativePluginSettings: fixture.settings }));
vi.mock("@/lib/api/native-secrets", () => ({ readSecrets: vi.fn(), putSecret: vi.fn(), removeSecret: vi.fn() }));
vi.mock("@/features/settings/sections/SecretTransfer", () => ({ SecretTransfer: () => null }));
vi.mock("@/app/native-session/context", () => ({ useNativeSessionOptional: () => ({ identity: session }) }));
vi.mock("@tanstack/react-query", () => ({
  useQuery: () => ({ data: { entries: [], generation: 1n, hostLabel: "Fixture", hostId: "fixture" }, isFetching: false }),
  useQueryClient: () => ({}),
}));
vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, params, hash, children }: { to: string; params: { section: string }; hash?: string; children: ReactNode }) =>
    createElement("a", { href: `${to.replace("$section", params.section)}${hash ? `#${hash}` : ""}` }, children),
  useLocation: () => fixture.hash,
  useNavigate: () => fixture.navigate,
}));
vi.mock("@/components/ui/dialog", () => ({
  Dialog: ({ open, children }: { open: boolean; children: ReactNode }) => (open ? children : null),
  DialogContent: ({ children }: { children: ReactNode }) => createElement("div", { role: "dialog" }, children),
  DialogHeader: ({ children }: { children: ReactNode }) => children,
  DialogTitle: ({ children }: { children: ReactNode }) => children,
  DialogDescription: ({ children }: { children: ReactNode }) => children,
  DialogFooter: ({ children }: { children: ReactNode }) => children,
}));
vi.mock("@/features/settings/installed-plugins/installed-plugin-queries", () => ({
  useInstalledPluginDetailQuery: fixture.detail,
  useInstalledPluginSettingsMutation: () => ({ mutateAsync: vi.fn() }),
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
  fixture.captureAdding = false;
  fixture.capturedAdding = false;
  fixture.adding = null;
  fixture.buttons.length = 0;
  fixture.hash = "";
  fixture.admitted.mockResolvedValue([]);
  fixture.installed.mockResolvedValue([entry]);
  fixture.settings.mockResolvedValue({
    settings: { contributions: [] },
    values: { address: "corrected" },
    discovered: {},
    secrets: [],
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
    assert.equal(detail?.error, null);
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
      error: null,
      settings: {
        contributions: [
          { kind: "text_area", id: "address", title: "Address", default: "", required: false },
          { kind: "secret", id: "token", title: "Token", required: true },
        ],
      },
      values: { address: "corrected" },
      discovered: {},
      secrets: [{ identifier: "token", available: false }],
    },
  }));
  const html = renderToStaticMarkup(createElement(PluginSettingsSurface, { pluginId: entry.install_id }));
  assert.equal(fixture.detail.mock.calls.length, 2);
  assert.deepEqual(
    fixture.detail.mock.calls.map(([id]) => id),
    entry.metadata?.owners,
  );
  assert.equal((html.match(/startup-refused/g) ?? []).length, 1);
  assert.ok(html.includes('href="/settings/secrets#add-secret=token"'));
  assert.ok(html.includes("<details"));
  assert.equal(html.includes("<details open"), false);
  assert.ok(html.includes("dependent workflow requires equipment"));
  assert.ok(html.includes('id="plugin-settings-equipment.main"'));
  assert.equal((html.match(/<textarea/g) ?? []).length, 2);
  assert.equal((html.match(/type="password"/g) ?? []).length, 0);
  assert.equal((html.match(/Add In Secret Storage/g) ?? []).length, 2);
  assert.equal((html.match(/<button/g) ?? []).length, 3);
  assert.equal(html.includes("disabled="), false);
  assert.equal((html.match(/>corrected<\/textarea>/g) ?? []).length, 2);
  assert.equal(entry.last_boot?.state, "failed");
  if (entry.last_boot?.state !== "failed") throw new Error("fixture must have a boot failure");
  for (const code of ["plugin_contract_unavailable", "plugin_contract_cycle"]) {
    fixture.installations.mockReturnValue({
      data: [{ ...entry, last_boot: { ...entry.last_boot, failure: { ...entry.last_boot.failure, code, missing_secret: null } } }],
    });
    const refused = renderToStaticMarkup(createElement(PluginSettingsSurface, { pluginId: entry.install_id }));
    assert.equal((refused.match(/startup-refused/g) ?? []).length, 1);
    assert.equal((refused.match(/type="password"/g) ?? []).length, 0);
    assert.ok(refused.includes(code));
    assert.equal(refused.includes('href="/settings/secrets#'), false);
  }
  fixture.installations.mockReturnValue({ data: [{ ...entry, state: "disabled", boot_failed: false }] });
  const disabled = renderToStaticMarkup(createElement(PluginSettingsSurface, { pluginId: entry.install_id }));
  assert.equal(disabled.includes("startup-refused"), false);
});

test("Declared Secret Add", () => {
  const detail: PluginDetail = {
    plugin_id: "equipment.main",
    owner: null,
    label: "Equipment",
    version: "1.0.0",
    enabled: true,
    runtime: "code",
    status: "failed",
    error: null,
    values: {},
    discovered: {},
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
  const render = () => {
    fixture.capturedAdding = false;
    return renderToStaticMarkup(createElement(InstalledPluginDetail, { detail, onSaveSettings: async () => {} }));
  };
  fixture.captureAdding = true;
  const initial = render();
  assert.ok(initial.includes("api_token"));
  assert.ok(initial.includes("Required · Missing"));
  assert.ok(initial.includes("Optional · Available"));
  assert.equal((initial.match(/Add In Secret Storage/g) ?? []).length, 1);
  assert.equal(initial.includes('type="password"'), false);
  assert.equal(fixture.buttons.length, 1);
  fixture.buttons[0]();
  const opened = render();
  assert.ok(opened.includes("Add Secret"));
  assert.ok(opened.includes('value="api_token"'));
  assert.ok(opened.includes('type="password"'));
});

test("Missing Secret Action", () => {
  fixture.hash = "add-secret=api_token";
  const html = renderToStaticMarkup(createElement(SecretsSection));
  assert.ok(html.includes('role="dialog"'));
  assert.ok(html.includes('value="api_token"'));
  assert.ok(html.includes('type="password"'));
  fixture.hash = "";
  const ordinary = renderToStaticMarkup(createElement(SecretsSection));
  assert.equal(ordinary.includes('role="dialog"'), false);
});
