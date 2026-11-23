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
 * Backend Registry Interaction Lifecycle
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { fireEvent, render } from "@testing-library/svelte";
import { onTestFinished, test, vi } from "vitest";

// Local Imports
import { NativeSession } from "@/app/native-session/session";
import BackendsTable from "@/features/settings/BackendsTable.svelte";
import { preferencesCommands } from "@/lib/desktop/settings";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Backend Action Menus Survive Owner Renders", async () => {
  vi.spyOn(preferencesCommands, "desktopBackendsList").mockResolvedValue({
    profiles: [
      {
        identifier: "lab",
        label: "Lab",
        endpoint: "http://127.0.0.1:8765",
        endpoints: [],
        pairing_file: "/fixture/lab.pair",
        expected_backend_id: null,
        certificate_pem: null,
      },
    ],
  });
  const owner = new NativeSession();
  const table = render(BackendsTable, { owner });
  onTestFinished(() => {
    table.unmount();
    vi.restoreAllMocks();
  });
  const trigger = await table.findByRole("button", { name: "Actions for lab" });
  await fireEvent.pointerDown(trigger, { button: 0 });
  assert.ok(await table.findByRole("menu"));
  await table.rerender({ owner, onConnect: vi.fn() });
  assert.ok(table.getByRole("menu"));
  await fireEvent.click(table.getByRole("menuitem"));
  assert.ok(await table.findByRole("dialog"));
});
