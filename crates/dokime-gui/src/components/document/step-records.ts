// Dokime — SPDX-License-Identifier: Apache-2.0
import type { VariantProps } from "class-variance-authority";
import { failureOf } from "@/lib/notification-details";
import type { StepExecutionRecord, StepFailureDetail } from "@/lib/types";
import type { badgeVariants } from "../ui/badge-variants";

type BadgeVariant = VariantProps<typeof badgeVariants>["variant"];

const RECORD_TYPE_ORDER: Record<StepExecutionRecord["record_type"], number> = {
  step: 0,
  criterion: 1,
  initialize: 2,
  finalize: 3,
  interrupt: 4,
};

export function stepFailure(detail: StepFailureDetail) {
  return {
    ...failureOf(detail, "Execution Failed"),
    causes: detail.chain,
    diagnostics: [detail.location, ...detail.span_trace, detail.backtrace].filter(Boolean).join("\n"),
  };
}

export function sortedStepRecords(records: StepExecutionRecord[]): StepExecutionRecord[] {
  const executionStarts = new Map<string, number>();
  for (const record of records) {
    const started = sortableTimestamp(record.started_at);
    executionStarts.set(record.test_execution_id, Math.min(executionStarts.get(record.test_execution_id) ?? started, started));
  }
  return [...records].sort((left, right) => {
    if (left.test_execution_id !== right.test_execution_id) {
      const leftStart = executionStarts.get(left.test_execution_id) ?? Number.POSITIVE_INFINITY;
      const rightStart = executionStarts.get(right.test_execution_id) ?? Number.POSITIVE_INFINITY;
      if (leftStart !== rightStart) {
        return leftStart < rightStart ? -1 : 1;
      }
      return left.test_execution_id.localeCompare(right.test_execution_id);
    }

    const orderDelta = left.order - right.order;
    if (orderDelta !== 0) {
      return orderDelta;
    }

    const recordTypeDelta = RECORD_TYPE_ORDER[left.record_type] - RECORD_TYPE_ORDER[right.record_type];
    if (recordTypeDelta !== 0) {
      return recordTypeDelta;
    }

    return left.step_id.localeCompare(right.step_id);
  });
}

export function stepBadgeVariant(status: StepExecutionRecord["status"]): BadgeVariant {
  if (status === "passed") {
    return "success";
  }
  if (status === "failed") {
    return "destructive";
  }
  // Its own register: not the green of a pass, not the red of a measured
  // failure, and not the grey of something never attempted.
  if (status === "indeterminate") {
    return "warning";
  }
  return "secondary";
}

export function stepRecordLabel(record: StepExecutionRecord, showRecordType: boolean): string {
  const label =
    typeof record.metadata.label === "string" && record.metadata.label.trim()
      ? record.metadata.label
      : record.method_name || record.step_id;
  if (!showRecordType || record.record_type === "step") {
    return label;
  }
  return `${record.record_type}: ${label}`;
}

function sortableTimestamp(value: string): number {
  const timestamp = Date.parse(value);
  return Number.isFinite(timestamp) ? timestamp : Number.POSITIVE_INFINITY;
}
