/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { LoaderCircle } from "lucide-react";
import type { ComponentProps } from "react";

/* === Local Imports === */

import { DocumentEmbeddedBlock, DocumentEmbeddedEmptyState } from "@/components/document/DocumentEmbeddedContent";
import { Badge } from "@/components/ui/badge";
import { ErrorCard } from "@/components/ErrorCard";
import { failureOf } from "@/lib/notification-details";
import { formatDuration } from "@/lib/format";
import type { StepExecutionRecord, StepFailureDetail } from "@/lib/types";

/* === Types === */

type BadgeVariant = ComponentProps<typeof Badge>["variant"];

/* === Constants === */

const RECORD_TYPE_ORDER: Record<StepExecutionRecord["record_type"], number> = {
  step: 0,
  criterion: 1,
  initialize: 2,
  finalize: 3,
  interrupt: 4,
};

/* === Components === */

export function StepRecordList({
  records,
  emptyMessage,
  maxItems,
}: {
  records: StepExecutionRecord[];
  emptyMessage: string;
  maxItems?: number | null;
}) {
  const sortedRecords = sortedStepRecords(records);
  const visibleRecords = maxItems === null || maxItems === undefined ? sortedRecords : sortedRecords.slice(0, Math.max(0, maxItems));
  const orderWidth = Math.max(2, String(visibleRecords.length).length);
  const showRecordType = hasMixedRecordTypes(visibleRecords);

  if (!visibleRecords.length) {
    return <DocumentEmbeddedEmptyState dashed>{emptyMessage}</DocumentEmbeddedEmptyState>;
  }

  return (
    <ol className="grid gap-2">
      {visibleRecords.map((record, index) => {
        // Only a measured duration is printed: a pending or running step has none yet.
        const duration = record.status === "pending" || record.status === "running" ? null : formatDuration(record.duration_s);
        return (
          <DocumentEmbeddedBlock
            as="li"
            className="grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 pr-1 py-2 text-sm"
            key={`${record.test_execution_id}:${record.record_type}:${record.step_id}:${record.order}:${record.started_at}`}
          >
            <span className="grid size-7 shrink-0 place-items-center rounded-lg border bg-muted/40 text-xs font-semibold tabular-nums text-muted-foreground">
              {String(index + 1).padStart(orderWidth, "0")}
            </span>
            <div className="min-w-0">
              <span className="block min-w-0 font-medium leading-tight [overflow-wrap:break-word]">
                {stepRecordLabel(record, showRecordType)}
              </span>
              {duration ? <span className="mt-1 block text-xs leading-tight text-muted-foreground">{duration}</span> : null}
              {record.detail ? <ErrorCard failure={stepFailure(record.detail)} /> : null}
            </div>
            <Badge className="justify-self-end gap-1 text-xs capitalize" variant={stepBadgeVariant(record.status)}>
              {record.status === "running" ? <LoaderCircle aria-hidden className="animate-spin" size={10} /> : null}
              {record.status}
            </Badge>
          </DocumentEmbeddedBlock>
        );
      })}
    </ol>
  );
}

/* === Public Functions === */

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

/* === Private Functions === */

function sortableTimestamp(value: string): number {
  const timestamp = Date.parse(value);
  return Number.isFinite(timestamp) ? timestamp : Number.POSITIVE_INFINITY;
}

function hasMixedRecordTypes(records: StepExecutionRecord[]): boolean {
  return new Set(records.map((record) => record.record_type)).size > 1;
}
