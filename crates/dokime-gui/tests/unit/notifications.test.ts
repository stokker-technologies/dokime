/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Notification History, Faults And Delivery.
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { forceCloseDatabase, IDBFactory, IDBKeyRange, IDBObjectStore } from "fake-indexeddb";
// Third Party Imports
import { createElement } from "react";
import { afterEach, beforeEach, test, vi } from "vitest";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

const shown = vi.hoisted(() => ({
  error: vi.fn((_title: string, options?: { id?: string | number; duration?: number }) => options?.id ?? "error"),
  info: vi.fn(() => "info"),
  success: vi.fn(() => "success"),
  warning: vi.fn(() => "warning"),
}));
vi.mock("sonner", () => ({ toast: shown }));
vi.mock("react", async (original) => ({
  ...(await original<typeof import("react")>()),
  useId: () => "fixture-feedback",
  useEffect: (effect: () => void) => effect(),
}));

beforeEach(() => {
  vi.resetModules();
  vi.clearAllMocks();
  vi.stubGlobal("indexedDB", new IDBFactory());
  vi.stubGlobal("IDBKeyRange", IDBKeyRange);
});
afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function record(index: number) {
  return {
    id: String(index).padStart(3, "0"),
    at: Math.floor(index / 2),
    source: "fixture",
    title: `row-${index}`,
    tone: "info" as const,
    unread: 1 as const,
  };
}

/* === Entry Point === */

test("History Paging", async () => {
  const history = await import("@/lib/notifications");
  assert.deepEqual((await history.notificationPage()).records, []);
  for (let index = 0; index < 53; index++) await history.recordNotification(record(index));
  vi.resetModules();
  const reopened = await import("@/lib/notifications");
  const first = await reopened.notificationPage();
  assert.deepEqual(
    first.records.map((value) => value.id),
    Array.from({ length: 25 }, (_, index) => record(52 - index).id),
  );
  assert.equal(first.newer, false);
  assert.equal(first.older, true);
  const second = await reopened.notificationPage(first.records.at(-1));
  assert.deepEqual(
    second.records.map((value) => value.id),
    Array.from({ length: 25 }, (_, index) => record(27 - index).id),
  );
  const last = await reopened.notificationPage(second.records.at(-1));
  assert.deepEqual(
    last.records.map((value) => value.id),
    ["002", "001", "000"],
  );
  assert.equal(last.older, false);
  assert.equal(last.newer, true);
  const previous = await reopened.notificationPage(last.records[0], "newer");
  assert.deepEqual(previous.records, second.records);
});

test("Unread And Clear", async () => {
  const history = await import("@/lib/notifications");
  await history.recordNotification(record(0));
  await history.recordNotification(record(1));
  await history.markNotificationsRead();
  assert.ok((await history.notificationPage()).records.every((value) => value.unread === 0));
  await history.recordNotification(record(2));
  assert.deepEqual(
    (await history.notificationPage()).records.map((value) => value.unread),
    [1, 0, 0],
  );
  const write = vi.spyOn(IDBObjectStore.prototype, "put").mockImplementationOnce(() => {
    throw new DOMException("fixture", "QuotaExceededError");
  });
  await assert.rejects(history.recordNotification({ ...record(0), title: "replacement" }), { name: "QuotaExceededError" });
  write.mockRestore();
  assert.equal((await history.notificationPage()).records.find((value) => value.id === "000")?.title, "row-0");
  await history.clearNotifications();
  assert.deepEqual((await history.notificationPage()).records, []);
});

test("Failed Open Retries", async () => {
  const history = await import("@/lib/notifications");
  const factory = indexedDB;
  vi.stubGlobal("indexedDB", {
    open: () => {
      throw new DOMException("fixture", "SecurityError");
    },
  });
  await assert.rejects(history.notificationPage(), { name: "SecurityError" });
  vi.stubGlobal("indexedDB", factory);
  await history.recordNotification(record(0));
  assert.equal((await history.notificationPage()).records.length, 1);
});

test("Sticky Error Delivery", async () => {
  const history = await import("@/lib/notifications");
  vi.spyOn(Date, "now").mockReturnValue(7);
  history.toast.error("fixture", { id: "fixture", source: "fixture-source", description: "fixture-detail", duration: 1 });
  assert.equal(shown.error.mock.calls.at(-1)?.[1]?.duration, Infinity);
  const page = await history.notificationPage();
  assert.equal(page.records.length, 1);
  assert.deepEqual(
    { at: page.records[0].at, source: page.records[0].source, description: page.records[0].description },
    { at: 7, source: "fixture-source", description: "fixture-detail" },
  );
});

test("Updated Toasts Stay Recorded", async () => {
  const history = await import("@/lib/notifications");
  history.toast.info("fixture-one", { id: "fixture" });
  history.toast.success("fixture-two", { id: "fixture" });
  const page = await history.notificationPage();
  assert.deepEqual(page.records.map((value) => value.title).sort(), ["fixture-one", "fixture-two"]);
});

test("Feedback Is Recorded", async () => {
  const { NotificationMessage } = await import("@/components/NotificationMessage");
  const history = await import("@/lib/notifications");
  assert.equal(
    NotificationMessage({ children: ["fixture-one", createElement("strong", null, "fixture-two")], source: "fixture-source" }),
    null,
  );
  const page = await history.notificationPage();
  assert.equal(page.records.length, 1);
  assert.equal(page.records[0].description, "fixture-one fixture-two");
  assert.equal(page.records[0].source, "fixture-source");
});

test("Closed Connections Recover", async () => {
  const opened = vi.spyOn(indexedDB, "open");
  const history = await import("@/lib/notifications");
  await history.recordNotification(record(0));
  const connection = opened.mock.results[0].value.result as IDBDatabase;
  const closed = new Promise<void>((resolve) => connection.addEventListener("close", () => resolve(), { once: true }));
  // The Driver Accepts An Instance; Its Declaration Names The Constructor.
  const close = forceCloseDatabase as unknown as (value: IDBDatabase) => void;
  close(connection);
  await closed;
  await history.recordNotification(record(1));
  assert.equal((await history.notificationPage()).records.length, 2);
});
