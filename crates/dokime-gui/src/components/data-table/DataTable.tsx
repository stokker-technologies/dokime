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

import { type Column, type RowData, useTable } from "@tanstack/react-table";
import { ArrowRight, ChevronLeft, ChevronRight, Eye, type LucideIcon, MoreHorizontal } from "lucide-react";
import { Children, type MouseEvent, type ReactNode, useMemo, useState } from "react";

/* === Local Imports === */

import { DataTableColumnHeader } from "@/components/data-table/DataTableColumnHeader";
import { type ActiveFilter, DataTableToolbar } from "@/components/data-table/DataTableToolbar";
import {
  type DataTableColumns,
  type DataTableFeatures,
  type DataTableProps,
  dataTableColumns,
  dataTableFeatures,
  facetFilter,
  filterSummary,
} from "@/components/data-table/data-table-model";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

/* === Types === */

/** A row command: an icon button named by its label, which its tooltip repeats. */
export type DataTableRowAction = { label: string; icon: LucideIcon; onSelect: () => void; disabled?: boolean };

/* === Constants === */

/** The outer cells carry the table inset so their text lines up with the page. */
const EDGE_CELL = "first:pl-[var(--table-inset)] last:pr-[var(--table-inset)]";

/** Undefined values (no owner, never updated) sort after every real one. */
const DEFAULT_COLUMN = { sortUndefined: "last" } as const;

/** Clicks on a row's own controls belong to those controls, not to the row. */
const INNER_CONTROL = "a, button, input, label, select, textarea, [role=checkbox], [role=switch]";

/** Fits a row's icon commands, centered under the header. */
const ACTIONS_WIDTH = "w-24";

/** Room a header needs beyond its name: the cell padding, then a sort glyph
    and a filter button when the column has them. */
const HEADER_PADDING_REM = 1.75;
const HEADER_SORT_REM = 1;
const HEADER_FILTER_REM = 1.75;

/* === Components === */

export function DataTable<TData extends RowData>({
  actions,
  "aria-label": ariaLabel,
  className,
  columns,
  data,
  defaultSort,
  description,
  emptyText,
  getRowId,
  onRowClick,
  search,
  selection,
  title,
}: DataTableProps<TData>) {
  const [inspectedId, setInspectedId] = useState<string | null>(null);
  // Resolve Columns
  const resolvedColumns = useMemo(() => {
    const filtered = columns.map((column) => {
      const id = column.id ?? ("accessorKey" in column ? String(column.accessorKey) : "");
      const inferredWidth = /^(status|state|outcome)$/.test(id)
        ? "w-36"
        : /^(updated|created|connected|lastSeen|started|finished)(At|AtIso)?$/.test(id)
          ? "w-44"
          : /^(count|calls|cases|caseCount|executionCount|records|attempts|enabled)$/.test(id)
            ? "w-24"
            : id === "actions"
              ? ACTIONS_WIDTH
              : undefined;
      const width = column.meta?.width ?? inferredWidth;
      // Commands sit centered in their own column, header included.
      const align = id === "actions" ? "center" : column.meta?.align;
      column = { ...column, meta: { ...column.meta, align, width } };
      const filter = column.meta?.filter;
      if (filter === "facet") return { ...column, filterFn: facetFilter };
      if (filter === "text") return { ...column, filterFn: "includesString" };
      return { ...column, filterFn: "includesString", meta: { ...column.meta, filter: "text" } };
    }) as DataTableColumns<TData>;
    const helper = dataTableColumns<TData>();
    if (!filtered.some((column) => column.id === "actions"))
      filtered.push(
        helper.display({
          id: "actions",
          header: "Actions",
          meta: { width: ACTIONS_WIDTH, align: "center" },
          cell: ({ row }) => (
            <DataTableRowActions
              label={row.id}
              primary={{
                label: onRowClick ? "Open" : "Inspect",
                icon: onRowClick ? ArrowRight : Eye,
                onSelect: () => (onRowClick ? onRowClick(row.original) : setInspectedId(row.id)),
              }}
            />
          ),
        }),
      );
    if (!selection) return filtered;

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
  }, [columns, selection, onRowClick]);

  const initialState = useMemo(
    () => ({ pagination: { pageIndex: 0, pageSize: 10 }, sorting: defaultSort ? [defaultSort] : [] }),
    [defaultSort],
  );

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
  const flexibleColumn = table.getAllLeafColumns().find((column) => column.id !== "select" && !column.columnDef.meta?.width)?.id;
  const inspected = table.getCoreRowModel().rows.find((row) => row.id === inspectedId);
  const activeFilters: ActiveFilter[] = table.state.columnFilters.map((filter) => ({
    id: filter.id,
    text: filterSummary(filter, table.getColumn(filter.id)?.columnDef.meta),
  }));
  const emptyMessage = data.length > 0 && (activeFilters.length || table.state.globalFilter) ? "No rows match these filters." : emptyText;

  function activate(event: MouseEvent<HTMLTableRowElement>, row: TData) {
    if (!event.currentTarget.contains(event.target as Node)) return;
    const control = (event.target as Element).closest(INNER_CONTROL);
    if (control) return;
    onRowClick?.(row);
  }

  // Rows keep the sidebar's pitch: its row height plus its 2px gap, the gap
  // being the 1px rule and 1px below the content box. A table beside the
  // navigation then reads as one grid with it.
  return (
    <div
      data-slot="data-table"
      className={cn(
        "grid min-w-0 content-start [--table-row-height:var(--list-row-height)] rounded-lg border bg-card text-card-foreground overflow-hidden",
        "[--table-inset:1rem]",
        className,
      )}
    >
      <DataTableToolbar
        actions={actions}
        description={description}
        filters={activeFilters}
        onClear={(id) => table.getColumn(id)?.setFilterValue(undefined)}
        onClearAll={() => table.resetColumnFilters()}
        search={
          search ?? { label: "Filter Rows", value: table.state.globalFilter ?? "", onChange: (value) => table.setGlobalFilter(value) }
        }
        title={title}
      />

      <Table aria-label={ariaLabel} className="table-fixed" style={{ minWidth: `${Math.max(480, 180 + (columnCount - 1) * 100)}px` }}>
        <TableHeader className="bg-muted/50 border-b">
          {table.getHeaderGroups().map((group) => (
            <TableRow className="hover:bg-transparent" key={group.id}>
              {group.headers.map((header) => {
                const meta = header.column.columnDef.meta;
                const sorted = header.column.getIsSorted();
                const width = meta?.width ?? (header.column.id === flexibleColumn ? undefined : "w-36");
                const length = widthLength(width);
                const reserved = length && header.column.id !== "select" ? `max(${length}, ${headerReserve(header.column)})` : length;
                return (
                  <TableHead
                    aria-sort={
                      sorted === "asc" ? "ascending" : sorted === "desc" ? "descending" : header.column.getCanSort() ? "none" : undefined
                    }
                    className={cn(
                      "h-[var(--table-row-height)] pt-0 pb-px",
                      EDGE_CELL,
                      length ? undefined : width,
                      meta?.align === "end" && "text-right",
                      meta?.align === "center" && "text-center",
                      "bg-muted/50",
                    )}
                    key={header.id}
                    style={reserved ? { width: reserved } : undefined}
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
                        meta?.align === "center" && "text-center",
                        cell.column.id !== "select" && "overflow-hidden",
                      )}
                      key={cell.id}
                    >
                      {meta?.primary && onRowClick ? (
                        <button
                          className="block min-h-[var(--control-height)] w-full min-w-0 rounded-lg text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
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
      <footer className="flex flex-wrap items-center justify-between gap-3 border-t px-[var(--table-inset)] py-3 text-sm text-muted-foreground">
        <span className="tabular-nums">{table.getFilteredRowModel().rows.length} Rows</span>
        <div className="flex flex-wrap items-center gap-3">
          <label className="flex items-center gap-2">
            Rows Per Page
            <select
              aria-label="Rows Per Page"
              className="h-[var(--control-height)] rounded-lg border bg-background px-2"
              value={table.state.pagination.pageSize}
              onChange={(event) => table.setPageSize(Number(event.target.value))}
            >
              {[10, 25, 50, 100].map((size) => (
                <option key={size} value={size}>
                  {size}
                </option>
              ))}
            </select>
          </label>
          <Button
            className="h-[var(--control-height)]"
            disabled={!table.getCanPreviousPage()}
            onClick={() => table.previousPage()}
            size="sm"
            type="button"
            variant="outline"
          >
            <ChevronLeft data-icon="inline-start" />
            Previous
          </Button>
          <span className="tabular-nums">
            {table.state.pagination.pageIndex + 1} / {Math.max(1, table.getPageCount())}
          </span>
          <Button
            className="h-[var(--control-height)]"
            disabled={!table.getCanNextPage()}
            onClick={() => table.nextPage()}
            size="sm"
            type="button"
            variant="outline"
          >
            Next
            <ChevronRight data-icon="inline-end" />
          </Button>
        </div>
      </footer>
      <Dialog
        open={Boolean(inspected)}
        onOpenChange={(open) => {
          if (!open) setInspectedId(null);
        }}
      >
        <DialogContent className="max-h-[85dvh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Row Details</DialogTitle>
          </DialogHeader>
          <dl className="grid gap-3">
            {inspected
              ?.getAllCells()
              .filter((cell) => cell.column.id !== "actions" && cell.column.id !== "select")
              .map((cell) => (
                <div key={cell.id} className="grid gap-1">
                  <dt className="text-xs text-muted-foreground">
                    {cell.column.columnDef.meta?.label ??
                      (typeof cell.column.columnDef.header === "string" ? cell.column.columnDef.header : cell.column.id)}
                  </dt>
                  <dd className="whitespace-pre-wrap break-words text-sm">{String(cell.getValue() ?? "—")}</dd>
                </div>
              ))}
          </dl>
        </DialogContent>
      </Dialog>
    </div>
  );
}

/** A row's commands: the primary one as an icon button, then a menu holding
    only the commands without a button of their own. The menu button is left
    out when nothing remains for it. */
export function DataTableRowActions({ label, primary, children }: { label: string; primary: DataTableRowAction; children?: ReactNode }) {
  const Icon = primary.icon;
  const overflow = Children.toArray(children);
  // Its own provider, so a table outside the shell (the report) still renders.
  return (
    <TooltipProvider>
      <div className="flex items-center justify-center gap-1">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              aria-label={primary.label}
              disabled={primary.disabled}
              onClick={primary.onSelect}
              size="icon-sm"
              type="button"
              variant="ghost"
            >
              <Icon />
            </Button>
          </TooltipTrigger>
          <TooltipContent>{primary.label}</TooltipContent>
        </Tooltip>
        {overflow.length ? (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button aria-label={`Actions for ${label}`} size="icon-sm" type="button" variant="ghost">
                <MoreHorizontal />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">{overflow}</DropdownMenuContent>
          </DropdownMenu>
        ) : null}
      </div>
    </TooltipProvider>
  );
}

/* === Private Functions === */

/** A Tailwind width class as a CSS length, so a header reserve can widen it. */
function widthLength(width: string | undefined): string | undefined {
  const scale = width?.match(/^w-(\d+(?:\.\d+)?)$/);
  if (scale) return `${Number(scale[1]) / 4}rem`;
  return width?.match(/^w-\[(.+)\]$/)?.[1];
}

/** The width a header needs to show its whole name beside its controls, so a
    short name such as Priority never clips. `ch` resolves at the header's own
    font size; one spare `ch` covers wide glyphs. */
function headerReserve<TData extends RowData>(column: Column<DataTableFeatures, TData, unknown>): string {
  const definition = column.columnDef;
  const name = typeof definition.header === "string" ? definition.header : (definition.meta?.label ?? "");
  const controls =
    HEADER_PADDING_REM +
    (column.getCanSort() ? HEADER_SORT_REM : 0) +
    (definition.meta?.filter && column.getCanFilter() ? HEADER_FILTER_REM : 0);
  return `calc(${name.length + 1}ch + ${controls}rem)`;
}
