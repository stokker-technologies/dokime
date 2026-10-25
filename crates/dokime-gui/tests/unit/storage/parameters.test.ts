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
  parameterOverride,
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
  return { values, read: vi.fn(async () => null), write: vi.fn() };
});

/* === Entry Point === */

vi.mock("@/lib/desktop-open", () => ({ isTauriDesktop: () => true }));
vi.mock("@/lib/desktop/commands", () => ({
  commands: { desktopParametersRead: fixture.read, desktopParametersWrite: fixture.write },
}));

/* === Tests === */

test("Typed Remembered Parameters", () => {
  const values = { template: { voltage: { raw: "115", valueType: "int", unit: "V" } } };
  assert.deepEqual(readPersistedParameterOverrides({ overrides: values }), values);
  assert.equal(parameterOverride(values, "template", "voltage"), "115");
  assert.deepEqual(readPersistedParameterOverrides(undefined), {});
  const unsupported = { overrides: { template: { voltage: "115" } } };
  assert.throws(() => readPersistedParameterOverrides(unsupported));
  assert.deepEqual(unsupported, { overrides: { template: { voltage: "115" } } });
});

test("Unsupported Browser Storage", async () => {
  const saved = JSON.stringify({ version: 1, state: { overrides: { template: { voltage: "115" } } } });
  fixture.values.set(PARAMETER_OVERRIDES_STORAGE_KEY, saved);
  fixture.write.mockClear();
  await useParameterOverridesStore.persist.rehydrate();
  assert.equal(useParameterOverridesStatus.getState().ready, false);
  assert.match(useParameterOverridesStatus.getState().error ?? "", /Unsupported browser-owned remembered parameters/);
  assert.equal(fixture.values.get(PARAMETER_OVERRIDES_STORAGE_KEY), saved);
  assert.equal(fixture.write.mock.calls.length, 0);
});
