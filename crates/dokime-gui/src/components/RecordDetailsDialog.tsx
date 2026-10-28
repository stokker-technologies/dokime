/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { AlertTriangle, Braces, Clipboard, Clock } from "lucide-react";

import { RecordGlyph } from "@/components/RecordGlyph";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { ValueDisplay } from "@/components/ValueDisplay";
import { recordTime, recordTitle } from "@/lib/dashboard";
import { recordToneShellClass } from "@/lib/record-display";
import { absoluteTimestampLabel } from "@/lib/time-format";
import type { HubRecord } from "@/lib/types";
import { isRecord } from "@/lib/utils";

type RecordDetailsDialogProps = {
  selected: HubRecord | null;
  onClose: () => void;
};

const STACKTRACE_KEYS = new Set(["stack", "stacktrace", "stack_trace", "traceback", "trace", "exception_stack", "error_stack"]);

export function RecordDetailsDialog({ selected, onClose }: RecordDetailsDialogProps) {
  const record = selected;
  if (!record) {
    return null;
  }

  const stacktrace = recordStacktrace(record);
  const rawRecord = JSON.stringify(record, null, 2);
  const title = recordTitle(record);
  const detailEntries = Object.entries(record.details).filter(([key]) => !STACKTRACE_KEYS.has(key.toLowerCase()));

  return (
    <Dialog open onOpenChange={(open) => (open ? undefined : onClose())}>
      <DialogContent className="grid max-h-[92vh] grid-rows-[auto_minmax(0,1fr)] gap-0 overflow-hidden p-0 sm:max-w-4xl">
        <header className="grid gap-3 border-b p-4">
          <div className="grid min-w-0 grid-cols-[auto_minmax(0,1fr)] gap-3">
            <RecordGlyph className="mt-1" display={record.display} />
            <div className="min-w-0">
              <div className="mb-2 flex flex-wrap items-center gap-2">
                <Badge
                  className={recordToneShellClass(record.display.tone)}
                  variant={record.level === "error" ? "destructive" : "secondary"}
                >
                  {record.level}
                </Badge>
                <span className="inline-flex items-center gap-2 text-xs text-muted-foreground">
                  <Clock size={13} />
                  {recordTime(record)}
                </span>
                {record.sequence !== undefined && record.sequence !== null ? (
                  <span className="text-xs text-muted-foreground">#{record.sequence}</span>
                ) : null}
              </div>
              <DialogTitle className="text-xl font-semibold leading-tight [overflow-wrap:anywhere]">{title}</DialogTitle>
              <p className="mt-1 text-sm text-muted-foreground [overflow-wrap:anywhere]">{record.record_type}</p>
            </div>
          </div>
        </header>

        <div className="min-h-0 overflow-y-auto p-4">
          <div className="grid min-w-0 gap-4">
            {record.level === "error" ? (
              <div className="flex min-w-0 gap-2 rounded-lg border border-destructive/25 bg-destructive/5 p-3 text-sm text-destructive">
                <AlertTriangle className="mt-1 shrink-0" size={16} />
                <p className="[overflow-wrap:anywhere]">{title}</p>
              </div>
            ) : null}

            <dl className="grid gap-2 text-sm sm:grid-cols-2 xl:grid-cols-4">
              <RecordMeta label="Source" value={record.source} />
              <RecordMeta label="Timestamp" value={absoluteTimestampLabel(record.timestamp)} />
              <RecordMeta label="Session" value={record.session_id ?? "-"} />
              <RecordMeta label="Record ID" value={record.record_id} />
              <RecordMeta label="Cycle" value={record.scope?.test_cycle_id ?? "-"} />
              <RecordMeta label="Execution" value={record.scope?.test_execution_id ?? "-"} />
              <RecordMeta label="Case" value={record.scope?.test_case_id ?? "-"} />
            </dl>

            {detailEntries.length ? (
              <section className="grid gap-2">
                <h3 className="text-sm font-semibold">Details</h3>
                <dl className="grid gap-2 text-sm sm:grid-cols-2">
                  {detailEntries.map(([key, value]) => (
                    <RecordMeta key={key} label={key} rawValue={value} value={formatDetailValue(value)} />
                  ))}
                </dl>
              </section>
            ) : null}

            {stacktrace ? <RawBlock label="Stacktrace" value={stacktrace} /> : null}
            <RawBlock label="Raw Record" value={rawRecord} />
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function RecordMeta({ label, rawValue, value }: { label: string; rawValue?: unknown; value: string }) {
  return (
    <div className="grid min-w-0 gap-2 rounded-lg border bg-muted/20 p-2">
      <dt className="truncate text-xs font-semibold uppercase text-muted-foreground">{label}</dt>
      <dd className="text-sm [overflow-wrap:anywhere]">
        <ValueDisplay label={label} value={rawValue ?? value} />
      </dd>
    </div>
  );
}

function RawBlock({ label, value }: { label: string; value: string }) {
  async function copyValue() {
    await navigator.clipboard?.writeText(value);
  }

  return (
    <section className="grid min-w-0 gap-2">
      <div className="flex items-center justify-between gap-3">
        <h3 className="inline-flex min-w-0 items-center gap-2 text-sm font-semibold">
          <Braces className="shrink-0" size={15} />
          <span className="truncate">{label}</span>
        </h3>
        <Button aria-label={`Copy ${label.toLowerCase()}`} size="sm" variant="outline" onClick={() => void copyValue()}>
          <Clipboard size={14} />
          Copy
        </Button>
      </div>
      <pre className="max-h-[min(--spacing(96),38vh)] min-h-32 max-w-full overflow-auto rounded-lg border bg-muted/35 p-3 text-xs leading-relaxed">
        <code>{value}</code>
      </pre>
    </section>
  );
}

function recordStacktrace(record: HubRecord): string | null {
  return findStacktrace(record.details) ?? findStacktrace({ message: record.message });
}

function findStacktrace(value: unknown): string | null {
  if (typeof value === "string") {
    return looksLikeStacktrace(value) ? value : null;
  }
  if (Array.isArray(value)) {
    for (const item of value) {
      const nested = findStacktrace(item);
      if (nested) {
        return nested;
      }
    }
    return null;
  }
  if (!isRecord(value)) {
    return null;
  }
  for (const [key, nestedValue] of Object.entries(value)) {
    if (STACKTRACE_KEYS.has(key.toLowerCase())) {
      return formatDetailValue(nestedValue);
    }
    const nested = findStacktrace(nestedValue);
    if (nested) {
      return nested;
    }
  }
  return null;
}

function looksLikeStacktrace(value: string): boolean {
  const lowered = value.toLowerCase();
  return (
    lowered.includes("traceback") || lowered.includes("stack trace") || lowered.includes("\n    at ") || lowered.includes('\n  file "')
  );
}

function formatDetailValue(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  if (value === null || value === undefined) {
    return "-";
  }
  return JSON.stringify(value, null, 2);
}
