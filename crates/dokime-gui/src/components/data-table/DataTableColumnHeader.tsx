/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* A column heading: its name, a sort toggle when the column sorts, and its
 * filter when it has one. The `th` around it carries `aria-sort`.
 */

/* === External Imports === */

import { FlexRender, type Header, type RowData } from "@tanstack/react-table";
import { ArrowDown, ArrowUp, ChevronsUpDown } from "lucide-react";

/* === Local Imports === */

import { DataTableColumnFilter } from "@/components/data-table/DataTableColumnFilter";
import type { DataTableFeatures } from "@/components/data-table/data-table-model";
import { cn } from "@/lib/utils";

/* === Types === */

type DataTableColumnHeaderProps<TData extends RowData> = {
  header: Header<DataTableFeatures, TData, unknown>;
};

/* === Components === */

export function DataTableColumnHeader<TData extends RowData>({
  header,
}: DataTableColumnHeaderProps<TData>) {
  const column = header.column;
  const meta = column.columnDef.meta;
  const sorted = column.getIsSorted();
  const filterable = Boolean(meta?.filter) && column.getCanFilter();
  const SortGlyph = sorted === "asc" ? ArrowUp : sorted === "desc" ? ArrowDown : ChevronsUpDown;

  const name = (
    <span className="truncate">
      <FlexRender header={header} />
    </span>
  );

  return (
    <div
      className={cn("flex min-w-0 items-center gap-1", meta?.align === "end" && "flex-row-reverse")}
    >
      {column.getCanSort() ? (
        <button
          className="inline-flex min-w-0 items-center gap-1 rounded-sm outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
          onClick={column.getToggleSortingHandler()}
          type="button"
        >
          {name}
          <SortGlyph
            aria-hidden
            className={cn("size-3 shrink-0", sorted ? "text-foreground" : "opacity-40")}
          />
        </button>
      ) : (
        name
      )}
      {filterable ? <DataTableColumnFilter column={column} /> : null}
    </div>
  );
}
