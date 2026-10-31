/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Host Firewall Notice And Explicit Action
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, test, vi } from "vitest";

// Local Imports
import { FirewallNotice } from "@/features/settings/sections/FirewallNotice";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  result: { required: true, message: null as string | null },
  invoke: vi.fn(),
  refetch: vi.fn(),
  click: null as (() => void) | null,
  inspect: null as (() => Promise<unknown>) | null,
  messages: [] as unknown[],
  label: "Fixture Lab",
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => fixture.invoke(...args) }));
vi.mock("@/lib/brand", () => ({ useBrand: () => ({ label: fixture.label }) }));
vi.mock("@tanstack/react-query", () => ({
  useQuery: ({ queryFn }: { queryFn: () => Promise<unknown> }) => {
    fixture.inspect = queryFn;
    return { data: fixture.result, refetch: fixture.refetch, error: null };
  },
}));
vi.mock("react", async (original) => ({
  ...(await original<typeof import("react")>()),
  useState: (initial: unknown) => [initial, (value: unknown) => fixture.messages.push(value)],
}));
vi.mock("@/components/ui/button", () => ({
  Button: ({ onClick }: { onClick: () => void }) => {
    fixture.click = onClick;
    return createElement("button", { onClick, type: "button" });
  },
}));

/* === Types === */

/* === Classes === */

/* === Public Functions === */

beforeEach(() => {
  vi.clearAllMocks();
  fixture.result = { required: true, message: null };
  fixture.click = null;
  fixture.inspect = null;
  fixture.messages = [];
});

test("Blocked Host Shows Its Brand And Only Elevates After The Operator Acts", async () => {
  const html = renderToStaticMarkup(
    createElement(FirewallNotice, { generation: "host-7", enabled: true, listenerAddress: "0.0.0.0:8765" }),
  );
  assert.ok(html.includes(fixture.label));
  assert.match(html, /role="status"/);
  assert.equal(fixture.invoke.mock.calls.length, 0);
  assert.ok(fixture.inspect);
  fixture.invoke.mockResolvedValueOnce(fixture.result);
  await fixture.inspect();
  assert.deepEqual(fixture.invoke.mock.calls, [["desktop_firewall", { generation: "host-7", allow: false }]]);
  assert.ok(fixture.click);
  fixture.invoke.mockResolvedValueOnce({ required: false, message: null });
  fixture.click();
  await vi.waitFor(() => assert.equal(fixture.refetch.mock.calls.length, 1));
  assert.deepEqual(fixture.invoke.mock.calls[1], ["desktop_firewall", { generation: "host-7", allow: true }]);
});

test("Allowed Or Inactive Host Does Not Offer An Action", () => {
  fixture.result.required = false;
  const html = renderToStaticMarkup(createElement(FirewallNotice, { generation: "host-8", enabled: false }));
  assert.equal(html, "");
  assert.equal(fixture.click, null);
  assert.equal(fixture.invoke.mock.calls.length, 0);
});

test("Cancelled Elevation Leaves Nonfatal Feedback Without Reporting Success", async () => {
  renderToStaticMarkup(createElement(FirewallNotice, { generation: "host-9", enabled: true, listenerAddress: "[::]:8765" }));
  fixture.invoke.mockRejectedValueOnce(new Error("cancelled"));
  assert.ok(fixture.click);
  fixture.click();
  await vi.waitFor(() => assert.ok(fixture.messages.some((value) => typeof value === "string" && value.length > 0)));
  assert.equal(fixture.refetch.mock.calls.length, 0);
});

test("Skipped Cleanup Is Visible Without Offering Elevation", () => {
  fixture.result = { required: false, message: "fixture-cleanup-skipped" };
  const html = renderToStaticMarkup(createElement(FirewallNotice, { generation: "host-10", enabled: false }));
  assert.match(html, /role="status"/);
  assert.equal(fixture.click, null);
  assert.equal(fixture.invoke.mock.calls.length, 0);
});

/* === Private Functions === */

/* === Entry Point === */
