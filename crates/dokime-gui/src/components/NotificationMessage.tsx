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

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { type ReactNode, useEffect, useId } from "react";

// Local Imports
import { notificationText, toast } from "@/lib/notifications";
import type { NotificationFailure } from "@/lib/notification-details";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type NotificationMessageProps = {
  children: ReactNode;
  title?: string;
  source?: string;
  tone?: "error" | "info" | "success";
  failure?: NotificationFailure;
  error?: unknown;
};

/* === Components === */

export function NotificationMessage({ children, source, title, tone = "error", failure, error }: NotificationMessageProps) {
  const id = useId();
  const description = notificationText(children);
  const heading = title ?? (tone === "error" ? "Operation Failed" : "Status Updated");
  useEffect(() => {
    if (description) toast[tone](heading, { description, id, source, failure, error });
  }, [description, heading, id, source, tone, failure, error]);
  return null;
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
