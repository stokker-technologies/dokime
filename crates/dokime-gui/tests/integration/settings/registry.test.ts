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
import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import { onTestFinished, test, vi } from "vitest";

// Local Imports
import { Controller } from "@/app/native-session/context";
import { NativeSession } from "@/app/native-session/session";
import { BackendsTable } from "@/features/settings/BackendsTable";
import { preferencesCommands } from "@/lib/desktop/settings";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Backend Action Menus Survive Owner Renders", async () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  vi.spyOn(preferencesCommands, "desktopBackendsList").mockResolvedValue({
    version: 1,
    default: null,
    profiles: [
      {
        identifier: "lab",
        label: "Lab",
        endpoint: "http://127.0.0.1:8765",
        endpoints: [],
        secret: "none",
        expected_backend_id: null,
        ca_certificate_pem: null,
      },
    ],
  });
  const owner = new NativeSession();
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  onTestFinished(async () => {
    await act(async () => root.unmount());
    host.remove();
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });
  const render = async () => {
    await act(async () => root.render(createElement(Controller.Provider, { value: owner }, createElement(BackendsTable))));
  };
  await render();
  const trigger = host.querySelector<HTMLButtonElement>("[aria-label='Actions For lab']");
  assert.ok(trigger);
  await act(async () => trigger.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true, button: 0 })));
  assert.ok(document.querySelector("[role=menu]"));
  await render();
  assert.ok(document.querySelector("[role=menu]"));
  const action = document.querySelector<HTMLElement>("[role=menuitem]");
  assert.ok(action);
  await act(async () => action.click());
  assert.ok(document.querySelector("[role=dialog]"));
});
