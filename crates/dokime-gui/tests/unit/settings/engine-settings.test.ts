/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The Engine Page's Stream Settings Are Read, Validated and Saved Through the Desktop Commands
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { createElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, test, vi } from "vitest";

// Local Imports
import { EngineSection } from "@/features/settings/sections/EngineSection";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  saved: { interval_window_ms: 100, interval_lag_windows: 10 },
  draft: null as { window: string; lag: string } | null,
  invoke: vi.fn(),
  read: null as (() => Promise<unknown>) | null,
  save: null as { onClick: () => void; disabled: boolean } | null,
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => fixture.invoke(...args) }));
vi.mock("@tanstack/react-query", () => ({
  useQueryClient: () => ({ setQueryData: vi.fn() }),
  useQuery: ({ queryFn }: { queryFn: () => Promise<unknown> }) => {
    fixture.read = queryFn;
    return { data: fixture.saved, isPending: false, isSuccess: true, error: null };
  },
  useMutation: ({ mutationFn }: { mutationFn: (settings: unknown) => Promise<unknown> }) => ({
    mutate: (settings: unknown) => void mutationFn(settings),
    isPending: false,
    error: null,
  }),
}));
vi.mock("react", async (original) => ({
  ...(await original<typeof import("react")>()),
  useState: () => [fixture.draft, vi.fn()],
}));
vi.mock("@dokime/plots/components/ui/button", () => ({
  Button: ({ children, disabled, onClick }: { children: ReactNode; disabled?: boolean; onClick: () => void }) => {
    fixture.save = { onClick, disabled: Boolean(disabled) };
    return createElement("button", { type: "button" }, children);
  },
}));

/* === Types === */

/* === Classes === */

/* === Public Functions === */

beforeEach(() => {
  vi.clearAllMocks();
  fixture.saved = { interval_window_ms: 100, interval_lag_windows: 10 };
  fixture.draft = null;
  fixture.save = null;
  fixture.invoke.mockResolvedValue(fixture.saved);
});

/* === Private Functions === */

function render() {
  return renderToStaticMarkup(createElement(EngineSection));
}

/* === Tests === */

test("Saved Values Show With Nothing to Save", async () => {
  const html = render();
  assert.ok(fixture.read);
  await fixture.read();
  assert.deepEqual(fixture.invoke.mock.calls, [["desktop_saved_settings_read", undefined]]);
  assert.ok(html.includes('value="100"'));
  assert.ok(html.includes('value="10"'));
  assert.equal(fixture.save?.disabled, true);
});

test("An Edited Window and Lag Save Through the Desktop", async () => {
  fixture.draft = { window: "250", lag: "3" };
  render();
  assert.equal(fixture.save?.disabled, false);
  fixture.save?.onClick();
  await vi.waitFor(() => assert.equal(fixture.invoke.mock.calls.length, 1));
  assert.deepEqual(fixture.invoke.mock.calls[0], [
    "desktop_saved_settings_update",
    { change: { interval_window_ms: 250, interval_lag_windows: 3 } },
  ]);
});

test("Zero, Fractional and Oversized Values Are Refused Before Saving", () => {
  for (const draft of [
    { window: "0", lag: "10" },
    { window: "2.5", lag: "10" },
    { window: "100", lag: "4294967296" },
  ]) {
    fixture.draft = draft;
    const html = render();
    assert.ok(html.includes('aria-invalid="true"'), JSON.stringify(draft));
    assert.equal(fixture.save?.disabled, true, JSON.stringify(draft));
  }
  assert.equal(fixture.invoke.mock.calls.length, 0);
});
