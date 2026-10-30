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
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, test, vi } from "vitest";
import { ApplicationPreferences } from "@/features/settings/ApplicationPreferences";
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
    },
  };
}

/* === Entry Point === */

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
    return { mutateAsync: vi.fn(), reset: vi.fn(), mutate: vi.fn(), isPending: false, error: fixture.saveError, isSuccess: false };
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
  assert.equal((advanced.match(/aria-label="[^"<>]* \[ms\]"/g) ?? []).length, 3);
  assert.equal((advanced.match(/aria-label=/g) ?? []).length, 13);
  for (const value of ["fixture", "3210", "1234", "5678", "42", "127.0.0.1:9876", "fixture.local, 127.0.0.1"])
    assert.ok(advanced.includes(`value="${value}"`), value);
  for (const value of ["dark", "compact", "lttb"]) assert.ok(advanced.includes(`value="${value}" selected=""`), value);
});

test("Scoped Saves Send All Fields And The Reviewed Revision", async () => {
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
  assert.ok(fixture.notices.mock.calls.length > 0);
  assert.doesNotMatch(html, /role="alert"/);
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

test("Autosave Blocks After Failure And Can Cancel Invalid Edits Or Flush On Navigation", async () => {
  vi.useFakeTimers();
  try {
    const states: string[] = [];
    const write = vi.fn().mockRejectedValueOnce(new Error("conflict")).mockResolvedValue(undefined);
    const writer = createDebouncedWriter(write, (state) => states.push(state.kind));
    writer.change(1);
    writer.cancel();
    await vi.advanceTimersByTimeAsync(100);
    assert.equal(write.mock.calls.length, 0);
    writer.change(2);
    writer.flush();
    await vi.advanceTimersByTimeAsync(0);
    assert.equal(write.mock.calls.length, 1);
    assert.equal(states.at(-1), "error");
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

test("Advanced View Persists Per Viewer In Desktop And Browser Storage", async () => {
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
