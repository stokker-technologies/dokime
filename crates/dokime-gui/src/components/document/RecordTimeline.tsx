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

import { RECORD_ROW_COLUMNS, RecordRowPanel } from "@/components/document/RecordRowPanel";
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
import { recordMessage, recordTitle } from "@/lib/dashboard";
import {
  recordClockLabel,
  recordLevelLabel,
  recordLevelVariant,
  recordToneRowClass,
} from "@/lib/record-display";
import type { HubRecord } from "@/lib/types";
import { cn } from "@/lib/utils";

/* === Constants === */

const FILTER_LABEL_CLASS = "text-xs font-bold uppercase tracking-[0.14em] text-muted-foreground";

/** One control height for every filter, whatever its primitive's own default. */
const FILTER_CONTROL_CLASS = "h-8";

const FILTER_HELPER_CLASS = "text-xs text-muted-foreground";

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
            the only border around this filter row. Every field spans the
            same three shared rows — label, control, helper — so labels share
            one baseline, controls one height, and helpers one line. */}
        {live ? (
          <div className="grid grid-rows-[auto_auto_auto] gap-x-3 gap-y-1.5 text-sm lg:grid-cols-[minmax(0,1fr)_16rem]">
            <div className="row-span-3 grid grid-rows-subgrid items-baseline">
              <Label className={FILTER_LABEL_CLASS} htmlFor={recordSearchId}>
                Search Records
              </Label>
              <div className="relative self-stretch">
                <Search
                  className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
                  size={14}
                />
                <Input
                  id={recordSearchId}
                  className={cn(FILTER_CONTROL_CLASS, "pl-9")}
                  placeholder="Search type, source, message, or details; * and ? supported"
                  value={query}
                  onChange={(record) => setQuery(record.target.value)}
                />
              </div>
              <span className={FILTER_HELPER_CLASS}>
                Leave blank for all records. Use * for any text or ? for one character.
              </span>
            </div>
            <div className="row-span-3 grid grid-rows-subgrid items-baseline">
              <Label className={FILTER_LABEL_CLASS} htmlFor={recordTypeId}>
                Record Type
              </Label>
              <Select value={recordType} onValueChange={setRecordType}>
                <SelectTrigger
                  className={cn(FILTER_CONTROL_CLASS, "w-full self-stretch bg-background")}
                  id={recordTypeId}
                >
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
              <span className={FILTER_HELPER_CLASS}>Narrow the timeline to one record type.</span>
            </div>
          </div>
        ) : null}

        <div className="grid grid-cols-[minmax(0,1fr)_auto] items-baseline gap-x-3 border-b pb-2 text-sm text-muted-foreground">
          <span className="truncate">
            {live
              ? `Showing ${visibleRecords.length} of ${records.length} records.`
              : `${records.length} ${records.length === 1 ? "record" : "records"}.`}
          </span>
          <span className="justify-self-end">The timeline is ordered newest first.</span>
        </div>

        {visibleRecords.length ? (
          /* One dense grid, live and frozen: time, level, source, message, and
             the technical record type where it belongs — at the row's end,
             muted, for the reader who needs it. The row keeps its message to
             one line; expanding it opens the panel with the full message and
             the payload behind it. */
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
                      RECORD_ROW_COLUMNS,
                      "w-full items-start px-3 py-1.5 text-left font-mono text-xs tabular-nums hover:bg-muted/40",
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
                    <span className="min-w-0 truncate">{recordLine(record)}</span>
                    <span className="truncate text-muted-foreground">{record.record_type}</span>
                  </button>
                  {expanded ? (
                    <RecordRowPanel
                      live={live}
                      onInspect={() => setSelectedRecord(record)}
                      record={record}
                    />
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
