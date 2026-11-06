/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Export Sealed Evidence Through Native Verified Delivery
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { failureOf } from "@/lib/notification-details";
import { FileArchive } from "lucide-react";
// Local Imports
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { toast } from "@/lib/notifications";
import { useCycleCommands } from "./cycle-queries";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */
type Props = {
  cycleId: string;
  executionId: string | null;
  sealed: boolean;
};

/* === Components === */
export function CycleExportMenuItem({ cycleId, executionId, sealed }: Props) {
  const { exportEvidence } = useCycleCommands(cycleId);
  const title = sealed
    ? "Download SQLite, Files and Saved Reports"
    : executionId
      ? "Wait for This Execution to Seal"
      : "Seal This Cycle Before Exporting";
  const save = () => {
    exportEvidence
      .mutateAsync({ executionId })
      .then((saved) => {
        if (saved) toast.success("Evidence Saved");
      })
      .catch((error: unknown) => toast.error(failureOf(error, "Evidence Could Not Be Saved")));
  };
  return (
    <DropdownMenuItem disabled={!sealed || exportEvidence.isPending} onSelect={save} title={title}>
      <FileArchive data-icon="inline-start" />
      {exportEvidence.isPending ? "Exporting Evidence…" : "Export Evidence"}
    </DropdownMenuItem>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
