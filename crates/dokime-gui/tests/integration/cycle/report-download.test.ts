// @vitest-environment happy-dom
// Dokime — SPDX-License-Identifier: Apache-2.0
import assert from "node:assert/strict";
import { fireEvent, render } from "@testing-library/svelte";
import { test, vi } from "vitest";
import DownloadReportAction from "@/features/cycle/DownloadReportAction.svelte";
import DownloadReportMenuProbe from "./DownloadReportMenuProbe.svelte";

test("Download Report Enables When the Seal Is Observed Without Remounting", async () => {
  const mutation = { isPending: false, mutateAsync: vi.fn().mockResolvedValue(null) };
  const props = { mutation, executionId: "execution", label: "Execution" };
  const view = render(DownloadReportMenuProbe, { ...props, sealed: false });
  try {
    await fireEvent.pointerDown(view.getByRole("button", { name: "Actions" }), { button: 0 });
    const item = view.getByRole("menuitem");
    assert.equal(item.getAttribute("aria-disabled"), "true");
    assert.match(item.title, /once the run is sealed/);
    await fireEvent.click(item);
    assert.equal(mutation.mutateAsync.mock.calls.length, 0);

    await view.rerender({ ...props, sealed: true });
    assert.equal(view.getByRole("menuitem"), item);
    assert.notEqual(item.getAttribute("aria-disabled"), "true");
    await fireEvent.click(item);
    assert.deepEqual(mutation.mutateAsync.mock.calls, [[{ executionId: "execution", label: "Execution" }]]);
  } finally {
    view.unmount();
  }
});

test("History Button Uses the Same Sealed-Only Download and Save State", async () => {
  const mutation = { isPending: false, mutateAsync: vi.fn().mockResolvedValue(null) };
  const props = { mutation, executionId: null, label: "Cycle", presentation: "button" as const };
  const view = render(DownloadReportAction, { ...props, sealed: false });
  try {
    const button = view.getByRole("button") as HTMLButtonElement;
    assert.equal(button.disabled, true);
    await view.rerender({ ...props, sealed: true });
    assert.equal(button.disabled, false);
    await fireEvent.click(button);
    assert.deepEqual(mutation.mutateAsync.mock.calls, [[{ executionId: null, label: "Cycle" }]]);
    await view.rerender({ ...props, sealed: true, mutation: { ...mutation, isPending: true } });
    assert.equal(button.disabled, true);
    assert.equal(button.textContent?.trim(), "Saving Report…");
  } finally {
    view.unmount();
  }
});
