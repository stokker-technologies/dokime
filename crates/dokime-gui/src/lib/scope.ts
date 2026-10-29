/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { RecordScope } from "@/lib/types";

/** Every selected owner must be present and equal in the record. */
export function scopeVisibleIn(scope: RecordScope | null | undefined, filter: RecordScope): boolean {
  return (
    fieldVisibleIn(scope?.session_id, filter.session_id) &&
    fieldVisibleIn(scope?.test_cycle_id, filter.test_cycle_id) &&
    fieldVisibleIn(scope?.test_case_id, filter.test_case_id) &&
    fieldVisibleIn(scope?.test_execution_id, filter.test_execution_id) &&
    fieldVisibleIn(scope?.step_id, filter.step_id) &&
    fieldVisibleIn(scope?.criterion_id, filter.criterion_id) &&
    fieldVisibleIn(scope?.template_id, filter.template_id) &&
    fieldVisibleIn(scope?.suite_id, filter.suite_id)
  );
}

function fieldVisibleIn(record: string | null | undefined, selected: string | null | undefined): boolean {
  return selected == null || record === selected;
}
