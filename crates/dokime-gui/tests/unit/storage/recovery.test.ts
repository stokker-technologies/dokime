// @vitest-environment happy-dom
/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { act, createElement } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, test, vi } from "vitest";

// Local Imports
import {
  STORAGE_RESET_CONFIRMATION,
  type StorageResetContext,
  storageResetAvailable,
  storageResetConfirmed,
} from "@/features/settings/storage-reset-model";
import { ConnectionScreen } from "@/app/native-session/ConnectionScreen";
import { storageRecoveryAvailable } from "@/app/native-session/errors";

/* === Re-Exports === */

/* === Constants === */

const fixture = vi.hoisted(() => ({
  state: { phase: "failed", error: { kind: "startup", message: "fixture", diagnostics: null } } as {
    phase: string;
    error: unknown;
    reset?: false | "reset" | "backup";
  },
  destination: vi.fn(),
  reset: vi.fn(),
  select: vi.fn(),
  status: vi.fn(),
}));

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function resetContext(overrides: Partial<StorageResetContext> = {}): StorageResetContext {
  return {
    development: true,
    desktop: true,
    ownership: "owned_local",
    directory: "/home/operator/.local/share/com.stokker.dokime/data",
    ...overrides,
  } as StorageResetContext;
}

/* === Entry Point === */

vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(async () => () => {}) }));
vi.mock("@/app/native-session/context", () => ({
  useNativeSessionController: () => ({
    subscribe: () => () => {},
    snapshot: () => fixture.state,
    select: fixture.select,
    backupAndResetLocalStorage: fixture.reset,
  }),
}));
vi.mock("@/lib/desktop/commands", () => ({
  commands: {
    desktopSessionStatus: fixture.status,
    desktopStorageBackupDestination: fixture.destination,
  },
}));
vi.mock("@/lib/brand", () => ({ defaultBrand: { label: "Fixture" } }));
vi.mock("@/shell/BrandMark", () => ({ BrandMark: () => null }));
vi.mock("@/features/settings/BackendsTable", () => ({ BackendsTable: () => null }));
vi.mock("@/features/settings/SettingsSurface", () => ({ DisconnectedSettings: () => null }));
vi.mock("@/components/NotificationMessage", () => ({ NotificationMessage: () => null }));
vi.mock("@/components/PathPill", () => ({
  PathPill: ({ value }: { value: string }) => createElement("code", { "data-path": value }, value),
}));

let root: Root;
let container: HTMLDivElement;

beforeEach(() => {
  vi.clearAllMocks();
  fixture.state = { phase: "failed", error: { kind: "startup", message: "fixture", diagnostics: null } };
  fixture.destination.mockResolvedValue({ directory: "/fixture/data", archive: "/fixture/archive.zip" });
  fixture.status.mockResolvedValue({ statusRevision: 1, phase: null, connection: { endpoint: null } });
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  container = document.createElement("div");
  document.body.append(container);
  root = createRoot(container);
});

afterEach(async () => {
  await act(async () => root.unmount());
  container.remove();
});

async function renderConnection() {
  await act(async () =>
    root.render(
      createElement(ConnectionScreen, {
        launch: { localBackendAvailable: true, target: null, expectedBackendId: null, error: null },
      }),
    ),
  );
}

test("Startup Recovery Confirms Both Paths Before Calling Reset and Allows Cancellation", async () => {
  await renderConnection();
  const trigger = container.querySelector<HTMLButtonElement>('[data-action="backup-reset"]');
  assert.ok(trigger);
  await act(async () => trigger.click());
  assert.equal(fixture.reset.mock.calls.length, 0);
  const dialog = document.querySelector('[role="dialog"]');
  assert.ok(dialog?.querySelector('[data-path="/fixture/data"]'));
  assert.ok(dialog?.querySelector('[data-path="/fixture/archive.zip"]'));
  await act(async () => document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true })));
  assert.equal(fixture.reset.mock.calls.length, 0);
  assert.equal(document.querySelector('[role="dialog"]'), null);
  await act(async () => trigger.click());
  const confirm = document.querySelector<HTMLButtonElement>('[data-action="confirm-backup-reset"]');
  assert.ok(confirm);
  await act(async () => confirm.click());
  assert.deepEqual(fixture.reset.mock.calls, [["/fixture/data", "/fixture/archive.zip"]]);
});

test("Another Backend's Folder Never Offers Recovery", async () => {
  fixture.state.error = { kind: "backend_running", root: "/fixture/data", message: "fixture" };
  await renderConnection();
  assert.equal(container.querySelector('[data-action="backup-reset"]'), null);
  assert.equal(storageRecoveryAvailable(fixture.state.error, true, true), false);
});

test("Local Startup Failures Do Not Require Diagnostics or Specific Error Prose", () => {
  for (const error of [
    { kind: "startup", message: "decode", diagnostics: null },
    { kind: "startup", message: "format", diagnostics: { cause: "corrupt", log: null } },
    { kind: "storage_reset", reason: "failed", message: "fixture" },
  ])
    assert.equal(storageRecoveryAvailable(error, true, true), true);
  assert.equal(storageRecoveryAvailable({ kind: "connection" }, false, true), false);
  assert.equal(storageRecoveryAvailable({ kind: "startup" }, true, false), false);
});

test("A Saved Archive Remains Visible After Restart Fails", async () => {
  fixture.state.error = { kind: "archived", archive: "/fixture/archive.zip", error: { kind: "startup", diagnostics: null } };
  await renderConnection();
  assert.ok(container.querySelector('[data-path="/fixture/archive.zip"]'));
});

test("Reset Progress Includes Archive and Reinstallation Phases", async () => {
  fixture.state = { phase: "connecting", reset: "backup", error: null };
  fixture.status.mockResolvedValue({ statusRevision: 2, phase: "reinstalling", connection: { endpoint: null } });
  await renderConnection();
  assert.deepEqual(
    [...container.querySelectorAll("li[data-state]")].map((step) => step.getAttribute("data-state")),
    ["done", "done", "done", "active", "waiting"],
  );
});

test("Storage Reset", () => {
  // A Development Desktop That Owns Its Backend Offers the Reset
  assert.equal(storageResetAvailable(resetContext()), true);

  // An Installer Build Never Offers the Reset
  assert.equal(storageResetAvailable(resetContext({ development: false })), false);

  // A Browser Build Never Offers the Reset
  assert.equal(storageResetAvailable(resetContext({ desktop: false })), false);

  // A Backend This Desktop Did Not Start Is Never Offered the Reset
  for (const ownership of ["attached", "none", null, undefined]) {
    const context = resetContext({ ownership: ownership as StorageResetContext["ownership"] });
    assert.equal(storageResetAvailable(context), false, String(ownership));
  }

  // The Reset Waits Until the Backend Names Its Data Directory
  for (const directory of ["", null, undefined]) {
    assert.equal(storageResetAvailable(resetContext({ directory })), false, String(directory));
  }

  // Only the Exact Confirmation Word Arms the Reset
  assert.equal(storageResetConfirmed(STORAGE_RESET_CONFIRMATION), true);
  for (const typed of ["", "reset", "Reset", " RESET", "RESET ", "RESETS", "RES"]) {
    assert.equal(storageResetConfirmed(typed), false, JSON.stringify(typed));
  }
});
