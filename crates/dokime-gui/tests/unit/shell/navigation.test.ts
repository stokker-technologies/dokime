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

import { Children, isValidElement, type ReactElement } from "react";
import { expect, test, vi } from "vitest";
import { navigationMode, settingsParent, sidebarWidth } from "@/shell/navigation-model";
import { Sidebar } from "@/shell/Sidebar";
import { SidebarFrame } from "@/shell/SidebarFrame";

const fixture = vi.hoisted(() => ({ width: 500, pathname: "/cycles", navigate: vi.fn() }));
vi.mock("react", async (original) => ({
  ...(await original<typeof import("react")>()),
  useState: (initial: unknown) => [initial, vi.fn()],
}));
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => fixture.navigate,
  useRouterState: ({ select }: { select: (state: unknown) => unknown }) => select({ location: { pathname: fixture.pathname, search: {} } }),
}));
vi.mock("@/shell/use-sidebar-state", async () => {
  const { navigationMode } = await import("@/shell/navigation-model");
  return {
    useSidebarState: () => ({ mode: navigationMode(fixture.width), collapsed: fixture.width < 1024, toggle: vi.fn(), width: "256px" }),
  };
});

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

test("destination switching", () => {
  fixture.width = 599;
  const phone = Sidebar({}) as ReactElement<{ children: unknown }>;
  const tabs = Children.toArray(phone.props.children as import("react").ReactNode).filter(isValidElement) as ReactElement<{
    onClick: () => void;
  }>[];
  expect(tabs).toHaveLength(4);
  for (const tab of tabs) tab.props.onClick();
  expect(fixture.navigate.mock.calls.map(([target]) => target.to)).toEqual(["/cycles", "/library", "/history", "/settings"]);
  for (const width of [600, 1023, 1024]) {
    fixture.width = width;
    const shell = Sidebar({}) as ReactElement<{ children: import("react").ReactNode }>;
    const frames = Children.toArray(shell.props.children).filter(isValidElement);
    expect(frames.some((node) => node.type === SidebarFrame)).toBe(true);
  }
  fixture.width = 599;
  expect((Sidebar({}) as ReactElement).type).toBe("nav");
});
