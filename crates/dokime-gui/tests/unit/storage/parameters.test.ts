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
import { test, vi } from "vitest";

// Local Imports
import {
  PARAMETER_OVERRIDES_STORAGE_KEY,
  readPersistedParameterOverrides,
  useParameterOverridesStatus,
  useParameterOverridesStore,
} from "@/stores/parameter-overrides-store";

/* === Constants === */

const fixture = vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("window", {
    localStorage: {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
      removeItem: (key: string) => values.delete(key),
    },
  });
  return { values, read: vi.fn(async () => ({ interval_window_ms: 100, interval_lag_windows: 10 })), write: vi.fn() };
});

/* === Entry Point === */

vi.mock("@/lib/surface-environment", () => ({ isTauriDesktop: () => true }));
vi.mock("@/lib/desktop/commands", () => ({
  commands: { desktopSavedSettingsRead: fixture.read, desktopSavedSettingsUpdate: fixture.write },
}));

/* === Tests === */

test("Typed Remembered Parameters", () => {
  const values = { template: { voltage: { raw: "115", valueType: "int", unit: "V" } } };
  assert.deepEqual(readPersistedParameterOverrides({ overrides: values }), values);
  assert.deepEqual(readPersistedParameterOverrides(undefined), {});
  const unsupported = { overrides: { template: { voltage: "115" } } };
  assert.throws(() => readPersistedParameterOverrides(unsupported));
  assert.deepEqual(unsupported, { overrides: { template: { voltage: "115" } } });
});

test("Unified Desktop Settings Supply Remembered Parameters", async () => {
  const parameters = { template: { voltage: { raw: "115", valueType: "int", unit: "V" } } };
  const saved = { interval_window_ms: 100, interval_lag_windows: 10, parameters };
  fixture.read.mockResolvedValue(saved);
  fixture.values.set(PARAMETER_OVERRIDES_STORAGE_KEY, "retired-browser-value");
  fixture.write.mockClear();
  await useParameterOverridesStore.persist.rehydrate();
  assert.equal(useParameterOverridesStatus.getState().ready, true);
  assert.deepEqual(useParameterOverridesStore.getState().overrides, parameters);
  assert.equal(fixture.values.get(PARAMETER_OVERRIDES_STORAGE_KEY), "retired-browser-value");
  assert.equal(fixture.write.mock.calls.length, 0);
});
