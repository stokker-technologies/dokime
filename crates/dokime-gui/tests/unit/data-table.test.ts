// @vitest-environment happy-dom
// Dokime — SPDX-License-Identifier: Apache-2.0
import assert from "node:assert/strict";
import { fireEvent, render } from "@testing-library/svelte";
import { createRawSnippet } from "svelte";
import { test, vi } from "vitest";
import SortableTableHeader from "@/components/ui/SortableTableHeader.svelte";
import TablePanel from "@/components/ui/TablePanel.svelte";
import TableRowActions from "@/components/ui/TableRowActions.svelte";

const columns = createRawSnippet(() => ({
  render: () => '<th scope="col">Value</th>',
}));
const icon = createRawSnippet(() => ({
  render: () => '<svg aria-hidden="true"></svg>',
}));

test("Unit Header Controls", async () => {
  const onclick = vi.fn();
  const view = render(SortableTableHeader, { label: "Fixture [Mbps]", direction: "descending", onclick });
  const header = view.getByRole("columnheader");
  assert.equal(header.getAttribute("aria-sort"), "descending");
  assert.ok(header.querySelector("svg"));
  await fireEvent.click(view.getByRole("button", { name: "Fixture [Mbps]" }));
  assert.equal(onclick.mock.calls.length, 1);
  view.unmount();
});

test("Empty Tables Keep Column Headers and an in Table Empty Row", () => {
  const view = render(TablePanel, { title: "Fixture", columns, columnCount: 1, rowCount: 0, empty: "Fixture Empty" });
  const table = view.getByRole("table", { name: "Fixture" });
  assert.equal(table.querySelectorAll("thead th").length, 1);
  assert.equal(table.querySelectorAll("tbody tr").length, 1);
  assert.equal(table.querySelector("tbody td")?.getAttribute("colspan"), "1");
  assert.equal(view.queryByRole("status"), null);
  view.unmount();
});

test("Loading and Failure Take the Body Slot While Rows Read Keep Their Place", async () => {
  const props = { title: "Fixture", columns, columnCount: 1, rowCount: 0, empty: "Fixture Empty" };
  const view = render(TablePanel, { ...props, loading: true });
  let table = view.getByRole("table");
  assert.equal(view.getByRole("status").closest("tbody"), table.querySelector("tbody"));
  assert.equal(table.getAttribute("aria-busy"), "true");
  const failure = createRawSnippet(() => ({ render: () => '<p role="alert">Fixture Failure</p>' }));
  const children = createRawSnippet(() => ({ render: () => '<tr data-row-id="fixture"><td>Saved Row</td></tr>' }));
  await view.rerender({ ...props, loading: false, rowCount: 1, children, failure });
  table = view.getByRole("table");
  assert.equal(view.getByRole("alert").closest("tr"), table.querySelector("tbody tr"));
  assert.ok(table.querySelector('tr[data-row-id="fixture"]'));
  assert.equal(view.queryByRole("status"), null);
  assert.equal(table.getAttribute("aria-busy"), "false");
  view.unmount();
});

test("Row Actions Are Named Icon Buttons with a Menu Only for the Rest", async () => {
  const onSelect = vi.fn();
  const props = { primary: "Open", label: "Fixture", icon, onSelect };
  const view = render(TableRowActions, props);
  const button = view.getByRole("button", { name: "Open" });
  assert.ok(button.querySelector("svg"));
  await fireEvent.click(button);
  assert.equal(onSelect.mock.calls.length, 1);
  assert.equal(view.queryByRole("button", { name: "Actions for Fixture" }), null);
  const children = createRawSnippet(() => ({ render: () => "<span>Fixture Action</span>" }));
  await view.rerender({ ...props, children });
  const trigger = view.getByRole("button", { name: "Actions for Fixture" });
  assert.equal(trigger.getAttribute("aria-haspopup"), "menu");
  view.unmount();
});
