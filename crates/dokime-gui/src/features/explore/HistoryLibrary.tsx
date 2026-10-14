/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { Cloud, Database, GitCompare, HardDrive } from "lucide-react";
import { useMemo } from "react";

/* === Local Imports === */

import { DataTable } from "@/components/data-table/DataTable";
import {
  type DataTableColumns,
  type DataTableSelection,
  dataTableColumns,
} from "@/components/data-table/data-table-model";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
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

/* === Constants === */

const column = dataTableColumns<ExploreEntry>();

const COLUMNS: DataTableColumns<ExploreEntry> = column.columns([
  column.accessor("title", {
    cell: ({ row }) => <EntryTitle entry={row.original} />,
    header: "Execution / Cycle",
    meta: { label: "Execution / Cycle", primary: true },
  }),
  column.accessor("failed_count", {
    cell: ({ row }) => (
      <ResultPills failed={row.original.failed_count} passed={row.original.passed_count} />
    ),
    header: "Result",
    meta: { label: "Result", width: "w-36" },
  }),
  column.accessor("lifecycle", {
    cell: ({ row }) => <span className="text-sm capitalize">{row.original.lifecycle}</span>,
    header: "Lifecycle",
    meta: { filter: "facet", label: "Lifecycle", width: "w-32" },
  }),
  column.accessor((entry) => entry.owner ?? undefined, {
    cell: ({ row }) => <span className="text-sm">{row.original.owner ?? "—"}</span>,
    header: "Owner",
    id: "owner",
    meta: { filter: "facet", label: "Owner", truncate: true, width: "w-32" },
  }),
  column.accessor((entry) => entry.source.label, {
    cell: ({ row }) => <SourceBadge source={row.original.source} />,
    header: "Source",
    id: "source",
    meta: { filter: "facet", label: "Source", width: "w-36" },
  }),
  column.accessor((entry) => Date.parse(entry.updated_at) || undefined, {
    cell: ({ row }) => (
      <span className="text-sm text-muted-foreground">{formatDate(row.original.updated_at)}</span>
    ),
    header: "Updated",
    id: "updated",
    meta: { align: "end", label: "Updated", width: "w-28" },
  }),
]);

/* === Private Functions === */

function formatDate(iso: string): string {
  const date = new Date(iso);
  return Number.isNaN(date.getTime()) ? iso : date.toLocaleDateString();
}

function entryKey(entry: ExploreEntry): string {
  return exploreEntryKey(entry.target);
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
     that arrived are the rows that matched. The table only renders the box. */
  const search = useMemo(
    () => ({ label: "Filter Executions", onChange: onQueryChange, value: query }),
    [onQueryChange, query],
  );
  const selection = useMemo<DataTableSelection<ExploreEntry>>(
    () => ({
      isSelected: (entry) => selectedIds.has(entryKey(entry)),
      label: (entry) => `Select ${entry.title} for comparison`,
      onToggle: (entry) => onToggleSelected(entryKey(entry)),
    }),
    [onToggleSelected, selectedIds],
  );

  const compare = (
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
  );

  return (
    <DataTable
      actions={compare}
      columns={COLUMNS}
      data={cycles}
      emptyText={loading ? "Loading history…" : "No history matches."}
      getRowId={entryKey}
      onRowClick={(entry) => onOpenEntry(entry.target)}
      search={search}
      selection={selection}
    />
  );
}

/* === Private Components === */

/** One line: the title, its qualifier, then the suite when it adds information. */
function EntryTitle({ entry }: { entry: ExploreEntry }) {
  return (
    <span className="flex min-w-0 items-center gap-2">
      <span className="truncate">{entry.title}</span>
      {entry.target.kind === "execution" ? (
        <Badge variant="outline">Standalone Execution</Badge>
      ) : null}
      {entry.suite_name ? (
        <span className="min-w-0 truncate text-xs text-muted-foreground">{entry.suite_name}</span>
      ) : null}
    </span>
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
