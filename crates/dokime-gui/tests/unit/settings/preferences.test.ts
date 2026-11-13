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
 * Scoped Preferences Behavior
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
// Local Imports
import type { SettingsSnapshot } from "@generated/application-settings";
import defaults from "@generated/application-settings.json";
// Third Party Imports
import { act, type ComponentProps, createElement } from "react";
import { createRoot } from "react-dom/client";
import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, onTestFinished, test, vi } from "vitest";
import { ApplicationPreferences } from "@/features/settings/ApplicationPreferences";
import { InstalledPluginSettingsForm } from "@/features/settings/installed-plugins/InstalledPluginSettingsForm";
import { preferencesKey, usePreferences, useSavePreferences } from "@/features/settings/application-preferences";
import { createDebouncedWriter } from "@/features/settings/settings-autosave";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  read: vi.fn(),
  notices: vi.fn(),
  update: vi.fn(),
  cache: vi.fn(),
  invalidate: vi.fn(),
  appearance: vi.fn(),
  session: null as { identity: { generation: string } } | null,
  snapshot: null as SettingsSnapshot | null,
  loading: false,
  backendError: null as unknown,
  saveError: null as unknown,
  queries: [] as { queryKey: readonly string[]; queryFn: () => Promise<SettingsSnapshot>; retry: boolean }[],
  mutations: [] as {
    mutationFn: (input: { previous: SettingsSnapshot; settings: SettingsSnapshot["settings"] }) => Promise<SettingsSnapshot>;
    onSuccess: (snapshot: SettingsSnapshot) => void;
  }[],
}));

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function SavePreferencesProbe({ scope }: { scope: "local" | "backend" }) {
  useSavePreferences(scope);
  return null;
}

function snapshot(): SettingsSnapshot {
  return {
    revision: 7,
    settings: {
      appearance: { theme_mode: "dark", theme_family: "fixture", density: "compact" },
      report: { ...defaults.report, plot_points: 3210, plot_downsample: "lttb" },
      client: { connect_timeout_ms: 1234, request_timeout_ms: 5678 },
      telemetry: { interval_ms: 42 },
      listener: {
        enabled: true,
        address: "127.0.0.1:9876",
        tls_enabled: true,
        authentication_enabled: false,
        certificate_names: ["fixture.local", "127.0.0.1"],
      },
      plugins: { startup_timeout_s: 75, stop_timeout_s: 15 },
    },
  };
}

/* === Entry Point === */

vi.mock("@/lib/notifications", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

vi.mock("@/components/NotificationMessage", () => ({
  NotificationMessage: (props: unknown) => {
    fixture.notices(props);
    return null;
  },
}));
vi.mock("@/app/native-session/context", () => ({ useNativeSessionOptional: () => fixture.session }));
vi.mock("@/lib/desktop/settings", () => ({
  preferencesCommands: {
    desktopSettingsRead: fixture.read,
    desktopSettingsUpdate: fixture.update,
  },
}));
vi.mock("@/stores/ui-store", () => ({ useUiStore: { getState: () => ({ setUiState: fixture.appearance }) } }));
vi.mock("@tanstack/react-query", () => ({
  useQuery: (options: (typeof fixture.queries)[number]) => {
    fixture.queries.push(options);
    const error = options.queryKey[2] === "backend" ? fixture.backendError : null;
    return { data: error ? null : fixture.snapshot, error, isPending: fixture.loading, refetch: fixture.read };
  },
  useMutation: (options: (typeof fixture.mutations)[number]) => {
    fixture.mutations.push(options);
    return {
      mutateAsync: async (input: Parameters<typeof options.mutationFn>[0]) => {
        const saved = await options.mutationFn(input);
        options.onSuccess(saved);
        return saved;
      },
      reset: vi.fn(),
      mutate: vi.fn(),
      isPending: false,
      error: fixture.saveError,
      isSuccess: false,
    };
  },
  useQueryClient: () => ({ setQueryData: fixture.cache, invalidateQueries: fixture.invalidate }),
}));

beforeEach(() => {
  vi.clearAllMocks();
  fixture.snapshot = snapshot();
  fixture.session = null;
  fixture.loading = false;
  fixture.backendError = null;
  fixture.saveError = null;
  fixture.queries.length = 0;
  fixture.mutations.length = 0;
  fixture.read.mockResolvedValue(fixture.snapshot);
  fixture.update.mockResolvedValue({ ...fixture.snapshot, revision: 8 });
});

test("Advanced Reveals Additional Preference Fields", () => {
  const simple =
    renderToStaticMarkup(createElement(ApplicationPreferences)) +
    renderToStaticMarkup(createElement(ApplicationPreferences, { section: "remote" }));
  const advanced =
    renderToStaticMarkup(createElement(ApplicationPreferences, { advanced: true })) +
    renderToStaticMarkup(createElement(ApplicationPreferences, { advanced: true, section: "remote" }));
  const visible = [
    "local-appearance.theme_mode",
    "local-appearance.theme_family",
    "local-appearance.density",
    "backend-report.plot_points",
    "backend-report.plot_downsample",
    "backend-listener.enabled",
    "backend-listener.address",
    "backend-listener.tls_enabled",
    "backend-listener.authentication_enabled",
  ];
  const additional = [
    "local-client.connect_timeout_ms",
    "local-client.request_timeout_ms",
    "backend-telemetry.interval_ms",
    "backend-listener.certificate_names",
  ];
  for (const id of visible) assert.ok(simple.includes(`id="${id}"`), id);
  for (const id of additional) {
    assert.ok(!simple.includes(`id="${id}"`), id);
    assert.ok(advanced.includes(`id="${id}"`), id);
  }
  for (const value of ["fixture", "3210", "1234", "5678", "42", "127.0.0.1:9876", "fixture.local, 127.0.0.1"])
    assert.ok(advanced.includes(`value="${value}"`), value);
});

test("Plugins Section Edits Only the Backend Plugin Budgets", () => {
  const simple = renderToStaticMarkup(createElement(ApplicationPreferences, { section: "plugins" }));
  const advanced = renderToStaticMarkup(createElement(ApplicationPreferences, { advanced: true, section: "plugins" }));
  const elsewhere = renderToStaticMarkup(createElement(ApplicationPreferences, { advanced: true }));
  assert.ok(simple.includes('id="backend-plugins.startup_timeout_s"'));
  assert.ok(simple.includes('value="75"'));
  assert.ok(!simple.includes('id="backend-plugins.stop_timeout_s"'));
  assert.ok(advanced.includes('id="backend-plugins.stop_timeout_s"'));
  assert.ok(advanced.includes('value="15"'));
  for (const id of ["local-appearance.theme_mode", "backend-report.plot_points", "backend-listener.enabled"])
    assert.ok(!advanced.includes(`id="${id}"`), id);
  for (const id of ["backend-plugins.startup_timeout_s", "backend-plugins.stop_timeout_s"])
    assert.ok(!elsewhere.includes(`id="${id}"`), id);
});

test("Scoped Saves Send All Fields and the Reviewed Revision", async () => {
  for (const scope of ["local", "backend"] as const) {
    renderToStaticMarkup(createElement(SavePreferencesProbe, { scope }));
    const mutation = fixture.mutations.at(-1);
    assert.ok(mutation);
    const previous = snapshot();
    const settings = structuredClone(previous.settings);
    if (scope === "local") {
      settings.appearance = { theme_mode: "light", theme_family: "other", density: "touch" };
      settings.client = { connect_timeout_ms: 2000, request_timeout_ms: 9000 };
    } else {
      settings.report = { ...settings.report, plot_points: 4000, plot_downsample: "raw" };
      settings.telemetry.interval_ms = 200;
      settings.plugins = { startup_timeout_s: 120, stop_timeout_s: 30 };
      settings.listener = {
        enabled: false,
        address: "0.0.0.0:8766",
        tls_enabled: false,
        authentication_enabled: true,
        certificate_names: ["other.local"],
      };
    }
    const saved = { revision: 8, settings };
    fixture.update.mockResolvedValueOnce(saved);
    assert.deepEqual(await mutation.mutationFn({ previous, settings }), saved);
    assert.deepEqual(fixture.update.mock.calls.at(-1), [scope, { expected_revision: 7, settings }]);
    mutation.onSuccess(saved);
    assert.deepEqual(fixture.cache.mock.calls.at(-1), [preferencesKey(scope), saved]);
    assert.deepEqual(previous, snapshot());
  }
  assert.equal(fixture.appearance.mock.calls.length, 1);
  assert.deepEqual(fixture.appearance.mock.calls[0], [{ density: "touch", themeFamily: "other" }]);
});

test("Failed Saves Preserve Revisions", async () => {
  for (const failure of [{ kind: "conflict" }, { kind: "persistence", message: "fixture failure" }]) {
    fixture.saveError = failure;
    fixture.update.mockRejectedValueOnce(failure);
    renderToStaticMarkup(createElement(SavePreferencesProbe, { scope: "local" }));
    const mutation = fixture.mutations.at(-1);
    assert.ok(mutation);
    await assert.rejects(mutation.mutationFn({ previous: snapshot(), settings: snapshot().settings }), (error) => error === failure);
    const html = renderToStaticMarkup(createElement(ApplicationPreferences, { advanced: true }));
    assert.ok(fixture.notices.mock.calls.length > 0);
    assert.doesNotMatch(html, /role="alert"/);
    assert.equal(fixture.cache.mock.calls.length, 0);
    assert.equal(fixture.snapshot?.revision, 7);
  }
});

test("Offline Local Edits Remain Available While Backend Reads Fail", async () => {
  fixture.backendError = { kind: "disconnected" };
  const html = renderToStaticMarkup(createElement(ApplicationPreferences));
  assert.ok(html.includes('aria-label="Theme Mode"'));
  assert.equal(fixture.notices.mock.calls.length, 0);
  assert.match(html, /role="alert"/);
  assert.ok(!html.includes('aria-label="Report Points"'));
  usePreferences("local");
  const query = fixture.queries.at(-1);
  assert.ok(query);
  await query.queryFn();
  assert.deepEqual(fixture.read.mock.calls.at(-1), ["local"]);
  useSavePreferences("local");
  const mutation = fixture.mutations.at(-1);
  assert.ok(mutation);
  await mutation.mutationFn({ previous: snapshot(), settings: snapshot().settings });
  assert.equal(fixture.update.mock.calls.at(-1)?.[0], "local");
  assert.deepEqual(preferencesKey("local", "new"), preferencesKey("local", "old"));
  assert.notDeepEqual(preferencesKey("backend", "new"), preferencesKey("backend", "old"));
});

test("Loading Does Not Render Editable Defaults", () => {
  fixture.loading = true;
  const html = renderToStaticMarkup(createElement(ApplicationPreferences, { advanced: true }));
  assert.equal(fixture.notices.mock.calls.length, 0);
  assert.match(html, /role="status"/);
  assert.ok(!html.includes("<form"));
});

test("Autosave Debounces Edits For 100ms And Serializes Pending Revisions", async () => {
  vi.useFakeTimers();
  try {
    const writes: number[] = [];
    let finish: (() => void) | undefined;
    const writer = createDebouncedWriter<number>(
      async (value) => {
        writes.push(value);
        if (value === 2)
          await new Promise<void>((resolve) => {
            finish = resolve;
          });
      },
      () => {},
    );
    writer.change(1);
    await vi.advanceTimersByTimeAsync(50);
    writer.change(2);
    await vi.advanceTimersByTimeAsync(99);
    assert.deepEqual(writes, []);
    await vi.advanceTimersByTimeAsync(1);
    assert.deepEqual(writes, [2]);
    writer.change(3);
    await vi.advanceTimersByTimeAsync(100);
    assert.deepEqual(writes, [2]);
    finish?.();
    await vi.advanceTimersByTimeAsync(0);
    assert.deepEqual(writes, [2, 3]);
  } finally {
    vi.useRealTimers();
  }
});

test("Autosave Blocks After Failure and Can Cancel Invalid Edits or Flush on Navigation", async () => {
  vi.useFakeTimers();
  try {
    const states: string[] = [];
    const failure = new Error("conflict", { cause: new Error("fixture-root") });
    const write = vi.fn().mockRejectedValueOnce(failure).mockResolvedValue(undefined);
    let retained: unknown;
    const writer = createDebouncedWriter(write, (state) => {
      states.push(state.kind);
      if (state.kind === "error") retained = state.failure;
    });
    writer.change(1);
    writer.cancel();
    await vi.advanceTimersByTimeAsync(100);
    assert.equal(write.mock.calls.length, 0);
    writer.change(2);
    writer.flush();
    await vi.advanceTimersByTimeAsync(0);
    assert.equal(write.mock.calls.length, 1);
    assert.equal(states.at(-1), "error");
    assert.equal(retained, failure);
    writer.change(3);
    await vi.advanceTimersByTimeAsync(100);
    assert.equal(write.mock.calls.length, 1);
    writer.reset();
    writer.change(4);
    writer.flush();
    await vi.advanceTimersByTimeAsync(0);
    assert.equal(write.mock.calls.length, 2);
    assert.equal(states.at(-1), "saved");
  } finally {
    vi.useRealTimers();
  }
});

test("Advanced View Persists per Viewer in Desktop and Browser Storage", async () => {
  const storage = new Map<string, string>();
  try {
    for (const desktop of [true, false]) {
      vi.stubGlobal("window", {
        ...(desktop ? { __TAURI_INTERNALS__: {} } : {}),
        localStorage: {
          getItem: (key: string) => storage.get(key) ?? null,
          setItem: (key: string, value: string) => storage.set(key, value),
          removeItem: (key: string) => storage.delete(key),
        },
      });
      const { useUiStore } = await vi.importActual<typeof import("@/stores/ui-store")>("@/stores/ui-store");
      useUiStore.getState().setUiState({ showAdvancedSettings: true });
      const key = desktop ? "dokime-test-hub.ui.sidebar" : "dokime-test-hub.ui";
      const saved = storage.get(key);
      assert.ok(saved);
      assert.equal(JSON.parse(saved).state.showAdvancedSettings, true);
      useUiStore.getState().setUiState({ showAdvancedSettings: false });
      storage.set(key, saved);
      await useUiStore.persist.rehydrate();
      assert.equal(useUiStore.getState().showAdvancedSettings, true);
      useUiStore.getState().resetUiStore();
      assert.equal(useUiStore.getState().showAdvancedSettings, true);
    }
  } finally {
    vi.unstubAllGlobals();
  }
});

test("Preference Choices Keep Their Values and Keyboard Selection", async () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  onTestFinished(() => {
    act(() => root.unmount());
    host.remove();
    vi.unstubAllGlobals();
  });
  await act(async () => root.render(createElement("form", null, createElement(ApplicationPreferences, { advanced: true }))));
  for (const [id, value, next] of [
    ["local-appearance.theme_mode", "dark", "system"],
    ["local-appearance.density", "compact", "comfortable"],
    ["backend-report.plot_downsample", "lttb", "auto"],
  ]) {
    const trigger = document.getElementById(id) as HTMLButtonElement;
    assert.equal(trigger.getAttribute("role"), "combobox");
    const label = trigger.closest("label") ?? host.querySelector(`label[for="${id}"]`);
    assert.ok(label);
    await act(async () => trigger.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true })));
    const selected = document.querySelector<HTMLElement>('[role="option"][aria-selected="true"]');
    assert.ok(selected);
    assert.equal(selected.dataset.state, "checked");
    const native = trigger.parentElement?.querySelector("select");
    assert.ok(native);
    assert.equal(native.value, value);
    const first = document.querySelector<HTMLElement>('[role="option"]');
    assert.ok(first);
    await act(async () => first.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true })));
    assert.equal(native.value, next);
    assert.equal(trigger.getAttribute("aria-expanded"), "false");
  }
});

test("Numeric Preferences Preserve Typing While Earlier Autosaves Complete", async () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  vi.useFakeTimers();
  fixture.cache.mockImplementation((_key, saved) => {
    fixture.snapshot = saved;
  });
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  onTestFinished(async () => {
    await act(async () => root.unmount());
    host.remove();
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });
  const render = () =>
    act(async () =>
      root.render(
        createElement(
          "div",
          null,
          createElement(ApplicationPreferences, { advanced: true }),
          createElement(ApplicationPreferences, { advanced: true, section: "plugins" }),
        ),
      ),
    );
  await render();
  const inputs = [...host.querySelectorAll<HTMLInputElement>('input[type="number"]')];
  assert.ok(inputs.length > 0);
  let revision = 7;
  for (const input of inputs) {
    let finish: ((saved: SettingsSnapshot) => void) | undefined;
    fixture.update.mockImplementationOnce(
      () =>
        new Promise<SettingsSnapshot>((resolve) => {
          finish = resolve;
        }),
    );
    fixture.update.mockImplementation((_scope, request) => ({ revision: ++revision, settings: request.settings }));
    input.focus();
    for (const value of ["", "2", "25", "250", "2500"]) {
      await act(async () => {
        Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set?.call(input, value);
        input.dispatchEvent(new Event("input", { bubbles: true }));
      });
      if (value === "2") await act(async () => vi.advanceTimersByTimeAsync(100));
      if (value === "25") {
        const complete = finish;
        assert.ok(complete, "the earlier draft is saving");
        const request = fixture.update.mock.calls.at(-1)?.[1];
        assert.ok(request);
        const saved = { revision: ++revision, settings: request.settings };
        fixture.snapshot = saved;
        await act(async () => complete(saved));
      }
      await render();
      assert.equal(document.getElementById(input.id), input);
      assert.equal(document.activeElement, input);
      assert.equal(input.value, value, input.id);
    }
    await act(async () => vi.advanceTimersByTimeAsync(100));
    assert.equal(input.value, "2500", input.id);
    const [group, key] = input.id.slice(input.id.indexOf("-") + 1).split(".");
    const request = fixture.update.mock.calls.at(-1)?.[1];
    assert.ok(request);
    const settings = request.settings;
    assert.equal((settings[group] as Record<string, unknown>)[key], 2500);
  }
});

test("Plugin Numeric Settings Preserve Incomplete Edits", async () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  vi.useFakeTimers();
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  onTestFinished(async () => {
    await act(async () => root.unmount());
    host.remove();
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });
  const save = vi.fn(async (): Promise<void> => {});
  let finish: (() => void) | undefined;
  save.mockImplementationOnce(
    () =>
      new Promise<void>((resolve) => {
        finish = resolve;
      }),
  );
  const props: ComponentProps<typeof InstalledPluginSettingsForm> = {
    schema: {
      contributions: [{ kind: "number", id: "budget", title: "Budget", description: "", default: 1000, min: 1000, max: 10000, step: 1 }],
    },
    values: { budget: 1000 },
    discovered: {},
    onSubmit: save,
  };
  await act(async () => root.render(createElement(InstalledPluginSettingsForm, props)));
  const input = host.querySelector<HTMLInputElement>('input[type="number"]');
  assert.ok(input);
  input.focus();
  for (const value of ["", "2", "25", "250", "2500", "250", "", "2", "25", "250", "2500"]) {
    await act(async () => {
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set?.call(input, value);
      input.dispatchEvent(new Event("input", { bubbles: true }));
    });
    assert.equal(input.value, value);
    assert.equal(document.activeElement, input);
    if (value === "250") await act(async () => vi.advanceTimersByTimeAsync(100));
    assert.equal(save.mock.calls.length, 0);
  }
  await act(async () => vi.advanceTimersByTimeAsync(100));
  assert.deepEqual(save.mock.calls, [[{ budget: 2500 }]]);
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set?.call(input, "3000");
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
  await act(async () => root.render(createElement(InstalledPluginSettingsForm, { ...props, values: { budget: 2500 } })));
  const complete = finish;
  assert.ok(complete);
  await act(async () => complete());
  assert.equal(input.value, "3000", "an earlier save cannot replace the newer edit");
  await act(async () => vi.advanceTimersByTimeAsync(100));
  assert.deepEqual(save.mock.calls, [[{ budget: 2500 }], [{ budget: 3000 }]]);
});
