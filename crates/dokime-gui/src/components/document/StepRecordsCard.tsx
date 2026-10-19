/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { ChevronDown, ListChecks, LoaderCircle } from "lucide-react";
import { type ComponentProps, useState } from "react";

/* === Local Imports === */

import { DocumentCard } from "@/components/document/DocumentCard";
import { DocumentEmbeddedBlock, DocumentEmbeddedEmptyState } from "@/components/document/DocumentEmbeddedContent";
import { Badge } from "@/components/ui/badge";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { formatDuration } from "@/lib/format";
import type { DocumentCardLayout, StepExecutionRecord, StepFailureDetail } from "@/lib/types";

/* === Types === */

type StepRecordsCardProps = {
  records: StepExecutionRecord[];
  emptyMessage?: string;
  eyebrow?: string;
  layout?: DocumentCardLayout;
  maxItems?: number | null;
  title?: string;
};

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

export function StepRecordsCard({
  records,
  emptyMessage = "No step records yet.",
  eyebrow = "Step",
  layout,
  maxItems,
  title = "Steps",
}: StepRecordsCardProps) {
  return (
    <DocumentCard contentClassName="p-3" eyebrow={eyebrow} icon={<ListChecks size={20} />} layout={layout} title={title}>
      <StepRecordList records={records} emptyMessage={emptyMessage} maxItems={maxItems} />
    </DocumentCard>
  );
}

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
        const duration = formatDuration(record.duration_s);
        return (
          <DocumentEmbeddedBlock
            as="li"
            className="grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 pr-0.5 py-1.5 text-[0.8125rem]"
            key={`${record.test_execution_id}:${record.record_type}:${record.step_id}:${record.order}:${record.started_at}`}
          >
            <span className="grid size-7 shrink-0 place-items-center rounded-md border bg-muted/40 text-xs font-semibold tabular-nums text-muted-foreground">
              {record.status === "running" ? (
                <LoaderCircle className="animate-spin" size={14} />
              ) : (
                String(index + 1).padStart(orderWidth, "0")
              )}
            </span>
            <div className="min-w-0">
              <span className="block min-w-0 font-medium leading-tight [overflow-wrap:break-word]">
                {stepRecordLabel(record, showRecordType)}
              </span>
              {duration ? <span className="mt-0.5 block text-[0.72rem] leading-tight text-muted-foreground">{duration}</span> : null}
              {record.detail ? <StepFailure detail={record.detail} /> : null}
            </div>
            <Badge className="justify-self-end text-[0.72rem] capitalize" variant={stepBadgeVariant(record.status)}>
              {record.status}
            </Badge>
          </DocumentEmbeddedBlock>
        );
      })}
    </ol>
  );
}

/**
 * Why one step failed: the line inline, the rest a click away.
 *
 * The message is what an operator has to read to know what went wrong, so it is
 * never hidden. The cause chain, the location, the spans and the frames are what
 * a maintainer needs and nobody else does, so they sit behind the expander.
 */
export function StepFailure({ detail }: { detail: StepFailureDetail }) {
  const [open, setOpen] = useState(false);
  const hasStory = detail.chain.length > 0 || detail.location.length > 0 || detail.span_trace.length > 0 || detail.backtrace.length > 0;

  return (
    <Collapsible onOpenChange={setOpen} open={open}>
      <p className="mt-1 text-[0.72rem] leading-snug text-destructive [overflow-wrap:anywhere]">{detail.message}</p>
      {hasStory ? (
        <>
          <CollapsibleTrigger className="mt-1 inline-flex items-center gap-1 text-[0.7rem] text-muted-foreground underline-offset-2 hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
            <ChevronDown className={open ? "rotate-180 transition-transform" : "transition-transform"} size={12} />
            {open ? "Hide details" : "Show details"}
          </CollapsibleTrigger>
          <CollapsibleContent>
            <div className="mt-1.5 grid gap-1.5 rounded-md border bg-muted/30 p-2 text-[0.7rem] leading-snug">
              {detail.chain.length ? (
                <span className="text-muted-foreground [overflow-wrap:anywhere] whitespace-pre-line">
                  {detail.chain.map((cause) => `caused by: ${cause}`).join("\n")}
                </span>
              ) : null}
              {detail.location ? <span className="text-muted-foreground [overflow-wrap:anywhere]">at {detail.location}</span> : null}
              {detail.span_trace.length ? (
                <span className="text-muted-foreground [overflow-wrap:anywhere] whitespace-pre-line">
                  {detail.span_trace.map((span) => `in ${span}`).join("\n")}
                </span>
              ) : null}
              {detail.backtrace ? (
                <pre className="max-h-64 overflow-auto whitespace-pre-wrap break-all text-[0.68rem] text-muted-foreground">
                  {detail.backtrace}
                </pre>
              ) : null}
            </div>
          </CollapsibleContent>
        </>
      ) : null}
    </Collapsible>
  );
}

/* === Public Functions === */

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
