/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* What a failed owned-backend start left behind, under the error headline.
 *
 * The exit and the full log file sit on the connect screen's two-column
 * label/value grid so they line up with the attach form. The resolved
 * executable and the redacted stderr tail fold into one Diagnostics block,
 * styled like the form's other disclosure.
 */

/* === External Imports === */

import type { ReactNode } from "react";

/* === Local Imports === */

import { PathPill } from "@/components/PathPill";
import type { BackendDiagnostics } from "@/lib/desktop/session";

/* === Types === */

type StartupDiagnosticsProps = {
  diagnostics: BackendDiagnostics;
};

type DetailProps = {
  children: ReactNode;
  label: string;
};

/* === Components === */

export function StartupDiagnostics({ diagnostics }: StartupDiagnosticsProps) {
  const exit = diagnostics.exitCode ?? diagnostics.exitStatus;

  return (
    <section aria-label="Backend Diagnostics" className="grid w-full gap-3 text-left">
      <dl className="grid gap-2">
        {exit !== null && (
          <Detail label="Exit Code">
            <span className="font-mono text-sm" title={diagnostics.exitStatus ?? undefined}>
              {exit}
            </span>
          </Detail>
        )}
        {diagnostics.log && (
          <Detail label="Log File">
            <PathPill value={diagnostics.log} />
          </Detail>
        )}
      </dl>
      <details className="text-sm">
        <summary className="cursor-pointer text-center text-muted-foreground">Diagnostics</summary>
        <div className="mt-3 grid gap-3">
          <dl className="grid gap-2">
            <Detail label="Executable">
              <PathPill value={diagnostics.executable} />
            </Detail>
          </dl>
          <pre className="max-h-64 overflow-auto whitespace-pre-wrap break-words rounded-lg border border-input bg-muted/40 p-2 font-mono text-xs">
            {diagnostics.tail || "The backend wrote nothing to its diagnostic output."}
          </pre>
        </div>
      </details>
    </section>
  );
}

/* === Private Components === */

/** One read-only label/value row on the connect screen's two-column grid. */
function Detail({ children, label }: DetailProps) {
  return (
    <div className="grid grid-cols-[10rem_minmax(0,1fr)] items-center gap-3">
      <dt className="text-sm text-muted-foreground">{label}</dt>
      <dd className="min-w-0">{children}</dd>
    </div>
  );
}
