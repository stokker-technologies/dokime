// @vitest-environment happy-dom
// Dokime — SPDX-License-Identifier: Apache-2.0

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import { test, vi } from "vitest";

// Local Imports
import { CycleReportMenuItem } from "@/features/cycle/CycleReportMenuItem";
import { DropdownMenu, DropdownMenuContent } from "@dokime/plots/components/ui/dropdown-menu";

/* === Constants === */

const download = vi.hoisted(() => ({ isPending: false, mutateAsync: vi.fn().mockResolvedValue(null) }));

/* === Entry Point === */

vi.mock("@/features/cycle/cycle-queries", () => ({ useCycleCommands: () => ({ downloadReport: download }) }));

test("Download Report Enables When the Seal Is Observed Without Remounting", async () => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  const element = document.createElement("div");
  document.body.append(element);
  const root = createRoot(element);
  const render = (sealed: boolean) =>
    createElement(DropdownMenu, { open: true },
      createElement(DropdownMenuContent, {},
        createElement(CycleReportMenuItem, { cycleId: "cycle", executionId: "execution", label: "Execution", sealed }),
      ),
    );
  const item = () => {
    const found = document.querySelector<HTMLElement>('[role="menuitem"]');
    assert.ok(found);
    return found;
  };
  try {
    await act(async () => root.render(render(false)));
    assert.equal(item().getAttribute("aria-disabled"), "true");
    assert.match(item().title, /once the run is sealed/);
    await act(async () => item().click());
    assert.equal(download.mutateAsync.mock.calls.length, 0);

    await act(async () => root.render(render(true)));
    assert.notEqual(item().getAttribute("aria-disabled"), "true");
    await act(async () => item().click());
    assert.deepEqual(download.mutateAsync.mock.calls, [[{ executionId: "execution", label: "Execution" }]]);
  } finally {
    await act(async () => root.unmount());
    element.remove();
    vi.unstubAllGlobals();
  }
});
