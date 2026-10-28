/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The Steps & Criteria section: what the execution did and what it judged.
 *
 * Two lists side by side on a wide viewport and stacked on a narrow one. Both
 * share one row grammar — ordinal, label, duration, status — on one column
 * template, so the status pills of both lists stand on the same column width
 * and every row's text shares a baseline with its pill. Every row keeps its
 * ordinal; a running step carries its spinner in its status pill and counts its
 * elapsed time up live, a pending one leaves the duration empty, an ended one
 * states the measured duration (`durationText`). A failure states its message
 * under the label, with the rest of its story a click away.
 */

import { LoaderCircle } from "lucide-react";

import { StepFailure, sortedStepRecords, stepBadgeVariant, stepRecordLabel } from "@/components/document/StepRecordsCard";
import { Badge } from "@/components/ui/badge";
import type { StepExecutionRecord } from "@/lib/types";
import { LIST_STATUS_PILL_CLASS } from "./cycle-workspace-model";
import { durationText, useElapsedSeconds } from "./use-elapsed";

export type ExecutionStepsCriteriaProps = {
  steps: StepExecutionRecord[];
  criteria: StepExecutionRecord[];
  /** False in a file: measured durations only, nothing ticks. */
  live?: boolean;
};

/** Ordinal, label, duration, status: the one row template of both lists. */
const ROW_COLUMNS = "grid-cols-(--layout-step-record)";

export function ExecutionStepsCriteria({ steps, criteria, live = true }: ExecutionStepsCriteriaProps) {
  return (
    <div className="grid min-w-0 gap-x-8 gap-y-4 lg:grid-cols-2">
      <RecordColumn empty="No steps recorded yet." live={live} records={steps} title="Steps" />
      <RecordColumn empty="No criteria judged yet." live={live} records={criteria} title="Criteria" />
    </div>
  );
}

function RecordColumn({ empty, live, records, title }: { empty: string; live: boolean; records: StepExecutionRecord[]; title: string }) {
  const ordered = sortedStepRecords(records);
  const ordinalWidth = Math.max(2, String(ordered.length).length);

  return (
    <section aria-label={title} className="grid min-w-0 content-start gap-2">
      <h3 className="grid grid-cols-[minmax(0,1fr)_auto] items-baseline gap-2 border-b pb-2 text-xs font-semibold text-muted-foreground uppercase">
        <span>{title}</span>
        <span className="tabular-nums">{ordered.length}</span>
      </h3>
      {ordered.length ? (
        <ol className={`grid min-w-0 ${ROW_COLUMNS} gap-x-3 gap-y-1`}>
          {ordered.map((record, index) => (
            <RecordRow
              key={`${record.record_type}:${record.step_id}:${record.order}`}
              live={live}
              ordinal={String(index + 1).padStart(ordinalWidth, "0")}
              record={record}
            />
          ))}
        </ol>
      ) : (
        <p className="text-sm text-muted-foreground">{empty}</p>
      )}
    </section>
  );
}

function RecordRow({ live, ordinal, record }: { live: boolean; ordinal: string; record: StepExecutionRecord }) {
  const running = record.status === "running";
  const elapsed = useElapsedSeconds(record.started_at, live && running && record.record_type === "step");
  const duration = record.record_type === "step" ? durationText(record, live, elapsed) : "";
  const message =
    record.record_type === "criterion" && typeof record.result.details.message === "string" ? record.result.details.message : null;

  return (
    <li className="col-span-4 grid min-w-0 grid-cols-subgrid items-baseline py-1 text-sm leading-5">
      <span className="text-xs text-muted-foreground tabular-nums">{ordinal}</span>
      <div className="min-w-0">
        <span className="block font-medium [overflow-wrap:break-word]">{stepRecordLabel(record, false)}</span>
        {message ? <span className="block text-xs text-muted-foreground [overflow-wrap:anywhere]">{message}</span> : null}
        {record.detail ? <StepFailure detail={record.detail} /> : null}
      </div>
      <span className="text-xs text-muted-foreground tabular-nums">{duration}</span>
      <Badge className={`${LIST_STATUS_PILL_CLASS} gap-1 capitalize`} variant={stepBadgeVariant(record.status)}>
        {running ? <LoaderCircle aria-hidden className="animate-spin" size={10} /> : null}
        {record.status}
      </Badge>
    </li>
  );
}
