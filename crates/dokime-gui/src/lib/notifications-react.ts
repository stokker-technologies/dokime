// Dokime — SPDX-License-Identifier: Apache-2.0
import { isValidElement, type ReactNode, useSyncExternalStore } from "react";
import { getNotificationSummary, subscribeNotificationSummary } from "./notifications";

export function useNotificationSummary() {
  return useSyncExternalStore(subscribeNotificationSummary, getNotificationSummary, getNotificationSummary);
}

export function notificationText(node: ReactNode): string {
  if (node == null || typeof node === "boolean") {
    return "";
  }
  if (typeof node === "string" || typeof node === "number" || typeof node === "bigint") {
    return String(node);
  }
  if (Array.isArray(node)) {
    return node.map(notificationText).filter(Boolean).join(" ");
  }
  if (isValidElement<{ children?: ReactNode }>(node)) {
    return notificationText(node.props.children);
  }
  return "";
}
