/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The Records section both viewers close their evidence with: the node's
 * records in the same card, the same list and the same empty state, live and
 * frozen alike.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import { RecordListContent } from "@/components/document/DocumentSurfaceView";
import type { HubRecord } from "@/lib/types";
import { ExecutionSectionCard } from "./ExecutionSectionCard";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type RecordsSectionProps = {
  records: HubRecord[];
  live: boolean;
};

/* === Components === */

export function RecordsSection({ live, records }: RecordsSectionProps) {
  return (
    <ExecutionSectionCard empty={records.length ? null : "No records yet."} title="Records">
      {records.length ? <RecordListContent live={live} records={records} /> : null}
    </ExecutionSectionCard>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
