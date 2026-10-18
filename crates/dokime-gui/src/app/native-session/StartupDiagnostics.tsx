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
 * The exit with its meaning, the log file and the executable sit on the
 * connect screen's two-column label/value grid so they line up with the attach
 * form; the log opens with the system handler through the shell. The redacted
 * stderr tail follows unfolded: an operator reporting a failed start must see
 * its cause without knowing to expand anything.
 */

/* === External Imports === */

import { type ReactNode, useState } from "react";

/* === Local Imports === */

import { PathPill } from "@/components/PathPill";
import { commands } from "@/lib/desktop/commands";
import type { BackendDiagnostics } from "@/lib/desktop/session";
import { backendExitMeaning } from "./errors";

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
  const meaning = backendExitMeaning(diagnostics.exitCode);
  const [openFailure, setOpenFailure] = useState<string | null>(null);

  // The shell opens the log it recorded; the WebView never names the path.
  async function openLog() {
    setOpenFailure(null);
    try {
      await commands.desktopOpenBackendLog();
    } catch (error) {
      setOpenFailure(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <section aria-label="Backend Diagnostics" className="grid w-full gap-3 text-left">
      <dl className="grid gap-2">
        {exit !== null && (
          <Detail label="Exit Code">
            <span className="text-sm" title={diagnostics.exitStatus ?? undefined}>
              <span className="font-mono">{exit}</span>
              {meaning ? <span className="text-muted-foreground"> · {meaning}</span> : null}
            </span>
          </Detail>
        )}
        {diagnostics.log && (
          <Detail label="Log File">
            <PathPill value={diagnostics.log} onOpen={() => void openLog()} />
          </Detail>
        )}
        {openFailure && (
          <Detail label="Open Failed">
            <span className="text-sm text-destructive" role="alert">
              {openFailure}
            </span>
          </Detail>
        )}
        <Detail label="Executable">
          <PathPill value={diagnostics.executable} />
        </Detail>
      </dl>
      <pre className="max-h-64 overflow-auto whitespace-pre-wrap break-words rounded-lg border border-input bg-muted/40 p-2 font-mono text-xs">
        {diagnostics.tail || "The backend wrote nothing to its diagnostic output."}
      </pre>
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
