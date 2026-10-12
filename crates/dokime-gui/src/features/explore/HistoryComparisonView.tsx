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

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { type ExploreEntry, exploreEntryKey } from "@/features/explore/explore-contracts";
import { formatDuration } from "@/lib/format";

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
  { label: "Suite", get: (cycle) => cycle.suite_name ?? "—" },
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

/* === Components === */

export function HistoryComparisonView({ cycles, onBack }: HistoryComparisonViewProps) {
  return (
    <div className="grid content-start">
      <div className="grid grid-cols-[auto_minmax(0,1fr)] items-center gap-[var(--shell-gap)] border-b px-[var(--shell-inset)] py-[var(--shell-gap)]">
        <Button className="h-[var(--control-height)]" onClick={onBack} size="sm" variant="ghost">
          <ArrowLeft />
          History Library
        </Button>
        <span className="text-sm text-muted-foreground">
          {cycles.length >= 2 ? `Comparing ${cycles.length} entries` : "Comparison"}
        </span>
      </div>

      <div className="p-[var(--shell-inset)]">
        {cycles.length < 2 ? (
          <p className="text-sm text-muted-foreground">
            Select at least two entries in the library to compare them.
          </p>
        ) : (
          <>
            <div className="overflow-hidden rounded-lg border">
              <table className="w-full border-collapse text-sm">
                <thead>
                  <tr className="border-b bg-muted/40">
                    <th className="p-3 text-left font-medium text-muted-foreground">Metric</th>
                    {cycles.map((cycle) => (
                      <th
                        className="p-3 text-left font-semibold"
                        key={exploreEntryKey(cycle.target)}
                      >
                        {cycle.title}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {COMPARISON_ROWS.map((row) => (
                    <tr className="border-b last:border-0" key={row.label}>
                      <td className="p-3 text-muted-foreground">{row.label}</td>
                      {cycles.map((cycle) => (
                        <td className="p-3" key={exploreEntryKey(cycle.target)}>
                          {row.get(cycle)}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <p className="mt-3 text-xs text-muted-foreground">
              Comparison covers the summary metrics above.
            </p>
          </>
        )}
      </div>
    </div>
  );
}
