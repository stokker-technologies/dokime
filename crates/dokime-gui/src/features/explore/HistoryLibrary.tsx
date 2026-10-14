/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { Cloud, Database, GitCompare, HardDrive, Search } from "lucide-react";

/* === Local Imports === */

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  type ExploreEntry,
  type ExploreSource,
  type ExploreTarget,
  exploreEntryKey,
} from "@/features/explore/explore-contracts";
import { cn } from "@/lib/utils";

/* === Types === */

type HistoryLibraryProps = {
  cycles: ExploreEntry[];
  loading: boolean;
  onCompare: () => void;
  onOpenEntry: (target: ExploreTarget) => void;
  onQueryChange: (query: string) => void;
  onToggleSelected: (entryKey: string) => void;
  /** The box's own value. The rows it selects are the server's answer. */
  query: string;
  selectedIds: Set<string>;
};

/* === Private Functions === */

function formatDate(iso: string): string {
  const date = new Date(iso);
  return Number.isNaN(date.getTime()) ? iso : date.toLocaleDateString();
}

/* === Components === */

export function HistoryLibrary({
  cycles,
  loading,
  onCompare,
  onOpenEntry,
  onQueryChange,
  onToggleSelected,
  query,
  selectedIds,
}: HistoryLibraryProps) {
  /* No client-side narrowing: `filter.text` is matched server-side against an
     entry's title, its execution identifier and its suite label, so the rows
     that arrived are the rows that matched. */
  const filtered = cycles;

  return (
    <div className="grid content-start">
      {/* One grid, one row: the search measure flexes and Compare keeps its
          intrinsic column so they stay on one baseline. The old All/Storage
          control is gone — `ExecutionFilter` carries no source or
          availability field, and every merged entry reports the one Storage
          source, so it offered a single option and selected nothing. */}
      <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-[var(--shell-gap)] border-b px-[var(--shell-inset)] py-[var(--shell-gap)]">
        <div className="relative min-w-48">
          <Search className="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            aria-label="Filter Executions"
            className="h-[var(--control-height)] pl-8"
            onChange={(event) => onQueryChange(event.target.value)}
            placeholder="Filter Executions"
            value={query}
          />
        </div>
        <Button
          className="h-[var(--control-height)]"
          disabled={selectedIds.size < 2}
          onClick={onCompare}
          size="sm"
          title={selectedIds.size < 2 ? "Select at least two entries to compare" : undefined}
        >
          <GitCompare />
          {selectedIds.size < 2
            ? `Compare (Select ${2 - selectedIds.size} More)`
            : `Compare Selected (${selectedIds.size})`}
        </Button>
      </div>

      <div>
        <Table>
          <TableHeader className="sticky top-0 z-10 bg-card">
            <TableRow>
              <TableHead className="w-10" />
              <TableHead>Execution / Cycle</TableHead>
              <TableHead>Result</TableHead>
              <TableHead>Owner</TableHead>
              <TableHead>Source</TableHead>
              <TableHead className="text-right">Updated</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {filtered.map((cycle) => {
              const selected = selectedIds.has(exploreEntryKey(cycle.target));
              return (
                <TableRow
                  className="cursor-pointer"
                  data-state={selected ? "selected" : undefined}
                  key={exploreEntryKey(cycle.target)}
                  onClick={() => onOpenEntry(cycle.target)}
                >
                  <TableCell
                    onClick={(event) => {
                      event.stopPropagation();
                      onToggleSelected(exploreEntryKey(cycle.target));
                    }}
                  >
                    <Checkbox
                      aria-label={`Select ${cycle.title} for comparison`}
                      checked={selected}
                      onClick={(event) => event.stopPropagation()}
                      onCheckedChange={() => onToggleSelected(exploreEntryKey(cycle.target))}
                    />
                  </TableCell>
                  <TableCell>
                    {/* Title and its qualifiers share one line. */}
                    <div className="flex min-w-0 items-center gap-2">
                      <span className="truncate font-medium">{cycle.title}</span>
                      {cycle.target.kind === "execution" ? (
                        <Badge variant="outline">Standalone execution</Badge>
                      ) : null}
                      {cycle.lifecycle !== "completed" ? (
                        <Badge variant="outline">{cycle.lifecycle}</Badge>
                      ) : null}
                    </div>
                    {/* Secondary detail only when it adds information: the
                        raw id here simply repeated the title. */}
                    {cycle.suite_name ? (
                      <div className="text-xs text-muted-foreground">{cycle.suite_name}</div>
                    ) : null}
                  </TableCell>
                  <TableCell>
                    <ResultPills failed={cycle.failed_count} passed={cycle.passed_count} />
                  </TableCell>
                  <TableCell className="text-sm">{cycle.owner ?? "—"}</TableCell>
                  <TableCell>
                    <SourceBadge source={cycle.source} />
                  </TableCell>
                  <TableCell className="text-right text-sm text-muted-foreground">
                    {formatDate(cycle.updated_at)}
                  </TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
        {filtered.length === 0 ? (
          <p className="p-6 text-center text-sm text-muted-foreground">
            {loading ? "Loading history…" : "No history matches."}
          </p>
        ) : null}
      </div>
    </div>
  );
}

function ResultPills({ failed, passed }: { failed: number; passed: number }) {
  return (
    <div className="flex items-center gap-1.5 text-xs font-medium">
      <span className="text-success">{passed} passed</span>
      <span className={cn(failed > 0 ? "text-destructive" : "text-muted-foreground")}>
        {failed} failed
      </span>
    </div>
  );
}

function SourceBadge({ source }: { source: ExploreSource }) {
  const Icon = source.kind === "storage" ? Database : source.kind === "local" ? HardDrive : Cloud;
  return (
    <Badge
      className={cn(
        "h-5 gap-1 rounded px-1.5 text-[0.7rem] font-medium [&>svg]:size-3",
        source.kind === "remote" ? "border-info/30 bg-info/10 text-info" : null,
      )}
      variant={source.kind === "remote" ? "secondary" : "outline"}
    >
      <Icon />
      {source.label}
    </Badge>
  );
}
