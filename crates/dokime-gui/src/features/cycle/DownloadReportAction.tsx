/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Download Report: a labelled button beside Export Evidence, in the same
 * style, in the action cluster of the cycle and the execution viewer.
 *
 * It creates one frozen report and saves its retained bytes through the native
 * dialog, named after the cycle or execution, then
 * opens the saved file in the default browser once the evidence is sealed.
 */

/* === External Imports === */

import { failureOf } from "@dokime/plots/lib/notification-details";
import { Download } from "lucide-react";
import { toast } from "@dokime/plots/lib/notifications";

/* === Local Imports === */

import { DropdownMenuItem } from "@dokime/plots/components/ui/dropdown-menu";
import { Button } from "@dokime/plots/components/ui/button";
import { useCycleCommands } from "./cycle-queries";
import { reportDownloadState } from "@dokime/document/features/cycle/cycle-workspace-model";

/* === Types === */

export type DownloadReportActionProps = {
  cycleId: string;
  /** The execution whose report is saved; `null` saves the cycle's. */
  executionId: string | null;
  /** The cycle or execution label the file is named after. */
  label: string;
  sealed: boolean;
  presentation?: "menu" | "button";
};

/* === Components === */

export function DownloadReportAction({ cycleId, executionId, label, sealed, presentation = "menu" }: DownloadReportActionProps) {
  const { downloadReport } = useCycleCommands(cycleId);
  const state = reportDownloadState(downloadReport.isPending, sealed);
  const save = () => {
    if (state.disabled) {
      return;
    }
    downloadReport
      .mutateAsync({ executionId, label })
      .then((saved) => {
        if (!saved) {
          return;
        }
        toast.success("Report Saved");
        if (saved.openFailure) {
          toast.error(saved.openFailure);
        }
      })
      .catch((error: unknown) => toast.error(failureOf(error, "The Report Could Not Be Saved")));
  };
  const content = (
    <>
      <Download data-icon="inline-start" />
      {downloadReport.isPending ? "Saving Report…" : state.label}
    </>
  );
  if (presentation === "button") {
    return (
      <Button disabled={state.disabled} onClick={save} size="sm" title={state.title} variant="outline">
        {content}
      </Button>
    );
  }
  return (
    <DropdownMenuItem disabled={state.disabled} onSelect={save} title={state.title}>
      {content}
    </DropdownMenuItem>
  );
}
