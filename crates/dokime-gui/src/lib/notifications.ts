/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Durable, Paged Notification History And Toast Delivery.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { isValidElement, type ReactNode, useSyncExternalStore } from "react";
import { toast as displayToast, type ExternalToast } from "sonner";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

const PAGE_SIZE = 25;

const listeners = new Set<() => void>();

let database: Promise<IDBDatabase> | undefined;

let summary: Summary = { count: 0, unread: 0, revision: 0 };

let refreshing: Promise<void> | undefined;

let dirty = false;

export const toast = {
  error: (title: string, options?: NotificationOptions) => publish("error", title, options),
  info: (title: string, options?: NotificationOptions) => publish("info", title, options),
  success: (title: string, options?: NotificationOptions) => publish("success", title, options),
  warning: (title: string, options?: NotificationOptions) => publish("warning", title, options),
};

/* === Types === */

export type NotificationTone = "error" | "info" | "success" | "warning";

export type NotificationRecord = {
  id: string;
  at: number;
  source: string;
  title: string;
  description?: string;
  tone: NotificationTone;
  unread: 0 | 1;
};

export type NotificationCursor = Pick<NotificationRecord, "at" | "id">;

export type NotificationPage = { records: NotificationRecord[]; older: boolean; newer: boolean };

type NotificationOptions = Omit<ExternalToast, "description"> & { description?: string | null; source?: string };

type Summary = { count: number; unread: number; revision: number; error?: string };

/* === Classes === */

/* === Public Functions === */

export async function recordNotification(record: NotificationRecord): Promise<void> {
  const connection = await open();
  const transaction = connection.transaction("notifications", "readwrite");
  transaction.objectStore("notifications").put(record);
  await completed(transaction);
  await refresh();
}

export async function notificationPage(from?: NotificationCursor, direction: "older" | "newer" = "older"): Promise<NotificationPage> {
  const connection = await open();
  const transaction = connection.transaction("notifications", "readonly");
  const index = transaction.objectStore("notifications").index("order");
  const boundary = from ? [from.at, from.id] : undefined;
  const range = boundary
    ? direction === "older"
      ? IDBKeyRange.upperBound(boundary, true)
      : IDBKeyRange.lowerBound(boundary, true)
    : undefined;
  const request = index.openCursor(range, direction === "older" ? "prev" : "next");
  const rows = new Promise<NotificationRecord[]>((resolve, reject) => {
    const records: NotificationRecord[] = [];
    request.onerror = () => reject(request.error);
    request.onsuccess = () => {
      const cursor = request.result;
      if (!cursor || records.length === PAGE_SIZE) {
        resolve(records);
        return;
      }
      records.push(cursor.value as NotificationRecord);
      cursor.continue();
    };
  });
  const [records] = await Promise.all([rows, completed(transaction)]);
  if (direction === "newer") records.reverse();
  if (!records.length) return { records, older: false, newer: false };
  const bounds = connection.transaction("notifications", "readonly");
  const ordered = bounds.objectStore("notifications").index("order");
  const newest = records[0];
  const oldest = records[records.length - 1];
  const [older, newer] = await Promise.all([
    result(ordered.count(IDBKeyRange.upperBound([oldest.at, oldest.id], true))),
    result(ordered.count(IDBKeyRange.lowerBound([newest.at, newest.id], true))),
    completed(bounds),
  ]);
  return { records, older: older > 0, newer: newer > 0 };
}

export async function markNotificationsRead(): Promise<void> {
  const connection = await open();
  const transaction = connection.transaction("notifications", "readwrite");
  const request = transaction.objectStore("notifications").index("unread").openCursor(IDBKeyRange.only(1));
  request.onsuccess = () => {
    const cursor = request.result;
    if (!cursor) return;
    cursor.update({ ...cursor.value, unread: 0 });
    cursor.continue();
  };
  await completed(transaction);
  await refresh();
}

export async function clearNotifications(): Promise<void> {
  const connection = await open();
  const transaction = connection.transaction("notifications", "readwrite");
  transaction.objectStore("notifications").clear();
  await completed(transaction);
  await refresh();
}

export function useNotificationSummary(): Summary {
  return useSyncExternalStore(
    subscribe,
    () => summary,
    () => summary,
  );
}

export function notificationText(node: ReactNode): string {
  if (node == null || typeof node === "boolean") return "";
  if (typeof node === "string" || typeof node === "number" || typeof node === "bigint") return String(node);
  if (Array.isArray(node)) return node.map(notificationText).filter(Boolean).join(" ");
  if (isValidElement<{ children?: ReactNode }>(node)) return notificationText(node.props.children);
  return "";
}

/* === Private Functions === */

function open(): Promise<IDBDatabase> {
  if (database) return database;
  const pending = new Promise<IDBDatabase>((resolve, reject) => {
    let blocked = false;
    const request = indexedDB.open("dokime-notifications", 1);
    request.onupgradeneeded = () => {
      const store = request.result.createObjectStore("notifications", { keyPath: "id" });
      store.createIndex("order", ["at", "id"]);
      store.createIndex("unread", "unread");
    };
    request.onsuccess = () => {
      const connection = request.result;
      if (blocked) {
        connection.close();
        return;
      }
      connection.onclose = () => {
        database = undefined;
      };
      connection.onversionchange = () => {
        connection.close();
        database = undefined;
      };
      resolve(connection);
    };
    request.onerror = () => reject(request.error);
    request.onblocked = () => {
      blocked = true;
      reject(new Error("Another window is keeping notification history open."));
    };
  }).catch((error: unknown) => {
    database = undefined;
    throw error;
  });
  database = pending;
  return pending;
}

function completed(transaction: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    transaction.oncomplete = () => resolve();
    transaction.onabort = () => reject(transaction.error ?? new Error("Notification history transaction was aborted."));
    transaction.onerror = () => reject(transaction.error ?? new Error("Notification history transaction failed."));
  });
}

function result<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

function historyFailure(error: unknown): void {
  summary = { ...summary, revision: summary.revision + 1, error: error instanceof Error ? error.message : String(error) };
  for (const listener of listeners) listener();
  displayToast.error("Notification History Unavailable", {
    id: "notification-history-unavailable",
    description: error instanceof Error ? error.message : String(error),
    duration: Infinity,
  });
}

async function refresh(): Promise<void> {
  dirty = true;
  if (refreshing) return refreshing;
  refreshing = (async () => {
    while (dirty) {
      dirty = false;
      const connection = await open();
      const transaction = connection.transaction("notifications", "readonly");
      const store = transaction.objectStore("notifications");
      const [count, unread] = await Promise.all([result(store.count()), result(store.index("unread").count(1)), completed(transaction)]);
      summary = { count, unread, revision: summary.revision + 1 };
      for (const listener of listeners) listener();
    }
  })().finally(() => {
    refreshing = undefined;
  });
  return refreshing;
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  void refresh().catch(historyFailure);
  return () => {
    listeners.delete(listener);
  };
}

function publish(tone: NotificationTone, title: string, options: NotificationOptions = {}): string | number {
  const { source, ...display } = options;
  const id = displayToast[tone](title, { ...display, ...(tone === "error" ? { duration: Infinity } : {}) });
  const identity = Array.from(crypto.getRandomValues(new Uint32Array(4)), (value) => value.toString(16)).join("-");
  void recordNotification({
    id: identity,
    at: Date.now(),
    title,
    tone,
    unread: 1,
    source: source ?? (typeof location === "undefined" ? "Desktop" : location.pathname),
    description: options.description ?? undefined,
  }).catch(historyFailure);
  return id;
}

/* === Entry Point === */
