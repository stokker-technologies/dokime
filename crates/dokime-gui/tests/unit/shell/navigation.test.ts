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
 * Responsive Navigation Behavior.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { fireEvent, render, within } from "@testing-library/svelte";
import { IDBFactory } from "fake-indexeddb";
import { expect, onTestFinished, test, vi } from "vitest";

// Local Imports
import { createDokimeQueryClient } from "@/app/query-client";
import { cyclesQueryOptions, pendingPromptsQueryOptions } from "@/features/cycles-home/cycles-query-options";
import { defaultBrand } from "@/lib/brand";
import { navigationMode, settingsParent, sidebarWidth } from "@/shell/navigation-model";
import Sidebar from "@/shell/Sidebar.svelte";

/* === Re-Exports === */

/* === Constants === */
/* === Types === */

/* === Classes === */

/* === Public Functions === */
test("breakpoint transitions", () => {
  expect([599, 600, 1023, 1024, 600, 599].map(navigationMode)).toEqual(["phone", "tablet", "tablet", "desktop", "tablet", "phone"]);
});
test("resize bounds", () => {
  expect([0, 199, 200, 253, 360, 500, Number.NaN].map(sidebarWidth)).toEqual([200, 200, 200, 252, 360, 360, 256]);
});
test("settings back", () => {
  expect(settingsParent("/settings/appearance")).toBe("/settings");
  expect(settingsParent("/settings/plugins/installed")).toBe("/settings/plugins");
});

test("Destination Switching", async () => {
  vi.stubGlobal("indexedDB", new IDBFactory());
  onTestFinished(() => {
    vi.unstubAllGlobals();
  });
  const identity = { generation: "navigation", isCurrent: () => true };
  const queryClient = createDokimeQueryClient();
  queryClient.setQueryData(cyclesQueryOptions(identity).queryKey, []);
  queryClient.setQueryData(pendingPromptsQueryOptions(identity).queryKey, []);
  const navigate = vi.fn();
  function props(width: number) {
    return {
      brand: defaultBrand,
      session: { identity, queryClient },
      sidebar: { mode: navigationMode(width), collapsed: width < 1024, toggle: vi.fn(), resize: vi.fn(), width: "256px" },
      pathname: "/cycles",
      cycleId: null,
      selection: {},
      selectCycleMember: vi.fn(),
      addCases: vi.fn(),
      navigate,
      openCycle: vi.fn(),
      openSettings: vi.fn(),
    };
  }

  const { container, getByRole, rerender, unmount } = render(Sidebar, props(599));
  onTestFinished(unmount);
  const tabs = within(getByRole("navigation")).getAllByRole("button");
  expect(tabs).toHaveLength(5);
  for (const tab of tabs) {
    await fireEvent.click(tab);
  }
  expect(navigate.mock.calls.map(([destination]) => destination)).toEqual([
    "/cycles",
    "/library",
    "/history",
    "/notifications",
    "/settings",
  ]);
  for (const width of [600, 1023, 1024]) {
    await rerender(props(width));
    expect(getByRole("navigation").querySelector('[data-slot="sidebar-brand"]')).not.toBeNull();
  }
  await rerender(props(599));
  expect(container.firstElementChild?.tagName).toBe("NAV");
});

/* === Private Functions === */

/* === Entry Point === */
