/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { ArrowLeft } from "lucide-react";
import { useMemo } from "react";

/* === Local Imports === */

import { DataTable } from "@/components/data-table/DataTable";
import { type DataTableColumns, dataTableColumns } from "@/components/data-table/data-table-model";
import { Button } from "@/components/ui/button";
import { type ExploreEntry, exploreEntryKey } from "@/features/explore/explore-contracts";
import { formatDuration } from "@/lib/format";
import { PageBleed, PageNotice, PageToolbar } from "@/shell/PageShell";

/* === Types === */

type HistoryComparisonViewProps = {
  cycles: ExploreEntry[];
  onBack: () => void;
};

type ComparisonRow = { get: (cycle: ExploreEntry) => string; label: string };

/* === Constants === */

const COMPARISON_ROWS: ComparisonRow[] = [
  { label: "Owner", get: (cycle) => cycle.owner ?? "—" },
  { label: "Source", get: (cycle) => cycle.source.label },
  { label: "Cases", get: (cycle) => String(cycle.case_count) },
  { label: "Passed", get: (cycle) => String(cycle.passed_count) },
  { label: "Failed", get: (cycle) => String(cycle.failed_count) },
  { label: "Duration", get: (cycle) => formatDuration(cycle.duration_s) ?? "—" },
  { label: "Updated", get: (cycle) => formatDate(cycle.updated_at) },
];

/* === Private Functions === */

function formatDate(iso: string): string {
  const date = new Date(iso);
  return Number.isNaN(date.getTime()) ? iso : date.toLocaleDateString();
}

function rowLabel(row: ComparisonRow): string {
  return row.label;
}

/* === Components === */

export function HistoryComparisonView({ cycles, onBack }: HistoryComparisonViewProps) {
  // One column per compared entry; the metric order is the data, so nothing sorts.
  const columns = useMemo(() => {
    const column = dataTableColumns<ComparisonRow>();
    const metric = column.accessor("label", {
      cell: ({ getValue }) => <span className="text-muted-foreground">{getValue()}</span>,
      enableSorting: false,
      header: "Metric",
      meta: { label: "Metric", width: "w-32" },
    });
    const entries = cycles.map((cycle) =>
      column.accessor((row) => row.get(cycle), {
        enableSorting: false,
        header: cycle.title,
        id: exploreEntryKey(cycle.target),
        meta: { label: cycle.title, truncate: true },
      }),
    );
    return [metric, ...entries] as DataTableColumns<ComparisonRow>;
  }, [cycles]);

  return (
    <>
      <PageToolbar>
        <Button className="h-[var(--control-height)]" onClick={onBack} size="sm" variant="ghost">
          <ArrowLeft />
          History Library
        </Button>
        <span className="text-sm text-muted-foreground">{cycles.length >= 2 ? `Comparing ${cycles.length} entries` : "Comparison"}</span>
      </PageToolbar>

      {cycles.length < 2 ? (
        <PageNotice>Select at least two entries in the library to compare them.</PageNotice>
      ) : (
        <>
          <PageBleed>
            <DataTable columns={columns} data={COMPARISON_ROWS} emptyText="Nothing to compare." getRowId={rowLabel} />
          </PageBleed>
          <p className="text-xs text-muted-foreground">Comparison covers the summary metrics above.</p>
        </>
      )}
    </>
  );
}
