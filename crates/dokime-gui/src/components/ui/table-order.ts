// Dokime — SPDX-License-Identifier: Apache-2.0

export type TableOrder<Column> = { column: Column; direction: "ascending" | "descending" };

export function nextTableOrder<Column>(order: TableOrder<Column> | undefined, column: Column): TableOrder<Column> | undefined {
  if (order?.column !== column) {
    return { column, direction: "ascending" };
  }
  return order.direction === "ascending" ? { column, direction: "descending" } : undefined;
}

export function orderTableRows<Row, Column extends keyof Row>(rows: Row[], order?: TableOrder<Column>): Row[] {
  if (!order) {
    return rows;
  }
  const { column, direction } = order;
  const sign = direction === "ascending" ? 1 : -1;
  return rows.toSorted((left, right) => {
    const a = left[column];
    const b = right[column];
    if (a == null || b == null) {
      return Number(a == null) - Number(b == null);
    }
    const comparison = typeof a === "number" && typeof b === "number" ? a - b : String(a).localeCompare(String(b));
    return sign * comparison;
  });
}
