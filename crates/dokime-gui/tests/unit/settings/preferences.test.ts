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
import type { PreferencesSnapshot } from "@generated/typescript/application-settings";
import defaults from "@generated/typescript/application-settings.json";
// Third Party Imports
import { QueryClient } from "@tanstack/svelte-query";
import { fireEvent, render, within } from "@testing-library/svelte";
import { type ComponentProps, tick } from "svelte";
import { beforeEach, onTestFinished, test, vi } from "vitest";
import ApplicationPreferences from "@/features/settings/ApplicationPreferences.svelte";
import InstalledPluginSettingsForm from "@/features/settings/installed-plugins/InstalledPluginSettingsForm.svelte";
import { preferencesKey, preferencesMutation, preferencesQuery } from "@/features/settings/application-preferences";
import { createDebouncedWriter } from "@/features/settings/settings-autosave";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  read: vi.fn(),
  update: vi.fn(),
  appearance: vi.fn(),
}));

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function newQueryClient() {
  const client = new QueryClient({ defaultOptions: { queries: { gcTime: Infinity, retry: false } } });
  onTestFinished(() => client.clear());
  return client;
}

function showPreferences(
  props: Omit<Parameters<typeof render<typeof ApplicationPreferences>>[1], "queryClient"> = {},
  queryClient = newQueryClient(),
) {
  const view = render(ApplicationPreferences, { ...props, queryClient });
  onTestFinished(() => {
    if (view.container.isConnected) {
      view.unmount();
    }
  });
  return view;
}

function snapshot(): PreferencesSnapshot {
  return {
    revision: [7],
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
    },
  };
}

/* === Entry Point === */

vi.mock("@/lib/notifications", async (original) => ({
  ...(await original<typeof import("@/lib/notifications")>()),
  toast: { success: vi.fn(), error: vi.fn() },
}));

vi.mock("@/lib/desktop/settings", () => ({
  preferencesCommands: {
    desktopSettingsRead: fixture.read,
    desktopSettingsUpdate: fixture.update,
  },
}));
vi.mock("@/stores/ui-store", () => ({ uiStore: { getState: () => ({ setUiState: fixture.appearance }) } }));
beforeEach(() => {
  vi.resetAllMocks();
  fixture.read.mockImplementation(async () => snapshot());
  fixture.update.mockImplementation(async (_scope, request) => ({ revision: [8], settings: request.settings }));
});

test("Advanced Reveals Additional Preference Fields", async () => {
  const simpleView = showPreferences();
  const simpleRemote = showPreferences({ section: "remote" });
  await simpleView.findByRole("combobox", { name: "Theme Mode" });
  await within(simpleRemote.container).findByRole("switch", { name: "Listener Enabled" });
  const simple = simpleView.container.innerHTML + simpleRemote.container.innerHTML;
  for (const view of [simpleView, simpleRemote]) {
    view.unmount();
    view.container.remove();
  }
  const advancedView = showPreferences({ advanced: true });
  const advancedRemote = showPreferences({ advanced: true, section: "remote" });
  await advancedView.findByRole("combobox", { name: "Theme Mode" });
  await within(advancedRemote.container).findByRole("switch", { name: "Listener Enabled" });
  const advanced = advancedView.container.innerHTML + advancedRemote.container.innerHTML;
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
  for (const id of visible) {
    assert.ok(simple.includes(`id="${id}"`), id);
  }
  for (const id of additional) {
    assert.ok(!simple.includes(`id="${id}"`), id);
    assert.ok(advanced.includes(`id="${id}"`), id);
  }
  for (const value of ["fixture", "3210", "1234", "5678", "42", "127.0.0.1:9876", "fixture.local, 127.0.0.1"]) {
    const inputs = [...advancedView.container.querySelectorAll("input"), ...advancedRemote.container.querySelectorAll("input")];
    assert.ok(
      inputs.some((input) => input.value === value),
      value,
    );
  }
});

test("Scoped Saves Send All Fields and the Reviewed Revision", async () => {
  for (const scope of ["local", "backend"] as const) {
    const client = newQueryClient();
    const mutation = preferencesMutation(scope, null, client);
    const previous = snapshot();
    const settings = structuredClone(previous.settings);
    if (scope === "local") {
      settings.appearance = { theme_mode: "light", theme_family: "other", density: "touch" };
      settings.client = { connect_timeout_ms: 2000, request_timeout_ms: 9000 };
    } else {
      settings.report = { ...settings.report, plot_points: 4000, plot_downsample: "raw" };
      settings.telemetry.interval_ms = 200;
      settings.listener = {
        enabled: false,
        address: "0.0.0.0:8766",
        tls_enabled: false,
        authentication_enabled: true,
        certificate_names: ["other.local"],
      };
    }
    const saved = { revision: [8], settings };
    fixture.update.mockResolvedValueOnce(saved);
    assert.deepEqual(await mutation.mutationFn({ previous, settings }), saved);
    assert.deepEqual(fixture.update.mock.calls.at(-1), [scope, { expected_revision: [7], settings }]);
    mutation.onSuccess(saved);
    assert.deepEqual(client.getQueryData(preferencesKey(scope)), saved);
    assert.deepEqual(previous, snapshot());
  }
  assert.equal(fixture.appearance.mock.calls.length, 1);
  assert.deepEqual(fixture.appearance.mock.calls[0], [{ density: "touch", themeFamily: "other" }]);
});

test("Offline Local Edits Remain Available While Backend Reads Fail", async () => {
  fixture.read.mockImplementation(async (scope) => {
    if (scope === "backend") {
      throw { kind: "disconnected" };
    }
    return snapshot();
  });
  const client = newQueryClient();
  const view = showPreferences({}, client);
  await view.findByRole("combobox", { name: "Theme Mode" });
  assert.ok(await view.findByRole("alert"));
  assert.equal(view.queryByLabelText("Report Points [points]"), null);
  await preferencesQuery("local", null).queryFn();
  assert.deepEqual(fixture.read.mock.calls.at(-1), ["local"]);
  await preferencesMutation("local", null, client).mutationFn({ previous: snapshot(), settings: snapshot().settings });
  assert.equal(fixture.update.mock.calls.at(-1)?.[0], "local");
  assert.deepEqual(preferencesKey("local", "new"), preferencesKey("local", "old"));
  assert.notDeepEqual(preferencesKey("backend", "new"), preferencesKey("backend", "old"));
});

test("Loading Does Not Render Editable Defaults", () => {
  fixture.read.mockImplementation(() => new Promise(() => {}));
  const view = showPreferences({ advanced: true });
  assert.ok(within(view.container).getAllByRole("status").length);
  assert.equal(view.container.querySelector("form"), null);
  assert.equal(view.queryByRole("combobox"), null);
});

test("Autosave Debounces Edits For 100ms And Serializes Pending Revisions", async () => {
  vi.useFakeTimers();
  try {
    const writes: number[] = [];
    let finish: (() => void) | undefined;
    const writer = createDebouncedWriter<number>(
      async (value) => {
        writes.push(value);
        if (value === 2) {
          await new Promise<void>((resolve) => {
            finish = resolve;
          });
        }
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
      if (state.kind === "error") {
        retained = state.failure;
      }
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
      const { uiStore } = await vi.importActual<typeof import("@/stores/ui-store")>("@/stores/ui-store");
      uiStore.getState().setUiState({ showAdvancedSettings: true });
      const key = desktop ? "dokime-test-hub.ui.sidebar" : "dokime-test-hub.ui";
      const saved = storage.get(key);
      assert.ok(saved);
      assert.equal(JSON.parse(saved).state.showAdvancedSettings, true);
      uiStore.getState().setUiState({ showAdvancedSettings: false });
      storage.set(key, saved);
      await uiStore.persist.rehydrate();
      assert.equal(uiStore.getState().showAdvancedSettings, true);
      uiStore.getState().resetUiStore();
      assert.equal(uiStore.getState().showAdvancedSettings, true);
    }
  } finally {
    vi.unstubAllGlobals();
  }
});

test("Preference Choices Keep Their Values and Keyboard Selection", async () => {
  const view = showPreferences({ advanced: true });
  await view.findByRole("combobox", { name: "Theme Mode" });
  for (const [id, value, next] of [
    ["local-appearance.theme_mode", "dark", "system"],
    ["local-appearance.density", "compact", "comfortable"],
    ["backend-report.plot_downsample", "lttb", "auto"],
  ]) {
    const trigger = document.getElementById(id);
    assert.ok(trigger instanceof HTMLButtonElement);
    assert.equal(trigger.getAttribute("role"), "combobox");
    const label = trigger.closest("label") ?? view.container.querySelector(`label[for="${id}"]`);
    assert.ok(label);
    trigger.focus();
    await fireEvent.keyDown(trigger, { key: "ArrowDown" });
    const listbox = await view.findByRole("listbox");
    assert.equal(trigger.getAttribute("aria-controls"), listbox.id);
    const choices = within(listbox);
    const selected = choices.getByRole("option", { selected: true });
    assert.equal(selected.getAttribute("aria-selected"), "true");
    const name = id.slice(id.indexOf("-") + 1);
    const native = view.container.querySelector<HTMLInputElement>(`input[name="${name}"]`);
    assert.ok(native);
    assert.equal(native.value, value);
    const [first] = choices.getAllByRole("option");
    await fireEvent.keyDown(trigger, { key: "Home" });
    assert.equal(trigger.getAttribute("aria-activedescendant"), first.id);
    await fireEvent.keyDown(trigger, { key: "Enter" });
    await tick();
    assert.equal(native.value, next);
    assert.equal(trigger.getAttribute("aria-expanded"), "false");
  }
});

test("Numeric Preferences Preserve Typing While Earlier Autosaves Complete", async () => {
  const view = showPreferences({ advanced: true });
  await view.findByRole("combobox", { name: "Theme Mode" });
  vi.useFakeTimers();
  onTestFinished(() => {
    vi.useRealTimers();
  });
  const inputs = [...view.container.querySelectorAll<HTMLInputElement>('input[type="number"]')];
  assert.ok(inputs.length > 0);
  let revision = 7;
  for (const input of inputs) {
    let finish: ((saved: PreferencesSnapshot) => void) | undefined;
    fixture.update.mockImplementationOnce(
      () =>
        new Promise<PreferencesSnapshot>((resolve) => {
          finish = resolve;
        }),
    );
    fixture.update.mockImplementation((_scope, request) => ({ revision: [++revision], settings: request.settings }));
    input.focus();
    for (const value of ["", "2", "25", "250", "2500"]) {
      await fireEvent.input(input, { target: { value } });
      if (value === "2") {
        await vi.advanceTimersByTimeAsync(100);
      }
      if (value === "25") {
        const complete = finish;
        assert.ok(complete, "the earlier draft is saving");
        const request = fixture.update.mock.calls.at(-1)?.[1];
        assert.ok(request);
        complete({ revision: [++revision], settings: request.settings });
        await vi.advanceTimersByTimeAsync(0);
      }
      await tick();
      assert.equal(document.getElementById(input.id), input);
      assert.equal(document.activeElement, input);
      assert.equal(input.value, value, input.id);
    }
    await vi.advanceTimersByTimeAsync(100);
    assert.equal(input.value, "2500", input.id);
    const [group, key] = input.id.slice(input.id.indexOf("-") + 1).split(".");
    const request = fixture.update.mock.calls.at(-1)?.[1];
    assert.ok(request);
    const settings = request.settings;
    assert.equal((settings[group] as Record<string, unknown>)[key], 2500);
  }
});

test("Plugin Numeric Settings Preserve Incomplete Edits", async () => {
  vi.useFakeTimers();
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
      contributions: [
        {
          kind: "setting",
          declaration: {
            name: "budget",
            title: "Budget",
            description: "",
            default: { value: { case: "number", value: 1000 } },
            minimum: { value: { case: "number", value: 1000 } },
            maximum: { value: { case: "number", value: 10000 } },
            choices: [],
            required: false,
          },
        },
      ],
    },
    values: { budget: 1000 },
    onSubmit: save,
  };
  const view = render(InstalledPluginSettingsForm, props);
  onTestFinished(() => {
    view.unmount();
    vi.useRealTimers();
  });
  const input = view.container.querySelector<HTMLInputElement>('input[type="number"]');
  assert.ok(input);
  input.focus();
  for (const value of ["", "2", "25", "250", "2500", "250", "", "2", "25", "250", "2500"]) {
    await fireEvent.input(input, { target: { value } });
    assert.equal(input.value, value);
    assert.equal(document.activeElement, input);
    if (value === "250") {
      await vi.advanceTimersByTimeAsync(100);
    }
    assert.equal(save.mock.calls.length, 0);
  }
  await vi.advanceTimersByTimeAsync(100);
  assert.deepEqual(save.mock.calls, [[{ budget: 2500 }]]);
  await fireEvent.input(input, { target: { value: "3000" } });
  await view.rerender({ values: { budget: 2500 } });
  const complete = finish;
  assert.ok(complete);
  complete();
  await vi.advanceTimersByTimeAsync(0);
  assert.equal(input.value, "3000", "an earlier save cannot replace the newer edit");
  await vi.advanceTimersByTimeAsync(100);
  assert.deepEqual(save.mock.calls, [[{ budget: 2500 }], [{ budget: 3000 }]]);
});

test("Shared Preference Failure Occupies Its Content Slot", async () => {
  const error = new Error("fixture-write");
  fixture.update.mockRejectedValueOnce(error);
  const client = newQueryClient();
  const view = showPreferences({ advanced: true, localOnly: true }, client);
  const input = await view.findByRole("textbox", { name: "Theme Family" });
  vi.useFakeTimers();
  try {
    await fireEvent.input(input, { target: { value: "other" } });
    await vi.advanceTimersByTimeAsync(100);
    await tick();
    assert.ok(view.getAllByRole("alert").some((alert) => alert.textContent?.includes(error.message)));
    assert.deepEqual(client.getQueryData<PreferencesSnapshot>(preferencesKey("local"))?.revision, snapshot().revision);
  } finally {
    vi.useRealTimers();
  }
});

test("Failed Saves Submit the Reviewed Revision and Changed Settings", async () => {
  const previous = snapshot();
  const refusal = { kind: "persistence", message: "fixture-write-refused" };
  fixture.update.mockRejectedValueOnce(refusal);
  const mutation = preferencesMutation("local", null, newQueryClient());
  const settings = { ...previous.settings, client: { ...previous.settings.client, request_timeout_ms: 6789 } };
  await mutation.mutationFn({ previous, settings }).catch(() => undefined);
  assert.deepEqual(fixture.update.mock.calls[0], ["local", { expected_revision: previous.revision, settings }]);
});
