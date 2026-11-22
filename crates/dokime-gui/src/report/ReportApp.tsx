/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { useState } from "react";
import { DokimeToaster } from "@/components/DokimeToaster";
import type { CycleSelection } from "@/features/cycle/cycle-workspace-model";
import type { CycleSnapshotSource } from "@/features/cycle/snapshot-source";
import { ReportCycleWorkspace } from "./ReportCycleWorkspace";
import { ReportShell } from "./ReportShell";

/* === Types === */

type ReportAppProps = {
  source: CycleSnapshotSource;
  executionId: string | null;
};

/* === Components === */

export function ReportApp({ source, executionId }: ReportAppProps) {
  const [selection, setSelection] = useState<CycleSelection>(executionId ? { executionId } : {});

  return (
    <>
      <ReportShell selection={selection} source={source} onSelect={executionId === null ? setSelection : undefined}>
        {source.warnings?.length ? (
          <aside aria-label="Report Notices" className="mx-4 mt-4 rounded-lg border border-warning/40 bg-warning/10 px-4 py-3 text-sm">
            <ul className="grid gap-2">
              {source.warnings.map((warning) => (
                <li key={warning}>{warning}</li>
              ))}
            </ul>
          </aside>
        ) : null}
        <ReportCycleWorkspace selection={selection} source={source} />
      </ReportShell>
      <DokimeToaster />
    </>
  );
}
