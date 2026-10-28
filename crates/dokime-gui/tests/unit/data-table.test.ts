/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Table Behavior
 */

import assert from "node:assert/strict";

import { type ReactTable, useTable } from "@tanstack/react-table";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test, vi } from "vitest";

import { DataTable, DataTableRowActions } from "@/components/data-table/DataTable";
import { type DataTableFeatures, dataTableColumns, dataTableFeatures } from "@/components/data-table/data-table-model";
import { Button } from "@/components/ui/button";

const column = dataTableColumns<{ id: string; value: string }>();
const columns = column.columns([column.accessor("value", { header: "Value", meta: { filter: "text" } })]);
const data = Array.from({ length: 24 }, (_, index) => ({ id: String(index), value: `item-${String(index).padStart(2, "0")}` }));

function table() {
  let view!: ReactTable<DataTableFeatures, { id: string; value: string }>;
  function Probe() {
    view = useTable({
      features: dataTableFeatures,
      columns,
      data,
      getRowId: (row) => row.id,
      initialState: { pagination: { pageIndex: 0, pageSize: 10 } },
    });
    return null;
  }
  renderToStaticMarkup(createElement(Probe));
  return view;
}

test("Sorting And Filtering Precede Pagination", () => {
  const view = table();
  view.setSorting([{ id: "value", desc: true }]);
  assert.deepEqual(
    view.getRowModel().rows.map((row) => row.id),
    ["23", "22", "21", "20", "19", "18", "17", "16", "15", "14"],
  );
  view.getColumn("value")?.setFilterValue("item-0");
  assert.equal(view.getFilteredRowModel().rows.length, 10);
  assert.equal(view.getRowModel().rows[0].id, "9");
  view.resetColumnFilters();
  view.setGlobalFilter("item-2");
  assert.deepEqual(
    view.getRowModel().rows.map((row) => row.id),
    ["23", "22", "21", "20"],
  );
});

test("Pagination And Page Size Keep All Rows Reachable", () => {
  const view = table();
  assert.equal(view.getCanPreviousPage(), false);
  view.nextPage();
  assert.equal(view.getRowModel().rows[0].id, "10");
  view.nextPage();
  assert.equal(view.getRowModel().rows.length, 4);
  assert.equal(view.getCanNextPage(), false);
  view.setPageSize(25);
  assert.equal(view.getRowModel().rows.length, 24);
  assert.equal(view.getPageCount(), 1);
});

test("Empty Tables Keep Column Headers And An In Table Empty Row", () => {
  const html = renderToStaticMarkup(
    createElement(DataTable<{ id: string; value: string }>, {
      columns,
      data: [],
      getRowId: (row: { id: string; value: string }) => row.id,
      emptyText: "fixture-empty",
    }),
  );
  assert.equal((html.match(/<thead/g) ?? []).length, 1);
  assert.equal((html.match(/<tbody/g) ?? []).length, 1);
  assert.match(html, /<td[^>]*colSpan="2"/i);
  assert.match(html, /fixture-empty/);
});

test("Row Actions Invoke The Primary Command And Expose A Menu", () => {
  const action = vi.fn();
  const tree = DataTableRowActions({ label: "fixture", primary: { label: "fixture-open", onSelect: action } });
  const children = tree.props.children;
  assert.equal(children[0].type, Button);
  children[0].props.onClick();
  assert.equal(action.mock.calls.length, 1);
  const html = renderToStaticMarkup(tree);
  assert.match(html, /aria-haspopup="menu"/);
});
