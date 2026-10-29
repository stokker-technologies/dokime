/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import type React from "react";
import { Toaster } from "sonner";

/* === Local Imports === */

import { useApplicationTheme } from "@/lib/application-theme";

/* === Constants === */

const DOKIME_TOAST_CLASSES = {
  toast:
    "relative flex w-full items-start gap-3 rounded-lg border border-border bg-popover p-4 pr-10 font-sans text-sm text-popover-foreground",
  title: "font-medium text-popover-foreground",
  description: "text-xs text-muted-foreground",
  content: "grid min-w-0 gap-1",
  icon: "mt-1 shrink-0",
  success: "border-success bg-popover",
  error: "border-destructive bg-popover",
  warning: "border-warning bg-popover",
  info: "border-info bg-popover",
  loading: "border-border bg-muted",
  actionButton: "shrink-0 whitespace-nowrap rounded-lg bg-primary px-3 py-2 text-primary-foreground",
  cancelButton: "rounded-lg bg-secondary px-2 py-1 text-secondary-foreground",
  closeButton:
    "absolute top-2 right-2 grid size-6 place-items-center rounded-lg text-muted-foreground hover:bg-muted hover:text-popover-foreground [&>svg]:size-4",
} as const;

/* === Types === */

/* === Components === */

/** The one toast surface, shared by the application and the frozen report. */
export function DokimeToaster() {
  const { resolvedTheme } = useApplicationTheme();

  return (
    <Toaster
      className="z-[100]"
      closeButton
      offset={{
        bottom: "calc(var(--status-bar-height) + var(--shell-gap))",
        right: "var(--shell-gap)",
      }}
      position="bottom-right"
      style={{ "--width": "28rem" } as React.CSSProperties}
      theme={resolvedTheme === "dark" ? "dark" : "light"}
      toastOptions={{ unstyled: true, classNames: DOKIME_TOAST_CLASSES }}
    />
  );
}
