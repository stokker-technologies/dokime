/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The bar above a table: its title on the left, its search box and actions
 * on the right of the same row, then one chip per active column filter and a
 * way to clear them all. On a narrow table the controls wrap under the title.
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
  description?: ReactNode;
  filters: ActiveFilter[];
  onClear: (id: string) => void;
  onClearAll: () => void;
  search?: DataTableSearch;
  title?: ReactNode;
};

/* === Components === */

export function DataTableToolbar({ actions, description, filters, onClear, onClearAll, search, title }: DataTableToolbarProps) {
  const heading = title || description;
  if (!heading && !search && !actions && !filters.length) {
    return null;
  }

  return (
    <div className="grid gap-2 border-b px-[var(--table-inset)] py-3">
      {heading || search || actions ? (
        <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
          {heading ? (
            <div className="min-w-48 flex-1">
              {title ? <h2 className="text-sm font-semibold">{title}</h2> : null}
              {description ? <p className="mt-1 text-xs text-muted-foreground">{description}</p> : null}
            </div>
          ) : null}
          {search || actions ? (
            <div className="ml-auto flex max-w-full flex-wrap items-center justify-end gap-[var(--shell-gap)]">
              {search ? (
                <div className="relative w-64 min-w-32 max-w-full">
                  <Search className="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    aria-label={search.label}
                    className="h-[var(--control-height)] pl-8"
                    onChange={(event) => search.onChange(event.target.value)}
                    placeholder={search.label}
                    value={search.value}
                  />
                </div>
              ) : null}
              {actions}
            </div>
          ) : null}
        </div>
      ) : null}

      {filters.length ? (
        <div className="flex min-w-0 items-center gap-2 overflow-x-auto" role="status">
          {filters.map((filter) => (
            <Badge className="h-6 gap-1 pr-1" key={filter.id} variant="secondary">
              <span className="max-w-64 truncate" title={filter.text}>
                {filter.text}
              </span>
              <button
                aria-label={`Clear ${filter.text}`}
                className="rounded-lg opacity-70 outline-none hover:opacity-100 focus-visible:ring-2 focus-visible:ring-ring"
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
