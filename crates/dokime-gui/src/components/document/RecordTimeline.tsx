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
 *
 * Expanding a row opens its panel beneath it — age, payload, provenance —
 * on the row's own column tracks; the row itself stays one line.
 */

/* === External Imports === */

import { Search } from "lucide-react";
import { useMemo, useState } from "react";

/* === Local Imports === */

import { RecordDetailsDialog } from "@/components/RecordDetailsDialog";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { recordMessage, recordTitle } from "@/lib/dashboard";
import { recordClockLabel, recordLevelLabel, recordLevelVariant, recordToneRowClass } from "@/lib/record-display";
import type { HubRecord } from "@/lib/types";
import { cn } from "@/lib/utils";

/* === Constants === */

/** One control height for every filter, whatever its primitive's own default. */
const FILTER_CONTROL_CLASS = "h-[var(--bar-control-height)]";

/** The record row's column tracks: time, level, source, message, type. */
const RECORD_ROW_COLUMNS = "grid grid-cols-[6rem_5.5rem_10rem_minmax(0,1fr)_auto] gap-x-3";

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
        return [record.record_type, record.source, recordTitle(record), recordMessage(record), stringifyRecordDetails(record.details)].some(
          (value) => matcher.test(value),
        );
      });
  }, [recordType, records, query]);

  return (
    <>
      <div className="grid gap-2">
        {/* One toolbar row, the same height as every other list bar: the
            search box carries its own hint, the type select sits beside it.
            The Records card already owns the only border around it. */}
        {live ? (
          <div className="grid items-center gap-x-3 text-sm sm:grid-cols-[minmax(0,1fr)_14rem]">
            <div className="relative">
              <Search
                aria-hidden
                className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground"
              />
              <Input
                aria-label="Search Records"
                className={cn(FILTER_CONTROL_CLASS, "pl-8")}
                id={recordSearchId}
                onChange={(record) => setQuery(record.target.value)}
                placeholder="Search records (* any text, ? one character)"
                value={query}
              />
            </div>
            <Select onValueChange={setRecordType} value={recordType}>
              <SelectTrigger aria-label="Record Type" className={cn(FILTER_CONTROL_CLASS, "w-full bg-background")} id={recordTypeId}>
                <SelectValue placeholder="All Records" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Records</SelectItem>
                {recordTypes.map((value) => (
                  <SelectItem key={value} value={value}>
                    {value}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}

        <div className="grid grid-cols-[minmax(0,1fr)_auto] items-baseline gap-x-3 border-b pb-1 text-xs text-muted-foreground">
          <span className="truncate">
            {live
              ? `Showing ${visibleRecords.length} of ${records.length} records`
              : `${records.length} ${records.length === 1 ? "record" : "records"}`}
          </span>
          <span className="justify-self-end">Newest first</span>
        </div>

        {visibleRecords.length ? (
          /* One dense grid, live and frozen: time, level, source, message, and
             the technical record type where it belongs — at the row's end,
             muted, for the reader who needs it. The row keeps its message to
             one line; opening it shows the whole record in the dialog. */
          /* The Records card already owns the only border around this list;
             a second one here would be a card inside a card. */
          <ol className="grid border-t" data-testid="record-rows">
            {visibleRecords.map((record) => {
              return (
                <li
                  className={cn("border-b border-l-2 border-b-border/60 last:border-b-0", recordToneRowClass(record.display.tone))}
                  key={record.record_id}
                >
                  <button
                    aria-haspopup="dialog"
                    className={cn(
                      RECORD_ROW_COLUMNS,
                      "w-full items-start px-3 py-1.5 text-left font-mono text-xs tabular-nums hover:bg-muted/40",
                    )}
                    onClick={() => setSelectedRecord(record)}
                    type="button"
                  >
                    <time className="truncate text-muted-foreground" dateTime={record.timestamp}>
                      {recordClockLabel(record.timestamp)}
                    </time>
                    <Badge className="h-4 justify-self-start px-1 text-[0.65rem]" variant={recordLevelVariant(record.level)}>
                      {recordLevelLabel(record.level)}
                    </Badge>
                    <span className="truncate text-muted-foreground">{record.source}</span>
                    <span className="min-w-0 truncate">{recordLine(record)}</span>
                    <span className="truncate text-muted-foreground">{record.record_type}</span>
                  </button>
                </li>
              );
            })}
          </ol>
        ) : (
          <p className="py-6 text-center text-sm text-muted-foreground">
            {live && (query.trim() || recordType !== "all") ? "No records match the current filters." : "No records yet."}
          </p>
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
