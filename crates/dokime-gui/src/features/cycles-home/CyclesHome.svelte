<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createMutation, createQuery } from "@tanstack/svelte-query";
  import { DropdownMenu } from "bits-ui";
  import { ArrowRight } from "lucide-svelte";
  import type { ConnectedSession } from "../../app/native-session/session";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import Badge from "../../components/ui/Badge.svelte";
  import Button from "../../components/ui/Button.svelte";
  import SortableTableHeader from "../../components/ui/SortableTableHeader.svelte";
  import Table from "../../components/ui/Table.svelte";
  import { nextTableOrder, orderTableRows, type TableOrder } from "../../components/ui/table-order";
  import { activateTableRow } from "../../components/ui/table-row";
  import TableRowActions from "../../components/ui/TableRowActions.svelte";
  import { setNativeCycleArchived } from "../../lib/api/native-execution";
  import { failureOf } from "../../lib/notification-details";
  import { toast } from "../../lib/notifications";
  import { LIST_STATUS_PILL_CLASS } from "../cycle/cycle-workspace-model";
  import {
    applyFrozenOrder,
    archivedCycleGroups,
    cycleRows,
    groupCycleRows,
    promptCountsByCycle,
    type CycleGroupId,
    type CycleRow,
  } from "./cycles-model";
  import { cyclesHomeQueryKeys, cyclesQueryOptions, pendingPromptsQueryOptions } from "./cycles-query-options";
  import CycleStatusPill from "./CycleStatusPill.svelte";

  let { session, onOpen }: { session: ConnectedSession; onOpen: (cycleId: string) => void } = $props();
  let archived = $state(false);
  let frozenIds = $state.raw<readonly string[] | null>(null);
  let orders = $state.raw<Partial<Record<CycleGroupId, TableOrder<Column>>>>({});
  const client = () => session.queryClient;
  const cycles = createQuery(() => cyclesQueryOptions(session.identity, { archived }), client);
  const prompts = createQuery(() => pendingPromptsQueryOptions(session.identity), client);
  const unarchive = createMutation(
    () => ({
      mutationFn: (cycleId: string) => setNativeCycleArchived(session.identity, cycleId, false),
      onSettled: () => session.queryClient.invalidateQueries({ queryKey: cyclesHomeQueryKeys.all }),
    }),
    client,
  );
  const rows = $derived(cycleRows(cycles.data ?? [], promptCountsByCycle(prompts.data ?? [])));
  const groups = $derived.by(() => {
    const groups = archived ? archivedCycleGroups(rows) : groupCycleRows(rows);
    return groups.map((group) => {
      const ordered = orderTableRows(group.rows, orders[group.id]);
      return { ...group, rows: frozenIds ? applyFrozenOrder(ordered, frozenIds) : ordered };
    });
  });

  const cell = "px-2 first:pl-4 last:pr-4";
  const columns = [
    { label: "Cycle", key: "label", style: "" },
    { label: "Attention", key: "attentionCount", style: "w-32" },
    { label: "Status", key: "statusLabel", style: "w-[calc(var(--pill-text)_*_11_+_2_*_var(--pill-pad-x)_+_--spacing(6))]" },
    { label: "Cases", key: "sealedCount", style: "w-28 text-right" },
    { label: "Updated", key: "updatedAtMs", style: "w-44 text-right" },
  ] as const;
  type Column = (typeof columns)[number]["key"];
  const columnCount = columns.length + 1;

  function freeze() {
    if (!frozenIds) {
      frozenIds = groups.flatMap((group) => group.rows.map((row) => row.cycleId));
    }
  }

  async function restore(cycleId: string) {
    try {
      await unarchive.mutateAsync(cycleId);
      toast.success("Unarchived");
    } catch (error: unknown) {
      toast.error(failureOf(error, "The Archive Mark Could Not Be Removed"));
    }
  }

  function instantsLabel(row: CycleRow): string | undefined {
    const instants = [
      row.updatedAtIso ? `Updated ${new Date(row.updatedAtIso).toLocaleString()}` : null,
      row.createdAtIso ? `Created ${new Date(row.createdAtIso).toLocaleString()}` : null,
    ].filter(Boolean);
    return instants.length ? instants.join("\n") : undefined;
  }
</script>

<svelte:window onpointerup={() => (frozenIds = null)} onpointercancel={() => (frozenIds = null)} />

{#each groups as group, index (group.id)}
  {@const heading = `cycle-group-${group.id}`}
  {@const failed = index === 0 && cycles.isError}
  {@const order = orders[group.id]}
  <section id={group.id} aria-labelledby={heading} class="min-w-0" onpointerdown={freeze}>
    <div data-slot="data-table" class="grid min-w-0 content-start overflow-hidden rounded-lg border bg-card text-card-foreground">
      <header class="flex flex-wrap items-center gap-2 border-b px-4 py-3">
        <div class="min-w-48 flex-1">
          <h2 id={heading} class="text-sm font-semibold">{group.title}</h2>
          <p class="mt-1 text-xs text-muted-foreground">Open a cycle to inspect its cases and executions.</p>
        </div>
        <Button aria-pressed={archived} variant="outline" onclick={() => (archived = !archived)}>Archived</Button>
      </header>
      <Table aria-label={group.title} aria-busy={cycles.isPending} class="table-fixed">
        <thead class="border-b bg-muted/50 text-left">
          <tr>
            {#each columns as column (column.key)}
              <SortableTableHeader
                class={`${cell} ${column.style}`}
                label={column.label}
                direction={order?.column === column.key ? order.direction : undefined}
                onclick={() => (orders = { ...orders, [group.id]: nextTableOrder(order, column.key) })}
              />
            {/each}
            <th scope="col" class={[cell, "h-(--list-row-height) w-24 text-right font-medium"]}>Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y">
          {#if failed}
            <tr>
              <td colspan={columnCount} class="px-4 py-3">
                <ErrorCard failure={failureOf(cycles.error, "Cycles Unavailable")} onRetry={() => void cycles.refetch()} />
              </td>
            </tr>
          {/if}
          {#each group.rows as row (row.cycleId)}
            <tr
              data-row-id={row.cycleId}
              class="h-(--list-row-height) cursor-pointer hover:bg-muted/50"
              {@attach (element) => activateTableRow(element, () => onOpen(row.cycleId))}
            >
              <td class={cell}>
                <Button variant="plain" size="inline" class="block w-full min-w-0 truncate text-left" onclick={() => onOpen(row.cycleId)}>
                  {row.label}
                </Button>
              </td>
              <td class={cell}>
                {#if row.attentionCount}
                  <Badge class={LIST_STATUS_PILL_CLASS} variant="warning">{row.attentionCount} Prompts</Badge>
                {/if}
              </td>
              <td class={cell}><CycleStatusPill label={row.statusLabel} tone={row.statusTone} /></td>
              <td
                class={[cell, "text-right font-mono text-xs tabular-nums text-muted-foreground"]}
                title="Cases Sealed, of the Cases This Cycle Plans"
              >
                {row.sealedCount}
                / {row.caseCount}
              </td>
              <td class={[cell, "text-right font-mono text-xs tabular-nums text-muted-foreground"]} title={instantsLabel(row)}>
                {row.updatedAtIso ? new Date(row.updatedAtIso).toLocaleString() : "-"}
              </td>
              <td class={cell}>
                {#snippet overflow()}
                  <DropdownMenu.Item
                    disabled={unarchive.isPending}
                    class="cursor-default rounded-lg px-2 py-1 text-sm outline-none data-highlighted:bg-accent data-disabled:opacity-50"
                    onSelect={() => void restore(row.cycleId)}
                    >Unarchive</DropdownMenu.Item
                  >
                {/snippet}
                <TableRowActions
                  primary="Open"
                  label={row.label}
                  onSelect={() => onOpen(row.cycleId)}
                  children={archived ? overflow : undefined}
                >
                  {#snippet icon()}
                    <ArrowRight />
                  {/snippet}
                </TableRowActions>
              </td>
            </tr>
          {:else}
            {#if !failed}
              <tr>
                <td colspan={columnCount} class="h-24 px-4 text-center text-muted-foreground">
                  <span role={cycles.isPending ? "status" : undefined}>{cycles.isPending ? "Loading Rows…" : group.emptyText}</span>
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </Table>
    </div>
  </section>
{/each}
