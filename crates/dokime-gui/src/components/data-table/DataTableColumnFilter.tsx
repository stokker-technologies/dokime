/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* One column's filter, behind a small button in its header.
 *
 * A text column takes a substring; an enum-like column lists the values its
 * rows actually hold, with their counts under the other active filters.
 */

/* === External Imports === */

import type { Column, RowData } from "@tanstack/react-table";
import { ListFilter } from "lucide-react";
import { useId } from "react";

/* === Local Imports === */

import { type DataTableFeatures, facetText } from "@/components/data-table/data-table-model";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { cn } from "@/lib/utils";

/* === Types === */

type DataTableColumnFilterProps<TData extends RowData> = {
  column: Column<DataTableFeatures, TData, unknown>;
};

/* === Components === */

export function DataTableColumnFilter<TData extends RowData>({
  column,
}: DataTableColumnFilterProps<TData>) {
  const meta = column.columnDef.meta;
  const label = meta?.label ?? column.id;
  const value = column.getFilterValue();
  const activeCount = Array.isArray(value) ? value.length : value === undefined ? 0 : 1;

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          aria-label={activeCount ? `Filter ${label} (${activeCount} Active)` : `Filter ${label}`}
          className={cn(
            "shrink-0 text-muted-foreground [[data-density=touch]_&]:size-9",
            activeCount ? "text-primary" : "opacity-60 hover:opacity-100",
          )}
          size="icon-xs"
          type="button"
          variant="ghost"
        >
          <ListFilter />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="start" className="grid w-64 gap-2 p-2">
        {meta?.filter === "facet" ? (
          <FacetOptions column={column} picked={Array.isArray(value) ? value : []} />
        ) : (
          <Input
            aria-label={`Filter ${label}`}
            autoFocus
            className="h-[var(--control-height)]"
            onChange={(event) => column.setFilterValue(event.target.value || undefined)}
            placeholder={`Filter ${label}`}
            value={typeof value === "string" ? value : ""}
          />
        )}
        <Button
          className="justify-self-end"
          disabled={!activeCount}
          onClick={() => column.setFilterValue(undefined)}
          size="sm"
          type="button"
          variant="ghost"
        >
          Clear
        </Button>
      </PopoverContent>
    </Popover>
  );
}

/* === Private Components === */

function FacetOptions<TData extends RowData>({
  column,
  picked,
}: {
  column: Column<DataTableFeatures, TData, unknown>;
  picked: unknown[];
}) {
  const idPrefix = useId();
  const meta = column.columnDef.meta;
  const options = [...column.getFacetedUniqueValues().entries()]
    .map(([value, count]) => ({ count, text: facetText(meta, value), value }))
    .sort((left, right) => left.text.localeCompare(right.text));

  function toggle(value: unknown) {
    const next = picked.includes(value)
      ? picked.filter((current) => current !== value)
      : [...picked, value];
    column.setFilterValue(next.length ? next : undefined);
  }

  if (!options.length) {
    return <p className="px-2 py-1 text-sm text-muted-foreground">No values to filter by.</p>;
  }

  return (
    <ul className="grid max-h-64 overflow-y-auto">
      {options.map((option, index) => (
        <li key={option.text}>
          <label
            htmlFor={`${idPrefix}-${index}`}
            className="flex min-h-[var(--control-height)] cursor-pointer items-center gap-2 rounded-md px-2 text-sm hover:bg-muted"
          >
            <Checkbox
              checked={picked.includes(option.value)}
              id={`${idPrefix}-${index}`}
              onCheckedChange={() => toggle(option.value)}
            />
            <span className="min-w-0 flex-1 truncate" title={option.text}>
              {option.text}
            </span>
            <span className="font-mono text-xs tabular-nums text-muted-foreground">
              {option.count}
            </span>
          </label>
        </li>
      ))}
    </ul>
  );
}
