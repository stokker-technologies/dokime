/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Capture Folders Are Read, Added and Removed Through the Desktop Commands
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { createElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, test, vi } from "vitest";

// Local Imports
import { CaptureFoldersSection } from "@/features/settings/sections/CaptureFoldersSection";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  folders: [] as string[],
  draft: "",
  invoke: vi.fn(),
  read: null as (() => Promise<unknown>) | null,
  write: null as ((folders: string[]) => Promise<unknown>) | null,
  buttons: new Map<string, { onClick: () => void; disabled: boolean }>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => fixture.invoke(...args) }));
vi.mock("@tanstack/react-query", () => ({
  useQueryClient: () => ({ setQueryData: vi.fn() }),
  useQuery: ({ queryFn }: { queryFn: () => Promise<unknown> }) => {
    fixture.read = queryFn;
    return { data: fixture.folders, isPending: false, isSuccess: true, error: null };
  },
  useMutation: ({ mutationFn }: { mutationFn: (folders: string[]) => Promise<unknown> }) => {
    fixture.write = mutationFn;
    return { mutate: (folders: string[]) => void mutationFn(folders), isPending: false, error: null };
  },
}));
vi.mock("react", async (original) => ({
  ...(await original<typeof import("react")>()),
  useState: () => [fixture.draft, vi.fn()],
}));
vi.mock("@/components/ui/button", () => ({
  Button: ({ "aria-label": label, children, disabled, onClick }: { "aria-label"?: string; children: ReactNode; disabled?: boolean; onClick: () => void }) => {
    fixture.buttons.set(label ?? String(children), { onClick, disabled: Boolean(disabled) });
    return createElement("button", { type: "button" });
  },
}));

/* === Types === */

/* === Classes === */

/* === Public Functions === */

beforeEach(() => {
  vi.clearAllMocks();
  fixture.folders = [];
  fixture.draft = "";
  fixture.buttons.clear();
  fixture.invoke.mockResolvedValue([]);
});

/* === Private Functions === */

function render() {
  return renderToStaticMarkup(createElement(CaptureFoldersSection));
}

/* === Tests === */

test("No Saved Folders Lists Nothing and Reads Them Through the Desktop", async () => {
  render();
  assert.ok(fixture.read);
  await fixture.read();
  assert.deepEqual(fixture.invoke.mock.calls, [["desktop_capture_folders_read", undefined]]);
  assert.equal(fixture.buttons.get("Add Folder")?.disabled, true);
  assert.equal([...fixture.buttons.keys()].filter((key) => key.startsWith("Remove ")).length, 0);
});

test("Adding a Folder Saves the Complete List Including It", async () => {
  fixture.folders = ["D:\\Bench\\Logs"];
  fixture.draft = "D:\\Bench\\Device";
  const html = render();
  assert.ok(html.includes("D:\\Bench\\Logs"));
  const add = fixture.buttons.get("Add Folder");
  assert.equal(add?.disabled, false);
  add?.onClick();
  await vi.waitFor(() => assert.equal(fixture.invoke.mock.calls.length, 1));
  assert.deepEqual(fixture.invoke.mock.calls[0], ["desktop_capture_folders_write", { folders: ["D:\\Bench\\Logs", "D:\\Bench\\Device"] }]);
});

test("Removing a Folder Saves the List Without It", async () => {
  fixture.folders = ["D:\\Bench\\Logs", "D:\\Bench\\Device"];
  render();
  fixture.buttons.get("Remove D:\\Bench\\Logs")?.onClick();
  await vi.waitFor(() => assert.equal(fixture.invoke.mock.calls.length, 1));
  assert.deepEqual(fixture.invoke.mock.calls[0], ["desktop_capture_folders_write", { folders: ["D:\\Bench\\Device"] }]);
});
