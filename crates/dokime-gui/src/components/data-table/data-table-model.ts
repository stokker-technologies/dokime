/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The one table model every list in the GUI is built on.
 *
 * Sorting, per-column filters and their facets are registered once here, so a
 * column opts into them through its meta rather than each surface wiring its
 * own row models.
 */

import {
  type ColumnDef,
  type ColumnFiltersState,
  type ColumnSort,
  columnFacetingFeature,
  columnFilteringFeature,
  createColumnHelper,
  createFacetedRowModel,
  createFacetedUniqueValues,
  createFilteredRowModel,
  createPaginatedRowModel,
  createSortedRowModel,
  type FilterFn,
  filterFn_includesString,
  globalFilteringFeature,
  metaHelper,
  type RowData,
  rowPaginationFeature,
  rowSortingFeature,
  sortFn_alphanumeric,
  sortFn_basic,
  sortFn_datetime,
  sortFn_text,
  tableFeatures,
} from "@tanstack/react-table";
import type { ReactNode } from "react";

/** How a column reads, sorts and filters; everything is opt-in. */
export type DataTableColumnMeta = {
  /** Numbers and dates sit on the right edge. */
  align?: "start" | "end";
  /** Labels a facet value; the raw value as text otherwise. */
  facetLabel?: (value: unknown) => string;
  /** `text` matches a substring; `facet` picks among the column's values. */
  filter?: "facet" | "text";
  /** The column's plain-text name for filter controls and chips. */
  label?: string;
  /** This cell is the row's keyboard control: it renders as a button that
      opens the row. */
  primary?: boolean;
  /** One line, clipped, with the full value on hover. */
  truncate?: boolean;
  /** A fixed width class, such as `w-28`; absent columns share the rest. */
  width?: string;
};

/** A search box the owner applies; the table renders it and never filters by it. */
export type DataTableSearch = {
  label: string;
  onChange: (value: string) => void;
  value: string;
};

/** A checkbox column whose state the owner keeps. */
export type DataTableSelection<TData> = {
  isSelected: (row: TData) => boolean;
  label: (row: TData) => string;
  onToggle: (row: TData) => void;
};

/** Heterogeneous accessor values, as TanStack's own `helper.columns` returns them. */
// biome-ignore lint/suspicious/noExplicitAny: column value types vary per column and are invariant.
export type DataTableColumns<TData extends RowData> = ColumnDef<DataTableFeatures, TData, any>[];

export type DataTableProps<TData extends RowData> = {
  /** Controls beside the search box, such as Compare. */
  actions?: ReactNode;
  "aria-label"?: string;
  className?: string;
  columns: DataTableColumns<TData>;
  data: TData[];
  defaultSort?: ColumnSort;
  emptyText: string;
  getRowId: (row: TData) => string;
  onRowClick?: (row: TData) => void;
  search?: DataTableSearch;
  selection?: DataTableSelection<TData>;
};

export type DataTableFeatures = typeof dataTableFeatures;

export const dataTableFeatures = tableFeatures({
  rowPaginationFeature,
  paginatedRowModel: createPaginatedRowModel(),
  globalFilteringFeature,
  columnFilteringFeature,
  filteredRowModel: createFilteredRowModel(),
  filterFns: { includesString: filterFn_includesString },
  columnFacetingFeature,
  facetedRowModel: createFacetedRowModel(),
  facetedUniqueValues: createFacetedUniqueValues(),
  rowSortingFeature,
  sortedRowModel: createSortedRowModel(),
  sortFns: {
    alphanumeric: sortFn_alphanumeric,
    basic: sortFn_basic,
    datetime: sortFn_datetime,
    text: sortFn_text,
  },
  columnMeta: metaHelper<DataTableColumnMeta>(),
});

/** Keeps a row whose value is one of the picked facet values. */
// biome-ignore lint/suspicious/noExplicitAny: one filter serves every row type.
export const facetFilter: FilterFn<DataTableFeatures, any> = (row, columnId, picked) =>
  Array.isArray(picked) && picked.includes(row.getValue(columnId));
facetFilter.autoRemove = (picked) => !Array.isArray(picked) || picked.length === 0;

export function dataTableColumns<TData extends RowData>() {
  return createColumnHelper<DataTableFeatures, TData>();
}

export function facetText(meta: DataTableColumnMeta | undefined, value: unknown): string {
  if (meta?.facetLabel) return meta.facetLabel(value);
  return value === undefined || value === null || value === "" ? "None" : String(value);
}

/** One readable line per active filter, for the chip that clears it. */
export function filterSummary(filter: ColumnFiltersState[number], meta: DataTableColumnMeta | undefined): string {
  const name = meta?.label ?? filter.id;
  if (!Array.isArray(filter.value)) return `${name}: “${String(filter.value)}”`;

  const values = filter.value.map((value) => facetText(meta, value));
  return values.length > 2 ? `${name}: ${values.slice(0, 2).join(", ")} +${values.length - 2}` : `${name}: ${values.join(", ")}`;
}
