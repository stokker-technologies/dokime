/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { Toaster } from "sonner";

/* === Local Imports === */

import { useApplicationTheme } from "@/lib/application-theme";

/* === Constants === */

const DOKIME_TOAST_CLASSES = {
  toast:
    "flex w-full items-start gap-3 rounded-2xl border border-border bg-popover p-4 font-sans text-sm text-popover-foreground",
  title: "font-medium text-popover-foreground",
  description: "text-xs text-muted-foreground",
  content: "grid min-w-0 gap-1",
  icon: "mt-0.5 shrink-0",
  success: "border-success bg-popover",
  error: "border-destructive bg-popover",
  warning: "border-warning bg-popover",
  info: "border-info bg-popover",
  loading: "border-border bg-muted",
  actionButton: "rounded-md bg-primary px-2 py-1 text-primary-foreground",
  cancelButton: "rounded-md bg-secondary px-2 py-1 text-secondary-foreground",
  closeButton:
    "rounded-md border border-border bg-popover text-muted-foreground hover:text-popover-foreground",
} as const;

/* === Types === */

/* === Components === */

/** The one toast surface, shared by the application and the frozen report. */
export function DokimeToaster() {
  const { resolvedTheme } = useApplicationTheme();

  return (
    <Toaster
      position="top-right"
      className="z-[100]"
      theme={resolvedTheme === "dark" ? "dark" : "light"}
      toastOptions={{ unstyled: true, classNames: DOKIME_TOAST_CLASSES }}
    />
  );
}
