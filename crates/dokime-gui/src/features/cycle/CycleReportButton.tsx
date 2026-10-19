/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Download Report: the last control of the page bar's group on the cycle page
 * and on the execution page.
 *
 * It saves through the native dialog the same report Open Report opens — one
 * rendered from the run so far, the sealed one, or the sealed one rendered
 * again when its file is gone — named after the cycle or execution, then
 * opens the saved file in the default browser. It waits only for a save
 * already in progress.
 */

/* === External Imports === */

import { Download } from "lucide-react";
import { toast } from "sonner";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { useCycleCommands } from "./cycle-queries";
import { reportDownloadState } from "./cycle-workspace-model";

/* === Types === */

export type CycleReportButtonProps = {
  cycleId: string;
  /** The execution whose report is saved; `null` saves the cycle's. */
  executionId: string | null;
  /** The cycle or execution label the file is named after. */
  label: string;
};

/* === Components === */

export function CycleReportButton({ cycleId, executionId, label }: CycleReportButtonProps) {
  const { downloadReport } = useCycleCommands(cycleId);
  const state = reportDownloadState(downloadReport.isPending);
  const save = () => {
    downloadReport
      .mutateAsync({ executionId, label })
      .then((saved) => {
        if (!saved) return;
        toast.success("Report Saved");
        if (saved.openFailure) toast.warning("The saved report could not be opened.", { description: saved.openFailure });
      })
      .catch((error: unknown) =>
        toast.error("The report could not be saved.", {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };
  return (
    <Button
      aria-label={state.label}
      className="size-[var(--bar-control-height)]"
      disabled={state.disabled}
      onClick={save}
      size="icon"
      title={state.title}
      variant="outline"
    >
      <Download />
    </Button>
  );
}
