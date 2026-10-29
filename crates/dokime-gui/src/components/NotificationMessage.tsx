/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Feedback Goes To Toasts And Durable History Without Moving Its Controls.
 */

import { type ReactNode, useEffect, useId } from "react";

import { notificationText, toast } from "@/lib/notifications";

type NotificationMessageProps = { children: ReactNode; title?: string; source?: string; tone?: "error" | "info" | "success" };

export function NotificationMessage({ children, source, title, tone = "error" }: NotificationMessageProps) {
  const id = useId();
  const description = notificationText(children);
  const heading = title ?? (tone === "error" ? "Operation Failed" : "Status Updated");
  useEffect(() => {
    if (description) toast[tone](heading, { description, id, source });
  }, [description, heading, id, source, tone]);
  return null;
}
