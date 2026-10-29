/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * A cycle state pill sized to the longest state label.
 */

import { Badge } from "@/components/ui/badge";
import { CYCLE_STATE_LABELS, STATUS_PILL_CLASS } from "@/features/cycle/cycle-workspace-model";
import type { CycleRow } from "@/features/cycles-home/cycles-model";

/**
 * A cycle's state pill, as wide as the longest state label wherever it sits.
 *
 * Every label stacks invisibly in one grid cell under the visible pill, so the
 * track is the widest label's max-content in every row and every list: no
 * label is clipped and no column shifts with the state it shows.
 */
export function CycleStatusPill({ label, tone }: { label: string; tone: CycleRow["statusTone"] }) {
  return (
    <span className="inline-grid align-middle" data-slot="cycle-status">
      {CYCLE_STATE_LABELS.map((sizer) => (
        <Badge aria-hidden className={`${STATUS_PILL_CLASS} invisible [grid-area:1/1]`} data-sizer="" key={sizer} variant="outline">
          {sizer}
        </Badge>
      ))}
      <Badge className={`${STATUS_PILL_CLASS} w-full [grid-area:1/1]`} variant={toneVariant(tone)}>
        {label}
      </Badge>
    </span>
  );
}

function toneVariant(tone: CycleRow["statusTone"]) {
  switch (tone) {
    case "good":
      return "success" as const;
    case "warn":
      return "warning" as const;
    case "bad":
      return "destructive" as const;
    default:
      return "outline" as const;
  }
}
