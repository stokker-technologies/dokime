// @vitest-environment happy-dom
// Dokime — SPDX-License-Identifier: Apache-2.0
import assert from "node:assert/strict";
import { QueryClient } from "@tanstack/svelte-query";
import { act, fireEvent, render } from "@testing-library/svelte";
import { beforeEach, onTestFinished, test, vi } from "vitest";
import FirewallNotice from "@/features/settings/sections/FirewallNotice.svelte";

const fixture = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: fixture.invoke }));
beforeEach(() => fixture.invoke.mockReset());

function showNotice(generation: string, enabled: boolean, listenerAddress?: string) {
  const queryClient = new QueryClient();
  const view = render(FirewallNotice, { generation, enabled, listenerAddress, label: "Fixture Lab", queryClient });
  onTestFinished(() => {
    view.unmount();
    queryClient.clear();
  });
  return view;
}

test("Blocked Host Shows Its Brand and Only Elevates After the Operator Acts", async () => {
  fixture.invoke.mockResolvedValueOnce({ required: true, message: null });
  const view = showNotice("host-7", true, "0.0.0.0:8765");
  const action = await view.findByRole("button", { name: "Allow In Firewall…" });
  assert.ok(view.getByRole("status").textContent?.includes("Fixture Lab"));
  assert.deepEqual(fixture.invoke.mock.calls, [["desktop_firewall", { generation: "host-7", allow: false }]]);
  fixture.invoke.mockResolvedValue({ required: false, message: null });
  await fireEvent.click(action);
  await view.findByText(/allows remote access/);
  assert.deepEqual(fixture.invoke.mock.calls.slice(1), [
    ["desktop_firewall", { generation: "host-7", allow: true }],
    ["desktop_firewall", { generation: "host-7", allow: false }],
  ]);
});

test("Allowed or Inactive Host Does Not Offer an Action", async () => {
  fixture.invoke.mockResolvedValueOnce({ required: false, message: null });
  const view = showNotice("host-8", false);
  await act();
  assert.equal(view.queryByRole("button"), null);
  assert.deepEqual(fixture.invoke.mock.calls, [["desktop_firewall", { generation: "host-8", allow: false }]]);
});

test("Served Network Details", async () => {
  const message = "Cable Ethernet (Unidentified network, Public network)";
  fixture.invoke.mockResolvedValueOnce({ required: true, message });
  const view = showNotice("host-public", true, "0.0.0.0:8765");
  assert.ok(await view.findByText(message));
  assert.ok(view.getByRole("button"));
  assert.deepEqual(fixture.invoke.mock.calls, [["desktop_firewall", { generation: "host-public", allow: false }]]);
});

test("Cancelled Elevation Leaves Nonfatal Feedback Without Reporting Success", async () => {
  fixture.invoke.mockResolvedValueOnce({ required: true, message: null });
  const view = showNotice("host-9", true, "[::]:8765");
  const action = await view.findByRole("button", { name: "Allow In Firewall…" });
  fixture.invoke.mockRejectedValueOnce(new Error("cancelled"));
  await fireEvent.click(action);
  assert.ok(await view.findByText(/Windows Firewall was not changed/));
  assert.equal(fixture.invoke.mock.calls.length, 2);
});

test("Skipped Cleanup Is Visible Without Offering Elevation", async () => {
  const message = "fixture-cleanup-skipped";
  fixture.invoke.mockResolvedValueOnce({ required: false, message });
  const view = showNotice("host-10", false);
  assert.equal(await view.findByRole("status"), view.getByText(message));
  assert.equal(view.queryByRole("button"), null);
  assert.deepEqual(fixture.invoke.mock.calls, [["desktop_firewall", { generation: "host-10", allow: false }]]);
});
