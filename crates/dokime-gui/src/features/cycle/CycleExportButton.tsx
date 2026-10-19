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
import { FileArchive } from "lucide-react";
import { toast } from "sonner";

// Local Imports
import { Button } from "@/components/ui/button";
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
export function CycleExportButton({ cycleId, executionId, sealed }: Props) {
  const { exportEvidence } = useCycleCommands(cycleId);
  const title = sealed
    ? "Download SQLite, Files And Saved Reports"
    : executionId
      ? "Wait For This Execution To Seal"
      : "Seal This Cycle Before Exporting";
  const save = () => {
    exportEvidence
      .mutateAsync({ executionId })
      .then((saved) => {
        if (saved) toast.success("Evidence Saved");
      })
      .catch((error: unknown) =>
        toast.error("Evidence Could Not Be Saved", {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };
  return (
    <Button disabled={!sealed || exportEvidence.isPending} onClick={save} title={title} variant="outline">
      <FileArchive />
      {exportEvidence.isPending ? "Exporting Evidence…" : "Export Evidence"}
    </Button>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
