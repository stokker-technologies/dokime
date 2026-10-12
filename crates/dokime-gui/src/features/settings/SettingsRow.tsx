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
 * Two columns: a fixed label column and the value. The one-line description
 * lives inside the label cell, so a row is 40px when it needs no explanation
 * and grows when it does — to two lines, wrapped rather than cut, never a
 * paragraph under the value that pushes the next label out of its column. The autosave state sits at the row's right
 * edge, where the eye already is after changing the value.
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
  htmlFor?: string;
  label: ReactNode;
  /** The autosave readout; see `SettingsFieldStatus`. */
  status?: ReactNode;
};

/* === Components === */

export function SettingsRow({
  children,
  className,
  control,
  description,
  htmlFor,
  label,
  status,
}: SettingsRowProps) {
  return (
    <div
      className={cn(
        "grid min-h-[var(--settings-row-height)] grid-cols-[var(--settings-label-width)_minmax(0,1fr)] items-center gap-x-6 px-4 py-1.5",
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
        {description ? (
          <p
            className="line-clamp-2 text-xs leading-snug text-muted-foreground"
            title={typeof description === "string" ? description : undefined}
          >
            {description}
          </p>
        ) : null}
      </div>
      <div className="flex min-w-0 items-center gap-3">
        <div className="flex min-w-0 max-w-[var(--settings-value-max)] flex-1 items-center gap-2">
          {control ?? children}
        </div>
        {status ? <div className="shrink-0">{status}</div> : null}
      </div>
    </div>
  );
}
