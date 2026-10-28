/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { ArtifactRecord, HealthItem, HubRecord, OperatorNote } from "@/lib/types";

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

export function buildEvidenceSummary({ artifacts, records, health, notes }: EvidenceSummaryInput): EvidenceSummary {
  return {
    artifacts: artifacts.length,
    records: records.length,
    healthIssues: health.filter((item) => item.level !== "info").length,
    notes: notes.length,
  };
}
