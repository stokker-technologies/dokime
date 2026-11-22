/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { EvidenceScope } from "@generated/host/model";
import type { Wire } from "@dokime/plots/lib/rpc/canonical";

/* === Types === */

/* === Public Functions === */

/** Every selected owner must be present and equal in the record. */
export function scopeVisibleIn(scope: Wire<EvidenceScope> | null | undefined, filter: Wire<EvidenceScope>): boolean {
  return (
    fieldVisibleIn(scope?.sessionId, filter.sessionId) &&
    fieldVisibleIn(scope?.cycleId, filter.cycleId) &&
    fieldVisibleIn(scope?.caseId, filter.caseId) &&
    fieldVisibleIn(scope?.executionId, filter.executionId) &&
    fieldVisibleIn(scope?.stepId, filter.stepId) &&
    fieldVisibleIn(scope?.criterionId, filter.criterionId) &&
    fieldVisibleIn(scope?.templateId, filter.templateId) &&
    fieldVisibleIn(scope?.suiteId, filter.suiteId)
  );
}

/* === Private Functions === */

function fieldVisibleIn(record: string | null | undefined, selected: string | null | undefined): boolean {
  return selected == null || record === selected;
}
