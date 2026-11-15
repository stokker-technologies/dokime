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
import { create } from "@bufbuild/protobuf";
import { createElement, type ComponentProps, type ReactNode } from "react";
import { InstalledPluginDetail } from "@/features/settings/installed-plugins/InstalledPluginDetail";
import type { InstalledPluginDetail as PluginDetail } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import { renderToStaticMarkup } from "react-dom/server";

// Local Imports
import { getInstalledPluginDetail } from "@/features/settings/installed-plugins/installed-plugin-data-source";
import { PluginSettingsSurface } from "@/features/settings/PluginSettingsSurface";
import { PluginList, statusPresentation } from "@/features/settings/plugins/PluginList";
import type { PluginTableRow } from "@/features/settings/plugins/PluginTable";
import { bundleSettingsPage } from "@/features/settings/plugins/PluginPrograms";
import { startNotification } from "@/features/settings/plugins/plugin-queries";
import { SecretsSection } from "@/features/settings/sections/SecretsSection";
import type { ManagedPlugin } from "@/lib/api/native-plugins";
import { FailureSchema, PluginEntrySchema, PluginState } from "@host/model";

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
  reload: vi.fn(),
  actions: new Map<string, () => void>(),
  rows: [] as PluginTableRow[],
}));
const session = { generation: "1", isCurrent: () => true };
const entry: ManagedPlugin = create(PluginEntrySchema, {
  name: "equipment",
  revision: new Uint8Array([1]),
  activation: 1n,
  state: PluginState.UNAVAILABLE,
  value: { enabled: true },
  failure: create(FailureSchema, { message: "activation-refused" }),
});

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
    if (onClick && !props.disabled && props.children !== "Copy") fixture.buttons.push(() => onClick({} as never));
    if (onClick && !props.disabled && props["aria-label"]) fixture.actions.set(props["aria-label"], () => onClick({} as never));
    return createElement("button", { type: "button", ...props });
  },
}));
vi.mock("@/features/settings/plugins/PluginTable", () => ({
  PluginTable: ({ rows }: { rows: PluginTableRow[] }) => {
    fixture.rows = rows;
    return createElement(
      "div",
      null,
      rows.map((row) => createElement("div", { key: row.key }, row.trailing)),
    );
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
  useInstalledPluginSecretMutation: () => ({ mutateAsync: vi.fn() }),
  usePluginRestartRefresh: vi.fn(),
}));
vi.mock("@/features/settings/plugins/plugin-queries", async (original) => ({
  ...(await original<typeof import("@/features/settings/plugins/plugin-queries")>()),
  usePluginsQuery: fixture.installations,
  useReloadPluginMutation: () => ({ mutate: fixture.reload, isPending: false, isError: false }),
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
  fixture.actions.clear();
  fixture.hash = "";
  fixture.admitted.mockResolvedValue([]);
  fixture.installed.mockResolvedValue([entry]);
  fixture.settings.mockResolvedValue({
    settings: { contributions: [] },
    values: { address: "corrected" },
    secrets: [],
  });
});

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

test("Failed Forms Keep Their Controls Editable", () => {
  fixture.installations.mockReturnValue({ data: [entry] });
  fixture.detail.mockImplementation((pluginId: string) => ({
    data: {
      plugin_id: pluginId,
      label: pluginId,
      version: "1.0.0",
      runtime: "code",
      status: "failed",
      error: null,
      settings: { contributions: [{ kind: "text_area", id: "address", title: "Address", default: "", required: false }] },
      values: { address: "corrected" },
      secrets: [],
    },
  }));
  const html = renderToStaticMarkup(createElement(PluginSettingsSurface, { pluginId: entry.name }));
  assert.deepEqual(
    fixture.detail.mock.calls.map(([id]) => id),
    [entry.name],
  );
  assert.equal((html.match(/<textarea/g) ?? []).length, 1);
  assert.equal(html.includes("disabled="), false);
});

test("Entry States Follow Current Activation and Enablement", () => {
  assert.equal(statusPresentation({ ...entry, failure: undefined, state: PluginState.READY }).variant, "success");
  assert.equal(statusPresentation(entry).reason, entry.failure?.message);
  assert.equal(statusPresentation({ ...entry, failure: undefined, value: { enabled: false } }).variant, "outline");
  assert.equal(startNotification({ ...entry, failure: undefined }), null);
  assert.notEqual(startNotification(entry), null);
});

test("Failed Entry Actions Address Its Persisted Name", () => {
  const remove = vi.fn();
  const enabled = vi.fn();
  renderToStaticMarkup(
    createElement(PluginList, {
      entries: [entry],
      onRemove: remove,
      onOpenSettings: vi.fn(),
      removingId: null,
      onSetEnabled: enabled,
      changingEnabled: false,
      settingsSectionOf: () => entry.name,
    }),
  );
  fixture.actions.get(`Delete ${entry.name}`)?.();
  fixture.actions.get(`Reload ${entry.name}`)?.();
  fixture.rows[0].enabled?.onChange(false);
  assert.deepEqual(remove.mock.calls, [[entry]]);
  assert.deepEqual(fixture.reload.mock.calls, [[entry.name]]);
  assert.deepEqual(enabled.mock.calls, [[entry, false]]);
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
    return renderToStaticMarkup(
      createElement(InstalledPluginDetail, { detail, onSaveSettings: async () => {}, onSaveBinding: async () => {} }),
    );
  };
  fixture.captureAdding = true;
  const initial = render();
  assert.ok(initial.includes("api_token"));
  assert.equal(initial.includes('type="password"'), false);
  assert.equal(fixture.buttons.length, 1);
  fixture.buttons[0]();
  const opened = render();
  assert.ok(opened.includes('role="dialog"'));
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
