// @vitest-environment happy-dom
// Dokime — SPDX-License-Identifier: Apache-2.0
import assert from "node:assert/strict";
import { QueryClient } from "@tanstack/svelte-query";
import { fireEvent, render } from "@testing-library/svelte";
import { beforeEach, onTestFinished, test, vi } from "vitest";
import EngineSection from "@/features/settings/sections/EngineSection.svelte";

const fixture = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: fixture.invoke }));

beforeEach(() => {
  fixture.invoke.mockReset();
  fixture.invoke.mockResolvedValue({ interval_window_ms: 100, interval_lag_windows: 10 });
});

async function showSettings() {
  const queryClient = new QueryClient();
  const view = render(EngineSection, { queryClient });
  onTestFinished(() => {
    view.unmount();
    queryClient.clear();
  });
  const window = await view.findByRole("textbox", { name: "Interval Window [ms]" });
  const lag = view.getByRole("textbox", { name: "Interval Lag [windows]" });
  const save = view.getByRole("button", { name: "Save" });
  assert.ok(window instanceof HTMLInputElement);
  assert.ok(lag instanceof HTMLInputElement);
  assert.ok(save instanceof HTMLButtonElement);
  return { window, lag, save };
}

test("Saved Values Show With Nothing to Save", async () => {
  const { window, lag, save } = await showSettings();
  assert.deepEqual(fixture.invoke.mock.calls, [["desktop_saved_settings_read", undefined]]);
  assert.equal(window.value, "100");
  assert.equal(lag.value, "10");
  assert.equal(save.disabled, true);
});

test("An Edited Window and Lag Save Through the Desktop", async () => {
  const { window, lag, save } = await showSettings();
  await fireEvent.input(window, { target: { value: "250" } });
  await fireEvent.input(lag, { target: { value: "3" } });
  assert.equal(save.disabled, false);
  let sent!: () => void;
  const requestSent = new Promise<void>((resolve) => {
    sent = resolve;
  });
  fixture.invoke.mockImplementationOnce(async () => {
    sent();
    return { interval_window_ms: 250, interval_lag_windows: 3 };
  });
  await fireEvent.click(save);
  await requestSent;
  assert.deepEqual(fixture.invoke.mock.calls[1], [
    "desktop_saved_settings_update",
    { change: { interval_window_ms: 250, interval_lag_windows: 3 } },
  ]);
});

test("Zero, Fractional and Oversized Values Are Refused Before Saving", async () => {
  const { window, lag, save } = await showSettings();
  for (const draft of [
    { window: "0", lag: "10" },
    { window: "2.5", lag: "10" },
    { window: "100", lag: "4294967296" },
  ]) {
    await fireEvent.input(window, { target: { value: draft.window } });
    await fireEvent.input(lag, { target: { value: draft.lag } });
    assert.ok(
      [window, lag].some((input) => input.getAttribute("aria-invalid") === "true"),
      JSON.stringify(draft),
    );
    assert.equal(save.disabled, true, JSON.stringify(draft));
  }
  assert.deepEqual(fixture.invoke.mock.calls, [["desktop_saved_settings_read", undefined]]);
});
