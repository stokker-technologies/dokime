/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { ArtifactRecord, HealthItem, HubRecord, OperatorNote } from "@/lib/types";

/* === Types === */

export type EvidenceSummary = {
  artifacts: number;
  records: number;
  healthIssues: number;
  notes: number;
};

type EvidenceSummaryInput = {
  artifacts: ArtifactRecord[];
  records: HubRecord[];
  health: HealthItem[];
  notes: OperatorNote[];
};

/* === Public Functions === */

export function buildEvidenceSummary({
  artifacts,
  records,
  health,
  notes,
}: EvidenceSummaryInput): EvidenceSummary {
  return {
    artifacts: artifacts.length,
    records: records.length,
    healthIssues: health.filter((item) => item.level !== "info").length,
    notes: notes.length,
  };
}
