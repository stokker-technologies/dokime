/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The Cycle Container Has One Finalization Action */

/* === Imports === */

// Third Party Imports
import { LockKeyhole } from "lucide-react";

// Local Imports
import { Button } from "@/components/ui/button";
import { useCycleDetailQuery } from "./cycle-queries";

/* === Constants === */

export const PAGE_BAR_CONTROL_CLASS = "h-[var(--bar-control-height)] px-3";
export const PAGE_BAR_CONTROLS_CLASS = "grid auto-cols-max grid-flow-col items-center gap-x-[var(--bar-gap)]";

/* === Types === */

export type CycleRunControlsProps = { cycleId: string; onSeal: () => void };

/* === Components === */

export function CycleRunControls({ cycleId, onSeal }: CycleRunControlsProps) {
  const detailQuery = useCycleDetailQuery(cycleId);
  const plan = detailQuery.data?.plan;
  if (!plan) return null;
  if (plan.test_cycle.phase === "sealed") return <span role="status">Sealed</span>;
  if (plan.test_cycle.phase === "sealing") return <span role="status">Sealing — Saving Evidence</span>;

  return (
    <fieldset aria-label="Cycle Controls" className={PAGE_BAR_CONTROLS_CLASS}>
      <Button className={PAGE_BAR_CONTROL_CLASS} onClick={onSeal} variant="outline">
        <LockKeyhole data-icon="inline-start" />
        Seal Cycle
      </Button>
    </fieldset>
  );
}

/* === Functions === */
