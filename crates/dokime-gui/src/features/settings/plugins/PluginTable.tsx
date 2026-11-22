// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ReactNode } from "react";
import { SvelteMount, useSvelteSlots } from "@/components/SvelteMount";
import PluginTableView, { type PluginRow } from "./PluginTable.svelte";

export type PluginTableRow = Omit<PluginRow, "alert" | "badge" | "trailing"> & {
  alert?: ReactNode;
  badge?: ReactNode;
  trailing?: ReactNode;
};

export function PluginTable({
  actions,
  failure,
  label,
  loading,
  rows,
}: {
  actions?: ReactNode;
  failure?: ReactNode;
  label: string;
  loading?: boolean;
  rows: PluginTableRow[];
}) {
  const slots: Record<string, ReactNode> = { actions, failure };
  for (const row of rows) {
    slots[`alert:${row.key}`] = row.alert;
    slots[`badge:${row.key}`] = row.badge;
    slots[`trailing:${row.key}`] = row.trailing;
  }
  const { rendered, portals } = useSvelteSlots(slots);
  const viewRows = rows.map((row) => ({
    ...row,
    alert: rendered[`alert:${row.key}`],
    badge: rendered[`badge:${row.key}`],
    trailing: rendered[`trailing:${row.key}`],
  }));
  return (
    <>
      <SvelteMount
        component={PluginTableView}
        props={{ label, loading, rows: viewRows, actions: rendered.actions, failure: rendered.failure }}
      />
      {portals}
    </>
  );
}
