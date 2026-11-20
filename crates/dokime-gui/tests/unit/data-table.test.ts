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

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { type ReactTable, useTable } from "@tanstack/react-table";
import { Eye } from "lucide-react";
import { createElement, isValidElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test, vi } from "vitest";

// Local Imports
import { DataTable, DataTableRowActions } from "@/components/data-table/DataTable";
import { DataTableColumnHeader } from "@/components/data-table/DataTableColumnHeader";
import { type DataTableFeatures, dataTableColumns, dataTableFeatures } from "@/components/data-table/data-table-model";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/** The first element in a returned tree, without rendering, whose props match. */
function findElement(node: ReactNode, match: (props: Record<string, unknown>) => boolean): Record<string, unknown> | null {
  if (Array.isArray(node)) {
    for (const child of node) {
      const found = findElement(child, match);
      if (found) return found;
    }
    return null;
  }
  if (!isValidElement(node)) return null;
  const props = node.props as Record<string, unknown>;
  if (match(props)) return props;
  return findElement(props.children as ReactNode, match);
}

/* === Entry Point === */

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

test("Sorting and Filtering Precede Pagination", () => {
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

test("Unit Header Controls", () => {
  const view = table();
  const header = view.getHeaderGroups()[0].headers[0];
  header.column.columnDef.header = "fixture [Mbps]";
  header.column.columnDef.meta = { label: "fixture [Mbps]", filter: "text" };
  const toggle = vi.fn();
  const handler = vi.spyOn(header.column, "getToggleSortingHandler").mockReturnValue(toggle);
  const heading = DataTableColumnHeader({ header });
  const name = findElement(heading, (props) => props.title === "fixture [Mbps]");
  assert.ok(name);
  assert.match(String(name.className), /\btruncate\b/);
  const sort = findElement(heading, (props) => typeof props.onClick === "function");
  assert.ok(sort);
  (sort.onClick as (event: object) => void)({ persist() {} });
  assert.equal(toggle.mock.calls.length, 1);
  handler.mockRestore();
  const html = renderToStaticMarkup(heading);
  assert.match(html, /aria-haspopup="dialog"/);
  assert.match(html, /<svg/);
  assert.doesNotMatch(html, /whitespace-normal|break-words|flex-wrap/);
});

test("Pagination and Page Size Keep All Rows Reachable", () => {
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

test("Empty Tables Keep Column Headers and an in Table Empty Row", () => {
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

test("Loading and Failure Take the Body Slot While Rows Read Keep Their Place", () => {
  const props = { columns, getRowId: (row: { id: string; value: string }) => row.id, emptyText: "fixture-empty" };
  const loading = renderToStaticMarkup(createElement(DataTable<{ id: string; value: string }>, { ...props, data: [], loading: true }));
  assert.match(loading, /<td[^>]*colSpan="2"[^>]*><span role="status">/i);
  assert.doesNotMatch(loading, /fixture-empty/);
  const failure = createElement("p", { "data-fixture": "failure" });
  const failed = renderToStaticMarkup(
    createElement(DataTable<{ id: string; value: string }>, { ...props, data: [{ id: "row", value: "kept" }], failure }),
  );
  assert.match(failed, /<tbody[^>]*><tr[^>]*><td[^>]*colSpan="2"[^>]*><p data-fixture="failure">/i);
  assert.match(failed, /data-row-id="row"/);
  assert.doesNotMatch(failed, /fixture-empty/);
  // One Page Keeps the Footer, With Both Paging Buttons Disabled Rather Than Hidden.
  const html = renderToStaticMarkup(createElement(DataTable<{ id: string; value: string }>, { ...props, data: [] }));
  const footer = html.match(/<footer[\s\S]*<\/footer>/)?.[0] ?? "";
  assert.equal((footer.match(/<button[^>]*disabled=""/g) ?? []).length, 2);
});

test("Only Declared Filters Are Offered and Actions Stay Last", () => {
  const helper = dataTableColumns<{ id: string; count: number }>();
  const html = renderToStaticMarkup(
    createElement(DataTable<{ id: string; count: number }>, {
      columns: helper.columns([
        helper.display({
          id: "actions",
          header: "fixture-actions",
          cell: () => createElement("button", { type: "button", "data-fixture": "action" }),
        }),
        helper.accessor("count", { header: "fixture-count", meta: { align: "end" } }),
        helper.accessor("id", { header: "fixture-id", meta: { filter: "text" } }),
      ]),
      data: [{ id: "row", count: 1200 }],
      getRowId: (row) => row.id,
      emptyText: "fixture-empty",
    }),
  );
  const headings = html.match(/<thead[\s\S]*?<\/thead>/)?.[0] ?? "";
  assert.ok(headings.indexOf("fixture-count") < headings.indexOf("fixture-id"));
  assert.ok(headings.indexOf("fixture-id") < headings.indexOf("fixture-actions"));
  assert.equal((headings.match(/aria-haspopup="dialog"/g) ?? []).length, 1);
  assert.match(html, /text-right[^>]*>1200<\/td>/);
});

test("Row Actions Are Named Icon Buttons with a Menu Only for the Rest", () => {
  const action = vi.fn();
  const primary = { label: "fixture-open", icon: Eye, onSelect: action };
  const alone = DataTableRowActions({ label: "fixture", primary });
  const button = findElement(alone, (props) => props["aria-label"] === "fixture-open");
  assert.ok(button);
  (button.onClick as () => void)();
  assert.equal(action.mock.calls.length, 1);
  const html = renderToStaticMarkup(alone);
  assert.match(html, /<button[^>]*aria-label="fixture-open"[^>]*><svg/);
  assert.doesNotMatch(html, /aria-haspopup="menu"/);

  const withMore = DataTableRowActions({
    children: createElement(DropdownMenuItem, { onSelect: vi.fn() }, "fixture-more"),
    label: "fixture",
    primary,
  });
  assert.match(
    renderToStaticMarkup(withMore),
    /aria-haspopup="menu"[^>]*aria-label="Actions for fixture"|aria-label="Actions for fixture"[^>]*aria-haspopup="menu"/,
  );
});
