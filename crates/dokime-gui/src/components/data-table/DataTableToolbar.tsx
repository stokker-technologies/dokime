/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The bar above a table: its search box and actions on one baseline, then
 * one chip per active column filter and a way to clear them all.
 *
 * Each line is one sidebar row tall, after the sidebar's own 8px top pad and
 * with its 2px gap, so the table header below lands on a navigation row.
 */

/* === External Imports === */

import { Search, X } from "lucide-react";
import type { ReactNode } from "react";

/* === Local Imports === */

import type { DataTableSearch } from "@/components/data-table/data-table-model";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

/* === Types === */

export type ActiveFilter = { id: string; text: string };

type DataTableToolbarProps = {
  actions?: ReactNode;
  filters: ActiveFilter[];
  onClear: (id: string) => void;
  onClearAll: () => void;
  search?: DataTableSearch;
};

/* === Components === */

export function DataTableToolbar({ actions, filters, onClear, onClearAll, search }: DataTableToolbarProps) {
  if (!search && !actions && !filters.length) {
    return null;
  }

  return (
    <div className="grid auto-rows-[var(--sidebar-row-height)] gap-0.5 px-[var(--table-inset)] pt-2 pb-0.5">
      {search || actions ? (
        <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-[var(--shell-gap)]">
          {search ? (
            <div className="relative min-w-48">
              <Search className="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                aria-label={search.label}
                className="h-[var(--control-height)] pl-8"
                onChange={(event) => search.onChange(event.target.value)}
                placeholder={search.label}
                value={search.value}
              />
            </div>
          ) : (
            <span />
          )}
          {actions ? <div className="flex items-center gap-[var(--shell-gap)]">{actions}</div> : null}
        </div>
      ) : null}

      {filters.length ? (
        <div className="flex min-w-0 items-center gap-1.5 overflow-x-auto" role="status">
          {filters.map((filter) => (
            <Badge className="h-6 gap-1 pr-1" key={filter.id} variant="secondary">
              <span className="max-w-64 truncate" title={filter.text}>
                {filter.text}
              </span>
              <button
                aria-label={`Clear ${filter.text}`}
                className="rounded-sm opacity-70 outline-none hover:opacity-100 focus-visible:ring-2 focus-visible:ring-ring"
                onClick={() => onClear(filter.id)}
                type="button"
              >
                <X className="size-3" />
              </button>
            </Badge>
          ))}
          <Button className="h-6" onClick={onClearAll} size="xs" type="button" variant="ghost">
            Clear All
          </Button>
        </div>
      ) : null}
    </div>
  );
}
