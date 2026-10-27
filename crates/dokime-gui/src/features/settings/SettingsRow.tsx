/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* One row shape for every settings surface.
 *
 * Two columns: a fixed label column and the value. The description lives
 * inside the label cell and wraps in full, so a row is 40px when it needs no
 * explanation and grows when it does. A field error sits under its control,
 * and the autosave state sits at the row's right edge, where the eye already
 * is after changing the value.
 */

/* === External Imports === */

import type { ReactNode } from "react";

/* === Local Imports === */

import { cn } from "@/lib/utils";

/* === Types === */

type SettingsRowProps = {
  children?: ReactNode;
  className?: string;
  control?: ReactNode;
  description?: ReactNode;
  /** An inline problem with this field, shown under its control. */
  error?: ReactNode;
  htmlFor?: string;
  label: ReactNode;
  /** The autosave readout; see `SettingsFieldStatus`. */
  status?: ReactNode;
};

/* === Components === */

export function SettingsRow({ children, className, control, description, error, htmlFor, label, status }: SettingsRowProps) {
  return (
    <div
      className={cn(
        "grid min-h-[var(--settings-row-height)] grid-cols-1 xl:grid-cols-(--layout-settings-row) items-center gap-x-6 gap-y-3 px-4 py-3",
        className,
      )}
    >
      <div className="min-w-0">
        {htmlFor ? (
          <label className="block text-sm leading-snug font-medium" htmlFor={htmlFor}>
            {label}
          </label>
        ) : (
          <div className="text-sm leading-snug font-medium">{label}</div>
        )}
        {description ? <p className="text-xs leading-snug text-pretty text-muted-foreground">{description}</p> : null}
      </div>
      <div className="grid min-w-0 gap-2">
        <div className="flex min-w-0 items-center justify-end gap-3">
          <div className="flex min-w-0 flex-1 flex-wrap items-center justify-end gap-2">{control ?? children}</div>
        </div>
        {status ? <div className="text-right">{status}</div> : null}
        {error ? (
          <div className="max-w-[var(--settings-value-max)] text-xs leading-snug text-destructive" role="alert">
            {error}
          </div>
        ) : null}
      </div>
    </div>
  );
}
