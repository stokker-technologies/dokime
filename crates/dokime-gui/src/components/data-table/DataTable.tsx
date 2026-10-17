/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Every list in the GUI, drawn one way.
 *
 * Columns come from TanStack defs; this component owns the markup: one cell
 * padding, the shared row height, a sticky header, sort toggles, per-column
 * filters, an optional checkbox column and row activation. It renders through
 * the `ui/table` primitives, so document tables share its cell padding.
 */

/* === External Imports === */

import { type RowData, useTable } from "@tanstack/react-table";
import { type MouseEvent, useMemo } from "react";

/* === Local Imports === */

import { DataTableColumnHeader } from "@/components/data-table/DataTableColumnHeader";
import { type ActiveFilter, DataTableToolbar } from "@/components/data-table/DataTableToolbar";
import {
  type DataTableColumns,
  type DataTableProps,
  dataTableColumns,
  dataTableFeatures,
  facetFilter,
  filterSummary,
} from "@/components/data-table/data-table-model";
import { Checkbox } from "@/components/ui/checkbox";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { cn } from "@/lib/utils";

/* === Constants === */

/** The outer cells carry the table inset so their text lines up with the page. */
const EDGE_CELL = "first:pl-[var(--table-inset)] last:pr-[var(--table-inset)]";

const INSET_CLASS = {
  cell: "[--table-inset:0.75rem]",
  shell: "[--table-inset:var(--shell-inset)]",
} as const;

/** Undefined values (no owner, never updated) sort after every real one. */
const DEFAULT_COLUMN = { sortUndefined: "last" } as const;

/** Clicks on a row's own controls belong to those controls, not to the row. */
const INNER_CONTROL = "a, button, input, label, select, textarea, [role=checkbox], [role=switch]";

/* === Components === */

export function DataTable<TData extends RowData>({
  actions,
  "aria-label": ariaLabel,
  className,
  columns,
  data,
  defaultSort,
  emptyText,
  getRowId,
  inset = "shell",
  onRowClick,
  search,
  selection,
  stickyHeader = true,
}: DataTableProps<TData>) {
  // Resolve Columns
  const resolvedColumns = useMemo(() => {
    const filtered = columns.map((column) => {
      const filter = column.meta?.filter;
      if (filter === "facet") return { ...column, filterFn: facetFilter };
      if (filter === "text") return { ...column, filterFn: "includesString" };
      return { ...column, enableColumnFilter: false };
    }) as DataTableColumns<TData>;
    if (!selection) return filtered;

    const helper = dataTableColumns<TData>();
    const select = helper.display({
      cell: ({ row }) => (
        <Checkbox
          aria-label={selection.label(row.original)}
          checked={selection.isSelected(row.original)}
          onCheckedChange={() => selection.onToggle(row.original)}
        />
      ),
      header: () => <span className="sr-only">Select</span>,
      id: "select",
      meta: { width: "w-10" },
    });
    return [select, ...filtered];
  }, [columns, selection]);

  const initialState = useMemo(() => ({ sorting: defaultSort ? [defaultSort] : [] }), [defaultSort]);

  const table = useTable({
    columns: resolvedColumns,
    data,
    defaultColumn: DEFAULT_COLUMN,
    features: dataTableFeatures,
    getRowId,
    initialState,
    sortDescFirst: false,
  });

  // Derive View State
  const rows = table.getRowModel().rows;
  const columnCount = table.getAllLeafColumns().length;
  const activeFilters: ActiveFilter[] = table.state.columnFilters.map((filter) => ({
    id: filter.id,
    text: filterSummary(filter, table.getColumn(filter.id)?.columnDef.meta),
  }));
  const emptyMessage = data.length > 0 && activeFilters.length ? "No rows match these filters." : emptyText;

  function activate(event: MouseEvent<HTMLTableRowElement>, row: TData) {
    const control = (event.target as Element).closest(INNER_CONTROL);
    if (control && event.currentTarget.contains(control)) return;
    onRowClick?.(row);
  }

  // Rows keep the sidebar's pitch: its row height plus its 2px gap, the gap
  // being the 1px rule and 1px below the content box. A table beside the
  // navigation then reads as one grid with it.
  return (
    <div
      className={cn(
        "grid min-w-0 content-start [--table-row-height:calc(var(--sidebar-row-height)+0.125rem)] [--table-surface:var(--background)]",
        INSET_CLASS[inset],
        className,
      )}
    >
      <DataTableToolbar
        actions={actions}
        filters={activeFilters}
        onClear={(id) => table.getColumn(id)?.setFilterValue(undefined)}
        onClearAll={() => table.resetColumnFilters()}
        search={search}
      />

      <Table aria-label={ariaLabel} className="table-fixed" containerClassName={stickyHeader ? "overflow-visible" : undefined}>
        <TableHeader className={cn(stickyHeader && "[&_tr]:border-b-0")}>
          {table.getHeaderGroups().map((group) => (
            <TableRow className="hover:bg-transparent" key={group.id}>
              {group.headers.map((header) => {
                const meta = header.column.columnDef.meta;
                const sorted = header.column.getIsSorted();
                return (
                  <TableHead
                    aria-sort={
                      sorted === "asc" ? "ascending" : sorted === "desc" ? "descending" : header.column.getCanSort() ? "none" : undefined
                    }
                    className={cn(
                      "h-[var(--table-row-height)] pt-0 pb-px",
                      EDGE_CELL,
                      meta?.width,
                      meta?.align === "end" && "text-right",
                      stickyHeader && "sticky top-0 z-10 bg-[var(--table-surface)] shadow-[inset_0_-1px_0_var(--border)]",
                    )}
                    key={header.id}
                  >
                    {header.isPlaceholder ? null : <DataTableColumnHeader header={header} />}
                  </TableHead>
                );
              })}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {rows.map((row) => {
            const selected = selection?.isSelected(row.original) ?? false;
            return (
              <TableRow
                className={cn("h-[var(--table-row-height)]", onRowClick && "cursor-pointer")}
                data-state={selected ? "selected" : undefined}
                key={row.id}
                onClick={onRowClick ? (event) => activate(event, row.original) : undefined}
              >
                {row.getAllCells().map((cell) => {
                  const meta = cell.column.columnDef.meta;
                  const value = cell.getValue();
                  const content = <table.FlexRender cell={cell} />;
                  return (
                    <TableCell
                      className={cn(
                        "pt-0 pb-px",
                        EDGE_CELL,
                        meta?.align === "end" && "text-right",
                        cell.column.id !== "select" && "overflow-hidden",
                      )}
                      key={cell.id}
                    >
                      {meta?.primary && onRowClick ? (
                        <button
                          className="block w-full min-w-0 rounded-sm text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
                          onClick={() => onRowClick(row.original)}
                          type="button"
                        >
                          {content}
                        </button>
                      ) : meta?.truncate ? (
                        <div
                          className="truncate"
                          title={typeof value === "string" || typeof value === "number" ? String(value) : undefined}
                        >
                          {content}
                        </div>
                      ) : (
                        content
                      )}
                    </TableCell>
                  );
                })}
              </TableRow>
            );
          })}
          {rows.length === 0 ? (
            <TableRow className="hover:bg-transparent">
              <TableCell className="h-24 px-[var(--table-inset)] text-center whitespace-normal text-muted-foreground" colSpan={columnCount}>
                {emptyMessage}
              </TableCell>
            </TableRow>
          ) : null}
        </TableBody>
      </Table>
    </div>
  );
}
