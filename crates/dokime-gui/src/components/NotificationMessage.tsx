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
import { failureOf, type NotificationFailure } from "@/lib/notification-details";

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
  context?: "action" | "background";
};

/* === Components === */

export function NotificationMessage({
  children,
  source,
  title,
  tone = "error",
  failure,
  error,
  context = "action",
}: NotificationMessageProps) {
  const id = useId();
  const description = notificationText(children);
  const heading = title ?? (tone === "error" ? "Operation Failed" : "Status Updated");
  const record =
    tone === "error"
      ? failureOf(
          failure ?? error ?? description,
          heading,
          (failure ?? error) && typeof (failure ?? error) === "object" ? undefined : `${id}:${description}`,
        )
      : undefined;
  useEffect(() => {
    if (record) toast.error(record, { id: record.id, source, context });
    else if (description && tone !== "error") toast[tone](heading, { description, id, source });
  }, [description, heading, id, source, tone, record, context]);
  return null;
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
