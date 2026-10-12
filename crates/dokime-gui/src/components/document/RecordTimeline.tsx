/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The captured records of one scope, newest first.
 *
 * One dense grid in both modes — `time | level | source | message` with the
 * record type muted at the row's end — the same row grammar the console
 * drawer uses, because a record is a record wherever it is read. Live, the
 * toolbar narrows a list that is still growing. In a frozen report there is
 * nothing to narrow towards and nothing more coming, so the search field and
 * the type select — controls that read as live — are absent and the records
 * are simply the records the file holds.
 */

/* === External Imports === */

import { Search } from "lucide-react";
import { useMemo, useState } from "react";

/* === Local Imports === */

import { RecordDetailsDialog } from "@/components/RecordDetailsDialog";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { formatValue, recordMessage, recordTitle } from "@/lib/dashboard";
import {
  recordClockLabel,
  recordLevelLabel,
  recordLevelVariant,
  recordToneRowClass,
} from "@/lib/record-display";
import { absoluteTimestampLabel } from "@/lib/time-format";
import type { HubRecord } from "@/lib/types";
import { cn } from "@/lib/utils";

/* === Components === */

export function RecordTimeline({
  live = true,
  records,
}: {
  /** False in the frozen report: no search field, no type select. */
  live?: boolean;
  records: HubRecord[];
}) {
  const [recordType, setRecordType] = useState("all");
  const [query, setQuery] = useState("");
  const [expandedRecordId, setExpandedRecordId] = useState<string | null>(null);
  const [selectedRecord, setSelectedRecord] = useState<HubRecord | null>(null);
  const recordSearchId = "record-timeline-search";
  const recordTypeId = "record-timeline-type";

  const recordTypes = useMemo(
    () =>
      [...new Set(records.map((record) => record.record_type))]
        .filter((value) => value.trim().length > 0)
        .sort((left, right) => left.localeCompare(right)),
    [records],
  );

  const visibleRecords = useMemo(() => {
    // Escape regex syntax: only * and ? are wildcards in the search textbox.
    const pattern = query
      .trim()
      .split(/([*?])/)
      .map((part) => {
        if (part === "*") return ".*";
        if (part === "?") return ".";

        return part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      })
      .join("");
    const matcher = pattern ? new RegExp(pattern, "is") : null;

    return [...records]
      .sort(compareRecordTimelineOrder)
      .filter((record) => recordType === "all" || record.record_type === recordType)
      .filter((record) => {
        if (!matcher) {
          return true;
        }
        return [
          record.record_type,
          record.source,
          recordTitle(record),
          recordMessage(record),
          stringifyRecordDetails(record.details),
        ].some((value) => matcher.test(value));
      });
  }, [recordType, records, query]);

  return (
    <>
      <div className="grid gap-4">
        {/* A plain toolbar, not a second card: the Records card already owns
            the only border around this filter row. */}
        {live ? (
          <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_16rem]">
            <div className="grid gap-1.5 text-sm">
              <Label
                className="text-xs font-bold uppercase tracking-[0.14em] text-muted-foreground"
                htmlFor={recordSearchId}
              >
                Search Records
              </Label>
              <div className="relative">
                <Search
                  className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
                  size={14}
                />
                <Input
                  id={recordSearchId}
                  className="pl-9"
                  placeholder="Search type, source, message, or details; * and ? supported"
                  value={query}
                  onChange={(record) => setQuery(record.target.value)}
                />
              </div>
              <span className="text-xs text-muted-foreground">
                Leave blank for all records. Use * for any text or ? for one character.
              </span>
            </div>
            <div className="grid gap-1.5 text-sm">
              <Label
                className="text-xs font-bold uppercase tracking-[0.14em] text-muted-foreground"
                htmlFor={recordTypeId}
              >
                Record Type
              </Label>
              <Select value={recordType} onValueChange={setRecordType}>
                <SelectTrigger className="bg-background" id={recordTypeId}>
                  <SelectValue placeholder="All records" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All records</SelectItem>
                  {recordTypes.map((value) => (
                    <SelectItem key={value} value={value}>
                      {value}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
        ) : null}

        <div className="flex flex-wrap items-center justify-between gap-2 border-b pb-2 text-sm text-muted-foreground">
          <span>
            {live
              ? `Showing ${visibleRecords.length} of ${records.length} records.`
              : `${records.length} ${records.length === 1 ? "record" : "records"}.`}
          </span>
          <span>The timeline is ordered newest first.</span>
        </div>

        {visibleRecords.length ? (
          /* One dense grid, live and frozen: time, level, source, message, and
             the technical record type where it belongs — at the row's end,
             muted, for the reader who needs it. The message is printed once;
             expanding a row reveals the details behind it. */
          /* The Records card already owns the only border around this list;
             a second one here would be a card inside a card. */
          <ol className="grid border-t" data-testid="record-rows">
            {visibleRecords.map((record) => {
              const expanded = expandedRecordId === record.record_id;
              return (
                <li
                  className={cn(
                    "border-b border-l-2 border-b-border/60 last:border-b-0",
                    recordToneRowClass(record.display.tone),
                  )}
                  key={record.record_id}
                >
                  <button
                    aria-expanded={expanded}
                    className={cn(
                      "grid w-full grid-cols-[6rem_5.5rem_10rem_minmax(0,1fr)_auto] items-start gap-3 px-3 py-1.5 text-left font-mono text-xs tabular-nums hover:bg-muted/40",
                    )}
                    onClick={() =>
                      setExpandedRecordId((current) =>
                        current === record.record_id ? null : record.record_id,
                      )
                    }
                    type="button"
                  >
                    <time className="truncate text-muted-foreground" dateTime={record.timestamp}>
                      {recordClockLabel(record.timestamp)}
                    </time>
                    <Badge
                      className="h-4 justify-self-start px-1 text-[0.65rem]"
                      variant={recordLevelVariant(record.level)}
                    >
                      {recordLevelLabel(record.level)}
                    </Badge>
                    <span className="truncate text-muted-foreground">{record.source}</span>
                    <span
                      className={cn(
                        "min-w-0",
                        expanded ? "break-words whitespace-pre-wrap" : "truncate",
                      )}
                    >
                      {recordLine(record)}
                    </span>
                    <span className="truncate text-muted-foreground">{record.record_type}</span>
                  </button>
                  {expanded ? (
                    <div className="grid gap-3 border-t bg-muted/10 px-3 py-3">
                      <div className="grid gap-2 lg:grid-cols-2">
                        <TimelineFact label="Source" value={record.source} />
                        <TimelineFact
                          label="Recorded At"
                          value={formatAbsoluteTimestamp(record.timestamp)}
                        />
                        <TimelineFact label="Scope" value={formatScopeSummary(record)} />
                        {record.sequence ? (
                          <TimelineFact label="Sequence" value={String(record.sequence)} />
                        ) : null}
                      </div>
                      {Object.keys(record.details).length ? (
                        <dl className="grid gap-2 rounded-md border bg-background/80 p-3">
                          {Object.entries(record.details).map(([key, value]) => (
                            <div
                              className="grid gap-1 md:grid-cols-[14rem_minmax(0,1fr)] md:gap-3"
                              key={key}
                            >
                              <dt className="text-xs font-semibold tracking-[0.12em] text-muted-foreground uppercase">
                                {key}
                              </dt>
                              <dd className="text-sm [overflow-wrap:anywhere]">
                                {formatValue(value)}
                              </dd>
                            </div>
                          ))}
                        </dl>
                      ) : null}
                      <div className="flex justify-end">
                        <button
                          className="text-sm font-semibold text-primary hover:underline"
                          onClick={() => setSelectedRecord(record)}
                          type="button"
                        >
                          Inspect full record details
                        </button>
                      </div>
                    </div>
                  ) : null}
                </li>
              );
            })}
          </ol>
        ) : (
          <div className="grid min-h-44 place-content-center gap-2 rounded-md border text-center text-muted-foreground">
            <strong>No records match the current filters</strong>
            <span>Adjust the search text or selected record type.</span>
          </div>
        )}
      </div>
      <RecordDetailsDialog selected={selectedRecord} onClose={() => setSelectedRecord(null)} />
    </>
  );
}

/* === Private Functions === */

function compareRecordTimelineOrder(left: HubRecord, right: HubRecord): number {
  const timestamp = right.timestamp.localeCompare(left.timestamp);
  if (timestamp !== 0) {
    return timestamp;
  }
  return (right.sequence ?? 0) - (left.sequence ?? 0);
}

function stringifyRecordDetails(details: Record<string, unknown>): string {
  try {
    return JSON.stringify(details);
  } catch {
    return "";
  }
}

function formatScopeSummary(record: HubRecord): string {
  const scope = record.scope;
  if (!scope) {
    return "Runtime";
  }
  return (
    [
      scope.test_cycle_id ? `cycle ${scope.test_cycle_id}` : null,
      scope.test_execution_id ? `execution ${scope.test_execution_id}` : null,
      scope.test_case_id ? `case ${scope.test_case_id}` : null,
    ]
      .filter(Boolean)
      .join(" / ") || "Runtime"
  );
}

function formatAbsoluteTimestamp(timestamp: string): string {
  return absoluteTimestampLabel(timestamp);
}

/**
 * The one line a record row prints.
 *
 * A record carries a title and a message, and printing both put the same
 * sentence on the row twice. The message is the record's own words; the title
 * is what stands in when it has none.
 */
function recordLine(record: HubRecord): string {
  return recordMessage(record) || recordTitle(record);
}

function TimelineFact({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid gap-1 rounded-lg border bg-background/80 p-3">
      <span className="text-xs font-semibold uppercase tracking-[0.12em] text-muted-foreground">
        {label}
      </span>
      <strong className="text-sm [overflow-wrap:anywhere]">{value}</strong>
    </div>
  );
}
