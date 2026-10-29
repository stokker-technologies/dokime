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

import { FlexRender, type Header, type RowData } from "@tanstack/react-table";
import { ArrowDown, ArrowUp, ChevronsUpDown } from "lucide-react";

import { DataTableColumnFilter } from "@/components/data-table/DataTableColumnFilter";
import type { DataTableFeatures } from "@/components/data-table/data-table-model";
import { cn } from "@/lib/utils";

type DataTableColumnHeaderProps<TData extends RowData> = {
  header: Header<DataTableFeatures, TData, unknown>;
};

export function DataTableColumnHeader<TData extends RowData>({ header }: DataTableColumnHeaderProps<TData>) {
  const column = header.column;
  const meta = column.columnDef.meta;
  const sorted = column.getIsSorted();
  const filterable = Boolean(meta?.filter) && column.getCanFilter();
  const SortGlyph = sorted === "asc" ? ArrowUp : sorted === "desc" ? ArrowDown : ChevronsUpDown;

  // Header Names Wrap Within The Available Column Width.
  const name = (
    <span className="min-w-0 break-words whitespace-normal">
      <FlexRender header={header} />
    </span>
  );

  return (
    <div
      className={cn(
        "flex min-w-0 flex-wrap items-center gap-1",
        meta?.align === "end" && "flex-row-reverse",
        meta?.align === "center" && "justify-center",
      )}
    >
      {column.getCanSort() ? (
        <button
          className="inline-flex min-w-0 items-center gap-1 rounded-lg text-left outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
          onClick={column.getToggleSortingHandler()}
          type="button"
        >
          {name}
          <SortGlyph aria-hidden className={cn("size-3 shrink-0", sorted ? "text-foreground" : "opacity-40")} />
        </button>
      ) : (
        name
      )}
      {filterable ? <DataTableColumnFilter column={column} /> : null}
    </div>
  );
}
