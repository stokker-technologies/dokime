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
 * and every row's text shares a baseline with its pill. A running step shows
 * its spinner in the ordinal slot; a failure states its message under the
 * label, with the rest of its story a click away.
 */

/* === External Imports === */

import { LoaderCircle } from "lucide-react";

/* === Local Imports === */

import { StepFailure, sortedStepRecords, stepBadgeVariant, stepRecordLabel } from "@/components/document/StepRecordsCard";
import { Badge } from "@/components/ui/badge";
import { formatDuration } from "@/lib/format";
import type { StepExecutionRecord } from "@/lib/types";
import { LIST_STATUS_PILL_CLASS } from "./cycle-workspace-model";

/* === Types === */

export type ExecutionStepsCriteriaProps = {
  steps: StepExecutionRecord[];
  criteria: StepExecutionRecord[];
};

/* === Constants === */

/** Ordinal, label, duration, status: the one row template of both lists. */
const ROW_COLUMNS = "grid-cols-[1.75rem_minmax(0,1fr)_auto_var(--pill-width)]";

/* === Components === */

export function ExecutionStepsCriteria({ steps, criteria }: ExecutionStepsCriteriaProps) {
  return (
    <div className="grid min-w-0 gap-x-8 gap-y-4 lg:grid-cols-2">
      <RecordColumn empty="No steps recorded yet." records={steps} title="Steps" />
      <RecordColumn empty="No criteria judged yet." records={criteria} title="Criteria" />
    </div>
  );
}

/* === Private Components === */

function RecordColumn({ empty, records, title }: { empty: string; records: StepExecutionRecord[]; title: string }) {
  const ordered = sortedStepRecords(records);
  const ordinalWidth = Math.max(2, String(ordered.length).length);

  return (
    <section aria-label={title} className="grid min-w-0 content-start gap-2">
      <h3 className="grid grid-cols-[minmax(0,1fr)_auto] items-baseline gap-2 border-b pb-1.5 text-xs font-semibold text-muted-foreground uppercase">
        <span>{title}</span>
        <span className="tabular-nums">{ordered.length}</span>
      </h3>
      {ordered.length ? (
        <ol className={`grid min-w-0 ${ROW_COLUMNS} gap-x-3 gap-y-1`}>
          {ordered.map((record, index) => (
            <RecordRow
              key={`${record.record_type}:${record.step_id}:${record.order}`}
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

function RecordRow({ ordinal, record }: { ordinal: string; record: StepExecutionRecord }) {
  const duration = record.record_type === "step" ? formatDuration(record.duration_s) : "";
  const message =
    record.record_type === "criterion" && typeof record.result.details.message === "string" ? record.result.details.message : null;

  return (
    <li className="col-span-4 grid min-w-0 grid-cols-subgrid items-baseline py-1 text-[0.8125rem] leading-5">
      <span className="text-xs text-muted-foreground tabular-nums">
        {record.status === "running" ? <LoaderCircle aria-label="Running" className="inline animate-spin" size={12} /> : ordinal}
      </span>
      <div className="min-w-0">
        <span className="block font-medium [overflow-wrap:break-word]">{stepRecordLabel(record, false)}</span>
        {message ? <span className="block text-xs text-muted-foreground [overflow-wrap:anywhere]">{message}</span> : null}
        {record.detail ? <StepFailure detail={record.detail} /> : null}
      </div>
      <span className="text-xs text-muted-foreground tabular-nums">{duration}</span>
      <Badge className={`${LIST_STATUS_PILL_CLASS} capitalize`} variant={stepBadgeVariant(record.status)}>
        {record.status}
      </Badge>
    </li>
  );
}
